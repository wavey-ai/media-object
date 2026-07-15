use media_object::{
    decode, decode_with_limits, encode, encoded_len, ClockConfidence, ClockTimestamp, Error,
    Limits, MediaObject, ObjectKey, ObjectKind, Stage, StageTimestamp, WriteDisposition,
    WIRE_MAGIC, WIRE_VERSION,
};

fn timestamp(time: i64, clock: &str) -> ClockTimestamp {
    ClockTimestamp::new(time, clock, ClockConfidence::synchronized(250_000)).unwrap()
}

fn dependency(track: &str, object: u64, payload: &[u8]) -> ObjectKey {
    ObjectKey::for_payload("tenant-a", "live", track, 7, 10, object, 1, payload).unwrap()
}

fn full_object() -> MediaObject {
    let payload = b"cmaf-media-payload".to_vec();
    let key =
        ObjectKey::for_payload("tenant-a", "live", "video-1080p", 7, 10, 4, 1, &payload).unwrap();
    MediaObject::builder(key, ObjectKind::Media, payload)
        .with_keyframe(true)
        .with_configuration_epoch(3)
        .with_deadline(timestamp(1_721_000_000_900_000_000, "ptp:gm-1"))
        .with_capture_timestamp(timestamp(1_721_000_000_100_000_000, "ptp:gm-1"))
        .with_stage_timestamp(StageTimestamp::new(
            Stage::Published,
            timestamp(1_721_000_000_300_000_000, "ptp:gm-1"),
        ))
        .with_stage_timestamp(StageTimestamp::new(
            Stage::IngressReceived,
            timestamp(1_721_000_000_200_000_000, "ptp:gm-1"),
        ))
        .with_dependency(dependency("audio", 3, b"audio-three"))
        .with_dependency(dependency("video-1080p", 2, b"video-two"))
        .with_metadata("content-type", b"video/mp4".to_vec())
        .with_metadata("codec", b"avc1.640028".to_vec())
        .build()
        .unwrap()
}

fn minimal_object(payload: &[u8]) -> MediaObject {
    let key = ObjectKey::for_payload("t", "s", "v", 1, 2, 3, 1, payload).unwrap();
    MediaObject::builder(key, ObjectKind::Media, payload.to_vec())
        .build()
        .unwrap()
}

#[test]
fn full_v1_object_round_trips_losslessly() {
    let object = full_object();
    let bytes = encode(&object).unwrap();

    assert_eq!(&bytes[..4], WIRE_MAGIC.as_slice());
    assert_eq!(u16::from_be_bytes([bytes[4], bytes[5]]), WIRE_VERSION);
    assert_eq!(encoded_len(&object).unwrap(), bytes.len());
    assert_eq!(decode(&bytes).unwrap(), object);

    assert_eq!(object.kind(), ObjectKind::Media);
    assert!(object.is_keyframe());
    assert_eq!(object.configuration_epoch(), 3);
    assert!(object.capture_timestamp().is_some());
    assert!(object.deadline().is_some());
    assert_eq!(object.stage_timestamps().len(), 2);
    assert_eq!(object.dependencies().len(), 2);
}

#[test]
fn canonical_encoding_is_independent_of_set_insertion_order() {
    let payload = b"same".to_vec();
    let key = ObjectKey::for_payload("tenant", "stream", "track", 1, 2, 3, 4, &payload).unwrap();
    let early = StageTimestamp::new(Stage::Normalized, timestamp(10, "clock-a"));
    let late = StageTimestamp::new(Stage::Published, timestamp(20, "clock-a"));
    let first_dependency = dependency("audio", 1, b"one");
    let second_dependency = dependency("video", 2, b"two");

    let first = MediaObject::builder(key.clone(), ObjectKind::Media, payload.clone())
        .with_stage_timestamp(late.clone())
        .with_stage_timestamp(early.clone())
        .with_dependency(second_dependency.clone())
        .with_dependency(first_dependency.clone())
        .with_metadata("z-last", b"z".to_vec())
        .with_metadata("a-first", b"a".to_vec())
        .build()
        .unwrap();
    let second = MediaObject::builder(key, ObjectKind::Media, payload)
        .with_stage_timestamp(early)
        .with_stage_timestamp(late)
        .with_dependency(first_dependency)
        .with_dependency(second_dependency)
        .with_metadata("a-first", b"a".to_vec())
        .with_metadata("z-last", b"z".to_vec())
        .build()
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(encode(&first).unwrap(), encode(&second).unwrap());
    assert_eq!(
        first.envelope_hash().unwrap(),
        second.envelope_hash().unwrap()
    );
}

#[test]
fn payload_tampering_is_rejected_during_decode() {
    let mut bytes = encode(&minimal_object(b"payload")).unwrap();
    *bytes.last_mut().unwrap() ^= 0x80;

    assert_eq!(decode(&bytes), Err(Error::PayloadHashMismatch));
}

#[test]
fn write_comparison_distinguishes_retry_conflict_and_distinct_key() {
    let stored = minimal_object(b"payload");
    assert_eq!(
        stored.compare_write(&stored.clone()),
        WriteDisposition::IdempotentReplay
    );

    let conflicting = MediaObject::builder(
        stored.key().clone(),
        ObjectKind::Media,
        stored.payload().to_vec(),
    )
    .with_metadata("source", b"other".to_vec())
    .build()
    .unwrap();
    assert_eq!(
        stored.compare_write(&conflicting),
        WriteDisposition::IdentityConflict
    );
    assert_eq!(
        stored.compare_write(&minimal_object(b"different")),
        WriteDisposition::DistinctObject
    );
}

#[test]
fn keyframe_flag_is_reserved_for_media_objects() {
    let payload = b"init".to_vec();
    let key = ObjectKey::for_payload("t", "s", "v", 1, 0, 0, 1, &payload).unwrap();
    let result = MediaObject::builder(key, ObjectKind::Initialization, payload)
        .with_keyframe(true)
        .build();

    assert!(matches!(
        result,
        Err(Error::InvalidField {
            field: "keyframe",
            ..
        })
    ));
}

#[test]
fn all_explicit_non_media_kinds_round_trip() {
    for kind in [
        ObjectKind::Initialization,
        ObjectKind::CodecConfiguration,
        ObjectKind::Discontinuity,
    ] {
        let payload = [kind as u8];
        let key = ObjectKey::for_payload("t", "s", "v", 2, 0, 0, 1, &payload).unwrap();
        let object = MediaObject::builder(key, kind, payload.to_vec())
            .build()
            .unwrap();
        assert_eq!(decode(&encode(&object).unwrap()).unwrap().kind(), kind);
    }
}

#[test]
fn self_dependencies_and_duplicate_set_values_are_rejected() {
    let object = minimal_object(b"payload");
    assert_eq!(
        MediaObject::builder(
            object.key().clone(),
            ObjectKind::Media,
            object.payload().to_vec()
        )
        .with_dependency(object.key().clone())
        .build(),
        Err(Error::SelfDependency)
    );

    let dependency = dependency("audio", 1, b"dependency");
    assert_eq!(
        MediaObject::builder(
            object.key().clone(),
            ObjectKind::Media,
            object.payload().to_vec()
        )
        .with_dependency(dependency.clone())
        .with_dependency(dependency)
        .build(),
        Err(Error::DuplicateValue("dependencies"))
    );

    assert_eq!(
        MediaObject::builder(
            object.key().clone(),
            ObjectKind::Media,
            object.payload().to_vec()
        )
        .with_metadata("codec", b"one".to_vec())
        .with_metadata("codec", b"two".to_vec())
        .build(),
        Err(Error::DuplicateValue("metadata"))
    );
}

#[test]
fn identifiers_are_nonempty_bounded_and_free_of_controls() {
    assert!(matches!(
        ObjectKey::for_payload("", "s", "v", 0, 0, 0, 0, b""),
        Err(Error::EmptyField("tenant"))
    ));
    assert!(matches!(
        ObjectKey::for_payload("tenant\n", "s", "v", 0, 0, 0, 0, b""),
        Err(Error::InvalidField {
            field: "tenant",
            ..
        })
    ));
    assert!(matches!(
        ObjectKey::for_payload("x".repeat(129), "s", "v", 0, 0, 0, 0, b""),
        Err(Error::LimitExceeded {
            field: "tenant",
            ..
        })
    ));
}

#[test]
fn clock_confidence_carries_numeric_uncertainty() {
    let known = ClockConfidence::traceable(42);
    assert_eq!(known.maximum_error_ns(), Some(42));
    assert_eq!(ClockConfidence::unknown().maximum_error_ns(), None);

    let stamp = ClockTimestamp::new(123, "ptp:grandmaster", known).unwrap();
    assert_eq!(stamp.unix_time_ns(), 123);
    assert_eq!(stamp.clock_id(), "ptp:grandmaster");
    assert_eq!(stamp.confidence(), known);
}

#[test]
fn lower_operational_limits_apply_to_build_validate_and_decode() {
    let object = minimal_object(b"four");
    let bytes = encode(&object).unwrap();
    let mut limits = Limits::HARD;
    limits.max_payload_bytes = 3;

    assert!(matches!(
        object.validate_with_limits(limits),
        Err(Error::LimitExceeded {
            field: "payload",
            actual: 4,
            maximum: 3,
        })
    ));
    assert!(matches!(
        decode_with_limits(&bytes, limits),
        Err(Error::LimitExceeded {
            field: "payload",
            actual: 4,
            maximum: 3,
        })
    ));
}

#[test]
fn limits_cannot_raise_compiled_safety_ceilings() {
    let bytes = encode(&minimal_object(b"x")).unwrap();
    let mut limits = Limits::HARD;
    limits.max_dependencies += 1;

    assert!(matches!(
        decode_with_limits(&bytes, limits),
        Err(Error::InvalidLimit {
            field: "max_dependencies",
            ..
        })
    ));
}

#[test]
fn aggregate_metadata_limit_is_enforced() {
    let object = minimal_object(b"x");
    let mut limits = Limits::HARD;
    limits.max_metadata_bytes = 5;
    let result = MediaObject::builder(
        object.key().clone(),
        ObjectKind::Media,
        object.payload().to_vec(),
    )
    .with_metadata("abc", b"def".to_vec())
    .build_with_limits(limits);

    assert!(matches!(
        result,
        Err(Error::LimitExceeded {
            field: "metadata",
            actual: 6,
            maximum: 5,
        })
    ));
}

#[test]
fn malicious_payload_length_is_rejected_before_payload_read() {
    let object = minimal_object(b"x");
    let mut bytes = encode(&object).unwrap();
    let payload_length_offset = bytes.len() - object.payload().len() - 4;
    bytes[payload_length_offset..payload_length_offset + 4]
        .copy_from_slice(&u32::MAX.to_be_bytes());

    assert!(matches!(
        decode(&bytes),
        Err(Error::LimitExceeded {
            field: "payload",
            ..
        })
    ));
}

#[test]
fn malicious_collection_counts_are_rejected_before_collection_allocation() {
    // Minimal v1 layout with one-byte tenant/stream/track values:
    // stage count @ 95, dependency count @ 97, metadata count @ 99.
    for (offset, field) in [
        (95, "stage_timestamps"),
        (97, "dependencies"),
        (99, "metadata_entries"),
    ] {
        let mut bytes = encode(&minimal_object(b"x")).unwrap();
        bytes[offset..offset + 2].copy_from_slice(&u16::MAX.to_be_bytes());
        assert!(matches!(
            decode(&bytes),
            Err(Error::LimitExceeded {
                field: actual_field,
                ..
            }) if actual_field == field
        ));
    }
}

#[test]
fn malformed_outer_frame_fields_are_rejected() {
    let bytes = encode(&minimal_object(b"x")).unwrap();

    let mut bad_magic = bytes.clone();
    bad_magic[0] ^= 1;
    assert_eq!(decode(&bad_magic), Err(Error::InvalidMagic));

    let mut bad_version = bytes.clone();
    bad_version[4..6].copy_from_slice(&2_u16.to_be_bytes());
    assert_eq!(decode(&bad_version), Err(Error::UnsupportedVersion(2)));

    let mut bad_header_reserved = bytes.clone();
    bad_header_reserved[7] = 1;
    assert_eq!(
        decode(&bad_header_reserved),
        Err(Error::ReservedField("header"))
    );

    let mut bad_object_reserved = bytes.clone();
    bad_object_reserved[15] = 1;
    assert_eq!(
        decode(&bad_object_reserved),
        Err(Error::ReservedField("object"))
    );

    let mut bad_length = bytes;
    let declared = u32::from_be_bytes(bad_length[8..12].try_into().unwrap());
    bad_length[8..12].copy_from_slice(&(declared + 1).to_be_bytes());
    assert!(matches!(
        decode(&bad_length),
        Err(Error::DeclaredLengthMismatch { .. })
    ));
}

#[test]
fn malformed_tags_booleans_and_utf8_are_rejected() {
    let bytes = encode(&minimal_object(b"x")).unwrap();

    let mut unknown_kind = bytes.clone();
    unknown_kind[12] = 99;
    assert_eq!(
        decode(&unknown_kind),
        Err(Error::UnknownTag {
            field: "object_kind",
            value: 99,
        })
    );

    let mut invalid_bool = bytes.clone();
    invalid_bool[13] = 2;
    assert_eq!(
        decode(&invalid_bool),
        Err(Error::InvalidBoolean {
            field: "keyframe",
            value: 2,
        })
    );

    // Header (12), kind/keyframe/reserved (4), tenant length (2), tenant byte.
    let mut invalid_utf8 = bytes;
    invalid_utf8[18] = 0xff;
    assert_eq!(decode(&invalid_utf8), Err(Error::InvalidUtf8("tenant")));
}

#[test]
fn noncanonical_metadata_order_is_rejected_on_the_wire() {
    let payload = b"x".to_vec();
    let key = ObjectKey::for_payload("t", "s", "v", 1, 2, 3, 1, &payload).unwrap();
    let object = MediaObject::builder(key, ObjectKind::Media, payload)
        .with_metadata("a", b"1".to_vec())
        .with_metadata("b", b"2".to_vec())
        .build()
        .unwrap();
    let mut bytes = encode(&object).unwrap();
    let pattern = [0, 1, b'a', 0, 0, 0, 1, b'1', 0, 1, b'b'];
    let position = bytes
        .windows(pattern.len())
        .position(|window| window == pattern)
        .unwrap();
    bytes[position + 2] = b'b';
    bytes[position + 10] = b'a';

    assert_eq!(decode(&bytes), Err(Error::NonCanonicalOrder("metadata")));
}

#[test]
fn every_truncated_prefix_is_rejected() {
    let bytes = encode(&full_object()).unwrap();
    for length in 0..bytes.len() {
        assert!(decode(&bytes[..length]).is_err(), "prefix length {length}");
    }
    assert!(decode(&bytes).is_ok());
}

#[test]
fn single_byte_mutations_never_panic_the_decoder() {
    let bytes = encode(&full_object()).unwrap();
    for index in 0..bytes.len() {
        for mask in [0x01, 0x80, 0xff] {
            let mut mutated = bytes.clone();
            mutated[index] ^= mask;
            let _result = decode(&mutated);
        }
    }
}

#[test]
fn envelope_has_a_stable_golden_digest() {
    let object = minimal_object(b"golden");
    assert_eq!(
        object.envelope_hash().unwrap().to_string(),
        "6cf7483e65bae6eafe6e856096beea264f6bbbe19b6278535d4d721a97c1392c"
    );
}

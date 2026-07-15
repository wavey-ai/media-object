use media_object::{
    AudienceId, AuthorizationFactId, CapabilityId, ContributorId, DescriptorId, EdgeId,
    EffectiveRole, EndpointId, MediaAuthorizationFactV1, MediaAuthorizationFactV1Params,
    MediaAuthorizationRequestV1, MediaAuthorizationRequestV1Params, MediaCapabilityClaimsV1,
    MediaCapabilityClaimsV1Params, MediaCapabilityValidationContextV1, MediaCaptureDisposition,
    MediaClass, MediaConfigurationId, MediaControlErrorCode, MediaEndpointDescriptorV1,
    MediaEndpointDescriptorV1Params, MediaEndpointTransport, MediaFrameConfigurationV1,
    MediaFrameConfigurationV1Params, MediaFrameEnvelopeV1, MediaFrameEnvelopeV1Params,
    MediaFramePayloadFormat, Operation, ParticipantId, SessionId, SessionMediaIdentityV1,
    SessionMediaIdentityV1Params, SessionWorkflowMode, SourceId, SubjectId, TakeId, TenantId,
    MEDIA_CONTROL_MAX_GENERATION,
};

const VALID_IDENTITY: &[u8] =
    include_bytes!("fixtures/media-control/v1/session-media-identity.json");
const VALID_CLAIMS: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-capability-claims.json");
const VALID_DESCRIPTOR: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-endpoint-descriptor.json");
const VALID_AUTHORIZATION_REQUEST: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-authorization-request.json");
const VALID_AUTHORIZATION_FACT: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-authorization-fact.json");
const VALID_FRAME_CONFIGURATION: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-frame-configuration.json");
const VALID_FRAME_ENVELOPE: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-frame-envelope.json");
const FUTURE_CLAIMS: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-capability-future-version.json");
const NONCANONICAL_CLAIMS: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-capability-noncanonical-edge-order.json");
const WRONG_SESSION_CLAIMS: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-capability-wrong-session.json");
const EXPIRED_CLAIMS: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-capability-expired.json");
const INVALID_TALKBACK_IDENTITY: &[u8] =
    include_bytes!("fixtures/media-control/v1/session-media-identity-talkback-with-source.json");
const DESCRIPTOR_WITH_SECRET: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-endpoint-descriptor-with-secret.json");
const AUTHORIZATION_REQUEST_WITH_RELAY_KEY: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-authorization-request-with-relay-key.json");
const AUTHORIZATION_FACT_WITH_SUBJECT: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-authorization-fact-with-subject.json");

fn tenant(value: &str) -> TenantId {
    TenantId::new(value).unwrap()
}

fn session(value: &str) -> SessionId {
    SessionId::new(value).unwrap()
}

fn participant(value: &str) -> ParticipantId {
    ParticipantId::new(value).unwrap()
}

fn endpoint(value: &str) -> EndpointId {
    EndpointId::new(value).unwrap()
}

fn contributor(value: &str) -> ContributorId {
    ContributorId::new(value).unwrap()
}

fn source(value: &str) -> SourceId {
    SourceId::new(value).unwrap()
}

fn audience(value: &str) -> AudienceId {
    AudienceId::new(value).unwrap()
}

fn edge(value: &str) -> EdgeId {
    EdgeId::new(value).unwrap()
}

fn subject(value: &str) -> SubjectId {
    SubjectId::new(value).unwrap()
}

fn take(value: &str) -> TakeId {
    TakeId::new(value).unwrap()
}

fn request_params() -> MediaAuthorizationRequestV1Params {
    MediaAuthorizationRequestV1Params {
        subject: subject("sub_zeroth_01"),
        endpoint_id: endpoint("ep_logic"),
        requested_operation: Operation::Publish,
        requested_media_class: MediaClass::Program,
        requested_source_ids: vec![source("src_mix")],
        requested_audience_ids: Vec::new(),
        take_id: None,
    }
}

fn fact_params() -> MediaAuthorizationFactV1Params {
    MediaAuthorizationFactV1Params {
        authorization_fact_id: AuthorizationFactId::new("maf_01").unwrap(),
        session_id: session("ses_mix"),
        session_epoch: 9,
        media_authorization_epoch: 14,
        subject_grant_epoch: 3,
        media_policy_version: 7,
        participant_id: participant("par_producer"),
        endpoint_id: endpoint("ep_logic"),
        effective_role: EffectiveRole::Producer,
        access_expires_at: Some(1_784_134_800),
        allowed_operations: vec![
            Operation::UploadTake,
            Operation::Subscribe,
            Operation::AcknowledgePlayout,
            Operation::ReadTake,
            Operation::Publish,
        ],
        allowed_media_classes: vec![
            MediaClass::Talkback,
            MediaClass::Source,
            MediaClass::TakeChunk,
            MediaClass::Program,
            MediaClass::Screen,
            MediaClass::Metadata,
        ],
        allowed_source_ids: vec![source("src_mix")],
        allowed_audience_ids: Vec::new(),
        requested_operation: Operation::Publish,
        requested_media_class: MediaClass::Program,
        take_id: None,
        workflow_mode: SessionWorkflowMode::FinalTake,
        evaluated_at: 1_784_131_200,
    }
}

fn claims_params() -> MediaCapabilityClaimsV1Params {
    MediaCapabilityClaimsV1Params {
        issuer: "https://control.infidelity.io".to_owned(),
        audience: "av-contrib".to_owned(),
        capability_id: CapabilityId::new("cap_publish_mix").unwrap(),
        tenant_id: tenant("ten_wavey"),
        session_id: session("ses_mix"),
        session_epoch: 9,
        media_authorization_epoch: 14,
        subject_grant_epoch: 3,
        media_policy_version: 7,
        class_authorization_epoch: Some(4),
        binding_generation: 8,
        participant_id: participant("par_producer"),
        endpoint_id: endpoint("ep_logic"),
        contributor_id: Some(contributor("con_logic")),
        operation: Operation::Publish,
        media_class: MediaClass::Program,
        source_ids: vec![source("src_mix")],
        audience_ids: Vec::new(),
        take_id: None,
        topology_generation: 52,
        edge_ids: vec![edge("edge_ams"), edge("edge_lon")],
        max_channels: 2,
        max_bitrate: 512_000,
        max_datagram_bytes: 1_200,
        client_key_thumbprint: Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned()),
        issued_at: 1_784_131_200,
        not_before: 1_784_131_195,
        expires_at: 1_784_131_260,
    }
}

fn frame_configuration_params() -> MediaFrameConfigurationV1Params {
    MediaFrameConfigurationV1Params {
        configuration_id: MediaConfigurationId::new("cfg_talkback_01").unwrap(),
        binding_generation: 8,
        configuration_ref: 17,
        configuration_epoch: 4,
        identity: SessionMediaIdentityV1::new(SessionMediaIdentityV1Params {
            tenant_id: tenant("ten_wavey"),
            session_id: session("ses_mix"),
            session_epoch: 9,
            participant_id: participant("par_listener"),
            endpoint_id: endpoint("ep_browser"),
            contributor_id: contributor("con_browser"),
            source_id: None,
            media_class: MediaClass::Talkback,
            audience_id: Some(audience("aud_producer_return")),
            take_id: None,
            topology_generation: 52,
        })
        .unwrap(),
        payload_format: MediaFramePayloadFormat::Opus,
        capture_timebase_hz: 48_000,
        channel_count: 1,
        max_payload_bytes: 400,
        capture_disposition: MediaCaptureDisposition::MonitorOnly,
    }
}

fn frame_envelope_params() -> MediaFrameEnvelopeV1Params {
    MediaFrameEnvelopeV1Params {
        binding_generation: 8,
        configuration_ref: 17,
        configuration_epoch: 4,
        sequence: 9_001,
        capture_pts: -960,
        duration_ticks: 960,
        payload_bytes: 160,
    }
}

fn validation_ids() -> (
    TenantId,
    SessionId,
    ParticipantId,
    EndpointId,
    ContributorId,
    SourceId,
    EdgeId,
) {
    (
        tenant("ten_wavey"),
        session("ses_mix"),
        participant("par_producer"),
        endpoint("ep_logic"),
        contributor("con_logic"),
        source("src_mix"),
        edge("edge_ams"),
    )
}

#[test]
fn canonical_fixtures_round_trip_byte_for_byte() {
    let request =
        MediaAuthorizationRequestV1::from_json_slice(VALID_AUTHORIZATION_REQUEST).unwrap();
    assert_eq!(
        request.to_canonical_json_vec().unwrap(),
        VALID_AUTHORIZATION_REQUEST
    );
    assert_eq!(request.subject().as_str(), "sub_zeroth_01");
    assert_eq!(request.endpoint_id().as_str(), "ep_logic");
    assert_eq!(request.requested_operation(), Operation::Publish);
    assert_eq!(request.requested_media_class(), MediaClass::Program);
    assert_eq!(request.requested_source_ids(), &[source("src_mix")]);
    assert!(request.requested_audience_ids().is_empty());
    assert!(request.take_id().is_none());

    let fact = MediaAuthorizationFactV1::from_json_slice(VALID_AUTHORIZATION_FACT).unwrap();
    assert_eq!(
        fact.to_canonical_json_vec().unwrap(),
        VALID_AUTHORIZATION_FACT
    );
    assert_eq!(fact.authorization_fact_id().as_str(), "maf_01");
    assert_eq!(fact.session_id().as_str(), "ses_mix");
    assert_eq!(fact.session_epoch(), 9);
    assert_eq!(fact.media_authorization_epoch(), 14);
    assert_eq!(fact.subject_grant_epoch(), 3);
    assert_eq!(fact.media_policy_version(), 7);
    assert_eq!(fact.participant_id().as_str(), "par_producer");
    assert_eq!(fact.endpoint_id().as_str(), "ep_logic");
    assert_eq!(fact.effective_role(), EffectiveRole::Producer);
    assert_eq!(fact.access_expires_at(), Some(1_784_134_800));
    assert_eq!(fact.requested_operation(), Operation::Publish);
    assert_eq!(fact.requested_media_class(), MediaClass::Program);
    assert_eq!(fact.workflow_mode(), SessionWorkflowMode::FinalTake);
    assert_eq!(fact.evaluated_at(), 1_784_131_200);
    assert!(fact.take_id().is_none());

    let identity = SessionMediaIdentityV1::from_json_slice(VALID_IDENTITY).unwrap();
    assert_eq!(identity.to_canonical_json_vec().unwrap(), VALID_IDENTITY);

    let claims = MediaCapabilityClaimsV1::from_json_slice(VALID_CLAIMS).unwrap();
    assert_eq!(claims.to_canonical_json_vec().unwrap(), VALID_CLAIMS);
    assert_eq!(claims.session_epoch(), 9);
    assert_eq!(claims.media_authorization_epoch(), 14);
    assert_eq!(claims.subject_grant_epoch(), 3);
    assert_eq!(claims.media_policy_version(), 7);
    assert_eq!(claims.class_authorization_epoch(), Some(4));
    assert_eq!(claims.binding_generation(), 8);
    assert_eq!(claims.topology_generation(), 52);

    let descriptor = MediaEndpointDescriptorV1::from_json_slice(VALID_DESCRIPTOR).unwrap();
    assert_eq!(
        descriptor.to_canonical_json_vec().unwrap(),
        VALID_DESCRIPTOR
    );

    let configuration =
        MediaFrameConfigurationV1::from_json_slice(VALID_FRAME_CONFIGURATION).unwrap();
    assert_eq!(
        configuration.to_canonical_json_vec().unwrap(),
        VALID_FRAME_CONFIGURATION
    );
    let envelope = MediaFrameEnvelopeV1::from_json_slice(VALID_FRAME_ENVELOPE).unwrap();
    assert_eq!(
        envelope.to_canonical_json_vec().unwrap(),
        VALID_FRAME_ENVELOPE
    );
    assert_eq!(
        envelope.resolve(&configuration).unwrap().media_class(),
        MediaClass::Talkback
    );
}

#[test]
fn workspace_documentation_mirrors_the_core_canonical_fixtures() {
    let docs = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../io.infidelity.docs/specs/media-control/v1/fixtures/valid");
    if !docs.is_dir() {
        return;
    }
    for (name, expected) in [
        (
            "media-authorization-request.json",
            VALID_AUTHORIZATION_REQUEST,
        ),
        ("media-authorization-fact.json", VALID_AUTHORIZATION_FACT),
        ("session-media-identity.json", VALID_IDENTITY),
        ("media-capability-claims.json", VALID_CLAIMS),
        ("media-endpoint-descriptor.json", VALID_DESCRIPTOR),
        ("media-frame-configuration.json", VALID_FRAME_CONFIGURATION),
        ("media-frame-envelope.json", VALID_FRAME_ENVELOPE),
    ] {
        assert_eq!(std::fs::read(docs.join(name)).unwrap(), expected, "{name}");
    }

    let invalid = docs.join("../invalid");
    for (name, expected) in [
        (
            "media-authorization-request-with-relay-key.json",
            AUTHORIZATION_REQUEST_WITH_RELAY_KEY,
        ),
        (
            "media-authorization-fact-with-subject.json",
            AUTHORIZATION_FACT_WITH_SUBJECT,
        ),
        ("media-capability-future-version.json", FUTURE_CLAIMS),
        (
            "media-capability-noncanonical-edge-order.json",
            NONCANONICAL_CLAIMS,
        ),
        ("media-capability-wrong-session.json", WRONG_SESSION_CLAIMS),
        ("media-capability-expired.json", EXPIRED_CLAIMS),
        (
            "session-media-identity-talkback-with-source.json",
            INVALID_TALKBACK_IDENTITY,
        ),
        (
            "media-endpoint-descriptor-with-secret.json",
            DESCRIPTOR_WITH_SECRET,
        ),
    ] {
        assert_eq!(
            std::fs::read(invalid.join(name)).unwrap(),
            expected,
            "{name}"
        );
    }
}

#[test]
fn constructors_canonicalize_sets_but_wire_input_must_already_be_canonical() {
    let mut request = request_params();
    request.requested_source_ids = vec![source("src_z"), source("src_a")];
    let request = MediaAuthorizationRequestV1::new(request).unwrap();
    assert_eq!(
        request.requested_source_ids(),
        &[source("src_a"), source("src_z")]
    );

    let fact = MediaAuthorizationFactV1::new(fact_params()).unwrap();
    assert_eq!(
        fact.to_canonical_json_vec().unwrap(),
        VALID_AUTHORIZATION_FACT
    );
    assert_eq!(
        fact.allowed_operations(),
        &[
            Operation::AcknowledgePlayout,
            Operation::Publish,
            Operation::ReadTake,
            Operation::Subscribe,
            Operation::UploadTake,
        ]
    );
    assert_eq!(
        fact.allowed_media_classes(),
        &[
            MediaClass::Metadata,
            MediaClass::Program,
            MediaClass::Screen,
            MediaClass::Source,
            MediaClass::TakeChunk,
            MediaClass::Talkback,
        ]
    );
    assert_eq!(fact.allowed_source_ids(), &[source("src_mix")]);
    assert!(fact.allowed_audience_ids().is_empty());

    let mut params = claims_params();
    params.source_ids = vec![source("src_z"), source("src_a")];
    let claims = MediaCapabilityClaimsV1::new(params).unwrap();
    assert_eq!(claims.source_ids(), &[source("src_a"), source("src_z")]);

    assert_eq!(
        MediaCapabilityClaimsV1::from_json_slice(NONCANONICAL_CLAIMS)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::NonCanonicalOrder
    );
}

#[test]
fn authorization_wire_sets_must_be_canonical_and_unique() {
    let noncanonical_request = String::from_utf8(VALID_AUTHORIZATION_REQUEST.to_vec())
        .unwrap()
        .replace("[\"src_mix\"]", "[\"src_z\",\"src_a\"]");
    assert_eq!(
        MediaAuthorizationRequestV1::from_json_slice(noncanonical_request.as_bytes())
            .unwrap_err()
            .code(),
        MediaControlErrorCode::NonCanonicalOrder
    );

    let noncanonical_fact = String::from_utf8(VALID_AUTHORIZATION_FACT.to_vec())
        .unwrap()
        .replace(
            "[\"acknowledge_playout\",\"publish\",\"read_take\",\"subscribe\",\"upload_take\"]",
            "[\"publish\",\"acknowledge_playout\",\"read_take\",\"subscribe\",\"upload_take\"]",
        );
    assert_eq!(
        MediaAuthorizationFactV1::from_json_slice(noncanonical_fact.as_bytes())
            .unwrap_err()
            .code(),
        MediaControlErrorCode::NonCanonicalOrder
    );

    let mut duplicate_request = request_params();
    duplicate_request.requested_audience_ids = vec![audience("aud_return"), audience("aud_return")];
    assert_eq!(
        MediaAuthorizationRequestV1::new(duplicate_request)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::DuplicateValue
    );

    let mut duplicate_fact = fact_params();
    duplicate_fact.allowed_operations[0] = Operation::Publish;
    assert_eq!(
        MediaAuthorizationFactV1::new(duplicate_fact)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::DuplicateValue
    );
}

#[test]
fn authorization_objects_are_closed_and_required_nullable_fields_stay_required() {
    assert_eq!(
        MediaAuthorizationRequestV1::from_json_slice(AUTHORIZATION_REQUEST_WITH_RELAY_KEY)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::MalformedJson
    );
    assert_eq!(
        MediaAuthorizationFactV1::from_json_slice(AUTHORIZATION_FACT_WITH_SUBJECT)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::MalformedJson
    );

    let mut request: serde_json::Value =
        serde_json::from_slice(VALID_AUTHORIZATION_REQUEST).unwrap();
    request
        .as_object_mut()
        .unwrap()
        .remove("requested_source_ids");
    assert_eq!(
        MediaAuthorizationRequestV1::from_json_slice(&serde_json::to_vec(&request).unwrap())
            .unwrap_err()
            .code(),
        MediaControlErrorCode::MalformedJson
    );

    let mut request_without_optional_take: serde_json::Value =
        serde_json::from_slice(VALID_AUTHORIZATION_REQUEST).unwrap();
    request_without_optional_take
        .as_object_mut()
        .unwrap()
        .remove("take_id");
    assert_eq!(
        MediaAuthorizationRequestV1::from_json_slice(
            &serde_json::to_vec(&request_without_optional_take).unwrap()
        )
        .unwrap_err()
        .code(),
        MediaControlErrorCode::MalformedJson
    );

    let mut request_with_null_take: serde_json::Value =
        serde_json::from_slice(VALID_AUTHORIZATION_REQUEST).unwrap();
    request_with_null_take["take_id"] = serde_json::Value::Null;
    assert!(MediaAuthorizationRequestV1::from_json_slice(
        &serde_json::to_vec(&request_with_null_take).unwrap()
    )
    .unwrap()
    .take_id()
    .is_none());

    let mut fact: serde_json::Value = serde_json::from_slice(VALID_AUTHORIZATION_FACT).unwrap();
    fact.as_object_mut().unwrap().remove("access_expires_at");
    assert_eq!(
        MediaAuthorizationFactV1::from_json_slice(&serde_json::to_vec(&fact).unwrap())
            .unwrap_err()
            .code(),
        MediaControlErrorCode::MalformedJson
    );

    let mut fact_with_null_expiry: serde_json::Value =
        serde_json::from_slice(VALID_AUTHORIZATION_FACT).unwrap();
    fact_with_null_expiry["access_expires_at"] = serde_json::Value::Null;
    assert!(MediaAuthorizationFactV1::from_json_slice(
        &serde_json::to_vec(&fact_with_null_expiry).unwrap()
    )
    .unwrap()
    .access_expires_at()
    .is_none());
    let mut fact: serde_json::Value = serde_json::from_slice(VALID_AUTHORIZATION_FACT).unwrap();
    fact.as_object_mut().unwrap().remove("take_id");
    assert_eq!(
        MediaAuthorizationFactV1::from_json_slice(&serde_json::to_vec(&fact).unwrap())
            .unwrap_err()
            .code(),
        MediaControlErrorCode::MalformedJson
    );

    let oversized = vec![b' '; 64 * 1024 + 1];
    assert_eq!(
        MediaAuthorizationRequestV1::from_json_slice(&oversized)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::LimitExceeded
    );
}

#[test]
fn optional_identity_and_capability_members_are_absent_only() {
    for field in ["source_id", "audience_id", "take_id"] {
        let mut value: serde_json::Value = serde_json::from_slice(VALID_IDENTITY).unwrap();
        value[field] = serde_json::Value::Null;
        assert_eq!(
            SessionMediaIdentityV1::from_json_slice(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .code(),
            MediaControlErrorCode::MalformedJson,
            "identity field {field} accepted explicit null"
        );
    }

    for field in [
        "class_authorization_epoch",
        "contributor_id",
        "take_id",
        "client_key_thumbprint",
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(VALID_CLAIMS).unwrap();
        value[field] = serde_json::Value::Null;
        assert_eq!(
            MediaCapabilityClaimsV1::from_json_slice(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .code(),
            MediaControlErrorCode::MalformedJson,
            "capability field {field} accepted explicit null"
        );
    }
}

#[test]
fn compact_frames_resolve_only_against_the_exact_authenticated_configuration() {
    let configuration = MediaFrameConfigurationV1::new(frame_configuration_params()).unwrap();
    let envelope = MediaFrameEnvelopeV1::new(frame_envelope_params()).unwrap();
    let identity = envelope.resolve(&configuration).unwrap();
    assert_eq!(identity.session_id().as_str(), "ses_mix");
    assert_eq!(identity.session_epoch(), 9);
    assert_eq!(identity.contributor_id().as_str(), "con_browser");
    assert_eq!(
        identity.audience_id().unwrap().as_str(),
        "aud_producer_return"
    );
    assert_eq!(identity.media_class(), MediaClass::Talkback);

    for mutate in [
        |params: &mut MediaFrameEnvelopeV1Params| params.binding_generation += 1,
        |params: &mut MediaFrameEnvelopeV1Params| params.configuration_ref += 1,
        |params: &mut MediaFrameEnvelopeV1Params| params.configuration_epoch += 1,
    ] {
        let mut params = frame_envelope_params();
        mutate(&mut params);
        assert_eq!(
            MediaFrameEnvelopeV1::new(params)
                .unwrap()
                .resolve(&configuration)
                .unwrap_err()
                .code(),
            MediaControlErrorCode::ConfigurationMismatch
        );
    }

    let mut oversized = frame_envelope_params();
    oversized.payload_bytes = configuration.max_payload_bytes() + 1;
    assert_eq!(
        MediaFrameEnvelopeV1::new(oversized)
            .unwrap()
            .resolve(&configuration)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::ConfigurationMismatch
    );
}

#[test]
fn frame_contract_enforces_capture_and_format_boundaries() {
    let mut recordable_talkback = frame_configuration_params();
    recordable_talkback.capture_disposition = MediaCaptureDisposition::Recordable;
    assert_eq!(
        MediaFrameConfigurationV1::new(recordable_talkback)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut stereo_talkback = frame_configuration_params();
    stereo_talkback.channel_count = 2;
    assert_eq!(
        MediaFrameConfigurationV1::new(stereo_talkback)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut wrong_opus_clock = frame_configuration_params();
    wrong_opus_clock.capture_timebase_hz = 44_100;
    assert_eq!(
        MediaFrameConfigurationV1::new(wrong_opus_clock)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut unsafe_sequence = frame_envelope_params();
    unsafe_sequence.sequence = MEDIA_CONTROL_MAX_GENERATION + 1;
    assert_eq!(
        MediaFrameEnvelopeV1::new(unsafe_sequence)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::LimitExceeded
    );

    let mut unsafe_pts = frame_envelope_params();
    unsafe_pts.capture_pts = -i64::try_from(MEDIA_CONTROL_MAX_GENERATION)
        .expect("the compiled media-control generation bound fits in i64")
        - 1;
    assert_eq!(
        MediaFrameEnvelopeV1::new(unsafe_pts).unwrap_err().code(),
        MediaControlErrorCode::LimitExceeded
    );

    let mut future: serde_json::Value = serde_json::from_slice(VALID_FRAME_ENVELOPE).unwrap();
    future["version"] = serde_json::json!(2);
    assert_eq!(
        MediaFrameEnvelopeV1::from_json_slice(&serde_json::to_vec(&future).unwrap())
            .unwrap_err()
            .code(),
        MediaControlErrorCode::UnsupportedVersion
    );
}

#[test]
fn authorization_requests_enforce_operation_class_and_take_invariants() {
    let mut upload_without_take = request_params();
    upload_without_take.requested_operation = Operation::UploadTake;
    upload_without_take.requested_media_class = MediaClass::TakeChunk;
    assert_eq!(
        MediaAuthorizationRequestV1::new(upload_without_take)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut live_with_take = request_params();
    live_with_take.take_id = Some(take("take_private"));
    assert_eq!(
        MediaAuthorizationRequestV1::new(live_with_take)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut invalid_ack = request_params();
    invalid_ack.requested_operation = Operation::AcknowledgePlayout;
    invalid_ack.requested_media_class = MediaClass::Metadata;
    assert_eq!(
        MediaAuthorizationRequestV1::new(invalid_ack)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut upload = request_params();
    upload.requested_operation = Operation::UploadTake;
    upload.requested_media_class = MediaClass::TakeChunk;
    upload.take_id = Some(take("take_01"));
    assert!(MediaAuthorizationRequestV1::new(upload).is_ok());
}

#[test]
fn authorization_facts_enforce_safe_generations_timestamps_and_semantics() {
    let mut zero_generation = fact_params();
    zero_generation.session_epoch = 0;
    assert_eq!(
        MediaAuthorizationFactV1::new(zero_generation)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidGeneration
    );

    let mut inexact_generation = fact_params();
    inexact_generation.media_policy_version = MEDIA_CONTROL_MAX_GENERATION + 1;
    assert_eq!(
        MediaAuthorizationFactV1::new(inexact_generation)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidGeneration
    );

    let mut negative_timestamp = fact_params();
    negative_timestamp.evaluated_at = -1;
    assert_eq!(
        MediaAuthorizationFactV1::new(negative_timestamp)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidTimestamp
    );

    let mut inexact_timestamp = fact_params();
    inexact_timestamp.access_expires_at = Some(9_007_199_254_740_992);
    assert_eq!(
        MediaAuthorizationFactV1::new(inexact_timestamp)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidTimestamp
    );

    let mut already_expired = fact_params();
    already_expired.access_expires_at = Some(already_expired.evaluated_at);
    assert_eq!(
        MediaAuthorizationFactV1::new(already_expired)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidTimestamp
    );

    let mut missing_allowed_operation = fact_params();
    missing_allowed_operation
        .allowed_operations
        .retain(|operation| *operation != Operation::Publish);
    assert_eq!(
        MediaAuthorizationFactV1::new(missing_allowed_operation)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut take_outside_final_take = fact_params();
    take_outside_final_take.requested_operation = Operation::ReadTake;
    take_outside_final_take.requested_media_class = MediaClass::TakeChunk;
    take_outside_final_take.take_id = Some(take("take_01"));
    take_outside_final_take.workflow_mode = SessionWorkflowMode::MixReview;
    assert_eq!(
        MediaAuthorizationFactV1::new(take_outside_final_take)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );
}

#[test]
fn future_versions_and_unknown_fields_fail_closed() {
    for error in [
        MediaAuthorizationRequestV1::from_json_slice(
            String::from_utf8(VALID_AUTHORIZATION_REQUEST.to_vec())
                .unwrap()
                .replacen("\"version\":1", "\"version\":2", 1)
                .as_bytes(),
        )
        .unwrap_err(),
        MediaAuthorizationFactV1::from_json_slice(
            String::from_utf8(VALID_AUTHORIZATION_FACT.to_vec())
                .unwrap()
                .replacen("\"version\":1", "\"version\":2", 1)
                .as_bytes(),
        )
        .unwrap_err(),
    ] {
        assert_eq!(error.code(), MediaControlErrorCode::UnsupportedVersion);
    }

    assert_eq!(
        MediaCapabilityClaimsV1::from_json_slice(FUTURE_CLAIMS)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::UnsupportedVersion
    );

    let unknown = String::from_utf8(VALID_CLAIMS.to_vec()).unwrap().replacen(
        "\"expires_at\":1784131260}",
        "\"expires_at\":1784131260,\"relay_key\":\"forbidden\"}",
        1,
    );
    assert_eq!(
        MediaCapabilityClaimsV1::from_json_slice(unknown.as_bytes())
            .unwrap_err()
            .code(),
        MediaControlErrorCode::MalformedJson
    );

    let oversized = vec![b' '; 64 * 1024 + 1];
    assert_eq!(
        MediaCapabilityClaimsV1::from_json_slice(&oversized)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::LimitExceeded
    );
}

#[test]
fn opaque_ids_are_bounded_tokens_and_debug_is_redacted() {
    assert_eq!(
        SessionId::new("").unwrap_err().code(),
        MediaControlErrorCode::InvalidIdentifier
    );
    assert_eq!(
        SessionId::new("session/secret").unwrap_err().code(),
        MediaControlErrorCode::InvalidIdentifier
    );
    assert_eq!(
        SessionId::new("s".repeat(129)).unwrap_err().code(),
        MediaControlErrorCode::LimitExceeded
    );
    assert_eq!(
        format!("{:?}", session("ses_private")),
        "SessionId(\"[REDACTED]\")"
    );
}

#[test]
fn media_identity_enforces_class_specific_namespaces() {
    let talkback = SessionMediaIdentityV1::new(SessionMediaIdentityV1Params {
        tenant_id: tenant("ten_wavey"),
        session_id: session("ses_mix"),
        session_epoch: 9,
        participant_id: participant("par_producer"),
        endpoint_id: endpoint("ep_logic"),
        contributor_id: contributor("con_logic"),
        source_id: None,
        media_class: MediaClass::Talkback,
        audience_id: Some(audience("aud_producer_return")),
        take_id: None,
        topology_generation: 52,
    })
    .unwrap();
    assert!(talkback.source_id().is_none());
    assert!(talkback.audience_id().is_some());

    assert_eq!(
        SessionMediaIdentityV1::from_json_slice(INVALID_TALKBACK_IDENTITY)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );
}

#[test]
fn claims_enforce_operation_class_scope_and_lifetime() {
    let mut duplicate = claims_params();
    duplicate.edge_ids.push(edge("edge_ams"));
    assert_eq!(
        MediaCapabilityClaimsV1::new(duplicate).unwrap_err().code(),
        MediaControlErrorCode::DuplicateValue
    );

    let mut talkback = claims_params();
    talkback.media_class = MediaClass::Talkback;
    talkback.source_ids.clear();
    talkback.audience_ids.push(audience("aud_producer_return"));
    assert!(MediaCapabilityClaimsV1::new(talkback).is_ok());

    let mut wrong_take = claims_params();
    wrong_take.operation = Operation::UploadTake;
    assert_eq!(
        MediaCapabilityClaimsV1::new(wrong_take).unwrap_err().code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut invalid_ack = claims_params();
    invalid_ack.operation = Operation::AcknowledgePlayout;
    invalid_ack.media_class = MediaClass::Metadata;
    assert_eq!(
        MediaCapabilityClaimsV1::new(invalid_ack)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );

    let mut long_lived = claims_params();
    long_lived.expires_at = long_lived.issued_at + 91;
    assert_eq!(
        MediaCapabilityClaimsV1::new(long_lived).unwrap_err().code(),
        MediaControlErrorCode::CapabilityLifetimeExceeded
    );

    let mut noncanonical_authority = claims_params();
    noncanonical_authority.audience = "av contrib".to_owned();
    assert_eq!(
        MediaCapabilityClaimsV1::new(noncanonical_authority)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::InvalidCombination
    );
}

#[test]
fn claims_authorize_only_the_exact_current_context() {
    let claims = MediaCapabilityClaimsV1::new(claims_params()).unwrap();
    let (tenant, session_id, participant, endpoint, contributor, source, edge) = validation_ids();
    let context = MediaCapabilityValidationContextV1 {
        expected_issuer: "https://control.infidelity.io",
        expected_audience: "av-contrib",
        tenant_id: &tenant,
        session_id: &session_id,
        session_epoch: 9,
        media_authorization_epoch: 14,
        subject_grant_epoch: 3,
        media_policy_version: 7,
        class_authorization_epoch: Some(4),
        binding_generation: 8,
        topology_generation: 52,
        participant_id: &participant,
        endpoint_id: &endpoint,
        contributor_id: Some(&contributor),
        operation: Operation::Publish,
        media_class: MediaClass::Program,
        source_id: Some(&source),
        audience_id: None,
        take_id: None,
        edge_id: Some(&edge),
        now: 1_784_131_220,
        clock_skew_seconds: 5,
    };
    assert_eq!(claims.authorize(&context), Ok(()));

    let missing_edge = MediaCapabilityValidationContextV1 {
        edge_id: None,
        ..context
    };
    assert_eq!(
        claims.authorize(&missing_edge).unwrap_err().code(),
        MediaControlErrorCode::InvalidCombination
    );

    let wrong_session = session("ses_other");
    let wrong_context = MediaCapabilityValidationContextV1 {
        session_id: &wrong_session,
        ..context
    };
    let error = claims.authorize(&wrong_context).unwrap_err();
    assert_eq!(error.code(), MediaControlErrorCode::AuthorizationMismatch);
    assert_eq!(error.field(), "session_id");

    let expired_context = MediaCapabilityValidationContextV1 {
        now: 1_784_131_265,
        clock_skew_seconds: 5,
        ..context
    };
    assert_eq!(
        claims.authorize(&expired_context).unwrap_err().code(),
        MediaControlErrorCode::Expired
    );
}

#[test]
fn context_dependent_negative_fixtures_have_stable_errors() {
    let (tenant, session_id, participant, endpoint, contributor, source, _) = validation_ids();
    let edge_lon = edge("edge_lon");
    let context = MediaCapabilityValidationContextV1 {
        expected_issuer: "https://control.infidelity.io",
        expected_audience: "av-contrib",
        tenant_id: &tenant,
        session_id: &session_id,
        session_epoch: 9,
        media_authorization_epoch: 14,
        subject_grant_epoch: 3,
        media_policy_version: 7,
        class_authorization_epoch: None,
        binding_generation: 8,
        topology_generation: 52,
        participant_id: &participant,
        endpoint_id: &endpoint,
        contributor_id: Some(&contributor),
        operation: Operation::Publish,
        media_class: MediaClass::Program,
        source_id: Some(&source),
        audience_id: None,
        take_id: None,
        edge_id: Some(&edge_lon),
        now: 1_784_131_220,
        clock_skew_seconds: 5,
    };

    let wrong_session = MediaCapabilityClaimsV1::from_json_slice(WRONG_SESSION_CLAIMS).unwrap();
    let error = wrong_session.authorize(&context).unwrap_err();
    assert_eq!(error.code(), MediaControlErrorCode::AuthorizationMismatch);
    assert_eq!(error.field(), "session_id");

    let expired = MediaCapabilityClaimsV1::from_json_slice(EXPIRED_CLAIMS).unwrap();
    let expired_context = MediaCapabilityValidationContextV1 {
        now: 1_784_131_265,
        ..context
    };
    let error = expired.authorize(&expired_context).unwrap_err();
    assert_eq!(error.code(), MediaControlErrorCode::Expired);
    assert_eq!(error.field(), "expires_at");
}

#[test]
fn endpoint_descriptors_are_closed_non_authorizing_routes() {
    let descriptor = MediaEndpointDescriptorV1::new(MediaEndpointDescriptorV1Params {
        descriptor_id: DescriptorId::new("dsc_lon_primary").unwrap(),
        tenant_id: tenant("ten_wavey"),
        session_id: session("ses_mix"),
        session_epoch: 9,
        endpoint_id: endpoint("ep_logic"),
        edge_id: edge("edge_lon"),
        binding_generation: 8,
        topology_generation: 52,
        transport: MediaEndpointTransport::WebtransportDatagram,
        origin: "https://media-lon.infidelity.io".to_owned(),
        path: "/v1/playback/dsc_lon_primary".to_owned(),
        expires_at: 1_784_131_290,
    })
    .unwrap();
    assert_eq!(descriptor.origin(), "https://media-lon.infidelity.io");
    assert!(!descriptor.path().contains('?'));

    assert_eq!(
        MediaEndpointDescriptorV1::from_json_slice(DESCRIPTOR_WITH_SECRET)
            .unwrap_err()
            .code(),
        MediaControlErrorCode::MalformedJson
    );

    for origin in [
        "http://media.infidelity.io",
        "https://user@media.infidelity.io",
        "https://media.infidelity.io/path",
        "https://media.infidelity.io?token=x",
        "https://média.infidelity.io",
    ] {
        let mut params = MediaEndpointDescriptorV1Params {
            descriptor_id: DescriptorId::new("dsc_test").unwrap(),
            tenant_id: tenant("ten_wavey"),
            session_id: session("ses_mix"),
            session_epoch: 9,
            endpoint_id: endpoint("ep_logic"),
            edge_id: edge("edge_lon"),
            binding_generation: 8,
            topology_generation: 52,
            transport: MediaEndpointTransport::WebtransportDatagram,
            origin: origin.to_owned(),
            path: "/v1/playback/dsc_test".to_owned(),
            expires_at: 1_784_131_290,
        };
        assert_eq!(
            MediaEndpointDescriptorV1::new(params.clone())
                .unwrap_err()
                .code(),
            MediaControlErrorCode::InvalidEndpoint
        );
        params.origin = "https://media.infidelity.io".to_owned();
    }
}

#[test]
fn safe_debug_views_do_not_contain_identity_or_route_values() {
    let request = MediaAuthorizationRequestV1::new(request_params()).unwrap();
    let debug = format!("{request:?}");
    for forbidden in ["sub_zeroth_01", "ep_logic", "src_mix"] {
        assert!(!debug.contains(forbidden));
    }
    assert!(debug.contains("requested_source_count: 1"));

    let fact = MediaAuthorizationFactV1::new(fact_params()).unwrap();
    let debug = format!("{fact:?}");
    for forbidden in [
        "maf_01",
        "ses_mix",
        "par_producer",
        "ep_logic",
        "src_mix",
        "Producer",
        "1784134800",
    ] {
        assert!(!debug.contains(forbidden));
    }
    let fact_json = String::from_utf8(fact.to_canonical_json_vec().unwrap()).unwrap();
    assert!(!fact_json.contains("\"subject\""));

    let claims = MediaCapabilityClaimsV1::new(claims_params()).unwrap();
    let debug = format!("{claims:?}");
    for forbidden in ["ses_mix", "src_mix", "edge_ams", "cap_publish_mix"] {
        assert!(!debug.contains(forbidden));
    }
    let redacted = serde_json::to_string(&claims.redacted()).unwrap();
    assert!(!redacted.contains("ses_mix"));
    assert!(redacted.contains("\"source_count\":1"));
}

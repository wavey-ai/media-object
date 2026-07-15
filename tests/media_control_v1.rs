use media_object::{
    AudienceId, CapabilityId, ContributorId, DescriptorId, EdgeId, EndpointId,
    MediaCapabilityClaimsV1, MediaCapabilityClaimsV1Params, MediaCapabilityValidationContextV1,
    MediaClass, MediaControlErrorCode, MediaEndpointDescriptorV1, MediaEndpointDescriptorV1Params,
    MediaEndpointTransport, Operation, ParticipantId, SessionId, SessionMediaIdentityV1,
    SessionMediaIdentityV1Params, SourceId, TenantId,
};

const VALID_IDENTITY: &[u8] =
    include_bytes!("fixtures/media-control/v1/session-media-identity.json");
const VALID_CLAIMS: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-capability-claims.json");
const VALID_DESCRIPTOR: &[u8] =
    include_bytes!("fixtures/media-control/v1/media-endpoint-descriptor.json");
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
}

#[test]
fn workspace_documentation_mirrors_the_core_canonical_fixtures() {
    let docs = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../io.infidelity.docs/specs/media-control/v1/fixtures/valid");
    if !docs.is_dir() {
        return;
    }
    for (name, expected) in [
        ("session-media-identity.json", VALID_IDENTITY),
        ("media-capability-claims.json", VALID_CLAIMS),
        ("media-endpoint-descriptor.json", VALID_DESCRIPTOR),
    ] {
        assert_eq!(std::fs::read(docs.join(name)).unwrap(), expected, "{name}");
    }

    let invalid = docs.join("../invalid");
    for (name, expected) in [
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
fn future_versions_and_unknown_fields_fail_closed() {
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
    let claims = MediaCapabilityClaimsV1::new(claims_params()).unwrap();
    let debug = format!("{claims:?}");
    for forbidden in ["ses_mix", "src_mix", "edge_ams", "cap_publish_mix"] {
        assert!(!debug.contains(forbidden));
    }
    let redacted = serde_json::to_string(&claims.redacted()).unwrap();
    assert!(!redacted.contains("ses_mix"));
    assert!(redacted.contains("\"source_count\":1"));
}

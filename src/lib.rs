//! Canonical, bounded media-object primitives.
//!
//! This crate owns the immutable object identity and its versioned binary
//! envelope. `RelaySession`, Needletail, `RaptorQ`, storage, and contribution
//! adapters own their respective session, routing, repair, persistence, and
//! protocol responsibilities.

mod control;
mod error;
mod model;
mod wire;

pub use control::{
    AudienceId, AuthorizationFactId, CapabilityId, ContributorId, DescriptorId, EdgeId,
    EffectiveRole, EndpointId, LiveMonitorTransport, MediaAuthorizationFactV1,
    MediaAuthorizationFactV1Params, MediaAuthorizationRequestV1, MediaAuthorizationRequestV1Params,
    MediaCapabilityClaimsV1, MediaCapabilityClaimsV1Params, MediaCapabilityTokenType,
    MediaCapabilityValidationContextV1, MediaCaptureDisposition, MediaClass, MediaConfigurationId,
    MediaControlError, MediaControlErrorCode, MediaControlResult, MediaEndpointDescriptorV1,
    MediaEndpointDescriptorV1Params, MediaEndpointTransport, MediaFrameConfigurationV1,
    MediaFrameConfigurationV1Params, MediaFrameEnvelopeV1, MediaFrameEnvelopeV1Params,
    MediaFramePayloadFormat, Operation, ParticipantId, RedactedMediaAuthorizationFactV1,
    RedactedMediaAuthorizationRequestV1, RedactedMediaCapabilityClaimsV1,
    RedactedMediaEndpointDescriptorV1, RedactedMediaFrameConfigurationV1,
    RedactedMediaFrameEnvelopeV1, RedactedSessionMediaIdentityV1, SessionId,
    SessionMediaIdentityV1, SessionMediaIdentityV1Params, SessionWorkflowMode, SourceId, SubjectId,
    TakeId, TenantId, MEDIA_CONTROL_MAX_CAPABILITY_LIFETIME_SECONDS,
    MEDIA_CONTROL_MAX_CLOCK_SKEW_SECONDS, MEDIA_CONTROL_MAX_GENERATION, MEDIA_CONTROL_MAX_ID_BYTES,
    MEDIA_CONTROL_MAX_JSON_BYTES, MEDIA_CONTROL_MAX_SCOPE_IDS, MEDIA_CONTROL_VERSION_V1,
    MEDIA_FRAME_MAX_PAYLOAD_BYTES, MEDIA_FRAME_MAX_TIMEBASE_HZ,
};
pub use error::{Error, Result};
pub use model::{
    ClockConfidence, ClockConfidenceLevel, ClockTimestamp, Limits, MediaObject, MediaObjectBuilder,
    ObjectKey, ObjectKind, PayloadHash, Stage, StageTimestamp, WriteDisposition,
};
pub use wire::{decode, decode_with_limits, encode, encoded_len, WIRE_MAGIC, WIRE_VERSION};

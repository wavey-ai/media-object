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
    AudienceId, CapabilityId, ContributorId, DescriptorId, EdgeId, EndpointId,
    LiveMonitorTransport, MediaCapabilityClaimsV1, MediaCapabilityClaimsV1Params,
    MediaCapabilityTokenType, MediaCapabilityValidationContextV1, MediaClass, MediaControlError,
    MediaControlErrorCode, MediaControlResult, MediaEndpointDescriptorV1,
    MediaEndpointDescriptorV1Params, MediaEndpointTransport, Operation, ParticipantId,
    RedactedMediaCapabilityClaimsV1, RedactedMediaEndpointDescriptorV1,
    RedactedSessionMediaIdentityV1, SessionId, SessionMediaIdentityV1,
    SessionMediaIdentityV1Params, SessionWorkflowMode, SourceId, TakeId, TenantId,
    MEDIA_CONTROL_MAX_CAPABILITY_LIFETIME_SECONDS, MEDIA_CONTROL_MAX_CLOCK_SKEW_SECONDS,
    MEDIA_CONTROL_MAX_GENERATION, MEDIA_CONTROL_MAX_ID_BYTES, MEDIA_CONTROL_MAX_JSON_BYTES,
    MEDIA_CONTROL_MAX_SCOPE_IDS, MEDIA_CONTROL_VERSION_V1,
};
pub use error::{Error, Result};
pub use model::{
    ClockConfidence, ClockConfidenceLevel, ClockTimestamp, Limits, MediaObject, MediaObjectBuilder,
    ObjectKey, ObjectKind, PayloadHash, Stage, StageTimestamp, WriteDisposition,
};
pub use wire::{decode, decode_with_limits, encode, encoded_len, WIRE_MAGIC, WIRE_VERSION};

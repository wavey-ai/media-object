//! Strict JSON contracts for media-control v1.
//!
//! These types are separate from the immutable `MOBJ` binary envelope. They
//! describe authorization identity, capability claims, non-authorizing
//! endpoint routing data, and the authenticated configuration behind compact
//! high-rate frame references. Deserialization rejects unknown fields and
//! invalid combinations; constructors canonicalize set-like scopes.

use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize};

/// The only media-control major version supported by this crate release.
pub const MEDIA_CONTROL_VERSION_V1: u16 = 1;
/// Maximum encoded length of every opaque media-control identifier.
pub const MEDIA_CONTROL_MAX_ID_BYTES: usize = 128;
/// Maximum complete JSON object accepted by the bounded v1 parsers.
pub const MEDIA_CONTROL_MAX_JSON_BYTES: usize = 64 * 1024;
/// Largest generation that round-trips exactly through JavaScript numbers.
pub const MEDIA_CONTROL_MAX_GENERATION: u64 = 9_007_199_254_740_991;
/// Maximum number of identifiers in one set-like capability scope.
pub const MEDIA_CONTROL_MAX_SCOPE_IDS: usize = 64;
/// Largest payload represented by the logical v1 media-frame envelope.
pub const MEDIA_FRAME_MAX_PAYLOAD_BYTES: u32 = 16 * 1024 * 1024;
/// Largest exact capture timebase admitted by the v1 frame configuration.
pub const MEDIA_FRAME_MAX_TIMEBASE_HZ: u32 = 1_000_000_000;
/// Hard maximum lifetime for any v1 media capability.
pub const MEDIA_CONTROL_MAX_CAPABILITY_LIFETIME_SECONDS: i64 = 90;
/// Hard maximum verifier clock-skew allowance.
pub const MEDIA_CONTROL_MAX_CLOCK_SKEW_SECONDS: i64 = 5;

const MAX_AUTHORITY_BYTES: usize = 512;
const MAX_EXACT_UNIX_SECONDS: i64 = 9_007_199_254_740_991;
const MAX_ORIGIN_BYTES: usize = 2_048;
const MAX_PATH_BYTES: usize = 512;
const MAX_CHANNELS: u16 = 128;
const MAX_BITRATE: u64 = 1_000_000_000;
const MIN_DATAGRAM_BYTES: u32 = 256;
const MAX_DATAGRAM_BYTES: u32 = 65_535;
const REDACTED: &str = "[REDACTED]";

/// Stable machine-readable media-control validation categories.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum MediaControlErrorCode {
    /// JSON is malformed or does not match the closed v1 shape.
    MalformedJson,
    /// The requested major contract version is unsupported.
    UnsupportedVersion,
    /// An opaque identifier is empty or outside the v1 token alphabet/bound.
    InvalidIdentifier,
    /// A numeric value or collection exceeds a fixed contract bound.
    LimitExceeded,
    /// A generation is zero or exceeds the cross-language exact-integer bound.
    InvalidGeneration,
    /// A set-like scope contains the same identifier more than once.
    DuplicateValue,
    /// A set-like scope is not in deterministic ascending order on input.
    NonCanonicalOrder,
    /// Fields are individually valid but form a forbidden semantic combination.
    InvalidCombination,
    /// Capability timestamps are inconsistent or outside the v1 range.
    InvalidTimestamp,
    /// The capability lifetime exceeds the v1 hard maximum.
    CapabilityLifetimeExceeded,
    /// A claim does not match the verifier's authenticated context.
    AuthorizationMismatch,
    /// A compact frame reference does not match its authenticated configuration.
    ConfigurationMismatch,
    /// The capability is not valid yet.
    NotYetValid,
    /// The capability has expired.
    Expired,
    /// An endpoint origin or path is not a safe non-authorizing descriptor.
    InvalidEndpoint,
}

/// A bounded media-control validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaControlError {
    code: MediaControlErrorCode,
    field: &'static str,
    reason: &'static str,
}

impl MediaControlError {
    const fn new(code: MediaControlErrorCode, field: &'static str, reason: &'static str) -> Self {
        Self {
            code,
            field,
            reason,
        }
    }

    /// Return the stable error category suitable for cross-language fixtures.
    #[must_use]
    pub const fn code(&self) -> MediaControlErrorCode {
        self.code
    }

    /// Return the contract field associated with the failure.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Return a bounded, value-free explanation safe for diagnostics.
    #[must_use]
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for MediaControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.reason)
    }
}

impl std::error::Error for MediaControlError {}

/// Result alias for media-control v1 parsing and validation.
pub type MediaControlResult<T> = std::result::Result<T, MediaControlError>;

macro_rules! opaque_id {
    ($name:ident, $field:literal) => {
        #[doc = concat!("A validated opaque `", $field, "` value.")]
        #[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[doc = concat!("Validate and construct an opaque `", $field, "`.")]
            ///
            /// # Errors
            ///
            /// Returns an error for an empty, oversized, or non-token value.
            pub fn new(value: impl Into<String>) -> MediaControlResult<Self> {
                let value = value.into();
                validate_opaque_id($field, &value)?;
                Ok(Self(value))
            }

            #[doc = concat!("Return the exact opaque `", $field, "` value.")]
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&REDACTED)
                    .finish()
            }
        }

        impl FromStr for $name {
            type Err = MediaControlError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

opaque_id!(TenantId, "tenant_id");
opaque_id!(SessionId, "session_id");
opaque_id!(SubjectId, "subject");
opaque_id!(ParticipantId, "participant_id");
opaque_id!(EndpointId, "endpoint_id");
opaque_id!(ContributorId, "contributor_id");
opaque_id!(SourceId, "source_id");
opaque_id!(AudienceId, "audience_id");
opaque_id!(TakeId, "take_id");
opaque_id!(CapabilityId, "capability_id");
opaque_id!(EdgeId, "edge_id");
opaque_id!(DescriptorId, "descriptor_id");
opaque_id!(AuthorizationFactId, "authorization_fact_id");
opaque_id!(MediaConfigurationId, "configuration_id");

/// Product workflow selected for a session.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionWorkflowMode {
    /// A bounded-latency stereo/program review session.
    MixReview,
    /// Multiple named sources rendered against a common playout anchor.
    SynchronizedStems,
    /// Source-local durable capture with eventual exact completion.
    FinalTake,
}

/// Effective identity-policy role carried by an authorization fact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectiveRole {
    /// May consume admitted media and publish only role-specific return media.
    Listener,
    /// May additionally contribute admitted program and source media.
    Contributor,
    /// May administer and contribute every policy-admitted media class.
    Producer,
}

/// Requested or admitted live-monitor representation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveMonitorTransport {
    /// Lossy, short-frame Opus live monitoring.
    Opus,
    /// PCM only when route and receiver admission explicitly allow it.
    PcmIfAdmitted,
    /// Let the controller select from admitted representations.
    Auto,
}

/// Semantic media class; display labels cannot change this value.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaClass {
    Program,
    Source,
    Talkback,
    Screen,
    Metadata,
    TakeChunk,
}

/// Capability operation checked at the first authoritative media boundary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Publish,
    Subscribe,
    UploadTake,
    ReadTake,
    AcknowledgePlayout,
}

/// Carrier selected for one endpoint descriptor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaEndpointTransport {
    WebtransportDatagram,
    NativeDatagram,
    LlHls,
}

/// Fixed token type preventing a different signed object from being substituted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaCapabilityTokenType {
    MediaCapability,
}

/// Inputs for constructing a canonical [`MediaAuthorizationRequestV1`].
#[derive(Clone, Eq, PartialEq)]
pub struct MediaAuthorizationRequestV1Params {
    pub subject: SubjectId,
    pub endpoint_id: EndpointId,
    pub requested_operation: Operation,
    pub requested_media_class: MediaClass,
    pub requested_source_ids: Vec<SourceId>,
    pub requested_audience_ids: Vec<AudienceId>,
    pub take_id: Option<TakeId>,
}

/// Authenticated internal request for an identity-policy authorization fact.
///
/// This object contains a raw identity subject and is therefore deliberately
/// redacted by `Debug`. It is an internal policy input, never a media
/// credential.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct MediaAuthorizationRequestV1 {
    version: u16,
    subject: SubjectId,
    endpoint_id: EndpointId,
    requested_operation: Operation,
    requested_media_class: MediaClass,
    requested_source_ids: Vec<SourceId>,
    requested_audience_ids: Vec<AudienceId>,
    take_id: Option<TakeId>,
}

impl MediaAuthorizationRequestV1 {
    /// Construct a request and canonicalize its set-like scopes.
    ///
    /// # Errors
    ///
    /// Returns an error for duplicate, oversized, or semantically invalid
    /// operation/class/take scopes.
    pub fn new(mut params: MediaAuthorizationRequestV1Params) -> MediaControlResult<Self> {
        canonicalize_set("requested_source_ids", &mut params.requested_source_ids)?;
        canonicalize_set("requested_audience_ids", &mut params.requested_audience_ids)?;
        validate_requested_operation_scope(
            params.requested_operation,
            params.requested_media_class,
            params.take_id.as_ref(),
            None,
        )?;
        Ok(Self {
            version: MEDIA_CONTROL_VERSION_V1,
            subject: params.subject,
            endpoint_id: params.endpoint_id,
            requested_operation: params.requested_operation,
            requested_media_class: params.requested_media_class,
            requested_source_ids: params.requested_source_ids,
            requested_audience_ids: params.requested_audience_ids,
            take_id: params.take_id,
        })
    }

    /// Parse a bounded, closed v1 JSON request.
    ///
    /// # Errors
    ///
    /// Returns a stable validation error for malformed, non-canonical, or
    /// semantically invalid input.
    pub fn from_json_slice(input: &[u8]) -> MediaControlResult<Self> {
        require_v1(input)?;
        let wire: MediaAuthorizationRequestV1Wire =
            serde_json::from_slice(input).map_err(malformed_json)?;
        validate_wire_canonical_set("requested_source_ids", &wire.requested_source_ids)?;
        validate_wire_canonical_set("requested_audience_ids", &wire.requested_audience_ids)?;
        Self::try_from(wire)
    }

    /// Encode the deterministic compact JSON fixture representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if the JSON serializer fails unexpectedly.
    pub fn to_canonical_json_vec(&self) -> MediaControlResult<Vec<u8>> {
        canonical_json(self)
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    /// Return the authenticated identity subject.
    ///
    /// Calling this crosses the request's privacy boundary; routine
    /// diagnostics should use [`Self::redacted`] instead.
    #[must_use]
    pub const fn subject(&self) -> &SubjectId {
        &self.subject
    }

    #[must_use]
    pub const fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    #[must_use]
    pub const fn requested_operation(&self) -> Operation {
        self.requested_operation
    }

    #[must_use]
    pub const fn requested_media_class(&self) -> MediaClass {
        self.requested_media_class
    }

    #[must_use]
    pub fn requested_source_ids(&self) -> &[SourceId] {
        &self.requested_source_ids
    }

    #[must_use]
    pub fn requested_audience_ids(&self) -> &[AudienceId] {
        &self.requested_audience_ids
    }

    #[must_use]
    pub const fn take_id(&self) -> Option<&TakeId> {
        self.take_id.as_ref()
    }

    /// Build the intentionally lossy representation permitted in diagnostics.
    #[must_use]
    pub fn redacted(&self) -> RedactedMediaAuthorizationRequestV1 {
        RedactedMediaAuthorizationRequestV1 {
            version: self.version,
            requested_operation: self.requested_operation,
            requested_media_class: self.requested_media_class,
            requested_source_count: self.requested_source_ids.len(),
            requested_audience_count: self.requested_audience_ids.len(),
            has_take: self.take_id.is_some(),
        }
    }
}

impl fmt::Debug for MediaAuthorizationRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Safe diagnostic projection of [`MediaAuthorizationRequestV1`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedMediaAuthorizationRequestV1 {
    pub version: u16,
    pub requested_operation: Operation,
    pub requested_media_class: MediaClass,
    pub requested_source_count: usize,
    pub requested_audience_count: usize,
    pub has_take: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaAuthorizationRequestV1Wire {
    version: u16,
    subject: SubjectId,
    endpoint_id: EndpointId,
    requested_operation: Operation,
    requested_media_class: MediaClass,
    requested_source_ids: Vec<SourceId>,
    requested_audience_ids: Vec<AudienceId>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    take_id: Option<TakeId>,
}

impl<'de> Deserialize<'de> for MediaAuthorizationRequestV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = MediaAuthorizationRequestV1Wire::deserialize(deserializer)?;
        validate_wire_canonical_set("requested_source_ids", &wire.requested_source_ids)
            .map_err(de::Error::custom)?;
        validate_wire_canonical_set("requested_audience_ids", &wire.requested_audience_ids)
            .map_err(de::Error::custom)?;
        Self::try_from(wire).map_err(de::Error::custom)
    }
}

impl TryFrom<MediaAuthorizationRequestV1Wire> for MediaAuthorizationRequestV1 {
    type Error = MediaControlError;

    fn try_from(wire: MediaAuthorizationRequestV1Wire) -> Result<Self, Self::Error> {
        if wire.version != MEDIA_CONTROL_VERSION_V1 {
            return Err(unsupported_version());
        }
        Self::new(MediaAuthorizationRequestV1Params {
            subject: wire.subject,
            endpoint_id: wire.endpoint_id,
            requested_operation: wire.requested_operation,
            requested_media_class: wire.requested_media_class,
            requested_source_ids: wire.requested_source_ids,
            requested_audience_ids: wire.requested_audience_ids,
            take_id: wire.take_id,
        })
    }
}

/// Inputs for constructing a canonical [`MediaAuthorizationFactV1`].
#[derive(Clone, Eq, PartialEq)]
pub struct MediaAuthorizationFactV1Params {
    pub authorization_fact_id: AuthorizationFactId,
    pub session_id: SessionId,
    pub session_epoch: u64,
    pub media_authorization_epoch: u64,
    pub subject_grant_epoch: u64,
    pub media_policy_version: u64,
    pub participant_id: ParticipantId,
    pub endpoint_id: EndpointId,
    pub effective_role: EffectiveRole,
    pub access_expires_at: Option<i64>,
    pub allowed_operations: Vec<Operation>,
    pub allowed_media_classes: Vec<MediaClass>,
    pub allowed_source_ids: Vec<SourceId>,
    pub allowed_audience_ids: Vec<AudienceId>,
    pub requested_operation: Operation,
    pub requested_media_class: MediaClass,
    pub take_id: Option<TakeId>,
    pub workflow_mode: SessionWorkflowMode,
    pub evaluated_at: i64,
}

/// Current identity-policy result returned to an authenticated media controller.
///
/// The fact intentionally has no raw identity subject and is not itself a
/// media credential. All identifiers, private role/scope details, and expiry
/// values are omitted from its `Debug` representation.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct MediaAuthorizationFactV1 {
    version: u16,
    authorization_fact_id: AuthorizationFactId,
    session_id: SessionId,
    session_epoch: u64,
    media_authorization_epoch: u64,
    subject_grant_epoch: u64,
    media_policy_version: u64,
    participant_id: ParticipantId,
    endpoint_id: EndpointId,
    effective_role: EffectiveRole,
    access_expires_at: Option<i64>,
    allowed_operations: Vec<Operation>,
    allowed_media_classes: Vec<MediaClass>,
    allowed_source_ids: Vec<SourceId>,
    allowed_audience_ids: Vec<AudienceId>,
    requested_operation: Operation,
    requested_media_class: MediaClass,
    take_id: Option<TakeId>,
    workflow_mode: SessionWorkflowMode,
    evaluated_at: i64,
}

impl MediaAuthorizationFactV1 {
    /// Construct a fact and canonicalize all set-like scopes.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid generations, timestamps, bounds, duplicate
    /// values, or operation/class/take/workflow combinations.
    pub fn new(mut params: MediaAuthorizationFactV1Params) -> MediaControlResult<Self> {
        for (field, generation) in [
            ("session_epoch", params.session_epoch),
            (
                "media_authorization_epoch",
                params.media_authorization_epoch,
            ),
            ("subject_grant_epoch", params.subject_grant_epoch),
            ("media_policy_version", params.media_policy_version),
        ] {
            validate_generation(field, generation)?;
        }
        validate_unix_seconds("evaluated_at", params.evaluated_at)?;
        if let Some(access_expires_at) = params.access_expires_at {
            validate_unix_seconds("access_expires_at", access_expires_at)?;
            if access_expires_at <= params.evaluated_at {
                return Err(MediaControlError::new(
                    MediaControlErrorCode::InvalidTimestamp,
                    "access_expires_at",
                    "access expiry must be later than authorization evaluation",
                ));
            }
        }
        canonicalize_operation_set(&mut params.allowed_operations)?;
        canonicalize_media_class_set(&mut params.allowed_media_classes)?;
        canonicalize_set("allowed_source_ids", &mut params.allowed_source_ids)?;
        canonicalize_set("allowed_audience_ids", &mut params.allowed_audience_ids)?;
        validate_requested_operation_scope(
            params.requested_operation,
            params.requested_media_class,
            params.take_id.as_ref(),
            Some(params.workflow_mode),
        )?;
        if !params
            .allowed_operations
            .contains(&params.requested_operation)
        {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidCombination,
                "allowed_operations",
                "requested operation is absent from the allowed operation set",
            ));
        }
        if !params
            .allowed_media_classes
            .contains(&params.requested_media_class)
        {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidCombination,
                "allowed_media_classes",
                "requested media class is absent from the allowed class set",
            ));
        }

        Ok(Self {
            version: MEDIA_CONTROL_VERSION_V1,
            authorization_fact_id: params.authorization_fact_id,
            session_id: params.session_id,
            session_epoch: params.session_epoch,
            media_authorization_epoch: params.media_authorization_epoch,
            subject_grant_epoch: params.subject_grant_epoch,
            media_policy_version: params.media_policy_version,
            participant_id: params.participant_id,
            endpoint_id: params.endpoint_id,
            effective_role: params.effective_role,
            access_expires_at: params.access_expires_at,
            allowed_operations: params.allowed_operations,
            allowed_media_classes: params.allowed_media_classes,
            allowed_source_ids: params.allowed_source_ids,
            allowed_audience_ids: params.allowed_audience_ids,
            requested_operation: params.requested_operation,
            requested_media_class: params.requested_media_class,
            take_id: params.take_id,
            workflow_mode: params.workflow_mode,
            evaluated_at: params.evaluated_at,
        })
    }

    /// Parse a bounded, closed v1 JSON fact.
    ///
    /// # Errors
    ///
    /// Returns a stable validation error for malformed, non-canonical, or
    /// semantically invalid input.
    pub fn from_json_slice(input: &[u8]) -> MediaControlResult<Self> {
        require_v1(input)?;
        let wire: MediaAuthorizationFactV1Wire =
            serde_json::from_slice(input).map_err(malformed_json)?;
        validate_fact_wire_sets(&wire)?;
        Self::try_from(wire)
    }

    /// Encode the deterministic compact JSON fixture representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if the JSON serializer fails unexpectedly.
    pub fn to_canonical_json_vec(&self) -> MediaControlResult<Vec<u8>> {
        canonical_json(self)
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn authorization_fact_id(&self) -> &AuthorizationFactId {
        &self.authorization_fact_id
    }

    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    #[must_use]
    pub const fn session_epoch(&self) -> u64 {
        self.session_epoch
    }

    #[must_use]
    pub const fn media_authorization_epoch(&self) -> u64 {
        self.media_authorization_epoch
    }

    #[must_use]
    pub const fn subject_grant_epoch(&self) -> u64 {
        self.subject_grant_epoch
    }

    #[must_use]
    pub const fn media_policy_version(&self) -> u64 {
        self.media_policy_version
    }

    #[must_use]
    pub const fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    #[must_use]
    pub const fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    #[must_use]
    pub const fn effective_role(&self) -> EffectiveRole {
        self.effective_role
    }

    #[must_use]
    pub const fn access_expires_at(&self) -> Option<i64> {
        self.access_expires_at
    }

    #[must_use]
    pub fn allowed_operations(&self) -> &[Operation] {
        &self.allowed_operations
    }

    #[must_use]
    pub fn allowed_media_classes(&self) -> &[MediaClass] {
        &self.allowed_media_classes
    }

    #[must_use]
    pub fn allowed_source_ids(&self) -> &[SourceId] {
        &self.allowed_source_ids
    }

    #[must_use]
    pub fn allowed_audience_ids(&self) -> &[AudienceId] {
        &self.allowed_audience_ids
    }

    #[must_use]
    pub const fn requested_operation(&self) -> Operation {
        self.requested_operation
    }

    #[must_use]
    pub const fn requested_media_class(&self) -> MediaClass {
        self.requested_media_class
    }

    #[must_use]
    pub const fn take_id(&self) -> Option<&TakeId> {
        self.take_id.as_ref()
    }

    #[must_use]
    pub const fn workflow_mode(&self) -> SessionWorkflowMode {
        self.workflow_mode
    }

    #[must_use]
    pub const fn evaluated_at(&self) -> i64 {
        self.evaluated_at
    }

    /// Build the intentionally lossy representation permitted in diagnostics.
    #[must_use]
    pub fn redacted(&self) -> RedactedMediaAuthorizationFactV1 {
        RedactedMediaAuthorizationFactV1 {
            version: self.version,
            requested_operation: self.requested_operation,
            requested_media_class: self.requested_media_class,
            workflow_mode: self.workflow_mode,
            allowed_operation_count: self.allowed_operations.len(),
            allowed_media_class_count: self.allowed_media_classes.len(),
            allowed_source_count: self.allowed_source_ids.len(),
            allowed_audience_count: self.allowed_audience_ids.len(),
            has_access_expiry: self.access_expires_at.is_some(),
            has_take: self.take_id.is_some(),
        }
    }
}

impl fmt::Debug for MediaAuthorizationFactV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Safe diagnostic projection of [`MediaAuthorizationFactV1`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedMediaAuthorizationFactV1 {
    pub version: u16,
    pub requested_operation: Operation,
    pub requested_media_class: MediaClass,
    pub workflow_mode: SessionWorkflowMode,
    pub allowed_operation_count: usize,
    pub allowed_media_class_count: usize,
    pub allowed_source_count: usize,
    pub allowed_audience_count: usize,
    pub has_access_expiry: bool,
    pub has_take: bool,
}

fn deserialize_required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

fn deserialize_absent_or_value<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaAuthorizationFactV1Wire {
    version: u16,
    authorization_fact_id: AuthorizationFactId,
    session_id: SessionId,
    session_epoch: u64,
    media_authorization_epoch: u64,
    subject_grant_epoch: u64,
    media_policy_version: u64,
    participant_id: ParticipantId,
    endpoint_id: EndpointId,
    effective_role: EffectiveRole,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    access_expires_at: Option<i64>,
    allowed_operations: Vec<Operation>,
    allowed_media_classes: Vec<MediaClass>,
    allowed_source_ids: Vec<SourceId>,
    allowed_audience_ids: Vec<AudienceId>,
    requested_operation: Operation,
    requested_media_class: MediaClass,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    take_id: Option<TakeId>,
    workflow_mode: SessionWorkflowMode,
    evaluated_at: i64,
}

impl<'de> Deserialize<'de> for MediaAuthorizationFactV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = MediaAuthorizationFactV1Wire::deserialize(deserializer)?;
        validate_fact_wire_sets(&wire).map_err(de::Error::custom)?;
        Self::try_from(wire).map_err(de::Error::custom)
    }
}

impl TryFrom<MediaAuthorizationFactV1Wire> for MediaAuthorizationFactV1 {
    type Error = MediaControlError;

    fn try_from(wire: MediaAuthorizationFactV1Wire) -> Result<Self, Self::Error> {
        if wire.version != MEDIA_CONTROL_VERSION_V1 {
            return Err(unsupported_version());
        }
        Self::new(MediaAuthorizationFactV1Params {
            authorization_fact_id: wire.authorization_fact_id,
            session_id: wire.session_id,
            session_epoch: wire.session_epoch,
            media_authorization_epoch: wire.media_authorization_epoch,
            subject_grant_epoch: wire.subject_grant_epoch,
            media_policy_version: wire.media_policy_version,
            participant_id: wire.participant_id,
            endpoint_id: wire.endpoint_id,
            effective_role: wire.effective_role,
            access_expires_at: wire.access_expires_at,
            allowed_operations: wire.allowed_operations,
            allowed_media_classes: wire.allowed_media_classes,
            allowed_source_ids: wire.allowed_source_ids,
            allowed_audience_ids: wire.allowed_audience_ids,
            requested_operation: wire.requested_operation,
            requested_media_class: wire.requested_media_class,
            take_id: wire.take_id,
            workflow_mode: wire.workflow_mode,
            evaluated_at: wire.evaluated_at,
        })
    }
}

/// Inputs for constructing a canonical [`SessionMediaIdentityV1`].
#[derive(Clone, Eq, PartialEq)]
pub struct SessionMediaIdentityV1Params {
    pub tenant_id: TenantId,
    pub session_id: SessionId,
    pub session_epoch: u64,
    pub participant_id: ParticipantId,
    pub endpoint_id: EndpointId,
    pub contributor_id: ContributorId,
    pub source_id: Option<SourceId>,
    pub media_class: MediaClass,
    pub audience_id: Option<AudienceId>,
    pub take_id: Option<TakeId>,
    pub topology_generation: u64,
}

/// Stable, namespaced identity for one media publication or object lane.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct SessionMediaIdentityV1 {
    version: u16,
    tenant_id: TenantId,
    session_id: SessionId,
    session_epoch: u64,
    participant_id: ParticipantId,
    endpoint_id: EndpointId,
    contributor_id: ContributorId,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_id: Option<SourceId>,
    media_class: MediaClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    audience_id: Option<AudienceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    take_id: Option<TakeId>,
    topology_generation: u64,
}

impl SessionMediaIdentityV1 {
    /// Construct and validate a v1 media identity.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid generations or class/scope combinations.
    pub fn new(params: SessionMediaIdentityV1Params) -> MediaControlResult<Self> {
        validate_generation("session_epoch", params.session_epoch)?;
        validate_generation("topology_generation", params.topology_generation)?;
        validate_identity_scope(
            params.media_class,
            params.source_id.as_ref(),
            params.audience_id.as_ref(),
            params.take_id.as_ref(),
        )?;
        Ok(Self {
            version: MEDIA_CONTROL_VERSION_V1,
            tenant_id: params.tenant_id,
            session_id: params.session_id,
            session_epoch: params.session_epoch,
            participant_id: params.participant_id,
            endpoint_id: params.endpoint_id,
            contributor_id: params.contributor_id,
            source_id: params.source_id,
            media_class: params.media_class,
            audience_id: params.audience_id,
            take_id: params.take_id,
            topology_generation: params.topology_generation,
        })
    }

    /// Parse a closed v1 JSON object, rejecting a future version first.
    ///
    /// # Errors
    ///
    /// Returns a stable validation error for malformed or invalid input.
    pub fn from_json_slice(input: &[u8]) -> MediaControlResult<Self> {
        require_v1(input)?;
        let wire: SessionMediaIdentityV1Wire =
            serde_json::from_slice(input).map_err(malformed_json)?;
        Self::try_from(wire)
    }

    /// Encode the deterministic compact JSON fixture representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if the JSON serializer fails unexpectedly.
    pub fn to_canonical_json_vec(&self) -> MediaControlResult<Vec<u8>> {
        canonical_json(self)
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    #[must_use]
    pub const fn session_epoch(&self) -> u64 {
        self.session_epoch
    }

    #[must_use]
    pub const fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    #[must_use]
    pub const fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    #[must_use]
    pub const fn contributor_id(&self) -> &ContributorId {
        &self.contributor_id
    }

    #[must_use]
    pub const fn source_id(&self) -> Option<&SourceId> {
        self.source_id.as_ref()
    }

    #[must_use]
    pub const fn media_class(&self) -> MediaClass {
        self.media_class
    }

    #[must_use]
    pub const fn audience_id(&self) -> Option<&AudienceId> {
        self.audience_id.as_ref()
    }

    #[must_use]
    pub const fn take_id(&self) -> Option<&TakeId> {
        self.take_id.as_ref()
    }

    #[must_use]
    pub const fn topology_generation(&self) -> u64 {
        self.topology_generation
    }

    /// Build the intentionally lossy representation permitted in diagnostics.
    #[must_use]
    pub const fn redacted(&self) -> RedactedSessionMediaIdentityV1 {
        RedactedSessionMediaIdentityV1 {
            version: self.version,
            media_class: self.media_class,
            has_source: self.source_id.is_some(),
            has_audience: self.audience_id.is_some(),
            has_take: self.take_id.is_some(),
        }
    }
}

impl fmt::Debug for SessionMediaIdentityV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Safe diagnostic projection of [`SessionMediaIdentityV1`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedSessionMediaIdentityV1 {
    pub version: u16,
    pub media_class: MediaClass,
    pub has_source: bool,
    pub has_audience: bool,
    pub has_take: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionMediaIdentityV1Wire {
    version: u16,
    tenant_id: TenantId,
    session_id: SessionId,
    session_epoch: u64,
    participant_id: ParticipantId,
    endpoint_id: EndpointId,
    contributor_id: ContributorId,
    #[serde(default, deserialize_with = "deserialize_absent_or_value")]
    source_id: Option<SourceId>,
    media_class: MediaClass,
    #[serde(default, deserialize_with = "deserialize_absent_or_value")]
    audience_id: Option<AudienceId>,
    #[serde(default, deserialize_with = "deserialize_absent_or_value")]
    take_id: Option<TakeId>,
    topology_generation: u64,
}

impl<'de> Deserialize<'de> for SessionMediaIdentityV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SessionMediaIdentityV1Wire::deserialize(deserializer)?;
        Self::try_from(wire).map_err(de::Error::custom)
    }
}

impl TryFrom<SessionMediaIdentityV1Wire> for SessionMediaIdentityV1 {
    type Error = MediaControlError;

    fn try_from(wire: SessionMediaIdentityV1Wire) -> Result<Self, Self::Error> {
        if wire.version != MEDIA_CONTROL_VERSION_V1 {
            return Err(unsupported_version());
        }
        Self::new(SessionMediaIdentityV1Params {
            tenant_id: wire.tenant_id,
            session_id: wire.session_id,
            session_epoch: wire.session_epoch,
            participant_id: wire.participant_id,
            endpoint_id: wire.endpoint_id,
            contributor_id: wire.contributor_id,
            source_id: wire.source_id,
            media_class: wire.media_class,
            audience_id: wire.audience_id,
            take_id: wire.take_id,
            topology_generation: wire.topology_generation,
        })
    }
}

/// Payload representation selected by an authenticated frame configuration.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaFramePayloadFormat {
    /// Low-delay Opus packet.
    Opus,
    /// Canonical packed signed 24-bit little-endian PCM.
    PcmS24le,
    /// A complete FLAC frame or bounded FLAC transport unit.
    Flac,
    /// UTF-8 JSON metadata bytes governed by a separate closed schema.
    Json,
    /// A format defined only by the authenticated configuration identifier.
    Opaque,
}

/// Whether a configured lane may contribute bytes to a recordable output.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaCaptureDisposition {
    /// The lane is eligible for an explicitly authorized recording workflow.
    Recordable,
    /// The lane is live-monitor-only and must never enter captured artifacts.
    MonitorOnly,
}

/// Inputs for an authenticated [`MediaFrameConfigurationV1`].
#[derive(Clone, Eq, PartialEq)]
pub struct MediaFrameConfigurationV1Params {
    pub configuration_id: MediaConfigurationId,
    pub binding_generation: u64,
    pub configuration_ref: u32,
    pub configuration_epoch: u64,
    pub identity: SessionMediaIdentityV1,
    pub payload_format: MediaFramePayloadFormat,
    pub capture_timebase_hz: u32,
    pub channel_count: u16,
    pub max_payload_bytes: u32,
    pub capture_disposition: MediaCaptureDisposition,
}

/// Authenticated mapping from a compact frame reference to complete media identity.
///
/// High-rate frames carry `binding_generation`, `configuration_ref`, and
/// `configuration_epoch` rather than repeating opaque identity strings. The
/// configuration is installed only through an authenticated control boundary;
/// resolving an envelope against any other mapping fails closed.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct MediaFrameConfigurationV1 {
    version: u16,
    configuration_id: MediaConfigurationId,
    binding_generation: u64,
    configuration_ref: u32,
    configuration_epoch: u64,
    identity: SessionMediaIdentityV1,
    payload_format: MediaFramePayloadFormat,
    capture_timebase_hz: u32,
    channel_count: u16,
    max_payload_bytes: u32,
    capture_disposition: MediaCaptureDisposition,
}

impl MediaFrameConfigurationV1 {
    /// Construct and validate a compact frame-reference mapping.
    ///
    /// # Errors
    ///
    /// Returns an error for unsafe numeric bounds or a media-class/format/
    /// recording combination that would weaken the canonical identity.
    pub fn new(params: MediaFrameConfigurationV1Params) -> MediaControlResult<Self> {
        validate_generation("binding_generation", params.binding_generation)?;
        validate_generation("configuration_epoch", params.configuration_epoch)?;
        if params.configuration_ref == 0 {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidGeneration,
                "configuration_ref",
                "configuration reference must be nonzero",
            ));
        }
        if params.capture_timebase_hz == 0
            || params.capture_timebase_hz > MEDIA_FRAME_MAX_TIMEBASE_HZ
        {
            return Err(MediaControlError::new(
                MediaControlErrorCode::LimitExceeded,
                "capture_timebase_hz",
                "capture timebase must be between one and 1000000000",
            ));
        }
        if params.channel_count == 0 || params.channel_count > MAX_CHANNELS {
            return Err(MediaControlError::new(
                MediaControlErrorCode::LimitExceeded,
                "channel_count",
                "channel count must be between one and 128",
            ));
        }
        if params.max_payload_bytes == 0 || params.max_payload_bytes > MEDIA_FRAME_MAX_PAYLOAD_BYTES
        {
            return Err(MediaControlError::new(
                MediaControlErrorCode::LimitExceeded,
                "max_payload_bytes",
                "frame payload bound must be between one and 16777216 bytes",
            ));
        }
        validate_frame_configuration_semantics(
            params.identity.media_class(),
            params.payload_format,
            params.capture_timebase_hz,
            params.channel_count,
            params.capture_disposition,
        )?;
        Ok(Self {
            version: MEDIA_CONTROL_VERSION_V1,
            configuration_id: params.configuration_id,
            binding_generation: params.binding_generation,
            configuration_ref: params.configuration_ref,
            configuration_epoch: params.configuration_epoch,
            identity: params.identity,
            payload_format: params.payload_format,
            capture_timebase_hz: params.capture_timebase_hz,
            channel_count: params.channel_count,
            max_payload_bytes: params.max_payload_bytes,
            capture_disposition: params.capture_disposition,
        })
    }

    /// Parse a bounded, closed v1 JSON configuration.
    ///
    /// # Errors
    ///
    /// Returns a stable validation error for malformed or invalid input.
    pub fn from_json_slice(input: &[u8]) -> MediaControlResult<Self> {
        require_v1(input)?;
        let wire: MediaFrameConfigurationV1Wire =
            serde_json::from_slice(input).map_err(malformed_json)?;
        Self::try_from(wire)
    }

    /// Encode the deterministic compact JSON representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if serialization fails unexpectedly.
    pub fn to_canonical_json_vec(&self) -> MediaControlResult<Vec<u8>> {
        canonical_json(self)
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn configuration_id(&self) -> &MediaConfigurationId {
        &self.configuration_id
    }

    #[must_use]
    pub const fn binding_generation(&self) -> u64 {
        self.binding_generation
    }

    #[must_use]
    pub const fn configuration_ref(&self) -> u32 {
        self.configuration_ref
    }

    #[must_use]
    pub const fn configuration_epoch(&self) -> u64 {
        self.configuration_epoch
    }

    #[must_use]
    pub const fn identity(&self) -> &SessionMediaIdentityV1 {
        &self.identity
    }

    #[must_use]
    pub const fn payload_format(&self) -> MediaFramePayloadFormat {
        self.payload_format
    }

    #[must_use]
    pub const fn capture_timebase_hz(&self) -> u32 {
        self.capture_timebase_hz
    }

    #[must_use]
    pub const fn channel_count(&self) -> u16 {
        self.channel_count
    }

    #[must_use]
    pub const fn max_payload_bytes(&self) -> u32 {
        self.max_payload_bytes
    }

    #[must_use]
    pub const fn capture_disposition(&self) -> MediaCaptureDisposition {
        self.capture_disposition
    }

    #[must_use]
    pub const fn redacted(&self) -> RedactedMediaFrameConfigurationV1 {
        RedactedMediaFrameConfigurationV1 {
            version: self.version,
            media_class: self.identity.media_class(),
            payload_format: self.payload_format,
            channel_count: self.channel_count,
            max_payload_bytes: self.max_payload_bytes,
            capture_disposition: self.capture_disposition,
        }
    }
}

impl fmt::Debug for MediaFrameConfigurationV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Value-free diagnostic projection of a media-frame configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedMediaFrameConfigurationV1 {
    pub version: u16,
    pub media_class: MediaClass,
    pub payload_format: MediaFramePayloadFormat,
    pub channel_count: u16,
    pub max_payload_bytes: u32,
    pub capture_disposition: MediaCaptureDisposition,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaFrameConfigurationV1Wire {
    version: u16,
    configuration_id: MediaConfigurationId,
    binding_generation: u64,
    configuration_ref: u32,
    configuration_epoch: u64,
    identity: SessionMediaIdentityV1,
    payload_format: MediaFramePayloadFormat,
    capture_timebase_hz: u32,
    channel_count: u16,
    max_payload_bytes: u32,
    capture_disposition: MediaCaptureDisposition,
}

impl<'de> Deserialize<'de> for MediaFrameConfigurationV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from(MediaFrameConfigurationV1Wire::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

impl TryFrom<MediaFrameConfigurationV1Wire> for MediaFrameConfigurationV1 {
    type Error = MediaControlError;

    fn try_from(wire: MediaFrameConfigurationV1Wire) -> Result<Self, Self::Error> {
        if wire.version != MEDIA_CONTROL_VERSION_V1 {
            return Err(unsupported_version());
        }
        Self::new(MediaFrameConfigurationV1Params {
            configuration_id: wire.configuration_id,
            binding_generation: wire.binding_generation,
            configuration_ref: wire.configuration_ref,
            configuration_epoch: wire.configuration_epoch,
            identity: wire.identity,
            payload_format: wire.payload_format,
            capture_timebase_hz: wire.capture_timebase_hz,
            channel_count: wire.channel_count,
            max_payload_bytes: wire.max_payload_bytes,
            capture_disposition: wire.capture_disposition,
        })
    }
}

/// Inputs for a compact [`MediaFrameEnvelopeV1`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaFrameEnvelopeV1Params {
    pub binding_generation: u64,
    pub configuration_ref: u32,
    pub configuration_epoch: u64,
    pub sequence: u64,
    pub capture_pts: i64,
    pub duration_ticks: u32,
    pub payload_bytes: u32,
}

/// Compact logical envelope carried by every high-rate media frame.
///
/// The envelope is intentionally free of repeated opaque strings. Its three
/// configuration coordinates must resolve against an authenticated
/// [`MediaFrameConfigurationV1`], which supplies the exact session epoch,
/// contributor/source or publisher/audience identity, media class, format, and
/// capture timebase.
#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
pub struct MediaFrameEnvelopeV1 {
    version: u16,
    binding_generation: u64,
    configuration_ref: u32,
    configuration_epoch: u64,
    sequence: u64,
    capture_pts: i64,
    duration_ticks: u32,
    payload_bytes: u32,
}

impl MediaFrameEnvelopeV1 {
    /// Construct a compact logical frame envelope.
    ///
    /// # Errors
    ///
    /// Returns an error for zero/out-of-range generations, sequence, PTS,
    /// duration, or payload length.
    pub fn new(params: MediaFrameEnvelopeV1Params) -> MediaControlResult<Self> {
        validate_generation("binding_generation", params.binding_generation)?;
        validate_generation("configuration_epoch", params.configuration_epoch)?;
        if params.configuration_ref == 0 {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidGeneration,
                "configuration_ref",
                "configuration reference must be nonzero",
            ));
        }
        if params.sequence > MEDIA_CONTROL_MAX_GENERATION {
            return Err(MediaControlError::new(
                MediaControlErrorCode::LimitExceeded,
                "sequence",
                "sequence must be an exact nonnegative JavaScript integer",
            ));
        }
        validate_signed_exact_integer("capture_pts", params.capture_pts)?;
        if params.duration_ticks == 0 {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidCombination,
                "duration_ticks",
                "frame duration must be nonzero",
            ));
        }
        if params.payload_bytes == 0 || params.payload_bytes > MEDIA_FRAME_MAX_PAYLOAD_BYTES {
            return Err(MediaControlError::new(
                MediaControlErrorCode::LimitExceeded,
                "payload_bytes",
                "frame payload length must be between one and 16777216 bytes",
            ));
        }
        Ok(Self {
            version: MEDIA_CONTROL_VERSION_V1,
            binding_generation: params.binding_generation,
            configuration_ref: params.configuration_ref,
            configuration_epoch: params.configuration_epoch,
            sequence: params.sequence,
            capture_pts: params.capture_pts,
            duration_ticks: params.duration_ticks,
            payload_bytes: params.payload_bytes,
        })
    }

    /// Parse a bounded, closed v1 JSON envelope.
    ///
    /// # Errors
    ///
    /// Returns a stable validation error for malformed or invalid input.
    pub fn from_json_slice(input: &[u8]) -> MediaControlResult<Self> {
        require_v1(input)?;
        let wire: MediaFrameEnvelopeV1Wire =
            serde_json::from_slice(input).map_err(malformed_json)?;
        Self::try_from(wire)
    }

    /// Encode the deterministic compact JSON representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if serialization fails unexpectedly.
    pub fn to_canonical_json_vec(&self) -> MediaControlResult<Vec<u8>> {
        canonical_json(self)
    }

    /// Resolve this compact reference against the authenticated configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigurationMismatch` for any coordinate or payload-bound
    /// mismatch. A caller must not parse or route payload bytes before this
    /// succeeds.
    pub fn resolve<'a>(
        &self,
        configuration: &'a MediaFrameConfigurationV1,
    ) -> MediaControlResult<&'a SessionMediaIdentityV1> {
        for (field, matches) in [
            (
                "binding_generation",
                self.binding_generation == configuration.binding_generation,
            ),
            (
                "configuration_ref",
                self.configuration_ref == configuration.configuration_ref,
            ),
            (
                "configuration_epoch",
                self.configuration_epoch == configuration.configuration_epoch,
            ),
            (
                "payload_bytes",
                self.payload_bytes <= configuration.max_payload_bytes,
            ),
        ] {
            if !matches {
                return Err(MediaControlError::new(
                    MediaControlErrorCode::ConfigurationMismatch,
                    field,
                    "frame does not match the authenticated configuration",
                ));
            }
        }
        Ok(&configuration.identity)
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn binding_generation(&self) -> u64 {
        self.binding_generation
    }

    #[must_use]
    pub const fn configuration_ref(&self) -> u32 {
        self.configuration_ref
    }

    #[must_use]
    pub const fn configuration_epoch(&self) -> u64 {
        self.configuration_epoch
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub const fn capture_pts(&self) -> i64 {
        self.capture_pts
    }

    #[must_use]
    pub const fn duration_ticks(&self) -> u32 {
        self.duration_ticks
    }

    #[must_use]
    pub const fn payload_bytes(&self) -> u32 {
        self.payload_bytes
    }

    #[must_use]
    pub const fn redacted(&self) -> RedactedMediaFrameEnvelopeV1 {
        RedactedMediaFrameEnvelopeV1 {
            version: self.version,
            sequence: self.sequence,
            duration_ticks: self.duration_ticks,
            payload_bytes: self.payload_bytes,
        }
    }
}

impl fmt::Debug for MediaFrameEnvelopeV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Value-free diagnostic projection of a media-frame envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedMediaFrameEnvelopeV1 {
    pub version: u16,
    pub sequence: u64,
    pub duration_ticks: u32,
    pub payload_bytes: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaFrameEnvelopeV1Wire {
    version: u16,
    binding_generation: u64,
    configuration_ref: u32,
    configuration_epoch: u64,
    sequence: u64,
    capture_pts: i64,
    duration_ticks: u32,
    payload_bytes: u32,
}

impl<'de> Deserialize<'de> for MediaFrameEnvelopeV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from(MediaFrameEnvelopeV1Wire::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

impl TryFrom<MediaFrameEnvelopeV1Wire> for MediaFrameEnvelopeV1 {
    type Error = MediaControlError;

    fn try_from(wire: MediaFrameEnvelopeV1Wire) -> Result<Self, Self::Error> {
        if wire.version != MEDIA_CONTROL_VERSION_V1 {
            return Err(unsupported_version());
        }
        Self::new(MediaFrameEnvelopeV1Params {
            binding_generation: wire.binding_generation,
            configuration_ref: wire.configuration_ref,
            configuration_epoch: wire.configuration_epoch,
            sequence: wire.sequence,
            capture_pts: wire.capture_pts,
            duration_ticks: wire.duration_ticks,
            payload_bytes: wire.payload_bytes,
        })
    }
}

/// Inputs for constructing canonical signed capability claims.
#[derive(Clone, Eq, PartialEq)]
#[allow(clippy::struct_field_names)]
pub struct MediaCapabilityClaimsV1Params {
    pub issuer: String,
    pub audience: String,
    pub capability_id: CapabilityId,
    pub tenant_id: TenantId,
    pub session_id: SessionId,
    pub session_epoch: u64,
    pub media_authorization_epoch: u64,
    pub subject_grant_epoch: u64,
    pub media_policy_version: u64,
    pub class_authorization_epoch: Option<u64>,
    pub binding_generation: u64,
    pub participant_id: ParticipantId,
    pub endpoint_id: EndpointId,
    pub contributor_id: Option<ContributorId>,
    pub operation: Operation,
    pub media_class: MediaClass,
    pub source_ids: Vec<SourceId>,
    pub audience_ids: Vec<AudienceId>,
    pub take_id: Option<TakeId>,
    pub topology_generation: u64,
    pub edge_ids: Vec<EdgeId>,
    pub max_channels: u16,
    pub max_bitrate: u64,
    pub max_datagram_bytes: u32,
    pub client_key_thumbprint: Option<String>,
    pub issued_at: i64,
    pub not_before: i64,
    pub expires_at: i64,
}

/// Strict v1 payload to be signed as a media capability.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct MediaCapabilityClaimsV1 {
    version: u16,
    issuer: String,
    audience: String,
    token_type: MediaCapabilityTokenType,
    capability_id: CapabilityId,
    tenant_id: TenantId,
    session_id: SessionId,
    session_epoch: u64,
    media_authorization_epoch: u64,
    subject_grant_epoch: u64,
    media_policy_version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    class_authorization_epoch: Option<u64>,
    binding_generation: u64,
    participant_id: ParticipantId,
    endpoint_id: EndpointId,
    #[serde(skip_serializing_if = "Option::is_none")]
    contributor_id: Option<ContributorId>,
    operation: Operation,
    media_class: MediaClass,
    source_ids: Vec<SourceId>,
    audience_ids: Vec<AudienceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    take_id: Option<TakeId>,
    topology_generation: u64,
    edge_ids: Vec<EdgeId>,
    max_channels: u16,
    max_bitrate: u64,
    max_datagram_bytes: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_key_thumbprint: Option<String>,
    issued_at: i64,
    not_before: i64,
    expires_at: i64,
}

impl MediaCapabilityClaimsV1 {
    /// Construct validated claims and canonicalize set-like scopes.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid bounds, timestamps, or semantic scopes.
    pub fn new(mut params: MediaCapabilityClaimsV1Params) -> MediaControlResult<Self> {
        validate_bounded_authority("issuer", &params.issuer)?;
        validate_bounded_authority("audience", &params.audience)?;
        for (field, generation) in [
            ("session_epoch", params.session_epoch),
            (
                "media_authorization_epoch",
                params.media_authorization_epoch,
            ),
            ("subject_grant_epoch", params.subject_grant_epoch),
            ("media_policy_version", params.media_policy_version),
            ("binding_generation", params.binding_generation),
            ("topology_generation", params.topology_generation),
        ] {
            validate_generation(field, generation)?;
        }
        if let Some(generation) = params.class_authorization_epoch {
            validate_generation("class_authorization_epoch", generation)?;
        }

        canonicalize_set("source_ids", &mut params.source_ids)?;
        canonicalize_set("audience_ids", &mut params.audience_ids)?;
        canonicalize_set("edge_ids", &mut params.edge_ids)?;
        validate_capability_scope(&params)?;
        validate_capability_limits(&params)?;
        validate_capability_times(params.issued_at, params.not_before, params.expires_at)?;
        if let Some(thumbprint) = &params.client_key_thumbprint {
            validate_thumbprint(thumbprint)?;
        }

        Ok(Self {
            version: MEDIA_CONTROL_VERSION_V1,
            issuer: params.issuer,
            audience: params.audience,
            token_type: MediaCapabilityTokenType::MediaCapability,
            capability_id: params.capability_id,
            tenant_id: params.tenant_id,
            session_id: params.session_id,
            session_epoch: params.session_epoch,
            media_authorization_epoch: params.media_authorization_epoch,
            subject_grant_epoch: params.subject_grant_epoch,
            media_policy_version: params.media_policy_version,
            class_authorization_epoch: params.class_authorization_epoch,
            binding_generation: params.binding_generation,
            participant_id: params.participant_id,
            endpoint_id: params.endpoint_id,
            contributor_id: params.contributor_id,
            operation: params.operation,
            media_class: params.media_class,
            source_ids: params.source_ids,
            audience_ids: params.audience_ids,
            take_id: params.take_id,
            topology_generation: params.topology_generation,
            edge_ids: params.edge_ids,
            max_channels: params.max_channels,
            max_bitrate: params.max_bitrate,
            max_datagram_bytes: params.max_datagram_bytes,
            client_key_thumbprint: params.client_key_thumbprint,
            issued_at: params.issued_at,
            not_before: params.not_before,
            expires_at: params.expires_at,
        })
    }

    /// Parse a closed v1 JSON object, rejecting a future version first.
    ///
    /// # Errors
    ///
    /// Returns a stable validation error for malformed or invalid input.
    pub fn from_json_slice(input: &[u8]) -> MediaControlResult<Self> {
        require_v1(input)?;
        let wire: MediaCapabilityClaimsV1Wire =
            serde_json::from_slice(input).map_err(malformed_json)?;
        validate_wire_canonical_set("source_ids", &wire.source_ids)?;
        validate_wire_canonical_set("audience_ids", &wire.audience_ids)?;
        validate_wire_canonical_set("edge_ids", &wire.edge_ids)?;
        Self::try_from(wire)
    }

    /// Encode the deterministic compact JSON fixture representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if the JSON serializer fails unexpectedly.
    pub fn to_canonical_json_vec(&self) -> MediaControlResult<Vec<u8>> {
        canonical_json(self)
    }

    /// Validate these claims against authenticated, current verifier state.
    ///
    /// Signature, `alg`, `kid`, replay, and one-use exchange checks happen in
    /// the signed-envelope verifier before this claims-level method.
    ///
    /// # Errors
    ///
    /// Returns a stable mismatch, not-yet-valid, or expired classification.
    pub fn authorize(
        &self,
        context: &MediaCapabilityValidationContextV1<'_>,
    ) -> MediaControlResult<()> {
        if context.clock_skew_seconds < 0
            || context.clock_skew_seconds > MEDIA_CONTROL_MAX_CLOCK_SKEW_SECONDS
        {
            return Err(MediaControlError::new(
                MediaControlErrorCode::LimitExceeded,
                "clock_skew_seconds",
                "clock skew must be between zero and five seconds",
            ));
        }
        validate_authorization_context(context)?;
        if context.now < self.not_before.saturating_sub(context.clock_skew_seconds) {
            return Err(MediaControlError::new(
                MediaControlErrorCode::NotYetValid,
                "not_before",
                "capability is not valid yet",
            ));
        }
        if context.now >= self.expires_at.saturating_add(context.clock_skew_seconds) {
            return Err(MediaControlError::new(
                MediaControlErrorCode::Expired,
                "expires_at",
                "capability has expired",
            ));
        }

        require_match("issuer", self.issuer == context.expected_issuer)?;
        require_match("audience", self.audience == context.expected_audience)?;
        require_match("tenant_id", &self.tenant_id == context.tenant_id)?;
        require_match("session_id", &self.session_id == context.session_id)?;
        require_match("session_epoch", self.session_epoch == context.session_epoch)?;
        require_match(
            "media_authorization_epoch",
            self.media_authorization_epoch == context.media_authorization_epoch,
        )?;
        require_match(
            "subject_grant_epoch",
            self.subject_grant_epoch == context.subject_grant_epoch,
        )?;
        require_match(
            "media_policy_version",
            self.media_policy_version == context.media_policy_version,
        )?;
        require_match(
            "class_authorization_epoch",
            self.class_authorization_epoch == context.class_authorization_epoch,
        )?;
        require_match(
            "binding_generation",
            self.binding_generation == context.binding_generation,
        )?;
        require_match(
            "topology_generation",
            self.topology_generation == context.topology_generation,
        )?;
        require_match(
            "participant_id",
            &self.participant_id == context.participant_id,
        )?;
        require_match("endpoint_id", &self.endpoint_id == context.endpoint_id)?;
        require_match("operation", self.operation == context.operation)?;
        require_match("media_class", self.media_class == context.media_class)?;
        if let Some(contributor_id) = context.contributor_id {
            require_match(
                "contributor_id",
                self.contributor_id.as_ref() == Some(contributor_id),
            )?;
        }
        if let Some(source_id) = context.source_id {
            require_match(
                "source_ids",
                self.source_ids.binary_search(source_id).is_ok(),
            )?;
        }
        if let Some(audience_id) = context.audience_id {
            require_match(
                "audience_ids",
                self.audience_ids.binary_search(audience_id).is_ok(),
            )?;
        }
        if let Some(take_id) = context.take_id {
            require_match("take_id", self.take_id.as_ref() == Some(take_id))?;
        }
        if let Some(edge_id) = context.edge_id {
            require_match("edge_ids", self.edge_ids.binary_search(edge_id).is_ok())?;
        }
        Ok(())
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    #[must_use]
    pub fn audience(&self) -> &str {
        &self.audience
    }

    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    #[must_use]
    pub const fn token_type(&self) -> MediaCapabilityTokenType {
        self.token_type
    }

    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    #[must_use]
    pub const fn session_epoch(&self) -> u64 {
        self.session_epoch
    }

    #[must_use]
    pub const fn media_authorization_epoch(&self) -> u64 {
        self.media_authorization_epoch
    }

    #[must_use]
    pub const fn subject_grant_epoch(&self) -> u64 {
        self.subject_grant_epoch
    }

    #[must_use]
    pub const fn media_policy_version(&self) -> u64 {
        self.media_policy_version
    }

    #[must_use]
    pub const fn class_authorization_epoch(&self) -> Option<u64> {
        self.class_authorization_epoch
    }

    #[must_use]
    pub const fn binding_generation(&self) -> u64 {
        self.binding_generation
    }

    #[must_use]
    pub const fn topology_generation(&self) -> u64 {
        self.topology_generation
    }

    #[must_use]
    pub const fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    #[must_use]
    pub const fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    #[must_use]
    pub const fn contributor_id(&self) -> Option<&ContributorId> {
        self.contributor_id.as_ref()
    }

    #[must_use]
    pub const fn operation(&self) -> Operation {
        self.operation
    }

    #[must_use]
    pub const fn media_class(&self) -> MediaClass {
        self.media_class
    }

    #[must_use]
    pub fn source_ids(&self) -> &[SourceId] {
        &self.source_ids
    }

    #[must_use]
    pub fn audience_ids(&self) -> &[AudienceId] {
        &self.audience_ids
    }

    #[must_use]
    pub fn edge_ids(&self) -> &[EdgeId] {
        &self.edge_ids
    }

    #[must_use]
    pub const fn take_id(&self) -> Option<&TakeId> {
        self.take_id.as_ref()
    }

    #[must_use]
    pub const fn max_channels(&self) -> u16 {
        self.max_channels
    }

    #[must_use]
    pub const fn max_bitrate(&self) -> u64 {
        self.max_bitrate
    }

    #[must_use]
    pub const fn max_datagram_bytes(&self) -> u32 {
        self.max_datagram_bytes
    }

    #[must_use]
    pub fn client_key_thumbprint(&self) -> Option<&str> {
        self.client_key_thumbprint.as_deref()
    }

    #[must_use]
    pub const fn issued_at(&self) -> i64 {
        self.issued_at
    }

    #[must_use]
    pub const fn not_before(&self) -> i64 {
        self.not_before
    }

    #[must_use]
    pub const fn expires_at(&self) -> i64 {
        self.expires_at
    }

    /// Build the intentionally lossy representation permitted in diagnostics.
    #[must_use]
    pub fn redacted(&self) -> RedactedMediaCapabilityClaimsV1 {
        RedactedMediaCapabilityClaimsV1 {
            version: self.version,
            token_type: self.token_type,
            operation: self.operation,
            media_class: self.media_class,
            source_count: self.source_ids.len(),
            audience_count: self.audience_ids.len(),
            edge_count: self.edge_ids.len(),
            has_take: self.take_id.is_some(),
            proof_bound: self.client_key_thumbprint.is_some(),
            issued_at: self.issued_at,
            expires_at: self.expires_at,
        }
    }
}

impl fmt::Debug for MediaCapabilityClaimsV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Safe diagnostic projection of capability claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedMediaCapabilityClaimsV1 {
    pub version: u16,
    pub token_type: MediaCapabilityTokenType,
    pub operation: Operation,
    pub media_class: MediaClass,
    pub source_count: usize,
    pub audience_count: usize,
    pub edge_count: usize,
    pub has_take: bool,
    pub proof_bound: bool,
    pub issued_at: i64,
    pub expires_at: i64,
}

/// Current verifier state against which signed claims are checked.
#[derive(Clone, Copy)]
#[allow(clippy::struct_field_names)]
pub struct MediaCapabilityValidationContextV1<'a> {
    pub expected_issuer: &'a str,
    pub expected_audience: &'a str,
    pub tenant_id: &'a TenantId,
    pub session_id: &'a SessionId,
    pub session_epoch: u64,
    pub media_authorization_epoch: u64,
    pub subject_grant_epoch: u64,
    pub media_policy_version: u64,
    pub class_authorization_epoch: Option<u64>,
    pub binding_generation: u64,
    pub topology_generation: u64,
    pub participant_id: &'a ParticipantId,
    pub endpoint_id: &'a EndpointId,
    pub contributor_id: Option<&'a ContributorId>,
    pub operation: Operation,
    pub media_class: MediaClass,
    pub source_id: Option<&'a SourceId>,
    pub audience_id: Option<&'a AudienceId>,
    pub take_id: Option<&'a TakeId>,
    pub edge_id: Option<&'a EdgeId>,
    pub now: i64,
    pub clock_skew_seconds: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaCapabilityClaimsV1Wire {
    version: u16,
    issuer: String,
    audience: String,
    token_type: MediaCapabilityTokenType,
    capability_id: CapabilityId,
    tenant_id: TenantId,
    session_id: SessionId,
    session_epoch: u64,
    media_authorization_epoch: u64,
    subject_grant_epoch: u64,
    media_policy_version: u64,
    #[serde(default, deserialize_with = "deserialize_absent_or_value")]
    class_authorization_epoch: Option<u64>,
    binding_generation: u64,
    participant_id: ParticipantId,
    endpoint_id: EndpointId,
    #[serde(default, deserialize_with = "deserialize_absent_or_value")]
    contributor_id: Option<ContributorId>,
    operation: Operation,
    media_class: MediaClass,
    source_ids: Vec<SourceId>,
    audience_ids: Vec<AudienceId>,
    #[serde(default, deserialize_with = "deserialize_absent_or_value")]
    take_id: Option<TakeId>,
    topology_generation: u64,
    edge_ids: Vec<EdgeId>,
    max_channels: u16,
    max_bitrate: u64,
    max_datagram_bytes: u32,
    #[serde(default, deserialize_with = "deserialize_absent_or_value")]
    client_key_thumbprint: Option<String>,
    issued_at: i64,
    not_before: i64,
    expires_at: i64,
}

impl TryFrom<MediaCapabilityClaimsV1Wire> for MediaCapabilityClaimsV1 {
    type Error = MediaControlError;

    fn try_from(wire: MediaCapabilityClaimsV1Wire) -> Result<Self, Self::Error> {
        if wire.version != MEDIA_CONTROL_VERSION_V1 {
            return Err(unsupported_version());
        }
        if wire.token_type != MediaCapabilityTokenType::MediaCapability {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidCombination,
                "token_type",
                "token type must be media_capability",
            ));
        }
        Self::new(MediaCapabilityClaimsV1Params {
            issuer: wire.issuer,
            audience: wire.audience,
            capability_id: wire.capability_id,
            tenant_id: wire.tenant_id,
            session_id: wire.session_id,
            session_epoch: wire.session_epoch,
            media_authorization_epoch: wire.media_authorization_epoch,
            subject_grant_epoch: wire.subject_grant_epoch,
            media_policy_version: wire.media_policy_version,
            class_authorization_epoch: wire.class_authorization_epoch,
            binding_generation: wire.binding_generation,
            participant_id: wire.participant_id,
            endpoint_id: wire.endpoint_id,
            contributor_id: wire.contributor_id,
            operation: wire.operation,
            media_class: wire.media_class,
            source_ids: wire.source_ids,
            audience_ids: wire.audience_ids,
            take_id: wire.take_id,
            topology_generation: wire.topology_generation,
            edge_ids: wire.edge_ids,
            max_channels: wire.max_channels,
            max_bitrate: wire.max_bitrate,
            max_datagram_bytes: wire.max_datagram_bytes,
            client_key_thumbprint: wire.client_key_thumbprint,
            issued_at: wire.issued_at,
            not_before: wire.not_before,
            expires_at: wire.expires_at,
        })
    }
}

impl<'de> Deserialize<'de> for MediaCapabilityClaimsV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = MediaCapabilityClaimsV1Wire::deserialize(deserializer)?;
        validate_wire_canonical_set("source_ids", &wire.source_ids).map_err(de::Error::custom)?;
        validate_wire_canonical_set("audience_ids", &wire.audience_ids)
            .map_err(de::Error::custom)?;
        validate_wire_canonical_set("edge_ids", &wire.edge_ids).map_err(de::Error::custom)?;
        Self::try_from(wire).map_err(de::Error::custom)
    }
}

/// Inputs for a non-authorizing media endpoint descriptor.
#[derive(Clone, Eq, PartialEq)]
pub struct MediaEndpointDescriptorV1Params {
    pub descriptor_id: DescriptorId,
    pub tenant_id: TenantId,
    pub session_id: SessionId,
    pub session_epoch: u64,
    pub endpoint_id: EndpointId,
    pub edge_id: EdgeId,
    pub binding_generation: u64,
    pub topology_generation: u64,
    pub transport: MediaEndpointTransport,
    pub origin: String,
    pub path: String,
    pub expires_at: i64,
}

/// A route descriptor that intentionally carries no reusable authorization.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct MediaEndpointDescriptorV1 {
    version: u16,
    descriptor_id: DescriptorId,
    tenant_id: TenantId,
    session_id: SessionId,
    session_epoch: u64,
    endpoint_id: EndpointId,
    edge_id: EdgeId,
    binding_generation: u64,
    topology_generation: u64,
    transport: MediaEndpointTransport,
    origin: String,
    path: String,
    expires_at: i64,
}

impl MediaEndpointDescriptorV1 {
    /// Construct a descriptor from safe origin/path components.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid generations, timestamps, origins, or paths.
    pub fn new(params: MediaEndpointDescriptorV1Params) -> MediaControlResult<Self> {
        validate_generation("session_epoch", params.session_epoch)?;
        validate_generation("binding_generation", params.binding_generation)?;
        validate_generation("topology_generation", params.topology_generation)?;
        validate_endpoint_origin(&params.origin)?;
        validate_endpoint_path(&params.path)?;
        validate_unix_seconds("expires_at", params.expires_at)?;
        Ok(Self {
            version: MEDIA_CONTROL_VERSION_V1,
            descriptor_id: params.descriptor_id,
            tenant_id: params.tenant_id,
            session_id: params.session_id,
            session_epoch: params.session_epoch,
            endpoint_id: params.endpoint_id,
            edge_id: params.edge_id,
            binding_generation: params.binding_generation,
            topology_generation: params.topology_generation,
            transport: params.transport,
            origin: params.origin,
            path: params.path,
            expires_at: params.expires_at,
        })
    }

    /// Parse a closed v1 JSON object, rejecting a future version first.
    ///
    /// # Errors
    ///
    /// Returns a stable validation error for malformed or invalid input.
    pub fn from_json_slice(input: &[u8]) -> MediaControlResult<Self> {
        require_v1(input)?;
        let wire: MediaEndpointDescriptorV1Wire =
            serde_json::from_slice(input).map_err(malformed_json)?;
        Self::try_from(wire)
    }

    /// Encode the deterministic compact JSON fixture representation.
    ///
    /// # Errors
    ///
    /// Returns an error only if the JSON serializer fails unexpectedly.
    pub fn to_canonical_json_vec(&self) -> MediaControlResult<Vec<u8>> {
        canonical_json(self)
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn descriptor_id(&self) -> &DescriptorId {
        &self.descriptor_id
    }

    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    #[must_use]
    pub const fn session_epoch(&self) -> u64 {
        self.session_epoch
    }

    #[must_use]
    pub const fn endpoint_id(&self) -> &EndpointId {
        &self.endpoint_id
    }

    #[must_use]
    pub const fn edge_id(&self) -> &EdgeId {
        &self.edge_id
    }

    #[must_use]
    pub const fn binding_generation(&self) -> u64 {
        self.binding_generation
    }

    #[must_use]
    pub const fn topology_generation(&self) -> u64 {
        self.topology_generation
    }

    #[must_use]
    pub const fn transport(&self) -> MediaEndpointTransport {
        self.transport
    }

    #[must_use]
    pub fn origin(&self) -> &str {
        &self.origin
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn expires_at(&self) -> i64 {
        self.expires_at
    }

    /// Build the intentionally lossy representation permitted in diagnostics.
    #[must_use]
    pub const fn redacted(&self) -> RedactedMediaEndpointDescriptorV1 {
        RedactedMediaEndpointDescriptorV1 {
            version: self.version,
            transport: self.transport,
            session_epoch: self.session_epoch,
            binding_generation: self.binding_generation,
            topology_generation: self.topology_generation,
            expires_at: self.expires_at,
        }
    }
}

impl fmt::Debug for MediaEndpointDescriptorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted().fmt(formatter)
    }
}

/// Safe diagnostic projection of an endpoint descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RedactedMediaEndpointDescriptorV1 {
    pub version: u16,
    pub transport: MediaEndpointTransport,
    pub session_epoch: u64,
    pub binding_generation: u64,
    pub topology_generation: u64,
    pub expires_at: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaEndpointDescriptorV1Wire {
    version: u16,
    descriptor_id: DescriptorId,
    tenant_id: TenantId,
    session_id: SessionId,
    session_epoch: u64,
    endpoint_id: EndpointId,
    edge_id: EdgeId,
    binding_generation: u64,
    topology_generation: u64,
    transport: MediaEndpointTransport,
    origin: String,
    path: String,
    expires_at: i64,
}

impl<'de> Deserialize<'de> for MediaEndpointDescriptorV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = MediaEndpointDescriptorV1Wire::deserialize(deserializer)?;
        Self::try_from(wire).map_err(de::Error::custom)
    }
}

impl TryFrom<MediaEndpointDescriptorV1Wire> for MediaEndpointDescriptorV1 {
    type Error = MediaControlError;

    fn try_from(wire: MediaEndpointDescriptorV1Wire) -> Result<Self, Self::Error> {
        if wire.version != MEDIA_CONTROL_VERSION_V1 {
            return Err(unsupported_version());
        }
        Self::new(MediaEndpointDescriptorV1Params {
            descriptor_id: wire.descriptor_id,
            tenant_id: wire.tenant_id,
            session_id: wire.session_id,
            session_epoch: wire.session_epoch,
            endpoint_id: wire.endpoint_id,
            edge_id: wire.edge_id,
            binding_generation: wire.binding_generation,
            topology_generation: wire.topology_generation,
            transport: wire.transport,
            origin: wire.origin,
            path: wire.path,
            expires_at: wire.expires_at,
        })
    }
}

fn validate_opaque_id(field: &'static str, value: &str) -> MediaControlResult<()> {
    if value.is_empty() {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidIdentifier,
            field,
            "opaque identifier must not be empty",
        ));
    }
    if value.len() > MEDIA_CONTROL_MAX_ID_BYTES {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            field,
            "opaque identifier exceeds 128 bytes",
        ));
    }
    let mut bytes = value.bytes();
    let first_is_valid = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric());
    let rest_is_valid =
        bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'));
    if !first_is_valid || !rest_is_valid {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidIdentifier,
            field,
            "opaque identifier must use the v1 ASCII token alphabet",
        ));
    }
    Ok(())
}

fn validate_generation(field: &'static str, value: u64) -> MediaControlResult<()> {
    if value == 0 || value > MEDIA_CONTROL_MAX_GENERATION {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidGeneration,
            field,
            "generation must be between one and 2^53-1",
        ));
    }
    Ok(())
}

fn validate_signed_exact_integer(field: &'static str, value: i64) -> MediaControlResult<()> {
    if !(-MAX_EXACT_UNIX_SECONDS..=MAX_EXACT_UNIX_SECONDS).contains(&value) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            field,
            "value must be an exact signed JavaScript integer",
        ));
    }
    Ok(())
}

fn validate_frame_configuration_semantics(
    media_class: MediaClass,
    payload_format: MediaFramePayloadFormat,
    capture_timebase_hz: u32,
    channel_count: u16,
    capture_disposition: MediaCaptureDisposition,
) -> MediaControlResult<()> {
    let format_allowed = match media_class {
        MediaClass::Program | MediaClass::Source => matches!(
            payload_format,
            MediaFramePayloadFormat::Opus
                | MediaFramePayloadFormat::PcmS24le
                | MediaFramePayloadFormat::Flac
                | MediaFramePayloadFormat::Opaque
        ),
        MediaClass::Talkback => payload_format == MediaFramePayloadFormat::Opus,
        MediaClass::Screen => payload_format == MediaFramePayloadFormat::Opaque,
        MediaClass::Metadata => matches!(
            payload_format,
            MediaFramePayloadFormat::Json | MediaFramePayloadFormat::Opaque
        ),
        MediaClass::TakeChunk => matches!(
            payload_format,
            MediaFramePayloadFormat::PcmS24le
                | MediaFramePayloadFormat::Flac
                | MediaFramePayloadFormat::Opaque
        ),
    };
    if !format_allowed {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "payload_format",
            "payload format is not valid for the configured media class",
        ));
    }
    if payload_format == MediaFramePayloadFormat::Opus && capture_timebase_hz != 48_000 {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "capture_timebase_hz",
            "Opus capture PTS uses the fixed 48000 Hz timebase",
        ));
    }
    if payload_format == MediaFramePayloadFormat::Json && channel_count != 1 {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "channel_count",
            "JSON metadata uses one logical channel",
        ));
    }
    if media_class == MediaClass::Talkback {
        if channel_count != 1 {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidCombination,
                "channel_count",
                "talkback v1 is mono",
            ));
        }
        if capture_disposition != MediaCaptureDisposition::MonitorOnly {
            return Err(MediaControlError::new(
                MediaControlErrorCode::InvalidCombination,
                "capture_disposition",
                "talkback is always monitor-only",
            ));
        }
    }
    if media_class == MediaClass::TakeChunk
        && capture_disposition != MediaCaptureDisposition::Recordable
    {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "capture_disposition",
            "take chunks are recordable artifacts",
        ));
    }
    Ok(())
}

fn validate_bounded_text(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> MediaControlResult<()> {
    if value.is_empty() {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            field,
            "value must not be empty",
        ));
    }
    if value.len() > maximum {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            field,
            "value exceeds its encoded byte bound",
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            field,
            "value must not contain control characters",
        ));
    }
    Ok(())
}

fn validate_bounded_authority(field: &'static str, value: &str) -> MediaControlResult<()> {
    validate_bounded_text(field, value, MAX_AUTHORITY_BYTES)?;
    if !value.is_ascii() || value.bytes().any(|byte| !byte.is_ascii_graphic()) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            field,
            "authority must contain only visible ASCII characters",
        ));
    }
    Ok(())
}

fn validate_identity_scope(
    media_class: MediaClass,
    source_id: Option<&SourceId>,
    audience_id: Option<&AudienceId>,
    take_id: Option<&TakeId>,
) -> MediaControlResult<()> {
    match media_class {
        MediaClass::Talkback
            if source_id.is_none() && audience_id.is_some() && take_id.is_none() =>
        {
            Ok(())
        }
        MediaClass::TakeChunk
            if source_id.is_some() && audience_id.is_none() && take_id.is_some() =>
        {
            Ok(())
        }
        MediaClass::Program | MediaClass::Source | MediaClass::Screen | MediaClass::Metadata
            if source_id.is_some() && audience_id.is_none() && take_id.is_none() =>
        {
            Ok(())
        }
        _ => Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "media_class",
            "media class has an invalid source, audience, or take identity",
        )),
    }
}

fn canonicalize_set<T: Ord>(field: &'static str, values: &mut [T]) -> MediaControlResult<()> {
    if values.len() > MEDIA_CONTROL_MAX_SCOPE_IDS {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            field,
            "scope exceeds 64 identifiers",
        ));
    }
    values.sort_unstable();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::DuplicateValue,
            field,
            "scope contains a duplicate identifier",
        ));
    }
    Ok(())
}

fn validate_wire_canonical_set<T: Ord>(
    field: &'static str,
    values: &[T],
) -> MediaControlResult<()> {
    if values.len() > MEDIA_CONTROL_MAX_SCOPE_IDS {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            field,
            "scope exceeds 64 identifiers",
        ));
    }
    for pair in values.windows(2) {
        match pair[0].cmp(&pair[1]) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => {
                return Err(MediaControlError::new(
                    MediaControlErrorCode::DuplicateValue,
                    field,
                    "scope contains a duplicate identifier",
                ));
            }
            std::cmp::Ordering::Greater => {
                return Err(MediaControlError::new(
                    MediaControlErrorCode::NonCanonicalOrder,
                    field,
                    "scope is not in canonical ascending order",
                ));
            }
        }
    }
    Ok(())
}

fn canonicalize_operation_set(values: &mut [Operation]) -> MediaControlResult<()> {
    canonicalize_ranked_set("allowed_operations", values, 5, operation_canonical_rank)
}

fn canonicalize_media_class_set(values: &mut [MediaClass]) -> MediaControlResult<()> {
    canonicalize_ranked_set(
        "allowed_media_classes",
        values,
        6,
        media_class_canonical_rank,
    )
}

fn canonicalize_ranked_set<T: Copy + Eq>(
    field: &'static str,
    values: &mut [T],
    maximum: usize,
    rank: fn(T) -> u8,
) -> MediaControlResult<()> {
    validate_nonempty_set_bound(field, values.len(), maximum)?;
    values.sort_unstable_by_key(|value| rank(*value));
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::DuplicateValue,
            field,
            "scope contains a duplicate value",
        ));
    }
    Ok(())
}

fn validate_wire_canonical_ranked_set<T: Copy + Eq>(
    field: &'static str,
    values: &[T],
    maximum: usize,
    rank: fn(T) -> u8,
) -> MediaControlResult<()> {
    validate_nonempty_set_bound(field, values.len(), maximum)?;
    for pair in values.windows(2) {
        let left = rank(pair[0]);
        let right = rank(pair[1]);
        if left == right {
            return Err(MediaControlError::new(
                MediaControlErrorCode::DuplicateValue,
                field,
                "scope contains a duplicate value",
            ));
        }
        if left > right {
            return Err(MediaControlError::new(
                MediaControlErrorCode::NonCanonicalOrder,
                field,
                "scope is not in canonical ascending order",
            ));
        }
    }
    Ok(())
}

fn validate_nonempty_set_bound(
    field: &'static str,
    length: usize,
    maximum: usize,
) -> MediaControlResult<()> {
    if length == 0 {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            field,
            "scope must contain at least one value",
        ));
    }
    if length > maximum {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            field,
            "scope exceeds the complete v1 enum vocabulary",
        ));
    }
    Ok(())
}

const fn operation_canonical_rank(operation: Operation) -> u8 {
    match operation {
        Operation::AcknowledgePlayout => 0,
        Operation::Publish => 1,
        Operation::ReadTake => 2,
        Operation::Subscribe => 3,
        Operation::UploadTake => 4,
    }
}

const fn media_class_canonical_rank(media_class: MediaClass) -> u8 {
    match media_class {
        MediaClass::Metadata => 0,
        MediaClass::Program => 1,
        MediaClass::Screen => 2,
        MediaClass::Source => 3,
        MediaClass::TakeChunk => 4,
        MediaClass::Talkback => 5,
    }
}

fn validate_fact_wire_sets(wire: &MediaAuthorizationFactV1Wire) -> MediaControlResult<()> {
    validate_wire_canonical_ranked_set(
        "allowed_operations",
        &wire.allowed_operations,
        5,
        operation_canonical_rank,
    )?;
    validate_wire_canonical_ranked_set(
        "allowed_media_classes",
        &wire.allowed_media_classes,
        6,
        media_class_canonical_rank,
    )?;
    validate_wire_canonical_set("allowed_source_ids", &wire.allowed_source_ids)?;
    validate_wire_canonical_set("allowed_audience_ids", &wire.allowed_audience_ids)
}

fn validate_requested_operation_scope(
    operation: Operation,
    media_class: MediaClass,
    take_id: Option<&TakeId>,
    workflow_mode: Option<SessionWorkflowMode>,
) -> MediaControlResult<()> {
    if !operation_allows_media_class(operation, media_class) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "requested_operation",
            "requested operation is not defined for this media class",
        ));
    }
    let take_operation = matches!(operation, Operation::UploadTake | Operation::ReadTake);
    if take_operation != take_id.is_some() {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "take_id",
            "take operations require an exact take ID and live operations forbid one",
        ));
    }
    if take_operation && workflow_mode.is_some_and(|mode| mode != SessionWorkflowMode::FinalTake) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "workflow_mode",
            "take operations require the final_take workflow",
        ));
    }
    Ok(())
}

fn validate_capability_scope(params: &MediaCapabilityClaimsV1Params) -> MediaControlResult<()> {
    if params.edge_ids.is_empty() {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "edge_ids",
            "at least one explicitly admitted edge is required",
        ));
    }
    if matches!(params.operation, Operation::Publish | Operation::UploadTake)
        && params.contributor_id.is_none()
    {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "contributor_id",
            "publish and upload operations require a contributor",
        ));
    }
    if !operation_allows_media_class(params.operation, params.media_class) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "operation",
            "operation is not defined for this media class",
        ));
    }
    if matches!(
        params.operation,
        Operation::UploadTake | Operation::ReadTake
    ) != (params.media_class == MediaClass::TakeChunk)
    {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "operation",
            "take operations and take_chunk media class must be used together",
        ));
    }
    match params.media_class {
        MediaClass::Talkback
            if params.source_ids.is_empty()
                && !params.audience_ids.is_empty()
                && params.take_id.is_none() =>
        {
            Ok(())
        }
        MediaClass::TakeChunk
            if !params.source_ids.is_empty()
                && params.audience_ids.is_empty()
                && params.take_id.is_some() =>
        {
            Ok(())
        }
        MediaClass::Program | MediaClass::Source | MediaClass::Screen | MediaClass::Metadata
            if !params.source_ids.is_empty()
                && params.audience_ids.is_empty()
                && params.take_id.is_none() =>
        {
            Ok(())
        }
        _ => Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "media_class",
            "capability has an invalid source, audience, or take scope",
        )),
    }
}

fn operation_allows_media_class(operation: Operation, media_class: MediaClass) -> bool {
    match operation {
        Operation::Publish | Operation::Subscribe => matches!(
            media_class,
            MediaClass::Program
                | MediaClass::Source
                | MediaClass::Talkback
                | MediaClass::Screen
                | MediaClass::Metadata
        ),
        Operation::AcknowledgePlayout => matches!(
            media_class,
            MediaClass::Program | MediaClass::Source | MediaClass::Talkback
        ),
        Operation::UploadTake | Operation::ReadTake => media_class == MediaClass::TakeChunk,
    }
}

fn validate_authorization_context(
    context: &MediaCapabilityValidationContextV1<'_>,
) -> MediaControlResult<()> {
    if context.edge_id.is_none() {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "edge_id",
            "authorization checks require the serving edge",
        ));
    }
    if matches!(
        context.operation,
        Operation::Publish | Operation::UploadTake
    ) && context.contributor_id.is_none()
    {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "contributor_id",
            "publish and upload checks require the contributor",
        ));
    }
    let valid_scope = match context.media_class {
        MediaClass::Talkback => {
            context.source_id.is_none()
                && context.audience_id.is_some()
                && context.take_id.is_none()
        }
        MediaClass::TakeChunk => {
            context.source_id.is_some()
                && context.audience_id.is_none()
                && context.take_id.is_some()
        }
        MediaClass::Program | MediaClass::Source | MediaClass::Screen | MediaClass::Metadata => {
            context.source_id.is_some()
                && context.audience_id.is_none()
                && context.take_id.is_none()
        }
    };
    if !valid_scope {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "media_class",
            "authorization check requires the exact class-specific scope",
        ));
    }
    Ok(())
}

fn validate_capability_limits(params: &MediaCapabilityClaimsV1Params) -> MediaControlResult<()> {
    if params.max_channels == 0 || params.max_channels > MAX_CHANNELS {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            "max_channels",
            "channel limit must be between one and 128",
        ));
    }
    if params.max_bitrate == 0 || params.max_bitrate > MAX_BITRATE {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            "max_bitrate",
            "bitrate limit must be between one and 1000000000",
        ));
    }
    if !(MIN_DATAGRAM_BYTES..=MAX_DATAGRAM_BYTES).contains(&params.max_datagram_bytes) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            "max_datagram_bytes",
            "datagram limit must be between 256 and 65535",
        ));
    }
    Ok(())
}

fn validate_capability_times(
    issued_at: i64,
    not_before: i64,
    expires_at: i64,
) -> MediaControlResult<()> {
    validate_unix_seconds("issued_at", issued_at)?;
    validate_unix_seconds("not_before", not_before)?;
    validate_unix_seconds("expires_at", expires_at)?;
    if expires_at <= issued_at || not_before > expires_at {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidTimestamp,
            "expires_at",
            "timestamps do not form a usable validity interval",
        ));
    }
    if expires_at - issued_at > MEDIA_CONTROL_MAX_CAPABILITY_LIFETIME_SECONDS {
        return Err(MediaControlError::new(
            MediaControlErrorCode::CapabilityLifetimeExceeded,
            "expires_at",
            "capability lifetime exceeds 90 seconds",
        ));
    }
    Ok(())
}

fn validate_unix_seconds(field: &'static str, value: i64) -> MediaControlResult<()> {
    if !(0..=MAX_EXACT_UNIX_SECONDS).contains(&value) {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidTimestamp,
            field,
            "Unix seconds must be a non-negative exact cross-language integer",
        ));
    }
    Ok(())
}

fn validate_thumbprint(value: &str) -> MediaControlResult<()> {
    if !(43..=128).contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidCombination,
            "client_key_thumbprint",
            "thumbprint must be bounded unpadded base64url",
        ));
    }
    Ok(())
}

fn validate_endpoint_origin(origin: &str) -> MediaControlResult<()> {
    validate_bounded_text("origin", origin, MAX_ORIGIN_BYTES)?;
    if !origin.is_ascii() {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidEndpoint,
            "origin",
            "endpoint origin must use an ASCII or punycode authority",
        ));
    }
    let Some(authority) = origin.strip_prefix("https://") else {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidEndpoint,
            "origin",
            "endpoint origin must use HTTPS",
        ));
    };
    let authority = authority.strip_suffix('/').unwrap_or(authority);
    if authority.is_empty()
        || authority.contains(['/', '?', '#', '@'])
        || authority.chars().any(char::is_whitespace)
        || !authority
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidEndpoint,
            "origin",
            "origin must contain only an HTTPS authority and no user info",
        ));
    }
    Ok(())
}

fn validate_endpoint_path(path: &str) -> MediaControlResult<()> {
    validate_bounded_text("path", path, MAX_PATH_BYTES)?;
    if !path.starts_with('/')
        || path.contains(['?', '#'])
        || path.contains("..")
        || !path.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'~' | b'-')
        })
    {
        return Err(MediaControlError::new(
            MediaControlErrorCode::InvalidEndpoint,
            "path",
            "path must be a bounded absolute non-authorizing token path",
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
struct VersionProbe {
    version: u64,
}

fn require_v1(input: &[u8]) -> MediaControlResult<()> {
    if input.len() > MEDIA_CONTROL_MAX_JSON_BYTES {
        return Err(MediaControlError::new(
            MediaControlErrorCode::LimitExceeded,
            "json",
            "media-control JSON exceeds 64 KiB",
        ));
    }
    let probe: VersionProbe = serde_json::from_slice(input).map_err(malformed_json)?;
    if probe.version != u64::from(MEDIA_CONTROL_VERSION_V1) {
        return Err(unsupported_version());
    }
    Ok(())
}

const fn unsupported_version() -> MediaControlError {
    MediaControlError::new(
        MediaControlErrorCode::UnsupportedVersion,
        "version",
        "unsupported media-control major version",
    )
}

fn malformed_json<E>(_error: E) -> MediaControlError {
    MediaControlError::new(
        MediaControlErrorCode::MalformedJson,
        "json",
        "input is not a closed valid media-control JSON object",
    )
}

fn canonical_json<T: Serialize>(value: &T) -> MediaControlResult<Vec<u8>> {
    let mut json = serde_json::to_vec(value).map_err(malformed_json)?;
    json.push(b'\n');
    Ok(json)
}

fn require_match(field: &'static str, matches: bool) -> MediaControlResult<()> {
    if matches {
        Ok(())
    } else {
        Err(MediaControlError::new(
            MediaControlErrorCode::AuthorizationMismatch,
            field,
            "claim does not match the authenticated verifier context",
        ))
    }
}

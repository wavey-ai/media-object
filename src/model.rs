use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// Hard safety ceilings compiled into media-object v1.
///
/// Services can pass smaller [`Limits`] to decoding and validation. They cannot
/// raise these ceilings without a reviewed crate release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_payload_bytes: usize,
    pub max_metadata_bytes: usize,
    pub max_metadata_entries: usize,
    pub max_metadata_key_bytes: usize,
    pub max_metadata_value_bytes: usize,
    pub max_dependencies: usize,
    pub max_stage_timestamps: usize,
    pub max_identifier_bytes: usize,
    pub max_clock_id_bytes: usize,
    pub max_envelope_bytes: usize,
}

impl Limits {
    pub const HARD: Self = Self {
        max_payload_bytes: 16 * 1024 * 1024,
        max_metadata_bytes: 64 * 1024,
        max_metadata_entries: 64,
        max_metadata_key_bytes: 128,
        max_metadata_value_bytes: 16 * 1024,
        max_dependencies: 64,
        max_stage_timestamps: 32,
        max_identifier_bytes: 128,
        max_clock_id_bytes: 128,
        max_envelope_bytes: 16 * 1024 * 1024 + 256 * 1024,
    };

    pub(crate) fn validate(self) -> Result<()> {
        macro_rules! within_hard_limit {
            ($field:ident) => {
                if self.$field > Self::HARD.$field {
                    return Err(Error::InvalidLimit {
                        field: stringify!($field),
                        actual: self.$field,
                        hard_maximum: Self::HARD.$field,
                    });
                }
            };
        }

        within_hard_limit!(max_payload_bytes);
        within_hard_limit!(max_metadata_bytes);
        within_hard_limit!(max_metadata_entries);
        within_hard_limit!(max_metadata_key_bytes);
        within_hard_limit!(max_metadata_value_bytes);
        within_hard_limit!(max_dependencies);
        within_hard_limit!(max_stage_timestamps);
        within_hard_limit!(max_identifier_bytes);
        within_hard_limit!(max_clock_id_bytes);
        within_hard_limit!(max_envelope_bytes);
        Ok(())
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::HARD
    }
}

/// A SHA-256 payload digest carried as part of an immutable object key.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PayloadHash([u8; 32]);

impl PayloadHash {
    #[must_use]
    pub fn digest(payload: &[u8]) -> Self {
        let digest: [u8; 32] = Sha256::digest(payload).into();
        Self(digest)
    }

    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for PayloadHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl From<[u8; 32]> for PayloadHash {
    fn from(value: [u8; 32]) -> Self {
        Self::from_bytes(value)
    }
}

/// Complete immutable identity for one media object.
///
/// The structured fields correspond to:
/// `tenant / stream / track / epoch / group / object / version / payload hash`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectKey {
    tenant: String,
    stream: String,
    track: String,
    epoch: u64,
    group: u64,
    object: u64,
    version: u32,
    payload_hash: PayloadHash,
}

impl ObjectKey {
    /// Construct a key from its complete identity components.
    ///
    /// # Errors
    ///
    /// Returns an error when an identifier is empty, contains a control
    /// character, or exceeds the hard identifier ceiling.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant: impl Into<String>,
        stream: impl Into<String>,
        track: impl Into<String>,
        epoch: u64,
        group: u64,
        object: u64,
        version: u32,
        payload_hash: PayloadHash,
    ) -> Result<Self> {
        let key = Self {
            tenant: tenant.into(),
            stream: stream.into(),
            track: track.into(),
            epoch,
            group,
            object,
            version,
            payload_hash,
        };
        key.validate_with_limits(Limits::HARD)?;
        Ok(key)
    }

    /// Construct a complete key while calculating `SHA-256(payload)`.
    ///
    /// # Errors
    ///
    /// Returns an error when an identifier is empty, contains a control
    /// character, or exceeds the hard identifier ceiling.
    #[allow(clippy::too_many_arguments)]
    pub fn for_payload(
        tenant: impl Into<String>,
        stream: impl Into<String>,
        track: impl Into<String>,
        epoch: u64,
        group: u64,
        object: u64,
        version: u32,
        payload: &[u8],
    ) -> Result<Self> {
        Self::new(
            tenant,
            stream,
            track,
            epoch,
            group,
            object,
            version,
            PayloadHash::digest(payload),
        )
    }

    #[must_use]
    pub fn tenant(&self) -> &str {
        &self.tenant
    }

    #[must_use]
    pub fn stream(&self) -> &str {
        &self.stream
    }

    #[must_use]
    pub fn track(&self) -> &str {
        &self.track
    }

    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    #[must_use]
    pub const fn group(&self) -> u64 {
        self.group
    }

    #[must_use]
    pub const fn object(&self) -> u64 {
        self.object
    }

    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    #[must_use]
    pub const fn payload_hash(&self) -> PayloadHash {
        self.payload_hash
    }

    pub(crate) fn validate_with_limits(&self, limits: Limits) -> Result<()> {
        validate_identifier("tenant", &self.tenant, limits.max_identifier_bytes)?;
        validate_identifier("stream", &self.stream, limits.max_identifier_bytes)?;
        validate_identifier("track", &self.track, limits.max_identifier_bytes)
    }
}

/// Semantic role of the object's payload.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ObjectKind {
    Media = 0,
    Initialization = 1,
    CodecConfiguration = 2,
    Discontinuity = 3,
}

impl ObjectKind {
    pub(crate) fn from_wire(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Media),
            1 => Ok(Self::Initialization),
            2 => Ok(Self::CodecConfiguration),
            3 => Ok(Self::Discontinuity),
            _ => Err(Error::UnknownTag {
                field: "object_kind",
                value,
            }),
        }
    }
}

/// Qualitative origin of a timestamp's confidence claim.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ClockConfidenceLevel {
    Unknown = 0,
    Estimated = 1,
    Synchronized = 2,
    Traceable = 3,
}

impl ClockConfidenceLevel {
    pub(crate) fn from_wire(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Estimated),
            2 => Ok(Self::Synchronized),
            3 => Ok(Self::Traceable),
            _ => Err(Error::UnknownTag {
                field: "clock_confidence_level",
                value,
            }),
        }
    }
}

/// Confidence attached to a wall-clock timestamp.
///
/// Known confidence levels include a numeric maximum error. This lets latency
/// analysis distinguish a measured delay from clock uncertainty.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClockConfidence {
    level: ClockConfidenceLevel,
    maximum_error_ns: Option<u64>,
}

impl ClockConfidence {
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            level: ClockConfidenceLevel::Unknown,
            maximum_error_ns: None,
        }
    }

    #[must_use]
    pub const fn estimated(maximum_error_ns: u64) -> Self {
        Self {
            level: ClockConfidenceLevel::Estimated,
            maximum_error_ns: Some(maximum_error_ns),
        }
    }

    #[must_use]
    pub const fn synchronized(maximum_error_ns: u64) -> Self {
        Self {
            level: ClockConfidenceLevel::Synchronized,
            maximum_error_ns: Some(maximum_error_ns),
        }
    }

    #[must_use]
    pub const fn traceable(maximum_error_ns: u64) -> Self {
        Self {
            level: ClockConfidenceLevel::Traceable,
            maximum_error_ns: Some(maximum_error_ns),
        }
    }

    pub(crate) fn from_parts(
        level: ClockConfidenceLevel,
        maximum_error_ns: Option<u64>,
    ) -> Result<Self> {
        let confidence = Self {
            level,
            maximum_error_ns,
        };
        confidence.validate()?;
        Ok(confidence)
    }

    #[must_use]
    pub const fn level(self) -> ClockConfidenceLevel {
        self.level
    }

    #[must_use]
    pub const fn maximum_error_ns(self) -> Option<u64> {
        self.maximum_error_ns
    }

    pub(crate) fn validate(self) -> Result<()> {
        match (self.level, self.maximum_error_ns) {
            (ClockConfidenceLevel::Unknown, None)
            | (
                ClockConfidenceLevel::Estimated
                | ClockConfidenceLevel::Synchronized
                | ClockConfidenceLevel::Traceable,
                Some(_),
            ) => Ok(()),
            (ClockConfidenceLevel::Unknown, Some(_)) => Err(Error::InvalidField {
                field: "clock_confidence",
                reason: "unknown confidence cannot claim a maximum error",
            }),
            (_, None) => Err(Error::InvalidField {
                field: "clock_confidence",
                reason: "known confidence requires a maximum error",
            }),
        }
    }
}

/// A Unix timestamp together with clock-domain and confidence provenance.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClockTimestamp {
    unix_time_ns: i64,
    clock_id: String,
    confidence: ClockConfidence,
}

impl ClockTimestamp {
    /// Construct a timestamp with explicit clock provenance.
    ///
    /// # Errors
    ///
    /// Returns an error when the clock identifier is invalid or the confidence
    /// value violates its level/error invariant.
    pub fn new(
        unix_time_ns: i64,
        clock_id: impl Into<String>,
        confidence: ClockConfidence,
    ) -> Result<Self> {
        let timestamp = Self {
            unix_time_ns,
            clock_id: clock_id.into(),
            confidence,
        };
        timestamp.validate_with_limits(Limits::HARD)?;
        Ok(timestamp)
    }

    #[must_use]
    pub const fn unix_time_ns(&self) -> i64 {
        self.unix_time_ns
    }

    #[must_use]
    pub fn clock_id(&self) -> &str {
        &self.clock_id
    }

    #[must_use]
    pub const fn confidence(&self) -> ClockConfidence {
        self.confidence
    }

    pub(crate) fn validate_with_limits(&self, limits: Limits) -> Result<()> {
        validate_identifier("clock_id", &self.clock_id, limits.max_clock_id_bytes)?;
        self.confidence.validate()
    }
}

/// End-to-end stage names shared by object metadata and telemetry observations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Stage {
    IngressReceived = 0,
    Normalized = 1,
    Packaged = 2,
    Published = 3,
    RelayReceived = 4,
    RelayForwarded = 5,
    EdgeAvailable = 6,
    PlayerPresented = 7,
}

impl Stage {
    pub(crate) fn from_wire(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::IngressReceived),
            1 => Ok(Self::Normalized),
            2 => Ok(Self::Packaged),
            3 => Ok(Self::Published),
            4 => Ok(Self::RelayReceived),
            5 => Ok(Self::RelayForwarded),
            6 => Ok(Self::EdgeAvailable),
            7 => Ok(Self::PlayerPresented),
            _ => Err(Error::UnknownTag {
                field: "stage",
                value,
            }),
        }
    }
}

/// Timestamp for one processing or delivery stage.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StageTimestamp {
    stage: Stage,
    timestamp: ClockTimestamp,
}

impl StageTimestamp {
    #[must_use]
    pub const fn new(stage: Stage, timestamp: ClockTimestamp) -> Self {
        Self { stage, timestamp }
    }

    #[must_use]
    pub const fn stage(&self) -> Stage {
        self.stage
    }

    #[must_use]
    pub const fn timestamp(&self) -> &ClockTimestamp {
        &self.timestamp
    }
}

/// Result of comparing two attempted writes at an immutable object boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteDisposition {
    /// The keys differ, so the candidate is a distinct object.
    DistinctObject,
    /// Key and complete canonical envelope match; the retry is idempotent.
    IdempotentReplay,
    /// The same key was presented with different immutable metadata or bytes.
    IdentityConflict,
}

/// One canonical media object.
///
/// Fields are private and the type exposes no mutators. Enrichment after publish
/// therefore requires a new version or an external observation keyed by
/// [`ObjectKey`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaObject {
    key: ObjectKey,
    kind: ObjectKind,
    keyframe: bool,
    configuration_epoch: u64,
    deadline: Option<ClockTimestamp>,
    capture_timestamp: Option<ClockTimestamp>,
    stage_timestamps: Vec<StageTimestamp>,
    dependencies: Vec<ObjectKey>,
    metadata: BTreeMap<String, Vec<u8>>,
    payload: Vec<u8>,
}

impl MediaObject {
    #[must_use]
    pub fn builder(key: ObjectKey, kind: ObjectKind, payload: Vec<u8>) -> MediaObjectBuilder {
        MediaObjectBuilder::new(key, kind, payload)
    }

    #[must_use]
    pub const fn key(&self) -> &ObjectKey {
        &self.key
    }

    #[must_use]
    pub const fn kind(&self) -> ObjectKind {
        self.kind
    }

    #[must_use]
    pub const fn is_keyframe(&self) -> bool {
        self.keyframe
    }

    #[must_use]
    pub const fn configuration_epoch(&self) -> u64 {
        self.configuration_epoch
    }

    #[must_use]
    pub const fn deadline(&self) -> Option<&ClockTimestamp> {
        self.deadline.as_ref()
    }

    #[must_use]
    pub const fn capture_timestamp(&self) -> Option<&ClockTimestamp> {
        self.capture_timestamp.as_ref()
    }

    #[must_use]
    pub fn stage_timestamps(&self) -> &[StageTimestamp] {
        &self.stage_timestamps
    }

    #[must_use]
    pub fn dependencies(&self) -> &[ObjectKey] {
        &self.dependencies
    }

    #[must_use]
    pub const fn metadata(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.metadata
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Recalculate and compare the payload hash in the immutable key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PayloadHashMismatch`] when the payload bytes and key
    /// digest differ.
    pub fn verify_payload_hash(&self) -> Result<()> {
        if PayloadHash::digest(&self.payload) == self.key.payload_hash {
            Ok(())
        } else {
            Err(Error::PayloadHashMismatch)
        }
    }

    /// Compare a candidate write with this immutable object.
    #[must_use]
    pub fn compare_write(&self, candidate: &Self) -> WriteDisposition {
        if self.key != candidate.key {
            WriteDisposition::DistinctObject
        } else if self == candidate {
            WriteDisposition::IdempotentReplay
        } else {
            WriteDisposition::IdentityConflict
        }
    }

    /// SHA-256 of the complete canonical envelope, suitable for auditing a retry.
    ///
    /// # Errors
    ///
    /// Returns an error when this object violates a model or size invariant.
    pub fn envelope_hash(&self) -> Result<PayloadHash> {
        Ok(PayloadHash::digest(&crate::wire::encode(self)?))
    }

    /// Validate all model and safety invariants using hard limits.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid identity, hash, kind, ordering, timestamp,
    /// dependency, metadata, or size state.
    pub fn validate(&self) -> Result<()> {
        self.validate_with_limits(Limits::HARD)
    }

    /// Validate with operational limits that are at or below hard ceilings.
    ///
    /// # Errors
    ///
    /// Returns an error when a supplied limit exceeds a hard ceiling or this
    /// object violates an invariant or supplied limit.
    pub fn validate_with_limits(&self, limits: Limits) -> Result<()> {
        limits.validate()?;
        self.key.validate_with_limits(limits)?;
        self.verify_payload_hash()?;
        enforce_limit("payload", self.payload.len(), limits.max_payload_bytes)?;

        if self.keyframe && self.kind != ObjectKind::Media {
            return Err(Error::InvalidField {
                field: "keyframe",
                reason: "only media objects can be keyframes",
            });
        }

        if let Some(deadline) = &self.deadline {
            deadline.validate_with_limits(limits)?;
        }
        if let Some(capture) = &self.capture_timestamp {
            capture.validate_with_limits(limits)?;
        }

        enforce_limit(
            "stage_timestamps",
            self.stage_timestamps.len(),
            limits.max_stage_timestamps,
        )?;
        for stage in &self.stage_timestamps {
            stage.timestamp.validate_with_limits(limits)?;
        }
        ensure_strictly_sorted(&self.stage_timestamps, "stage_timestamps")?;

        enforce_limit(
            "dependencies",
            self.dependencies.len(),
            limits.max_dependencies,
        )?;
        for dependency in &self.dependencies {
            dependency.validate_with_limits(limits)?;
            if dependency == &self.key {
                return Err(Error::SelfDependency);
            }
        }
        ensure_strictly_sorted(&self.dependencies, "dependencies")?;

        enforce_limit(
            "metadata_entries",
            self.metadata.len(),
            limits.max_metadata_entries,
        )?;
        let mut metadata_bytes = 0usize;
        for (key, value) in &self.metadata {
            validate_identifier("metadata_key", key, limits.max_metadata_key_bytes)?;
            enforce_limit(
                "metadata_value",
                value.len(),
                limits.max_metadata_value_bytes,
            )?;
            metadata_bytes = metadata_bytes
                .checked_add(key.len())
                .and_then(|sum| sum.checked_add(value.len()))
                .ok_or(Error::LimitExceeded {
                    field: "metadata",
                    actual: usize::MAX,
                    maximum: limits.max_metadata_bytes,
                })?;
        }
        enforce_limit("metadata", metadata_bytes, limits.max_metadata_bytes)?;

        enforce_limit(
            "envelope",
            crate::wire::encoded_len_unchecked(self),
            limits.max_envelope_bytes,
        )
    }
}

/// Builder that canonicalizes set-like fields before producing an immutable object.
#[derive(Clone, Debug)]
pub struct MediaObjectBuilder {
    key: ObjectKey,
    kind: ObjectKind,
    keyframe: bool,
    configuration_epoch: u64,
    deadline: Option<ClockTimestamp>,
    capture_timestamp: Option<ClockTimestamp>,
    stage_timestamps: Vec<StageTimestamp>,
    dependencies: Vec<ObjectKey>,
    metadata: Vec<(String, Vec<u8>)>,
    payload: Vec<u8>,
}

impl MediaObjectBuilder {
    #[must_use]
    pub fn new(key: ObjectKey, kind: ObjectKind, payload: Vec<u8>) -> Self {
        let configuration_epoch = key.epoch;
        Self {
            key,
            kind,
            keyframe: false,
            configuration_epoch,
            deadline: None,
            capture_timestamp: None,
            stage_timestamps: Vec::new(),
            dependencies: Vec::new(),
            metadata: Vec::new(),
            payload,
        }
    }

    #[must_use]
    pub const fn with_keyframe(mut self, keyframe: bool) -> Self {
        self.keyframe = keyframe;
        self
    }

    #[must_use]
    pub const fn with_configuration_epoch(mut self, configuration_epoch: u64) -> Self {
        self.configuration_epoch = configuration_epoch;
        self
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: ClockTimestamp) -> Self {
        self.deadline = Some(deadline);
        self
    }

    #[must_use]
    pub fn with_capture_timestamp(mut self, capture_timestamp: ClockTimestamp) -> Self {
        self.capture_timestamp = Some(capture_timestamp);
        self
    }

    #[must_use]
    pub fn with_stage_timestamp(mut self, stage_timestamp: StageTimestamp) -> Self {
        self.stage_timestamps.push(stage_timestamp);
        self
    }

    #[must_use]
    pub fn with_dependency(mut self, dependency: ObjectKey) -> Self {
        self.dependencies.push(dependency);
        self
    }

    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Canonicalize set-like fields and build using hard safety ceilings.
    ///
    /// # Errors
    ///
    /// Returns an error for duplicate values, an invalid payload hash or model
    /// invariant, or a hard-limit violation.
    pub fn build(self) -> Result<MediaObject> {
        self.build_with_limits(Limits::HARD)
    }

    /// Canonicalize set-like fields and build using lower operational limits.
    ///
    /// # Errors
    ///
    /// Returns an error for duplicate values, an invalid payload hash or model
    /// invariant, or an invalid/exceeded limit.
    pub fn build_with_limits(mut self, limits: Limits) -> Result<MediaObject> {
        self.dependencies.sort_unstable();
        reject_duplicates(&self.dependencies, "dependencies")?;
        self.stage_timestamps.sort_unstable();
        reject_duplicates(&self.stage_timestamps, "stage_timestamps")?;

        let mut metadata = BTreeMap::new();
        for (key, value) in self.metadata {
            if metadata.insert(key, value).is_some() {
                return Err(Error::DuplicateValue("metadata"));
            }
        }

        let object = MediaObject {
            key: self.key,
            kind: self.kind,
            keyframe: self.keyframe,
            configuration_epoch: self.configuration_epoch,
            deadline: self.deadline,
            capture_timestamp: self.capture_timestamp,
            stage_timestamps: self.stage_timestamps,
            dependencies: self.dependencies,
            metadata,
            payload: self.payload,
        };
        object.validate_with_limits(limits)?;
        Ok(object)
    }
}

fn validate_identifier(field: &'static str, value: &str, maximum: usize) -> Result<()> {
    if value.is_empty() {
        return Err(Error::EmptyField(field));
    }
    enforce_limit(field, value.len(), maximum)?;
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidField {
            field,
            reason: "control characters are not permitted",
        });
    }
    Ok(())
}

pub(crate) fn enforce_limit(field: &'static str, actual: usize, maximum: usize) -> Result<()> {
    if actual > maximum {
        Err(Error::LimitExceeded {
            field,
            actual,
            maximum,
        })
    } else {
        Ok(())
    }
}

fn reject_duplicates<T: Ord>(values: &[T], field: &'static str) -> Result<()> {
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(Error::DuplicateValue(field))
    } else {
        Ok(())
    }
}

fn ensure_strictly_sorted<T: Ord>(values: &[T], field: &'static str) -> Result<()> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(Error::NonCanonicalOrder(field))
    } else {
        Ok(())
    }
}

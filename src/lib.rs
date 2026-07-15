//! Canonical, bounded media-object primitives.
//!
//! This crate owns the immutable object identity and its versioned binary
//! envelope. `RelaySession`, Needletail, `RaptorQ`, storage, and contribution
//! adapters own their respective session, routing, repair, persistence, and
//! protocol responsibilities.

mod error;
mod model;
mod wire;

pub use error::{Error, Result};
pub use model::{
    ClockConfidence, ClockConfidenceLevel, ClockTimestamp, Limits, MediaObject, MediaObjectBuilder,
    ObjectKey, ObjectKind, PayloadHash, Stage, StageTimestamp, WriteDisposition,
};
pub use wire::{decode, decode_with_limits, encode, encoded_len, WIRE_MAGIC, WIRE_VERSION};

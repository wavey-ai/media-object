# media-object

`media-object` is the canonical artifact boundary shared by Wavey contribution,
relay, cache, and playback services. It defines an immutable media-object v1 key,
a bounded model, and a deterministic binary envelope.

The canonical key is:

```text
tenant / stream / track / epoch / group / object / version / SHA-256(payload)
```

That identity stays stable whether an object arrives from a primary parent, a
secondary parent supplying repair symbols, a reliable fetch, or local cache. A
relay can therefore deduplicate and verify the same object across both paths in
the dual-parent DAG.

## Media-control v1

The crate also exposes strict JSON types for the control-plane boundary:

- `SessionMediaIdentityV1` is the complete tenant/session/participant/endpoint/
  contributor namespace with class-specific source, audience, or take scope
- `MediaCapabilityClaimsV1` distinguishes session incarnation, session-wide
  authorization, per-subject grant, policy, optional class authorization,
  binding, and topology generations
- `MediaEndpointDescriptorV1` is a closed, HTTPS-only route locator with no
  token, key, header, cookie, query value, or arbitrary parameter map
- `MediaFrameConfigurationV1` authenticates the complete session/media
  identity, payload format, capture timebase, and recording disposition behind
  one compact binding-scoped reference
- `MediaFrameEnvelopeV1` carries that reference plus exact sequence, mandatory
  capture PTS, duration, configuration epoch, and payload length on every
  high-rate frame.

Opaque IDs are typed, bounded ASCII tokens. Constructors canonicalize set-like
scopes. JSON input must already use canonical ordering and rejects unknown
fields. The bounded `from_json_slice` entry points reject objects above 64 KiB
before parsing. Network consumers should use those entry points rather than
calling Serde directly. `authorize` compares claims with authenticated current verifier state,
including exact generations and a maximum five-second clock-skew allowance.

Capability lifetime is capped at 90 seconds. Signature/header, key selection,
and replay checks remain the issuer/verifier's responsibility and happen before
claims authorization.

`MediaFrameEnvelopeV1::resolve` is mandatory before parsing or routing frame
payload. It fails closed unless binding generation, configuration reference,
configuration epoch, and payload bound match the authenticated configuration
exactly. Talkback configurations are fixed to mono 48 kHz Opus and
`monitor_only`. They cannot be relabelled recordable.

Core values and IDs use redacted `Debug` output. Explicit serialization and
identifier `as_str()` access are trust-boundary operations.

## Model

Each `MediaObject` carries:

- one complete `ObjectKey`
- an explicit `Media`, `Initialization`, `CodecConfiguration`, or
  `Discontinuity` kind
- keyframe status and codec-configuration epoch
- a playout deadline with clock provenance
- an optional capture timestamp and bounded stage timestamps
- full immutable dependency keys
- sorted, bounded opaque metadata
- bounded payload bytes whose SHA-256 must match the key.

Clock confidence combines a named level with a numeric maximum error in
nanoseconds. This keeps glass-to-glass measurements honest when two stages use
different clocks. `Unknown` explicitly carries no error claim.

Objects are write-once. Stage information known at publication is included in
the object. Later relay, edge, and player observations reference the same
`ObjectKey` and reuse `StageTimestamp` in the observability path. Metadata
enrichment produces a new object version.

## Safety ceilings

The built-in v1 ceilings are:

| Field | Hard ceiling |
| --- | ---: |
| Envelope | 16.25 MiB |
| Payload | 16 MiB |
| Metadata total | 64 KiB |
| Metadata entries | 64 |
| Metadata key | 128 bytes |
| Metadata value | 16 KiB |
| Dependencies | 64 |
| Stage timestamps | 32 |
| Tenant/stream/track identifier | 128 bytes each |
| Clock identifier | 128 bytes |

Services can supply smaller `Limits` for a listener, tenant, or lane. The
decoder validates counts and lengths before owned allocation. The transport
framer should apply the same envelope limit before buffering a complete frame.

## Example

```rust
use media_object::{
    decode, encode, ClockConfidence, ClockTimestamp, MediaObject, ObjectKey,
    ObjectKind,
};

let payload = b"one CMAF object".to_vec();
let key = ObjectKey::for_payload(
    "tenant-a", "live", "video-1080p", 7, 42, 3, 1, &payload,
)?;
let deadline = ClockTimestamp::new(
    1_721_000_000_900_000_000,
    "ptp:grandmaster-1",
    ClockConfidence::synchronized(250_000),
)?;

let object = MediaObject::builder(key, ObjectKind::Media, payload)
    .with_keyframe(true)
    .with_configuration_epoch(3)
    .with_deadline(deadline)
    .build()?;

let envelope = encode(&object)?;
assert_eq!(decode(&envelope)?, object);
# Ok::<(), media_object::Error>(())
```

## Ownership boundaries

This crate owns object identity, validation, canonical ordering, hashing, and the
v1 envelope.

RelaySession owns authenticated carrier sessions, explicit subscriptions,
announcements, datagrams, reliable fetch, congestion feedback, and priority
queues. `raptor-fec` owns adaptive RaptorQ repair and deadline scheduling.
Needletail's controller owns DAG levels, dual-parent selection, bounded child
degree, and failure-domain diversity. Contribution adapters own protocol parsing
and normalization. Cache implementations own retention and atomic write policy.

The object format remains route-neutral. Either parent can deliver source or
repair data for the same verifiable identity. Object semantics remain stable
across routing generations.

The precise binary-object field order and compatibility rules live in
[WIRE.md](WIRE.md). Media-control JSON schemas and the cross-language fixture
corpus live in the adjacent `io.infidelity.docs/specs/media-control/v1`
repository.

## Development

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

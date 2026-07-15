# Media-object binary envelope v1

This file specifies only the immutable `MOBJ` binary envelope. The separate
media-control v1 JSON contracts do not add fields to, or reinterpret, this wire
version.

This document fixes the canonical byte representation for wire version 1. All
integers use network byte order (big-endian). All strings are UTF-8. Byte counts
measure encoded UTF-8 bytes.

## Outer frame

| Field | Encoding | Meaning |
| --- | --- | --- |
| Magic | 4 bytes | ASCII `MOBJ` |
| Wire version | `u16` | `1` |
| Reserved | `u16` | zero |
| Body length | `u32` | exact number of following bytes |
| Body | bytes | v1 fields below |

The body length must equal the available input exactly.

## Body

Fields appear in this order:

1. object kind: `u8`;
2. keyframe: canonical boolean `u8` (`0` or `1`);
3. reserved: zero `u16`;
4. object key;
5. codec-configuration epoch: `u64`;
6. deadline: presence boolean followed by a timestamp when present;
7. capture timestamp: presence boolean followed by a timestamp when present;
8. stage timestamp count: `u16`, followed by that many stage records;
9. dependency count: `u16`, followed by that many object keys;
10. metadata count: `u16`, followed by that many entries;
11. payload length: `u32`, followed by the payload bytes.

Object kind tags are:

| Tag | Kind |
| ---: | --- |
| 0 | Media |
| 1 | Initialization |
| 2 | Codec configuration |
| 3 | Discontinuity |

Only a media object may set the keyframe boolean.

## Object key

An object key is encoded as:

1. tenant: `u16` byte length + UTF-8 bytes;
2. stream: `u16` byte length + UTF-8 bytes;
3. track: `u16` byte length + UTF-8 bytes;
4. epoch: `u64`;
5. group: `u64`;
6. object: `u64`;
7. object version: `u32`;
8. SHA-256 of the exact payload: 32 bytes.

Identifiers are non-empty and contain no Unicode control characters. The hash
is verified after the payload is decoded. Dependencies carry this complete key,
so reliable fetch and repair requests identify immutable bytes precisely.

An epoch identifies a continuous decoding timeline. A discontinuity starts a
new epoch. The codec-configuration epoch identifies the initialization/config
state needed by media objects and can change within the broader stream epoch.

## Clock timestamp

A timestamp is encoded as:

1. signed Unix nanoseconds: `i64`;
2. clock identifier: `u16` byte length + UTF-8 bytes;
3. confidence level: `u8`;
4. maximum-error presence: canonical boolean `u8`;
5. maximum error in nanoseconds: `u64` when present.

Confidence tags are `0` unknown, `1` estimated, `2` synchronized, and `3`
traceable. Unknown confidence carries no maximum error. Every known confidence
level carries one.

A stage record contains its stage tag followed by a clock timestamp:

| Tag | Stage |
| ---: | --- |
| 0 | Ingress received |
| 1 | Normalized |
| 2 | Packaged |
| 3 | Published |
| 4 | Relay received |
| 5 | Relay forwarded |
| 6 | Edge available |
| 7 | Player presented |

## Metadata

Each metadata entry contains:

1. key: `u16` byte length + UTF-8 bytes;
2. value: `u32` byte length + opaque bytes.

Keys are non-empty and contain no Unicode control characters.

## Canonical ordering and idempotency

Dependencies are strictly increasing by the structured `ObjectKey` ordering.
Stage timestamps are strictly increasing by stage, timestamp, clock identifier,
and confidence. Metadata keys are strictly increasing by UTF-8 string ordering.
Duplicate set values are invalid.

The builder sorts these fields. The decoder requires canonical order, making one
valid byte representation for a model value. A retry with the same key and the
same complete envelope is an idempotent replay. Reusing a key with different
immutable metadata is an identity conflict. A changed payload naturally creates
a distinct key because the payload hash is part of identity.

## Decode safety

The decoder applies operational limits before allocating each collection,
string, metadata value, or payload. Operational limits can lower each compiled
hard ceiling. Wire tags, booleans, reserved fields, UTF-8, canonical ordering,
full consumption, model invariants, and the payload hash are all verified.

Transport framing applies the envelope ceiling before buffering the frame.
Relay-session policy can select tighter limits per tenant or subscription lane.

Any field addition or semantic reinterpretation requires a new wire version and
a new golden-vector test. Existing v1 bytes retain their meaning permanently.

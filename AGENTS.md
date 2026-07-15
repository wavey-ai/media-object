# Repository agent notes

## Scope

Keep this crate focused on the canonical media artifact boundary: immutable
identity, model validation, bounded encoding, deterministic decoding, and hash
verification.

Relay transport, dual-parent topology, subscriptions, congestion control,
adaptive repair, cache eviction, protocol parsing, and service policy belong in
their owning crates and services.

## Wire compatibility

- Treat `WIRE.md` and the golden digest test as compatibility contracts.
- Assign new wire tags explicitly and preserve all existing numeric values.
- Introduce a new wire version for field additions or semantic changes.
- Check every network-derived length or count before allocating.
- Keep set-like fields canonical and reject duplicates on decode.
- Preserve complete payload-hash verification.

## Validation

Run before handing off changes:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

The repository uses Rust 1.81 as its minimum supported toolchain.


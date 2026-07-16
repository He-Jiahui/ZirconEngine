---
related_code:
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/serialization/mod.rs
  - zircon_runtime_interface/src/serialization/schema_id.rs
  - zircon_runtime_interface/src/serialization/payload_header.rs
  - zircon_runtime_interface/src/serialization/versioned_schema.rs
  - zircon_runtime_interface/src/serialization/loaded.rs
  - zircon_runtime_interface/src/serialization/text
  - zircon_runtime_interface/src/serialization/binary
  - zircon_runtime_interface/src/serialization/migration
  - zircon_runtime_interface/src/serialization/load.rs
  - zircon_runtime_interface/src/serialization/error.rs
  - zircon_runtime_interface/src/serialization/format.rs
  - zircon_runtime_interface/src/tests/boundary.rs
implementation_files:
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/serialization/mod.rs
  - zircon_runtime_interface/src/serialization/schema_id.rs
  - zircon_runtime_interface/src/serialization/payload_header.rs
  - zircon_runtime_interface/src/serialization/versioned_schema.rs
  - zircon_runtime_interface/src/serialization/loaded.rs
  - zircon_runtime_interface/src/serialization/text/mod.rs
  - zircon_runtime_interface/src/serialization/text/document.rs
  - zircon_runtime_interface/src/serialization/text/envelope.rs
  - zircon_runtime_interface/src/serialization/text/wire.rs
  - zircon_runtime_interface/src/serialization/text/canonical.rs
  - zircon_runtime_interface/src/serialization/binary/mod.rs
  - zircon_runtime_interface/src/serialization/binary/envelope.rs
  - zircon_runtime_interface/src/serialization/binary/wire.rs
  - zircon_runtime_interface/src/serialization/binary/encode.rs
  - zircon_runtime_interface/src/serialization/binary/decode.rs
  - zircon_runtime_interface/src/serialization/binary/value/mod.rs
  - zircon_runtime_interface/src/serialization/binary/value/binary_value.rs
  - zircon_runtime_interface/src/serialization/binary/value/from_json.rs
  - zircon_runtime_interface/src/serialization/binary/value/into_json.rs
  - zircon_runtime_interface/src/serialization/binary/value/error.rs
  - zircon_runtime_interface/src/serialization/migration/mod.rs
  - zircon_runtime_interface/src/serialization/migration/step.rs
  - zircon_runtime_interface/src/serialization/migration/chain.rs
  - zircon_runtime_interface/src/serialization/migration/error.rs
  - zircon_runtime_interface/src/serialization/migration/validate.rs
  - zircon_runtime_interface/src/serialization/migration/execute.rs
  - zircon_runtime_interface/src/serialization/load.rs
  - zircon_runtime_interface/src/serialization/error.rs
  - zircon_runtime_interface/src/serialization/write.rs
  - zircon_runtime_interface/src/serialization/write_error.rs
  - zircon_runtime_interface/src/serialization/format.rs
  - zircon_runtime_interface/src/tests/boundary.rs
plan_sources:
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime_interface/src/serialization/tests/mod.rs
  - zircon_runtime_interface/src/serialization/tests/binary_contract.rs
  - zircon_runtime_interface/src/serialization/tests/binary_malformed_contract.rs
  - zircon_runtime_interface/src/serialization/tests/load_contract.rs
  - zircon_runtime_interface/src/serialization/tests/legacy_detection.rs
  - zircon_runtime_interface/src/serialization/tests/malformed_contract.rs
  - zircon_runtime_interface/src/serialization/tests/migration_contract.rs
  - zircon_runtime_interface/src/serialization/tests/migration_failure_contract.rs
  - zircon_runtime_interface/src/serialization/tests/schema_id_contract.rs
  - zircon_runtime_interface/src/serialization/tests/write_contract.rs
  - cargo test -p zircon_runtime_interface --locked --offline -- --test-threads=1
doc_type: module-detail
---

# Runtime Interface Versioned Serialization

## Purpose

`zircon_runtime_interface::serialization` owns the neutral version envelope and value-domain migration contract shared by runtime, editor, Hub, headless tools, and plugin SDK consumers. It does not own any concrete scene, asset, settings, layout, or editor schema.

## Contract

Each persisted type implements `VersionedSchema` with a stable `SchemaId`, current `VERSION`, and an explicit static `MigrationChain<Self>`. A `MigrationStep` advances exactly one version and transforms `serde_json::Value`; old Rust DTO definitions are not retained. `Loaded<T>` reports the final value and the original migrated version so an owner can issue a one-time resave notification.

Text payloads reserve one top-level `$zircon` magic member whose value contains `PayloadHeader { schema_id, schema_version }` plus `payload`. A text value without `$zircon` is version zero and must traverse every required migration step; its business schema may therefore own fields named `header` and `payload` without ambiguity. Once `$zircon` is present, document, envelope, and header all reject unknown fields. Schema mismatch, versions newer than the reader, malformed text/envelopes, payload decode failures, or invalid migration tables return distinct typed errors instead of silently guessing.

Every load validates the complete migration table before payload decoding, including current-version payloads that require no migration. A schema at version `N` must declare exactly one step for each source version in the ordered range `0..N`; missing, duplicate, out-of-order, and extra steps are rejected. A failing step is wrapped with schema id, source version, and the original typed cause.

## Format Boundary

`Format::Text` uses canonical pretty JSON for authoring. `Format::Binary` is a permanent M3 wire intended for cooked artifacts and other compact exchange paths. Both formats contain the same `PayloadHeader` and serialize through the same `serde_json::Value` migration domain, so a schema owns one migration chain instead of separate text and binary histories.

M1.2 adds the canonical text writer. It serializes the same `$zircon` envelope used by the loader, recursively orders object members, uses `serde_json`'s shortest round-trippable finite-number representation, emits pretty JSON, and normalizes every successful document to exactly one trailing newline.

The binary wire begins with the fixed eight-byte `ZRPAYLD\0` magic and a little-endian `u16` wire version. The body is bincode 1.3 with varint integers, little-endian scalars, bounded decode, and trailing-byte rejection explicitly selected in `binary/wire.rs`; no library default is part of the compatibility contract. The body field order is permanently `PayloadHeader` followed by `BinaryValue`. Decode reads and validates the header first, so a future schema version is rejected before any untrusted payload value stream is deserialized.

`BinaryValue` is a flat, self-describing node stream rather than a recursive Serde enum. The fixed v1 variant order distinguishes null, boolean, signed `i64`, unsigned `u64`, finite `f64`, string, array header, object header, and object key. There is no separate decimal variant: JSON's canonical numeric domain is represented only by `i64`/`u64`/finite `f64`, which keeps every production wire path reachable and deterministic. Object keys are canonicalized before encoding; duplicate keys and non-finite malicious numbers are rejected during decode rather than normalized silently. The writer performs a format-independent Serde traversal before JSON conversion and rejects `NaN`/positive infinity/negative infinity with `WriteError::NonFiniteFloat`, preventing `serde_json` from silently turning them into `null` for either Text or Binary output.

The v1 writer and decoder apply the same explicit resource ceilings: binary body at most 64 MiB, string/key at most 16 MiB, one container at most 1,000,000 entries, one payload at most 2,000,000 nodes, and nesting depth at most 128. The writer applies the 64 MiB limit to bincode serialization and returns `WriteError::BinaryPayloadTooLarge`; it cannot emit a file that the same-version reader rejects solely for body size. The flat node representation avoids recursive bincode deserialization; the iterative reconstruction stack enforces container completeness, exactly one root, object-key placement, duplicate-key rejection, and depth before typed payload decode. These limits are part of the wire-v1 acceptance contract and require a deliberate wire-version decision if changed incompatibly.

Direct `bincode<T>` was rejected because an old binary shape could not be reconstructed without retaining old Rust DTOs. Embedding canonical JSON bytes was rejected because it would preserve text size and parsing cost. The selected value-domain adapter follows Bevy's reflected Postcard round-trip pattern while retaining the explicit magic/version discipline used by Godot binary resources and Fyrox `Visitor`; bincode was already pinned in this workspace and therefore becomes a production dependency without adding a second compact codec.

## Structure And Error Discipline

The root `serialization/mod.rs` contains only child declarations and selected exports. Schema id, header, schema trait, and loaded result each have one declaration owner. Text wire identity/document/envelope live under `text/`. Binary prefix, envelope, encoding, decoding, value declaration, and value conversions each have a narrow owner under `binary/`. Migration step/chain/error/validation/execution live under `migration/`; loading, format, and typed load/write errors remain separate. Behavior tests are folder-backed under `serialization/tests/`. Production loading contains no `unwrap`, panic fallback, schema alias, compatibility reader, implicit migration skip, or table-order fallback.

## Current Validation

The M1 managed `zircon_runtime_interface` package gate covered the text envelope and migration-chain contracts. M3.1 adds contracts for current binary round-trip, Text→Binary→Text canonical equivalence, every JSON scalar class, deterministic output, representative bulk-size comparison, legacy binary migration, schema mismatch, truncated or invalid prefix, future wire/schema versions, trailing bytes, duplicate object keys, and non-finite values. A Layout15 current-source build exposed 11 E0364/E0365/E0603 diagnostics because the value leaf declarations were narrower than their parent re-exports; both declarations and the value-owner re-exports now use `pub(in crate::serialization)`, while binary-root test helpers remain `cfg(test)` and the public API remains unchanged. Moving bincode from a test-only codec to the production binary backend initially tripped the manifest boundary guard; `src/tests/boundary.rs` now explicitly classifies it as an allowed serialization dependency and permits no dev dependencies.

Windows managed job `8be9b8f3a08f4b7f85da9115c65637ca` performed the original fresh-target M3.1 gate: 263 passed / 0 failed, binary subset 13/13, dependency boundary 1/1, and doc-tests passed. The deterministic 256-row selection fixture encoded to 12094 binary bytes versus 32554 canonical-text bytes, a 62.85% reduction.

The first hardening review identified three Important gaps: recursive/unbounded payload allocation, implicit enum/field wire layout without a golden contract, and non-finite floats silently normalizing to JSON `null`. Windows managed reservation `216fc15a3e6e4079b2016d3899bb88b3`, job `6d91251a318e48448da30ef199979aa6`, run `5aae44f0adb94c81a4c6832f7ca04850` verified those fixes with 277 library tests and 3 integration tests passing. Re-review closed all three but found one further Important read/write asymmetry: only the reader enforced the 64 MiB body ceiling. TDD job `f2b9ae3ae8e544538f54506f1e0ed6d6` / run `7ed714818064494b91bdfe67da0b8d1b` first failed on the missing typed writer error; job `7e80283433204fa69c532680ec0a2036` / run `aba491a1a1c440edb4ef8c8ad5fe5d2f` then passed the focused writer-limit contract. Final managed job `30d14024f27c4c26a32e7537c93b1bd9` / run `08c3a06c27d1470d9e5aa41d7066b29d` executed `cargo test -p zircon_runtime_interface --locked --offline -- --test-threads=1`: 278 library tests passed, 3 integration tests passed, 0 failed, and doc-tests passed. Final independent review reported Critical/Important/Minor `0/0/0` for exact24 hash `70e0389e5fa650e5a16fb742c2a5f5dbbd25de171dc6dbfac2d1a68c7d2e8e4e`. The contracts now lock exact v1 bytes across the header and every node variant, prove the future-schema header wins over an invalid payload body, enforce symmetric body limits, reject excessive nesting, and reject non-finite floats in both writers. Plan 11 M3 still requires the M3.2 cook consumer, cross-schema conversion matrix, 5k-entity measurements, and overall milestone acceptance; M3.1's permanent wire slice is accepted.

## Reference Evidence

- Bevy `dev/bevy/crates/bevy_world_serialization/src/serde.rs` and `dev/bevy/crates/bevy_reflect/src/serde/ser/mod.rs` prove reflected values can round-trip through a compact non-self-describing codec when the reflection layer supplies explicit value/type structure.
- Godot `dev/godot/core/io/resource_format_binary.cpp` keeps binary magic and format version explicit and rejects future formats; `dev/godot/tests/core/io/test_resource.cpp` verifies equivalent resource behavior through binary and text saves.
- Fyrox `dev/Fyrox/fyrox-core/src/visitor/mod.rs` separates binary and collaborative text formats behind one visitor value model, prefixes each format with magic, and documents binary as the production-size path; its derive tests save both representations and load the binary result.

---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/serialization/mod.rs
  - zircon_runtime_interface/src/serialization/schema_id.rs
  - zircon_runtime_interface/src/serialization/payload_header.rs
  - zircon_runtime_interface/src/serialization/versioned_schema.rs
  - zircon_runtime_interface/src/serialization/loaded.rs
  - zircon_runtime_interface/src/serialization/text
  - zircon_runtime_interface/src/serialization/migration
  - zircon_runtime_interface/src/serialization/load.rs
  - zircon_runtime_interface/src/serialization/error.rs
  - zircon_runtime_interface/src/serialization/format.rs
implementation_files:
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
plan_sources:
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime_interface/src/serialization/tests/mod.rs
  - zircon_runtime_interface/src/serialization/tests/load_contract.rs
  - zircon_runtime_interface/src/serialization/tests/legacy_detection.rs
  - zircon_runtime_interface/src/serialization/tests/malformed_contract.rs
  - zircon_runtime_interface/src/serialization/tests/migration_contract.rs
  - zircon_runtime_interface/src/serialization/tests/migration_failure_contract.rs
  - zircon_runtime_interface/src/serialization/tests/schema_id_contract.rs
  - zircon_runtime_interface/src/serialization/tests/write_contract.rs
  - cargo test -p zircon_runtime_interface --locked --offline serialization -- --test-threads=1
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

`Format::Text` is implemented in M1.1. `Format::Binary` is present in the public entry point but returns `LoadError::UnsupportedFormat` until Plan 11 M3 performs the required encoding benchmark and selects the permanent fixed-header binary representation. This deliberately avoids creating a temporary binary wire format that would later require a compatibility reader.

M1.2 adds the canonical text writer. It serializes the same `$zircon` envelope used by the loader, recursively orders object members, uses `serde_json`'s shortest round-trippable finite-number representation, emits pretty JSON, and normalizes every successful document to exactly one trailing newline. `Format::Binary` remains explicitly unavailable and returns a typed `WriteError::UnsupportedFormat`.

## Structure And Error Discipline

The root `serialization/mod.rs` contains only child declarations and selected exports. Schema id, header, schema trait, and loaded result each have one declaration owner. Text wire identity/document/envelope live under `text/`; the parent loader addresses the private `text::document` and `text::wire` children explicitly, avoiding an invalid restricted re-export while keeping those wire details out of the public interface. Migration step/chain/error/validation/execution live under `migration/`; loading, format, and load errors remain separate. Behavior tests are folder-backed under `serialization/tests/`. Production loading contains no `unwrap`, panic fallback, schema alias, implicit migration skip, or table-order fallback.

## Current Validation

The initial TDD run failed only on the missing public serialization contracts. Independent review then drove explicit RED/GREEN coverage for legacy field-name ambiguity, duplicate/out-of-order/extra migration steps, current-version bad-chain validation, strict header fields, phase-specific load errors, owned `SchemaId`, step-failure context, deterministic member ordering, shortest float text, and the binary write rejection. The managed `zircon_runtime_interface` package gate passed after correcting the private canonicalization helper visibility.

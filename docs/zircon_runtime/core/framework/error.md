---
related_code:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
implementation_files:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/mod.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Framework Error Contracts

`zircon_runtime::core::framework::error` owns the shared runtime error contracts. `CoreError` and `ZirconError` are not concrete runtime behavior; they cross framework traits, runtime descriptors, manager access, config loading, event publishing, and thread-spawn helpers.

The 2026-06-12 runtime 02 M2.1 migration moved the former `core/error.rs` root fragment into this framework owner. The curated `zircon_runtime::core::{CoreError, ZirconError}` facade remains because these error types are stable public contracts used across runtime modules and downstream callers.

## Ownership Boundary

- `ZirconError` covers low-level shared failures such as channel send and thread spawn errors.
- `CoreError` covers runtime registration, dependency, resolution, lifecycle, downcast, and config errors.
- `core::framework::error` defines only the error enums and formatting. It does not own registry storage, lifecycle transitions, service factories, or config parsing behavior.
- Runtime implementation files import the facade through `crate::core::CoreError`, so the physical owner can remain framework-owned without reviving `crate::core::error` as a root fragment.

## Validation

The root-surface guard rejects a revived `mod error;`, `pub use error::...`, or retired `src/core/error.rs` file. Source scans also reject `crate::core::error` / `zircon_runtime::core::error` imports after the migration.

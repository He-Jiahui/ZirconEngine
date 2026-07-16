---
related_code:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
implementation_files:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/prelude.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - tools/tests/test_frameworks_02_core_error_single_source.py
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Framework Error Contracts

`zircon_runtime::core::framework::error` owns the single shared runtime error contract, `CoreError`. It is not concrete runtime behavior; it crosses framework traits, runtime descriptors, manager access, config loading, event publishing, channel admission, and thread-spawn helpers.

The 2026-06-12 runtime 02 M2.1 migration moved the former `core/error.rs` root fragment into this framework owner. Frameworks 02 M1 later completed the direct cutover to the curated `zircon_runtime::core::{CoreError, CoreResult}` facade: channel-send and thread-spawn failures are variants of `CoreError`, and the retired parallel error enum has no export, alias, or compatibility wrapper.

## Ownership Boundary

- `CoreError` covers low-level channel/thread failures together with runtime registration, dependency, resolution, lifecycle, downcast, resource, and config errors.
- `core::framework::error` defines only the canonical enum, result alias, and formatting. It does not own registry storage, lifecycle transitions, service factories, task execution, asset admission, or config parsing behavior.
- Runtime implementation files import the facade through `crate::core::CoreError`, so the physical owner can remain framework-owned without reviving `crate::core::error` as a root fragment.

## Validation

The root-surface guard rejects a revived `mod error;`, `pub use error::...`, or retired `src/core/error.rs` file. The Frameworks 02 single-source guard scans every live Rust source under `zircon_runtime/src`, requires the channel/thread variants on `CoreError`, and rejects any consumer or export of the retired parallel error type.

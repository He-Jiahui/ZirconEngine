---
related_code:
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
implementation_files:
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/mod.rs
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

# Runtime Lifecycle Vocabulary

`zircon_runtime::core::runtime::lifecycle` owns the service and module lifecycle vocabulary used by runtime registration, activation, deactivation, and resolution. The former `core/lifecycle.rs` root fragment was moved here during runtime plan 02 M2.2.

## Ownership Boundary

- `StartupMode` describes whether a registered service starts immediately or waits for lazy resolution.
- `LifecycleState` describes the runtime state of modules and services: registered, initializing, running, stopping, or unloaded.
- `ServiceKind` is the canonical driver/manager/plugin classifier used by `RegistryName`, dependency validation, and service table logic.
- The curated `zircon_runtime::core::{LifecycleState, StartupMode, ServiceKind}` facade remains because these types are public runtime vocabulary, but the physical owner is now the runtime kernel.

The lifecycle module defines vocabulary only. Registration ordering, dependency validation, activation, deactivation, and resolution behavior stay in their existing runtime handle and descriptor owners.

## Validation

The root-surface guard rejects a revived `mod lifecycle;`, `pub use lifecycle::...`, or retired `src/core/lifecycle.rs` file. Source scans reject `crate::core::lifecycle` and `zircon_runtime::core::lifecycle` imports after the migration.

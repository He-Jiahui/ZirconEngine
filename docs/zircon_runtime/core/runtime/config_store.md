---
related_code:
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
implementation_files:
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/core/runtime/config_store.rs::config_store_accessors_recover_poisoned_values_lock
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store
doc_type: module-detail
---

# Runtime Config Store

## Purpose

`ConfigStore` is the in-memory JSON backing store for runtime configuration values. `CoreRuntimeInner` owns one instance, and `CoreHandle` exposes `store_config_value`, `load_config_value`, `snapshot_config_values`, `store_config`, and `load_config` as the public runtime-facing accessors.

The store owns raw `serde_json::Value` entries keyed by string. Typed `store<T>` and `load<T>` only perform serialization/deserialization at this owner boundary and continue to report `CoreError::ConfigParse` or `CoreError::MissingConfig`.

## Ownership Boundary

`core::runtime::config_store` owns the runtime backing store. Foundation config modules may wrap the `CoreHandle` API for module registration and file IO, but they do not own the in-memory map.

The values map is private to `ConfigStore`. Callers must use the store/load/snapshot methods rather than opening the mutex directly.

## Poison Handling

Runtime 15 M3 config store lock poison recovery / `runtime_15_config_store_lock_poison_recovery_static_passed_cargo_deferred` adds the private `lock_values()` helper. `store_value`, `load_value`, and `snapshot_values` now recover a poisoned values mutex with `unwrap_or_else(|poisoned| poisoned.into_inner())` instead of panicking the runtime configuration path.

`config_store_accessors_recover_poisoned_values_lock` covers the module-local recovery path. `structure_convention/lock_poison_policy.rs::runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store` rejects production direct lock unwrap and keeps the helper plus status anchors from drifting.

## Validation

Scoped static validation for the 2026-06-24 Runtime 15 slice used rustfmt on touched Rust files, production direct-lock scans, docs/status/date/session anchor scans, trailing-whitespace scans, and scoped `git diff --check`. Cargo validation was deferred because external Cargo/Rust lanes were active, so no package-level Cargo pass is claimed for this slice.

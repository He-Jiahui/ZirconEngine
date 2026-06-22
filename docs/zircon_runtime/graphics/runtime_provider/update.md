---
related_code:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/update.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/update.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs::runtime_15_provider_update_uses_shared_stats_owner
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime Provider Update

`zircon_runtime::graphics::runtime_provider::update` owns the shared stats storage used by runtime-provider update wrappers. It stays crate-private because the feature-specific types remain the public and crate-visible contract: `HybridGiRuntimeUpdate` and `VirtualGeometryRuntimeUpdate`.

`RuntimeProviderUpdate<S>` stores the update stats payload once and exposes shared construction/access behavior. `define_runtime_provider_update!` generates the provider-specific wrappers while preserving their existing public shape:

- `HybridGiRuntimeUpdate::stats()` still returns `HybridGiRuntimeStats` by value.
- `VirtualGeometryRuntimeUpdate::stats()` still returns `&VirtualGeometryRuntimeStats`.

This closes only the F13 runtime-update stats sub-slice. Runtime-feedback shared payload storage is covered by `docs/zircon_runtime/graphics/runtime_provider/feedback.md`; shared prepare-input frame context is covered by `docs/zircon_runtime/graphics/runtime_provider/prepare_input.md`.

Runtime 15 F13 provider update shared stats owner status: `runtime_15_provider_update_shared_stats_owner_coremin_check_passed`. Guard: `runtime_15_provider_update_uses_shared_stats_owner`. Validation boundary: scoped rustfmt and core-min `cargo check` passed; standalone guard/status-output binary startup was blocked by Windows `ResourceUnavailable` / user-cancel state, and the focused Cargo test timed out without a result.

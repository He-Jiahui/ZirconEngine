---
related_code:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider_registration.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs::runtime_15_provider_registration_uses_shared_owner
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime Provider Registration

`zircon_runtime::graphics::runtime_provider` owns the shared registration storage for runtime-backed render providers. It is graphics-internal infrastructure, not a framework DTO: the public provider-specific surfaces stay `HybridGiRuntimeProviderRegistration`, `VirtualGeometryRuntimeProviderRegistration`, and `SolariRuntimeProviderRegistration`.

`RuntimeProviderRegistration<P: ?Sized>` stores the common registration fields:

- provider ID
- integer priority
- provider trait object
- provider-specific debug struct name

The provider-specific registration modules now use `define_runtime_provider_registration!` to generate the public wrapper methods. This keeps the existing constructor and accessor surface (`new`, `provider_id`, `priority`, `with_priority`, `provider`) while preventing HGI, Virtual Geometry, and Solari from each owning their own copy of the same storage and debug boilerplate.

This closes only the F13 registration sub-slice. Runtime-update stats storage is covered by `docs/zircon_runtime/graphics/runtime_provider/update.md`; shared feedback payload storage is covered by `docs/zircon_runtime/graphics/runtime_provider/feedback.md`; shared prepare-input frame context is covered by `docs/zircon_runtime/graphics/runtime_provider/prepare_input.md`.

Runtime 15 F13 provider registration shared owner status: `runtime_15_provider_registration_shared_owner_coremin_check_passed`. Guard: `runtime_15_provider_registration_uses_shared_owner`.

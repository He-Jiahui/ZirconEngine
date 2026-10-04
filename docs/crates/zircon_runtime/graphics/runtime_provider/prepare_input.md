---
related_code:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate.rs::runtime_15_provider_prepare_input_uses_shared_extract_generation_owner
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime Provider Prepare Input

`zircon_runtime::graphics::runtime_provider::prepare_input` owns the common frame context used by provider-specific prepare input wrappers. It stays crate-private because HGI and Virtual Geometry still expose their existing public prepare input types and constructor/getter surface.

`RuntimeProviderPrepareInput<'a, E>` stores:

- optional frame extract reference
- frame generation

Feature-specific prepare input wrappers keep fields whose lifetime and meaning differ by renderer. HGI keeps mesh snapshots, directional/point/spot lights, and its visibility update plan. Virtual Geometry keeps its page upload plan, visible clusters, and draw segments. This avoids forcing unlike provider inputs into one false common payload while removing the duplicated extract/generation storage and accessors.

This closes only the F13 prepare-input shared frame owner sub-slice. Registration, update stats, and feedback payload shared owners are covered by sibling runtime-provider docs; the full `runtime_15_no_duplicated_provider_boilerplate` / `module_convention_gate` audit remains pending.

Runtime 15 F13 provider prepare input shared frame owner status: `runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed`. Guard: `runtime_15_provider_prepare_input_uses_shared_extract_generation_owner`. Validation: scoped rustfmt, standalone structure guard 1/1, standalone status-output all-subplans guard 1/1, and core-min `cargo check` passed with existing warnings. The broader `status_output` batch still has an unrelated Runtime 06 F8 row-drift failure outside this slice.

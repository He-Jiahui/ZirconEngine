---
related_code:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/feedback.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/feedback.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs::runtime_15_provider_feedback_uses_shared_payload_owner
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime Provider Feedback

`zircon_runtime::graphics::runtime_provider::feedback` owns the shared payload pair used by provider runtime feedback wrappers. It is graphics-internal infrastructure: HGI and Virtual Geometry still expose the public feedback types consumed by render submission and tests.

`RuntimeProviderFeedback<G, V>` stores:

- optional GPU completion payload
- optional visibility feedback payload

Feature-specific wrappers keep their own provider-only fields. HGI keeps `evictable_probe_ids`; Virtual Geometry keeps `node_and_cluster_cull_page_requests`, `evictable_page_ids`, and `generation`. This avoids flattening unlike lifecycle data into a false common type while still removing the duplicated common feedback payload storage and accessors.

This closes only the F13 runtime-feedback shared payload sub-slice. Shared prepare-input extract/generation storage is covered by `docs/zircon_runtime/graphics/runtime_provider/prepare_input.md`.

Runtime 15 F13 provider feedback shared payload owner status: `runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed`. Guard: `runtime_15_provider_feedback_uses_shared_payload_owner`. Validation: scoped rustfmt, standalone structure guard, standalone status-output guards, and core-min `cargo check` passed with existing warnings.

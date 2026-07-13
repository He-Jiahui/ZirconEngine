---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/depth_prepass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/opaque_base.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/transparent.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/taa_reactive_mask.rs
plan_sources:
  - docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
doc_type: module-detail
---

# Mesh pass processors

## Responsibility

Mesh pass processors are the last material-aware filter before renderer command
creation. Each processor maps one `MeshBatchRef` into zero or one command for its
render phase, selecting a `MeshPassPipelineKind` and resolving a shader variant from
the batch's `PipelineKey`.

The processors do not rebuild shader or material state. The batch already contains
the resolved queue phase, disabled-pass mask, material bindings, visibility, and
pipeline key produced by the asset/resource path.

## Material contract

- The resolved material queue selects opaque, alpha-mask, or transparent command
  routing before sort-key ordering.
- `MaterialDisabledPasses` suppresses base, depth, shadow, velocity, and TAA material
  mask commands at the owning processor.
- `PipelineKey.material_option_bits` is copied into `ShaderVariantKey`, so two
  otherwise identical materials with different authored options receive distinct
  shader variants.
- Disabled passes do not allocate a variant or append a draw command.
- Resolving an already registered pass and pipeline shape records a memory hit rather
  than a compile miss.

## Combined acceptance guard

`render_material_options_disabled_passes_and_queue_drive_mesh_commands_together`
exercises the three contracts in one command list. It creates default and option-bit
opaque materials, disables shadow only on the option material, and adds a transparent
material. The test verifies phase ordering, option-bit preservation, one shadow
command, three unique variants, one memory reuse hit, and zero compile misses.

Keeping these assertions together prevents individually correct option packing,
disabled-pass filtering, and queue routing from drifting at their integration point.

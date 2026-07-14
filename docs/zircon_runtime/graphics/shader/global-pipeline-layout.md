---
related_code:
  - zircon_runtime/src/core/framework/render/shader/compute_dispatch.rs
  - zircon_runtime/src/core/framework/render/shader/fullscreen_pass.rs
  - zircon_runtime/src/graphics/shader/builtin_global_shader_contracts.rs
  - zircon_runtime/src/graphics/shader/global_pipeline_layout.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_fullscreen_triangle.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/hzb.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/motion_vector_tile_max.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/hzb_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/motion_vector_tile_max_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/execute_hzb_build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_motion_vector_tile_max/execute_motion_vector_tile_max.rs
implementation_files:
  - zircon_runtime/src/graphics/shader/builtin_global_shader_contracts.rs
  - zircon_runtime/src/graphics/shader/global_pipeline_layout.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_fullscreen_triangle.wgsl
plan_sources:
  - user: 2026-07-14 complete the Shader architecture plan and prioritize code-structure/review findings
  - docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/graphics/shader/builtin_global_shader_contracts.rs
  - zircon_runtime/src/graphics/shader/global_pipeline_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_hzb_build/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process/motion_blur.rs
doc_type: module-detail
---

# Global Shader Pipeline Layout

## Purpose

This module closes the boundary between the backend-independent compute/fullscreen
authoring contracts and WGPU pipeline construction. A `ComputeDispatchPlan` or
`FullscreenPassPlan` owns resource names, access modes, generated group/binding
indices, entry points, workgroup sizes, and cache identity. The graphics layer adds
only the backend details that the schema cannot infer, such as depth sampling,
multisampling, texture view dimension, and storage texture format.

The split prevents feature descriptors, WGPU bind-group layouts, executors, and WGSL
from maintaining independent binding-number tables.

## Behavior Model

`builtin_global_shader_contracts.rs` is the single contract owner for the migrated
samples:

- HZB compute reserves group 0 binding 0 for generated parameters and assigns
  `scene-depth`, `source_hzb`, and `target_hzb` to bindings 1, 2, and 3.
- Single-sample and multisample HZB use distinct option bits and pipeline labels but
  share the same named-resource ABI.
- Motion-vector tile-max uses the fullscreen ABI: frame group 0 is reserved, pass
  inputs begin at group 1 binding 0, and the vertex entry is
  `zr_fullscreen_triangle_vs`.

`global_pipeline_layout.rs` projects those names into `wgpu::BindGroupLayoutEntry`
values. It returns `GlobalShaderPipelineLayoutError` for missing, unknown, duplicate,
wrong-kind, or wrong-group resource descriptions before WGPU object creation. This is
the typed-error boundary required by the June review findings.

## Runtime Flow

1. A feature descriptor obtains the shared HZB or fullscreen plan.
2. Render-graph workload metadata uses the plan's pipeline label, workgroup size, and
   dispatch extent.
3. WGPU construction supplies backend texture details by resource name; generated ABI
   indices come only from the plan.
4. The executor looks up each binding by resource name and binds at the generated
   group/binding index.
5. HZB WGSL follows the generated compute order. Motion-vector fragment WGSL contains
   no vertex stage and is assembled with the shared fullscreen-triangle WGSL.

## Design Constraints

- Backend texture shape/format remains in `graphics`; it is not leaked into the core
  shader contract.
- `with_*` methods on compute/fullscreen builders consume and return `Self`, keeping
  validation exclusively in `build()` as required by the builder convention.
- Built-in pipeline constructors may treat a rejected static contract as an invariant
  violation, but reusable WGPU projection APIs return typed errors.
- The migrated motion-vector shader declares no fullscreen parameters, so its pipeline
  omits the unused frame group 0 and parameter group 2. A reusable upload path for a
  non-empty `FullscreenPassPlan::parameters` block and backend-construction errors
  forwarded into `FrameDiagnostics` remain separate SH04-M3 work; this sample does not
  claim those paths.
- No compatibility wrapper remains for the deleted feature-local fullscreen contract.

## Test Coverage

The module tests verify generated HZB and fullscreen ABI indices plus missing-resource
and wrong-kind WGPU diagnostics. The HZB WGPU regression renders a nonuniform 4x4 depth
pattern, gives each 4x MSAA sample a distinct value, then asserts furthest, closest, and
span channels independently for both mip levels through the production contract-derived
layout. The motion-blur product regression executes the fullscreen tile-max pass in the
complete post-process chain and checks a nonzero final-frame delta.

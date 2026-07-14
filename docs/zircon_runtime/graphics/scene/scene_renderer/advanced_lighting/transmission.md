---
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_usage.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/screen_space_transmission.rs
  - zircon_runtime/src/graphics/pipeline/declarations/advanced_pbr_pass_contract.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/material_feature_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/transmission
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/material_feature_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/transmission/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/transmission/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/transmission/steps.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
plan_sources:
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
tests:
  - render_graph_compile_frame_fingerprint_tracks_compile_extract_inputs
  - render_advanced_material_scene_copy_is_absent_without_specular_transmission
  - render_advanced_material_scene_copy_runs_after_sky_before_transparency
  - render_advanced_opaque_forward_runs_after_sky_before_scene_copy
  - render_advanced_lighting_material_usage_ignores_meshes_outside_selected_camera_layers
  - frame_binder_reuses_fixed_scene_color_and_depth_targets
  - render_screen_space_transmission_settings_normalize_step_budget
  - render_transmission_steps_partition_commands_without_overlap
  - render_transmission_steps_do_not_emit_empty_ranges
  - render_advanced_material_transmission_steps_alternate_copy_and_nonoverlapping_draws
  - render_advanced_material_zero_copy_steps_keep_one_environment_only_draw
  - render_transmission_zero_step_fallback_marks_scene_copy_unavailable
  - render_product_advanced_pbr_three_spheres_execute_owned_passes
  - export_render_product_advanced_pbr_three_spheres_png
doc_type: module-detail
---

# Screen-Space Transmission

This module owns the WGPU resources and graph-execution contract required by
Standard PBR specular transmission. The renderer-neutral material contract and
normalization rules remain in `core::framework`; this module starts where a
view-local extract has established that visible material usage needs a
scene-color copy.

## Activation And Graph Shape

Material-usage extraction filters mesh instances through the selected camera's
render-layer mask before setting `requires_scene_color_copy`. That bit is part
of `RenderGraphCompileFrameFingerprint`, preventing reuse of a graph compiled
for a view with a different transmission requirement.

When no view-visible material uses transmission, the compiled graph contains
no transmission node or transient texture. A diffuse-only transmission view,
or a specular view configured with zero copy steps, receives one
environment-only transmission draw. Positive step budgets emit alternating
`transmission.scene_copy[.N]` and `transmission-mesh.N` passes after sky and the
late forward opaque material pass, then before ordinary transparent/OIT work.
Each copy reads fixed `SCENE_COLOR` and writes
`TRANSMISSION_SCENE_COLOR`; declaring the source as a read is required for
correct graph dependency and lifetime analysis.

## Physical Texture Ownership

`SCENE_COLOR` is a renderer-owned imported target rather than a graph-owned
transient. Frame binding therefore supplies three related values under the
same graph name:

- the borrowed texture view used by render attachments and sampled bindings;
- the physical `wgpu::Texture` owner required by texture-copy encoding;
- the physical `TextureDesc` used for format, sample-count, extent, and bounds
  validation.

The execution-resource table does not report this imported texture as an owned
transient. `TRANSMISSION_SCENE_COLOR` is graph-owned, single-sampled
`Rgba16Float`, and exactly matches the view-local render extent. The copy
executor rejects multisampled, format-mismatched, depth, or out-of-bounds
source/destination pairs before recording `copy_texture_to_texture`.

## Shader Binding

The material bind-group layout reserves binding `31` for the copied
scene-color `texture_2d<f32>` and binding `32` for its filtering sampler. A
one-pixel fallback texture keeps non-transmission variants bind-compatible;
only the `PBR_TRANSMISSION` variant samples the copied resource through
`zr_pbr_extras.wgsl`. The fallback texel is transparent black: alpha zero is
the explicit "scene copy unavailable" marker. The shader then uses the
already-evaluated environment lighting as the transmission source. A physical
scene-color copy carries positive alpha and selects the refracted scene sample,
so zero-step quality does not darken the material through an opaque black
placeholder.

## Step Execution

The authorable step budget is clamped to four. The transmission queue is
separate from late-forward opaque and ordinary transparent command lists while
retaining queue value `2900` and back-to-front sorting. A list of `N` commands
is split into at most `min(N, steps)` contiguous, non-overlapping ranges; any
division remainder is assigned to the earliest, back-most ranges.

Every non-empty positive step records one scene-color copy followed by exactly
one range draw. Empty ranges skip both operations. Range streams intentionally
drop the cross-command indirect batch execution because a batch may cross a
step boundary; command-local indirect draws remain valid. Ordinary
transparent, sprites, and OIT commands are never replayed by a transmission
step.

## Product Evidence

The DX12 product fixture renders baseline and advanced views of clearcoat,
anisotropy, and specular-transmission spheres. The advanced view executes
`advanced-pbr-opaque`, `transmission.scene_copy`, and
`transmission-mesh.0` in that order. Region comparisons recorded 6,403 changed
clearcoat pixels, 7,028 changed anisotropy pixels, and 1,876 changed glass
pixels after both baseline and advanced frames enabled clustered lighting.

The side-by-side WGPU product is
`docs/tests/runtime/render/plan18_advanced_pbr_clearcoat_anisotropy_glass_three_spheres_wgpu_20260714.png`
with SHA-256
`606BEB60CD15394D356210B3EF68E27060B1EBE83EDF03A9815DA8AA60592C56`.
The advanced DX12 RenderDoc capture is
`docs/tests/runtime/render/plan18_advanced_pbr_clearcoat_anisotropy_glass_dx12_renderdoc_20260714.rdc`
with SHA-256
`1ECA3ED304AA042131B4674DB7040D1CEB92416399238F32E2B4F953988D24F2`;
`renderdoccmd replay --loops 1` completed successfully with RenderDoc 1.44.
The capture runs the advanced-only fixture after the clustered-lighting
correction, so it cannot stop on the baseline frame first.

---
related_code:
  - dev/bevy/crates/bevy_light/src/lib.rs
  - dev/bevy/crates/bevy_light/src/ambient_light.rs
  - dev/bevy/crates/bevy_light/src/rect_light.rs
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh_view_types.wgsl
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
implementation_files:
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
plan_sources:
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - user: 2026-06-11 vampire runtime point light illumination
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/core/framework/render/light/readiness.rs::light_status_counts_split_ready_and_degraded_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs::pack_light_slices_encodes_directional_shadow_and_layer_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::scene_uniform_packs_authored_point_lights
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::fallback_mesh_shader_applies_shadow_visibility_to_directional_light_and_adds_point_lights
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_receives_scene_point_lights
  - zircon_runtime/src/graphics/tests/render_product_submit.rs::render_product_pbr_submit_reports_material_fallback_and_light_stats
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Light Contracts

`zircon_runtime::core::framework::render::light` owns the neutral light DTOs used by render extraction and submit diagnostics. This makes the light product surface explicit, matching Bevy's split between authoring-facing `bevy_light` components and renderer-facing GPU light data in `bevy_pbr::render::light`.

The module currently defines snapshot rows for directional, point, spot, ambient, rect, reflection-probe, and baked-lighting inputs. `LightingExtract` in `frame_extract.rs` still owns the frame-level aggregation because it combines light rows with reflection, baked lighting, and Hybrid GI sidebands, but the row vocabulary no longer lives in `scene_extract.rs`.

Directional, point, spot, and rect light snapshots carry `layer_mask: RenderLayerSet`. Scene authoring still stores the legacy entity render-layer mask, so `World` wraps that `u32` at the extraction boundary with `RenderLayerSet::from_legacy_mask(...)`. GPU light packing is the only remaining legacy adapter: `light_buffer.rs` writes `layer_mask.to_legacy_mask_lossy()` into `GpuLightData.shadow_slot_layer[1]` to preserve the current shader-facing buffer ABI while the CPU render DTOs use the same typed layer set as cameras and volumes.

Readiness is intentionally conservative, but it now mirrors the light slots that the basic renderer actually shades. The basic Zircon renderer reports one directional light as ready because `SceneUniform` consumes a single directional slot. It also reports up to `BASIC_SCENE_UNIFORM_POINT_LIGHT_LIMIT` point lights as ready because `SceneUniform` packs those point lights into fixed `vec4` arrays and the forward fallback mesh shader plus deferred lighting shader consume them with range falloff. Authored ambient lights are ready when they are not marked renderer-degraded. Extra point lights beyond the fixed uniform limit, spot lights, rect lights, and extra directional lights remain degraded until clustered/Forward+ and area-light shading paths land. `RenderLightReadinessReport` centralizes those counts so submit stats and future diagnostics share one rule instead of duplicating light-family assumptions.

`collect_runtime_diagnostics(...)` mirrors the submit stats into runtime `DiagnosticStore` paths per family: `render.light.directional.*`, `render.light.point.*`, `render.light.spot.*`, `render.light.ambient.*`, and `render.light.rect.*`. Each family has `count`, `ready_count`, and `degraded_count` rows, giving tools a stable readiness surface without treating degraded point/spot/rect rows as implemented shading support.

This does not implement point-light shadows, clustered light lists, storage-buffer light culling, spot lighting, or rectangular area-light shading. Those stay in the PBR and clustered-lighting milestones; this module is the baseline contract that makes the current fixed uniform light support and the remaining renderer gaps visible.

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
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
plan_sources:
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/core/framework/render/light/readiness.rs::light_status_counts_split_ready_and_degraded_slots
  - zircon_runtime/src/graphics/tests/render_product_submit.rs::render_product_pbr_submit_reports_material_fallback_and_light_stats
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Light Contracts

`zircon_runtime::core::framework::render::light` owns the neutral light DTOs used by render extraction and submit diagnostics. This makes the light product surface explicit, matching Bevy's split between authoring-facing `bevy_light` components and renderer-facing GPU light data in `bevy_pbr::render::light`.

The module currently defines snapshot rows for directional, point, spot, ambient, rect, reflection-probe, and baked-lighting inputs. `LightingExtract` in `frame_extract.rs` still owns the frame-level aggregation because it combines light rows with reflection, baked lighting, and Hybrid GI sidebands, but the row vocabulary no longer lives in `scene_extract.rs`.

Readiness is intentionally conservative. The basic Zircon renderer reports one directional light as ready because `SceneUniform` consumes a single directional slot. Authored ambient lights are ready when they are not marked renderer-degraded. Point, spot, rect, and extra directional lights remain degraded until the clustered/Forward+ and area-light shading paths land. `RenderLightReadinessReport` centralizes those counts so submit stats and future diagnostics share one rule instead of duplicating light-family assumptions.

This does not implement shadows, clusters, multi-light GPU buffers, or rectangular area-light shading. Those stay in the PBR and clustered-lighting milestones; this module is the baseline contract that makes the missing renderer work visible.

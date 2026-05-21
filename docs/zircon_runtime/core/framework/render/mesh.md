---
related_code:
  - dev/bevy/crates/bevy_mesh/src/lib.rs
  - dev/bevy/crates/bevy_mesh/src/mesh.rs
  - dev/bevy/crates/bevy_mesh/src/components.rs
  - dev/bevy/crates/bevy_mesh/src/index.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/core/framework/render/mesh/mesh_kind.rs
  - zircon_runtime/src/core/framework/render/mesh/topology.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/core/framework/render/mesh/mesh_kind.rs
  - zircon_runtime/src/core/framework/render/mesh/topology.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
plan_sources:
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
tests:
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_model_metadata_exposes_mesh_bounds_and_vg_presence
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Render Mesh Contracts

## Purpose

`zircon_runtime::core::framework::render::mesh` owns the neutral mesh metadata needed before a concrete renderer chooses vertex layouts, buffers, culling, batching, or material pipeline variants. This follows Bevy's separation between `bevy_mesh::Mesh` asset data and renderer-specific mesh preparation.

The module does not own imported model parsing, scene component storage, GPU buffer allocation, skinning, morph targets, or material binding. It is the shared descriptor surface that assets, scene extraction, product profiles, and diagnostics can use without depending on WGPU implementation details.

## Product Surface

`RenderMeshDescriptor` carries topology, bounds, primitive kind, 2D/3D suitability flags, vertex count, index count, primitive count, and whether a Virtual Geometry payload is present.

`RenderMeshTopology` mirrors the topology family used by Bevy and WGPU: triangle list, triangle strip, line list, line strip, and point list. The descriptor intentionally records topology before pipeline selection so Core2d/Core3d phase queueing can stay independent from concrete index-buffer details.

`RenderMeshBounds` stores min, max, center, and radius. `RenderMeshBounds::from_positions(...)` computes a neutral AABB plus bounding radius from positions; graphics culling and editor debug overlays can consume it without reopening asset payloads.

`RenderMeshKind` distinguishes planar 2D meshes from spatial 3D meshes. Asset projection currently treats all-`z == 0` position sets as `Planar2d`, marks those as suitable for both 2D and 3D, and marks non-planar meshes as `Spatial3d`.

## Asset Projection

`MeshAsset::render_mesh_descriptor()` projects authored mesh attributes into the neutral descriptor after validating that position data exists. `ModelPrimitiveAsset::render_mesh_descriptor()` projects model primitives and records Virtual Geometry payload presence.

This keeps model import and mesh validation in the asset layer, while the render framework sees only the product metadata needed for phase selection, fallback diagnostics, and future render asset readiness reporting.

## Current Limits

This is not a full Bevy `MeshPlugin`. It does not register mesh assets, mark changed `Mesh3d` entities, prepare GPU mesh slabs, or own `Mesh2d`/`Mesh3d` render components. `Mesh2dComponent` exists as scene data, but materialized Mesh2d draw execution remains future product work.

The descriptor does not yet expose vertex attribute layouts, morph targets, skinning inverse bind poses, tangent generation, or per-attribute upload readiness. Those belong in later mesh asset and renderer preparation milestones.

## Test Coverage

`render_product_assets_model_metadata_exposes_mesh_bounds_and_vg_presence` proves model primitives project topology, planar kind, 2D/3D suitability, counts, bounds, and Virtual Geometry presence.

The sprite/2D tests prove `Mesh2dComponent` is stored as 2D scene data without being misclassified as a sprite product path. That separation is intentional until Mesh2d rendering has its own acceptance slice.

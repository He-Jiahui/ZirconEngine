---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/attribute.rs
  - zircon_runtime/src/asset/assets/mesh/constants.rs
  - zircon_runtime/src/asset/assets/mesh/indices.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/mesh/usage.rs
  - zircon_runtime/src/asset/assets/mesh/validation.rs
  - zircon_runtime/src/asset/assets/mesh/zmesh_document.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/importer/ingest/import_obj.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
implementation_files:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/attribute.rs
  - zircon_runtime/src/asset/assets/mesh/constants.rs
  - zircon_runtime/src/asset/assets/mesh/indices.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/mesh/usage.rs
  - zircon_runtime/src/asset/assets/mesh/validation.rs
  - zircon_runtime/src/asset/assets/mesh/zmesh_document.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
plan_sources:
  - user: 2026-05-20 implement ZirconEngine 资产、Texture、模型、ZShader/ZMaterial/ZMesh 缺口补齐计划
  - dev/bevy/crates/bevy_mesh/src/mesh.rs
  - dev/bevy/crates/bevy_gltf/src/label.rs
tests:
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - cargo check -p zircon_runtime --locked --all-targets
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_asset_importer_model_runtime --locked --jobs 1
doc_type: module-detail
---

# Mesh Asset And ZMesh

## Purpose

`MeshAsset` makes mesh payloads a first-class runtime asset instead of only embedding mesh data inside `ModelAsset.primitives`. This is the M2 foundation for the Bevy-style asset plan: typed `Handle<MeshAsset>`, `Assets<MeshAsset>`, `ImportedAsset::Mesh`, `AssetKind::Mesh`, artifact storage, project loading, and resource registration all use the same facade path as texture, shader, material, and model assets.

The authoring format is `.zmesh`, represented by `ZMeshDocument`. The runtime type remains `MeshAsset`, matching the existing naming split between `MaterialAsset`/`ZMaterialDocument` and `ShaderAsset`/`ZShaderDocument`.

## Behavior Model

`MeshAsset` stores a render topology, a named attribute map, optional `u16` or `u32` indices, Bevy-style main-world/render-world residency intent, and optional Virtual Geometry payload. The built-in attribute names are `position`, `normal`, `uv0`, `color`, `joint_index`, and `joint_weight`. Positions are required and must be `float32x3`; every present attribute must have the same vertex count. The representation follows the core idea in Bevy's `Mesh` attribute map while keeping Zircon's current `RenderMeshDescriptor` and `MeshVertex` compatibility layer.

`.zmesh` TOML imports through `zircon.builtin.zmesh`. A valid document becomes `ImportedAsset::Mesh`; invalid position data or mismatched attribute lengths fail during import with a deterministic parse diagnostic. `MeshAsset::render_mesh_descriptor()` computes bounds, primitive kind, 2D/3D suitability, vertex/index counts, primitive count, and Virtual Geometry presence from the attribute map and indices.

## Model Subassets

Model import paths now keep the old root `ModelAsset { primitives }` payload for renderer compatibility while also emitting labeled `MeshAsset` subassets. The current labels are `Mesh{n}/Primitive0`, so a single OBJ triangle imported from `res://models/triangle.obj` also produces `res://models/triangle.obj#Mesh0/Primitive0`. The root import entry records those subasset locators as dependencies, and each subasset has its own artifact and `ResourceRecord`.

This is intentionally a compatibility phase. The renderer can continue consuming `ModelAsset.primitives`; the new mesh assets give future renderer and editor paths a typed handle target. glTF labels currently use the shared model primitive label shape. The full Bevy label matrix (`Scene{n}`, `Node{n}`, `Mesh{m}/Primitive{p}`, `Material{n}`, `Texture{n}`, animation and skin labels) remains a later M4 expansion.

## Design And Rationale

The mesh module is folder-backed because mesh data already has separate concerns: attribute values, index format, validation, authoring document conversion, render descriptor projection, and legacy model primitive conversion. `mod.rs` only re-exports the public surface.

`MeshAsset::from_model_primitive()` is the shared support layer for current model importers. OBJ, glTF, STL, PLY, DXF, and `.model.toml` imports all preserve the legacy primitive and derive a matching `MeshAsset` from the same data, avoiding importer-specific render descriptor construction.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/mesh.rs` covers `.zmesh` roundtrip, missing position, attribute length mismatch, u16/u32 index conversion, bounds/descriptor projection, Virtual Geometry preservation, model primitive conversion, and default `.zmesh` importer routing.

`zircon_runtime/src/asset/tests/assets/importer.rs` covers mesh subasset emission for model imports. The OBJ, glTF, and model importer plugin package tests also assert that a model import outcome includes a mesh subasset entry.

## Open Issues

Texture container GPU upload, full glTF label/type coverage, animation/skin subassets, and renderer consumption of mesh handles are not completed in this slice. Those belong to the later M3-M5 milestones from the 2026-05-20 plan.


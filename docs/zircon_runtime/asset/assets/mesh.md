---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/attribute.rs
  - zircon_runtime/src/asset/assets/mesh/constants.rs
  - zircon_runtime/src/asset/assets/mesh/indices.rs
  - zircon_runtime/src/asset/assets/mesh/metadata.rs
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
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
implementation_files:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/attribute.rs
  - zircon_runtime/src/asset/assets/mesh/constants.rs
  - zircon_runtime/src/asset/assets/mesh/indices.rs
  - zircon_runtime/src/asset/assets/mesh/metadata.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/mesh/usage.rs
  - zircon_runtime/src/asset/assets/mesh/validation.rs
  - zircon_runtime/src/asset/assets/mesh/zmesh_document.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
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

`MeshAsset` stores a render topology, a named attribute map, optional `u16` or `u32` indices, Bevy-style main-world/render-world residency intent, morph target metadata, optional skin inverse bind matrices, and optional Virtual Geometry payload. The built-in attribute names are `position`, `normal`, `tangent`, `uv0`, `color`, `joint_index`, and `joint_weight`. Positions are required and must be `float32x3`; every present root attribute must have the same vertex count. When an index buffer is present, its highest index must reference an existing vertex so malformed `.zmesh` documents and importer outputs fail before render descriptor projection.

Morph targets are stored as named attribute maps on `MeshMorphTargetAsset`. Each morph target attribute must match the root mesh vertex count, but the target is not required to redeclare every root attribute. This keeps the authoring format ready for position/normal/tangent deltas without forcing current importers to synthesize unused channels. `MeshSkinAsset.inverse_bind_matrices` stores skin bind-pose matrices beside the mesh payload so future glTF skin labels and renderer skinning paths can reference the same mesh artifact instead of creating a parallel skin metadata store. The representation follows the core idea in Bevy's `Mesh` attribute map while keeping Zircon's current `RenderMeshDescriptor` and `MeshVertex` compatibility layer.

`.zmesh` TOML imports through `zircon.builtin.zmesh`. A valid document becomes `ImportedAsset::Mesh`; invalid position data or mismatched attribute lengths fail during import with a deterministic parse diagnostic. `MeshAsset::render_mesh_descriptor()` computes bounds, primitive kind, 2D/3D suitability, vertex/index counts, primitive count, and Virtual Geometry presence from the attribute map and indices.

## Model Subassets

Model import paths now keep the old root `ModelAsset { primitives }` payload for renderer compatibility while also emitting labeled `MeshAsset` subassets. The current labels are `Mesh{n}/Primitive0`, so a single OBJ triangle imported from `res://models/triangle.obj` also produces `res://models/triangle.obj#Mesh0/Primitive0`. The root import entry records those subasset locators as dependencies, and each subasset has its own artifact and `ResourceRecord`.

This is intentionally a compatibility phase. The renderer can continue consuming `ModelAsset.primitives`; the new mesh assets give future renderer and editor paths a typed handle target. Built-in and plugin glTF import now emit Bevy-style labels for scenes, nodes, meshes, mesh primitives, materials, textures, and diagnostic animation/skin placeholders. glTF primitive morph target position, normal, and tangent displacements are copied into `MeshAsset.morph_targets` on each primitive subasset. When a glTF node binds a mesh to a skin with inverse bind matrices, both import paths attach those matrices to the corresponding `MeshAsset.skin` on the primitive subassets. The standalone `Skin{n}` labels still remain diagnostic `DataAsset` placeholders until full skin asset payloads are introduced.

## Design And Rationale

The mesh module is folder-backed because mesh data already has separate concerns: attribute values, index format, validation, authoring document conversion, render descriptor projection, and legacy model primitive conversion. `mod.rs` only re-exports the public surface.

`MeshAsset::from_model_primitive()` is the shared support layer for current model importers. OBJ, glTF, STL, PLY, DXF, and `.model.toml` imports all preserve the legacy primitive and derive a matching `MeshAsset` from the same data, avoiding importer-specific render descriptor construction. The conversion preserves the existing joint index/weight vertex channels, but leaves `morph_targets` empty and `skin` unset because the legacy `ModelPrimitiveAsset` does not own morph target or inverse-bindpose metadata. The built-in and plugin glTF importers layer primitive morph target maps and node-level skin data onto the derived mesh subasset after this shared conversion, keeping the common path simple while preserving glTF metadata where it exists.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/mesh.rs` covers `.zmesh` roundtrip, morph target and skin inverse-bindpose persistence, missing position, root and morph target attribute length mismatch, out-of-range index rejection, u16/u32 index conversion, bounds/descriptor projection, Virtual Geometry preservation, model primitive conversion, and default `.zmesh` importer routing.

`zircon_runtime/src/asset/tests/assets/importer.rs` covers mesh subasset emission for model imports, Bevy-style glTF labeled subassets, and propagation of glTF morph target position deltas plus inverse bind matrices into `MeshAsset`. The OBJ, glTF, and model importer plugin package tests also assert that a model import outcome includes a mesh subasset entry, and the glTF plugin test mirrors the built-in morph target and inverse-bind matrix assertions.

## Open Issues

Texture container GPU upload, tangent generation, glTF morph weights/animation playback, full animation/skin subasset payloads, multiple skin bindings per shared mesh, and renderer consumption of mesh handles are not completed in this slice. Those belong to the later M3-M5 milestones from the 2026-05-20 plan.

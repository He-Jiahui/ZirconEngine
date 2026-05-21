---
related_code:
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/generate_preview_artifact.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/preview_palette.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/layouts/views/asset_surface_presentation.rs
implementation_files:
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/generate_preview_artifact.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/preview_palette.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/layouts/views/asset_surface_presentation.rs
plan_sources:
  - user: 2026-05-02 sound plugin validation closeout
  - user: 2026-05-08 implement Bevy-Style Asset Stack Completion Plan M1
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M2
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M3
  - .codex/plans/Sound 插件核心完善计划.md
  - .codex/plans/多插件组合可选功能规则设计.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - user: 2026-05-20 implement ZirconEngine asset/texture/model/ZShader/ZMaterial/ZMesh completion plan
tests:
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/core/resource/tests.rs
  - cargo check -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short
  - cargo test -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short
doc_type: module-detail
---

# Project Asset Type Convergence

## Purpose

The asset system now exposes more authoring kinds through `ResourceKind` and `ImportedAsset`: data, mesh, material graphs, navigation assets, terrain, tile maps, prefabs, and related layer/set assets. Any shared loader or editor status surface that pattern matches those enums must carry the same type set, otherwise unrelated plugin validation can fail before the target plugin is compiled.

## Behavior

Project asset loading maps every resource kind to a typed `load_*_asset` method, then wraps it back into `ImportedAsset` for generic callers. Runtime registration and payload storage likewise store every concrete asset variant in `ResourceManager`.

`MeshAsset` is now part of this convergence. `ResourceKind::Mesh`, `MeshMarker`, `ImportedAsset::Mesh`, `load_mesh_asset`, `load_imported_asset`, resource sync, artifact storage, and editor-facing labels all route through the same exhaustive enum surface as the older `ModelAsset` path.

M1 of the Bevy-style asset stack adds a generic facade over the same mapping. `Asset` implementations bind each concrete payload to its existing resource marker, `Handle<TAsset>` wraps the existing marker handle, and `Assets<TAsset>` reads, acquires, inserts, removes, and subscribes through `ResourceManager` rather than introducing a parallel store. `ProjectAssetManager::load<TAsset>(locator)` verifies the locator kind, ensures the payload is ready or rehydrated through the existing typed loader path, and returns a typed facade handle. The old `load_*_asset` methods remain implementation-backed entry points while callers migrate to the generic surface.

M2 adds the dependency graph to the same converged path. Importers declare dependency locators on each `ImportedAssetEntry`, and native importer response entries expose the same field. `ProjectManager::scan_and_import()` persists those locators into `AssetMetaDocument.entries[*].dependencies`, then resolves the completed project registry into `ResourceRecord.dependency_ids`. The graph therefore stays in the resource record and travels through existing project-to-runtime sync into `ResourceManager`, while the meta file carries enough locator data to restore the graph after a restart. Missing dependency locators are recorded as resource diagnostics instead of silently disappearing.

M3 hard-cuts the importer result from a single payload to `AssetImportOutcome { entries: Vec<ImportedAssetEntry> }`. A valid import must contain exactly one root entry whose locator has no `#label`, while subassets use the same source locator with a label such as `res://bundle.gltf#Mesh0`. `ProjectManager` writes one library artifact and one `ResourceRecord` per entry, persists those rows in `AssetMetaDocument.entries`, and derives each ID with `AssetId::from_asset_uuid(entry.uuid)`. Duplicate labels are recorded as a failed root import through `AssetImportError::DuplicateAssetLabel`, and loading a missing label from an imported source returns `AssetImportError::MissingAssetLabel` instead of an unstructured parse error.

Editor asset surfaces use placeholder thumbnails and labels for newly introduced non-texture kinds. Mesh assets use the same placeholder-preview path as models while keeping their own `ResourceKind::Mesh` palette entry. Reference analysis delegates graph-like authoring assets to their `direct_references()` implementations so scene terrain, tilemap, prefab, material graph, terrain layer stack, tile set, and tile map references remain discoverable without duplicating traversal logic in the editor; standalone mesh assets currently report no direct asset references.

## Validation Note

These fixes were added as narrow support work while validating the sound plugin. They are not a new asset importer implementation; they keep shared enum handling exhaustive so independent plugin packages can be checked and tested while asset importer pluginization continues in parallel.

---
related_code:
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/generate_preview_artifact.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/preview_palette.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/layouts/views/asset_surface_presentation.rs
implementation_files:
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/generate_preview_artifact.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/preview_palette.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/layouts/views/asset_surface_presentation.rs
plan_sources:
  - user: 2026-05-02 sound plugin validation closeout
  - .codex/plans/Sound 插件核心完善计划.md
  - .codex/plans/多插件组合可选功能规则设计.md
tests:
  - cargo check -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short
  - cargo test -p zircon_plugin_sound_runtime -p zircon_plugin_sound_editor --locked --message-format short
doc_type: module-detail
---

# Project Asset Type Convergence

## Purpose

The asset system now exposes more authoring kinds through `ResourceKind` and `ImportedAsset`: data, material graphs, navigation assets, terrain, tile maps, prefabs, and related layer/set assets. Any shared loader or editor status surface that pattern matches those enums must carry the same type set, otherwise unrelated plugin validation can fail before the target plugin is compiled.

## Behavior

Project asset loading maps every resource kind to a typed `load_*_asset` method, then wraps it back into `ImportedAsset` for generic callers. Runtime registration and payload storage likewise store every concrete asset variant in `ResourceManager`.

Editor asset surfaces use placeholder thumbnails and labels for newly introduced non-texture kinds. Reference analysis delegates graph-like authoring assets to their `direct_references()` implementations so scene terrain, tilemap, prefab, material graph, terrain layer stack, tile set, and tile map references remain discoverable without duplicating traversal logic in the editor.

## Validation Note

These fixes were added as narrow support work while validating the sound plugin. They are not a new asset importer implementation; they keep shared enum handling exhaustive so independent plugin packages can be checked and tested while asset importer pluginization continues in parallel.

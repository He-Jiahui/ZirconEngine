---
related_code:
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/resource_handle.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/core/resource/lease.rs
  - zircon_runtime/src/core/resource/runtime.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/meta.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager/mod.rs
  - zircon_asset/src/project/manager/open.rs
  - zircon_asset/src/project/manager/scan_and_import.rs
  - zircon_asset/src/project/manager/registry_access.rs
  - zircon_asset/src/project/manager/asset_lookup.rs
  - zircon_asset/src/project/manager/artifact_access.rs
  - zircon_asset/src/project/manager/source_path_for_uri.rs
  - zircon_asset/src/project/manager/source_uri_for_path.rs
  - zircon_asset/src/project/manager/collect_files.rs
  - zircon_asset/src/project/manager/asset_kind.rs
  - zircon_asset/src/project/manager/hash_bytes.rs
  - zircon_asset/src/project/manager/source_mtime_unix_ms.rs
  - zircon_asset/src/project/manager/meta_path_for_source.rs
  - zircon_asset/src/project/manager/is_meta_sidecar.rs
  - zircon_asset/src/project/manager/load_or_create_meta.rs
  - zircon_asset/src/editor/api.rs
  - zircon_asset/src/editor/catalog.rs
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/reference_graph.rs
  - zircon_asset/src/editor/preview.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/editor/resolver.rs
  - zircon_asset/src/pipeline/manager/mod.rs
  - zircon_asset/src/pipeline/manager/asset_manager/mod.rs
  - zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_asset/src/pipeline/manager/asset_manager/asset_manager_handle.rs
  - zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs
  - zircon_asset/src/pipeline/manager/driver/mod.rs
  - zircon_asset/src/pipeline/manager/driver/asset_io_driver.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/mod.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/construction.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/mod.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/acquire_asset.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/load_typed.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/acquire_typed.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/ensure_resident.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/mod.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/resource_manager_contract.rs
  - zircon_asset/src/pipeline/manager/resource_sync/mod.rs
  - zircon_asset/src/pipeline/manager/resource_sync/project_locators.rs
  - zircon_asset/src/pipeline/manager/resource_sync/clear_removed_project_resources.rs
  - zircon_asset/src/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_asset/src/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_asset/src/pipeline/manager/records/asset_pipeline_info.rs
  - zircon_asset/src/pipeline/manager/records/project_info.rs
  - zircon_asset/src/pipeline/manager/records/asset_status_record.rs
  - zircon_asset/src/pipeline/manager/records/project_info_from_project.rs
  - zircon_asset/src/pipeline/manager/records/status_record.rs
  - zircon_asset/src/pipeline/manager/records/metadata_import_state.rs
  - zircon_asset/src/pipeline/manager/builtins/mod.rs
  - zircon_asset/src/pipeline/manager/registration/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_asset/src/pipeline/manager/registration/service_names.rs
  - zircon_asset/src/pipeline/manager/errors/mod.rs
  - zircon_asset/src/pipeline/manager/errors/asset_error.rs
  - zircon_asset/src/pipeline/manager/errors/asset_error_message.rs
  - zircon_asset/src/formats/mod.rs
  - zircon_asset/src/formats/obj/mod.rs
  - zircon_asset/src/formats/obj/decode_obj_file.rs
  - zircon_asset/src/formats/obj/obj_vertex_key.rs
  - zircon_asset/src/formats/obj/parsed_obj_vertex.rs
  - zircon_asset/src/formats/obj/parse_obj_scalar.rs
  - zircon_asset/src/formats/obj/parse_obj_face_vertex.rs
  - zircon_asset/src/formats/obj/resolve_obj_index.rs
  - zircon_asset/src/watch/mod.rs
  - zircon_asset/src/watch/asset_change.rs
  - zircon_asset/src/watch/asset_change_kind.rs
  - zircon_asset/src/watch/asset_change_new.rs
  - zircon_asset/src/watch/asset_watch_event.rs
  - zircon_asset/src/watch/asset_watcher.rs
  - zircon_asset/src/watch/default.rs
  - zircon_asset/src/watch/spawn.rs
  - zircon_asset/src/watch/fold_events.rs
  - zircon_asset/src/watch/drop_impl.rs
  - zircon_asset/src/watch/watch_loop.rs
  - zircon_asset/src/watch/map_notify_event.rs
  - zircon_asset/src/watch/watched_asset_uri_for_path.rs
  - zircon_asset/src/watch/asset_uri_for_path.rs
  - zircon_asset/src/watch/watch_io_error.rs
  - zircon_asset/src/watch/is_meta_sidecar.rs
  - zircon_asset/src/watch/recommended_watcher.rs
  - zircon_asset/src/assets/material.rs
  - zircon_asset/src/assets/scene.rs
  - zircon_manager/src/lib.rs
  - zircon_manager/src/service_names.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/world/world.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/module/level_display_name.rs
  - zircon_runtime/src/scene/module/core_error.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/icons/viewport_icon_atlas/mod.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_graphics/src/service/mod.rs
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor/src/editing/asset_workspace.rs
  - zircon_editor/src/workbench/snapshot/mod.rs
  - zircon_editor/src/workbench/project/mod.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/event_bridge.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/src/host/bridge/viewport.rs
implementation_files:
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/resource_handle.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/core/resource/lease.rs
  - zircon_runtime/src/core/resource/runtime.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/meta.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager/mod.rs
  - zircon_asset/src/project/manager/open.rs
  - zircon_asset/src/project/manager/scan_and_import.rs
  - zircon_asset/src/project/manager/registry_access.rs
  - zircon_asset/src/project/manager/asset_lookup.rs
  - zircon_asset/src/project/manager/artifact_access.rs
  - zircon_asset/src/project/manager/source_path_for_uri.rs
  - zircon_asset/src/project/manager/source_uri_for_path.rs
  - zircon_asset/src/project/manager/collect_files.rs
  - zircon_asset/src/project/manager/asset_kind.rs
  - zircon_asset/src/project/manager/hash_bytes.rs
  - zircon_asset/src/project/manager/source_mtime_unix_ms.rs
  - zircon_asset/src/project/manager/meta_path_for_source.rs
  - zircon_asset/src/project/manager/is_meta_sidecar.rs
  - zircon_asset/src/project/manager/load_or_create_meta.rs
  - zircon_asset/src/editor/api.rs
  - zircon_asset/src/editor/catalog.rs
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/reference_graph.rs
  - zircon_asset/src/editor/preview.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/editor/resolver.rs
  - zircon_asset/src/pipeline/manager/mod.rs
  - zircon_asset/src/pipeline/manager/asset_manager/mod.rs
  - zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_asset/src/pipeline/manager/asset_manager/asset_manager_handle.rs
  - zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs
  - zircon_asset/src/pipeline/manager/driver/mod.rs
  - zircon_asset/src/pipeline/manager/driver/asset_io_driver.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/mod.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/project_asset_manager.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/construction.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/mod.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/acquire_asset.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/load_typed.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/acquire_typed.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/loading/ensure_resident.rs
  - zircon_asset/src/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/mod.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/resource_manager_contract.rs
  - zircon_asset/src/pipeline/manager/resource_sync/mod.rs
  - zircon_asset/src/pipeline/manager/resource_sync/project_locators.rs
  - zircon_asset/src/pipeline/manager/resource_sync/clear_removed_project_resources.rs
  - zircon_asset/src/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_asset/src/pipeline/manager/resource_sync/store_runtime_payload.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_asset/src/pipeline/manager/records/asset_pipeline_info.rs
  - zircon_asset/src/pipeline/manager/records/project_info.rs
  - zircon_asset/src/pipeline/manager/records/asset_status_record.rs
  - zircon_asset/src/pipeline/manager/records/project_info_from_project.rs
  - zircon_asset/src/pipeline/manager/records/status_record.rs
  - zircon_asset/src/pipeline/manager/records/metadata_import_state.rs
  - zircon_asset/src/pipeline/manager/builtins/mod.rs
  - zircon_asset/src/pipeline/manager/registration/mod.rs
  - zircon_asset/src/pipeline/manager/registration/module_descriptor.rs
  - zircon_asset/src/pipeline/manager/registration/service_names.rs
  - zircon_asset/src/pipeline/manager/errors/mod.rs
  - zircon_asset/src/pipeline/manager/errors/asset_error.rs
  - zircon_asset/src/pipeline/manager/errors/asset_error_message.rs
  - zircon_asset/src/formats/mod.rs
  - zircon_asset/src/formats/obj/mod.rs
  - zircon_asset/src/formats/obj/decode_obj_file.rs
  - zircon_asset/src/formats/obj/obj_vertex_key.rs
  - zircon_asset/src/formats/obj/parsed_obj_vertex.rs
  - zircon_asset/src/formats/obj/parse_obj_scalar.rs
  - zircon_asset/src/formats/obj/parse_obj_face_vertex.rs
  - zircon_asset/src/formats/obj/resolve_obj_index.rs
  - zircon_asset/src/watch/mod.rs
  - zircon_asset/src/watch/asset_change.rs
  - zircon_asset/src/watch/asset_change_kind.rs
  - zircon_asset/src/watch/asset_change_new.rs
  - zircon_asset/src/watch/asset_watch_event.rs
  - zircon_asset/src/watch/asset_watcher.rs
  - zircon_asset/src/watch/default.rs
  - zircon_asset/src/watch/spawn.rs
  - zircon_asset/src/watch/fold_events.rs
  - zircon_asset/src/watch/drop_impl.rs
  - zircon_asset/src/watch/watch_loop.rs
  - zircon_asset/src/watch/map_notify_event.rs
  - zircon_asset/src/watch/watched_asset_uri_for_path.rs
  - zircon_asset/src/watch/asset_uri_for_path.rs
  - zircon_asset/src/watch/watch_io_error.rs
  - zircon_asset/src/watch/is_meta_sidecar.rs
  - zircon_asset/src/watch/recommended_watcher.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/world/world.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_manager/src/lib.rs
  - zircon_manager/src/service_names.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/module/level_display_name.rs
  - zircon_runtime/src/scene/module/core_error.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_graphics/src/types/mod.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor/src/editing/asset_workspace.rs
  - zircon_editor/src/workbench/snapshot/mod.rs
  - zircon_editor/src/workbench/project/mod.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/event_bridge.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_runtime_interface/src/resource/resource_id.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_editor/src/tests/host/resource_access/mod.rs
plan_sources:
  - user: 2026-04-13 实现目录式 Project 资源抽象优先全链路替换计划
  - user: 2026-04-14 编辑器资源管理器 UI 真正接到 EditorAssetManager / EditorAssetServer
  - user: 2026-04-14 编辑器 Builtin 资产归位与 Revision 稳定化计划
  - user: 2026-04-16 全仓库模块边界拆分与根入口去逻辑化
  - user: 2026-04-17 继续扫描明显错包模块并按方案2把 editor asset API 从 zircon_manager 迁回 zircon_asset
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M2
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M3
  - docs/superpowers/plans/2026-04-17-asset-editor-api-boundary-migration.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/编辑器资源管理器双模式 UI 接线计划.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
tests:
  - zircon_runtime/src/core/resource/tests.rs
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_asset/src/tests/project/manifest.rs
  - zircon_asset/src/tests/project/manager.rs
  - zircon_asset/src/tests/editor/boundary.rs
  - zircon_asset/src/tests/editor/manager.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_asset/src/tests/watcher.rs
  - zircon_scene/src/lib.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/editing/import.rs
  - zircon_editor/src/tests/host/asset_manager_boundary/mod.rs
  - zircon_editor/src/tests/host/resource_access/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/asset/direct_dispatch.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/asset/template_bridge.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/mod.rs
  - zircon_editor/src/tests/host/slint_event_bridge/mod.rs
  - zircon_editor/src/tests/host/slint_asset_refresh/mod.rs
  - zircon_editor/src/tests/host/slint_builtin_assets.rs
  - cargo test -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --message-format short --color never
  - cargo test -p zircon_runtime --lib core::resource --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-boundary --message-format short --color never
  - cargo test -p zircon_manager manager_public_surface_excludes_editor_asset_api --locked
  - cargo test -p zircon_asset editor_asset_api_boundary_lives_in_zircon_asset --locked
  - cargo test -p zircon_editor editor_asset_boundary_lives_in_asset_crate --locked
  - cargo test -p zircon_asset --offline
  - cargo test -p zircon_scene --offline
  - cargo test --workspace --locked
doc_type: module-detail
---

# Directory Project Asset Rendering

## Purpose

这一轮实现把主链替换为“目录式项目 + 资源抽象层 + UUID/meta 持久化 + runtime lease/refcount + revision 驱动 prepare/cache + editor catalog/reference/preview”模型：

- `zircon_runtime_interface::resource` 定义跨 crate 的 locator、typed handle、record、state、event、marker 等 ABI/DTO/序列化合同
- `zircon_runtime::core::resource` 保留 `ResourceManager`、IO、lease、registry、resident payload 等执行逻辑，并重导出 interface 合同
- `zircon_asset::pipeline::manager::AssetManager` 负责 runtime resident 资源生命周期
- `zircon_asset::editor::DefaultEditorAssetManager` 负责 editor catalog/meta/reference/preview 生命周期，并由 `zircon_asset::editor` 自己公开 `EditorAssetManager` / records / resolver
- `zircon_scene_protocol::{WorldHandle, LevelSummary}` 现在作为 `LevelManager` 的 scene 协议面，不再挂在 `zircon_manager`
- `zircon_scene::Scene` 运行时只持 typed handle，不再持路径语义
- `zircon_runtime::scene::LevelSystem` 托管运行中的 world、metadata 和子系统生命周期
- `zircon_graphics` 按 `ResourceId + revision` 准备 GPU 资源
- `zircon_editor` 通过 `AssetManager + ResourceManager + zircon_asset::editor::EditorAssetManager` 消费这些层

目标不是先堆更多 importer 分支，而是先把“project -> resource -> scene -> render -> editor”变成统一的框架主链。

这一轮模块边界重构没有改掉这条主链的行为语义，但把两个历史聚合点降成了纯结构入口：

- `zircon_asset/src/pipeline/manager/mod.rs` 现在只负责声明 folder-backed 子模块；驱动声明、`ProjectAssetManager` 构造、runtime resident 载入、watcher 同步、service contract 实现、builtin 资源和模块描述符分别落到 `pipeline/manager/` 下
- `zircon_scene/src/module.rs` 现在只保留 scene 模块导出层；`DefaultLevelManager`、level project I/O、manager service contract、descriptor 和 service name 已拆到 `module/` 子树
- `zircon_scene/src/world/mod.rs` 继续作为 world 子系统边界，但 `World` 结构定义本身已经独立到 `zircon_scene/src/world/world.rs`

## Resource Foundation

资源基础层现在分成两段：`zircon_runtime_interface::resource` 拥有跨 app/runtime/editor/plugin 的稳定 DTO 合同，`zircon_runtime::core::resource` 拥有运行时执行逻辑并重导出这些合同类型。核心对象包括：

- `ResourceLocator`
  - 统一支持 `res://`、`lib://`、`builtin://`、`mem://`
  - 负责规范化、越界拒绝和 `#label` 子资源语法
- `AssetUuid` / `AssetReference`
  - 项目资产稳定身份改成 `UUID 主、locator 辅`
  - 旧 locator-only TOML 会在读取时按 locator 稳定派生 UUID
- `ResourceHandle<TMarker>` / `UntypedResourceHandle`
  - `ModelMarker`、`MaterialMarker`、`TextureMarker`、`ShaderMarker`、`SceneMarker` 把运行时引用类型化
- `zircon_runtime::asset::{Asset, Handle<TAsset>, Assets<TAsset>, AssetEvent<TAsset>, AssetEventReceiver<TAsset>, AssetLoadState}`
  - Bevy-style typed asset facade，建立在 `ResourceHandle<TMarker>`、`ResourceManager`、`ResourceRecord`、`ResourceEvent` 和 `ResourceLease` 之上，不新建资产存储
- `ResourceLease<T>`
  - 运行时获取资源时返回 typed lease
  - lease drop 后递减 refcount，最后一个 lease 释放时 resident payload 转 `Unloaded`
- `ResourceRecord`
  - 权威索引项，记录 `id`、`primary_locator`、`artifact_locator`、`revision`、`state`、`dependency_ids`、`diagnostics`
- `ResourceManager`
  - 管 ready payload、runtime refcount、resident unload/reload 和 reload failure
  - 重载失败时保留 last-good payload，只把 record 状态与诊断改成错误态

M1 Bevy-style facade keeps this foundation authoritative. `Handle<TAsset>` is a cheap typed identity value, while residency still comes from `ResourceLease<T>` acquired through `Assets<TAsset>`. `ProjectAssetManager::load<TAsset>(locator)` verifies the resource kind, reuses existing typed load/rehydration paths, and returns a typed facade handle. `AssetLoadState` maps missing records to `NotLoaded`, pending/runtime loading to `Loading`, resident ready payloads to `Loaded`, errors to `Failed`, and Zircon hot reload to an explicit loading-class `Reloading` state.

M2 dependency graph data follows the same authority model. Each `ImportedAssetEntry.dependencies` list records dependency locators from importers, `AssetMetaDocument.entries[*].dependencies` persists those locators for artifact restore, and `ProjectManager::scan_and_import()` resolves the final project registry into `ResourceRecord.dependency_ids`. The runtime graph is therefore a field on the canonical resource record instead of a parallel asset-side graph store. Unresolved locators are retained as `ResourceDiagnostic` rows, while `ProjectAssetManager::recursive_dependency_load_state()` walks the resource records to produce Bevy-style dependency state.

M3 extends this to multi-asset imports. `AssetImportOutcome` now owns an entry list, with one unlabeled root entry and zero or more labeled subasset entries. Each entry writes its own artifact under `Project/library`, records its artifact locator in `AssetMetaDocument.entries`, and registers a distinct `ResourceRecord`. The stable ID formula is `AssetId::from_asset_uuid_label(meta.asset_uuid, locator.label())`, so `res://bundle.multi` and `res://bundle.multi#Texture0` share the source UUID while staying separate handles, artifacts, dependency rows, diagnostics, and load targets.

`ResourceLocator`、`AssetUuid`、`ResourceId`、`ResourceHandle`、`ResourceRecord`、`ResourceEvent` 与 marker/status DTO 的源码只落在 `zircon_runtime_interface/src/resource/**`。`zircon_runtime::core::resource` 只保留 `ResourceData`、`ResourceIo`、`ResourceLease`、`ResourceManager`、`ResourceRegistry`、`ResourceRuntimeInfo` 等执行层文件，不再保留 resource DTO 的第二套 owner 路径。

`res://`、`lib://`、`builtin://` 的 `ResourceId` 都由规范化 locator 稳定派生。项目源资源的主 id 现在改为 `AssetUuid + #label` 稳定派生，`mem://` 则只在当前进程内稳定，不能写回 project/scene/material 文件。

## Project And Import Layer

目录式项目根继续固定为：

- `Project/zircon-project.toml`
- `Project/assets/`
- `Project/library/`

`ProjectManager` 负责：

- manifest / path layout
- 扫描 `assets/`
- 为缺失资源补写 `*.meta.toml`
- 调 importer 解析 PNG/JPEG、WGSL、TOML material、TOML scene、OBJ、glTF/GLB
- 把导入物写到 `library/`
- 生成 `ResourceRecord` 元数据和 `AssetUuid` 驱动的 `ResourceId`

实现上，`zircon_asset/src/project/manager/mod.rs` 现在只保留 `ProjectManager` 结构定义与子模块声明；`open`、`scan_and_import`、registry/lookup、artifact 访问和本地文件/meta helper 全部下沉到 `zircon_asset/src/project/manager/` 子树，避免 project root manager 继续堆叠 importer 与文件系统逻辑。

当前 public surface 也已经跟随收束：workspace 调用点统一通过 `zircon_asset::project::{ProjectManager, ProjectManifest, ProjectPaths}` 访问目录式项目 API，不再从 `zircon_asset` 根 crate 平铺拿这组三元组。

sidecar meta 文件当前固定为 `foo.ext.meta.toml`，至少记录：

- `asset_uuid`
- `primary_locator`
- `kind`
- `source_mtime_unix_ms`
- `source_hash`
- `preview_state`
- `dependencies`
- `entries`

`entries` is the current authority for imported root/subasset rows. Older single-artifact meta files are migrated on restore by synthesizing one root entry from `artifact_locator` and `dependencies`, but new successful imports always write the entry list. Duplicate labeled entries fail the import and keep only an error-state root record. Loading an unknown label from a known source returns `AssetImportError::MissingAssetLabel`, which gives editor and tooling code a structured “label missing” diagnostic instead of a generic missing metadata string.

`AssetManager` 现在是 runtime 资产管理器，而不是 project/editor 混合 service contract。它内部组合：

- `ProjectManager`
- `ResourceManager`
- watcher / broadcaster
- runtime lease/refcount 恢复逻辑

`EditorAssetManager` 是新的 sibling manager，负责：

- 基于 project scan 建 catalog
- 载入 `*.meta.toml`
- 解析 material/scene 直接引用
- 维护“谁引用我 / 我引用谁”的直接引用图
- 管理 `library/editor-previews/` 的缓存路径和 dirty/visible refresh 策略

`AssetManager` 继续负责项目打开、重导入、watch 生命周期。  
`ResourceManager` 负责 locator 解析、resource status/revision 查询和资源事件订阅。  
`EditorAssetManager` 负责 catalog、引用图和 preview 刷新；它的 trait、records、resolver、handle 和 service-name 现在都归 `zircon_asset`，不再经过 `zircon_manager` contract。

实现上，`zircon_asset/src/pipeline/manager/project_asset_manager/loading/` 现在只保留 runtime asset loading 这一层，并进一步拆成 imported-asset dispatch、typed load、typed acquire 和 resident restore 四个家族；`zircon_asset/src/formats/obj/` 也已拆成 decode、vertex declaration、face/scalar parse 与 index normalize 叶子，避免继续把 OBJ 语法细节堆在单文件里。

## Scene Runtime

`zircon_scene` 的关键切换是 `LevelManager -> LevelSystem -> World` 分层下的 handle-runtime：

- `MeshRenderer.model: ResourceHandle<ModelMarker>`
- `MeshRenderer.material: ResourceHandle<MaterialMarker>`
- `RenderMeshSnapshot` 也直接携带这两个 typed handle

`SceneAsset` 和 `MaterialAsset` 文件现在统一存 `AssetReference { uuid, locator }`。加载规则是：

- `res://` 先按 UUID 命中 project catalog，再按 locator 回退修复旧引用
- `builtin://` 直接由 locator 派生 stable id
- 找不到的 `res://` 会回退到 `builtin://missing-model` 或 `builtin://missing-material`

保存规则也做了硬切换：

- `res://` 原样写回
- `builtin://` 原样写回
- 没有持久 locator 的运行时资源直接报错

这次已经删除 `builtin://cube <-> res://models/cube.obj` 这类隐式改写。

运行中 scene 不再暴露旧的 session contract。实际持有 world 与生命周期的是 `LevelSystem`，而 `SceneAssetSerializer` 负责 `SceneAsset <-> World` 边界。

## Graphics Prepare And Cache

`zircon_graphics::ResourceStreamer` 不再把 importer DTO 当作业务缓存键。当前实现按 `ResourceId + revision` 维护：

- prepared model
- prepared material
- prepared texture
- prepared shader

prepare 流程是：

1. scene render extract 提供 typed handle
2. streamer 用 handle.id() 查询 `ResourceRecord.revision`
3. cache miss 或 revision 变化时，从 `AssetManager` 取 ready payload 重建 GPU 资源
4. `SceneRendererCore` 按 `shader ResourceId + shader revision + double_sided + alpha_mode` 缓存 pipeline

shader 缺失时回退到 `builtin://shader/pbr.wgsl`。  
当前材质工作流仍然是 glTF metallic-roughness 的核心最小集，重点是先把资源抽象和 revision invalidation 跑通。

## Editor Asset Layer

`EditorAssetManager` 当前已经具备第一批可用能力，对应 concrete type 为 `DefaultEditorAssetManager`：

- `AssetCatalogRecord`
  - 持有 `asset_uuid`、`asset_id`、`locator`、`meta_path`、`preview_state`、`preview_artifact_path`
- `ReferenceGraph`
  - 直接边当前覆盖 material -> shader/texture、scene -> model/material
  - 解析时执行“UUID 优先、locator 回退”的迁移修复
- `PreviewCache`
  - 统一放在 `Project/library/editor-previews/`
- `PreviewScheduler`
  - 文件变更后先标 `Dirty`
  - 只有 `visible = true` 时才刷新 preview artifact
  - 不可见资源保留旧缓存

当前 runtime/editor 合流后的错误资产边界是：project scan 可以把缺少插件 importer、解析失败或没有 artifact locator 的源文件登记为 `ResourceState::Error`，但 editor catalog sync 不能因此中断。`DefaultEditorAssetManager::sync_from_project(...)` 只对 ready 资源读取 artifact 并解析直接引用；error 资源仍进入 catalog、保留 diagnostics 和 preview state，引用边保持为空。2026-05-03 的回归 `sync_from_project_keeps_error_assets_without_artifacts_in_catalog` 覆盖了这个路径。

同一轮 runtime graphics project-render 测试也显式安装第一波插件 importer fixture 后再打开 PNG/WGSL/OBJ 测试项目。这样测试验证的是“插件 importer 已安装时目录项目能渲染”，而不是把这些格式重新塞回 `ProjectAssetManager::default()` 的生产默认路径。

## Editor Flow

`zircon_editor` 现在通过共享的 `AssetWorkspaceState` 维护 editor 资产会话，再统一投影为 `AssetWorkspaceSnapshot`。运行时与编辑器资产层的职责已经稳定分开：

- `AssetManager`
  - 目录项目生命周期、导入、watch、runtime resident load/unload
- `ResourceManager`
  - ready/error/reloading 状态、revision、typed handle 解析
- `EditorAssetManager`
  - folder tree、catalog details、reference graph、preview policy
  - 这组 editor-facing 资产工作区 API 现在直接由 `zircon_asset` 导出

UI 侧不再消费 `asset_entries: Vec<String>` 这类降级模型，也不再保留旧 `iced` fallback host：

- Slint 正式宿主
  - `host/slint_host/app.rs` 从 `EditorAssetManager` 拉取 catalog/details/change
  - `host/slint_host/ui.rs` 把 `AssetWorkspaceSnapshot` 投影成 Unity-first `Assets` 和 Unreal-first `AssetBrowser`
  - 只对当前可见 tile、选中资源预览、展开 preview tab 请求刷新
- 共享 editor 状态层
  - `AssetWorkspaceState` / `AssetWorkspaceSnapshot` 作为宿主无关数据模型保留
  - 旧 `iced` host、presenter、viewport bridge 已删除，不再保留并行宿主实现

目录项目打开流程：

1. `AssetManager::open_project` 打开并扫描 project
2. editor 读取 manifest 默认 scene locator
3. `LevelManager` 通过 `SceneAssetSerializer` 和 locator 实例化 `LevelSystem`，并以 `zircon_scene_protocol::WorldHandle` / `LevelSummary` 暴露 scene 协议面
4. editor host 通过 `AssetManager` 读项目生命周期，通过 `EditorAssetManager` 读 folder tree/detail/reference/preview，通过 `ResourceManager` 读 ready 状态和 revision
5. viewport render service 只在 GPU bridge 边界拿到 service contract 或 helper，避免把资源实现细节继续扩散到宿主编辑逻辑

资源刷新流程：

1. watcher 折叠 `assets/` 文件事件
2. `AssetManager` 重新导入并更新 `ResourceRecord.revision`
3. `EditorAssetManager` 同步 catalog/details/reference/preview 状态，`ResourceManager` 保留 runtime revision/state
4. `zircon_editor::host::resource_access` 只负责把 `ResourceRecord` 解析成 ready typed handle，不再承担 UI 列表组装
5. editor 收到 change record 后重建 `AssetWorkspaceSnapshot`，并在需要时重置 viewport render service
6. 新 render service 按最新 revision 重建 prepared GPU 资源

实现上，`zircon_asset/src/watch/mod.rs` 现在只保留 change/event/watcher 的结构导出；watcher 线程生命周期、notify 事件折叠、rename 映射、URI/path 解析与 meta-sidecar 过滤全部下沉到 `zircon_asset/src/watch/` 子树，避免 crate 根层 watcher 入口继续混装 runtime 与文件系统细节。

## Editor Builtin Assets And Revision Stability

### Builtin Icon Authority

这一轮把 editor chrome 还残留在仓库根 `dev/` 的图标依赖彻底收回到 crate 本地：

- `zircon_editor/ui/workbench/chrome.slint` 只允许从 `zircon_editor/assets/icons/ionicons/` 读取静态 SVG
- editor icon 的统一资源命名空间固定为 `builtin://editor/icons/<file>.svg`
- `zircon_asset::pipeline::manager` 会把同一批 icon 注册进 builtin registry，使 editor builtin 资产在资源系统里也是一等 locator

当前 Slint 仍直接读取 crate 本地 SVG 文件显示图标，不经过 builtin locator bridge；builtin registry 则先注册为 texture-kind placeholder 资源，目标不是立刻参与 Slint 渲染，而是先稳定 `builtin://editor/icons/...` 这条引擎级命名路径，后续再接真正的 SVG -> runtime image bridge。

### Asset Workspace Refresh Boundary

asset workspace 现在明确拆成两条链：

- UI 本地交互
  - 搜索、filter、folder 选择、item 选择、view mode、utility tab、引用跳转
  - 这些事件只会在 `EditorEventEffect` 中发出 `AssetDetailsRefreshRequested` 或 `AssetPreviewRefreshRequested`
  - Slint host 只做局部 `asset_details()` 查询或 `request_preview_refresh()`，不再顺手调用 `sync_asset_workspace()`
- 后端真实数据变化
  - `EditorAssetChangeRecord::{CatalogChanged, PreviewChanged, ReferenceChanged}`
  - `ResourceEvent { kind: ResourceEventKind::{Added, Updated, Removed, Renamed, ReloadFailed}, resource_kind, .. }`
  - 这些后台事件才允许触发 catalog/resource snapshot 重同步，必要时重载默认 scene

这样 editor UI 不再把“我改了一个搜索词”误当成“runtime/resource 发生了真实变化”。

### Revision Stability Contract

这一轮把 editor 可见的 `resource_revision` 稳定契约补齐为：

- idle tick、布局调整、搜索、筛选、tab 切换、普通选择变化都不能导致 runtime `revision` 漂移
- `PreviewChanged` 只更新 editor catalog/details/thumbnail 呈现，不顺带重拉 runtime resource list
- visible preview 刷新继续采用“可见即刷新，否则保留 last-good cache”的策略，但 preview/meta 写回不会形成新的 resource revision loop
- `zircon_runtime::core::resource::ResourceManager::register_ready()` 保持幂等；同一 ready record 重复注册不发 updated event，也不 bump revision

因此现在面板里的 `r####` 只应该在源文件、导入结果或真实 resource record 变化时增长，而不会随着 editor 每帧刷新自己累加。

## Constraints

- `ResourceLocator` 拒绝绝对路径、`..` 逃逸和空路径
- `library/` 只存可重建导入物，不是权威源文件
- watcher 只观察 `assets/`，不观察 `library/`
- 外部 `.gltf` 仍要求用户先处理外部依赖目录；单文件 `.glb` 是当前推荐路径
- `FBX`、`ASTC`、`PVRTexTool` 目前只保留扩展位，不承诺真实导入链

## Test Coverage

当前主链已经有直接证据覆盖：

- `zircon_runtime/src/core/resource/tests.rs` 与 `zircon_runtime_interface/src/tests/contracts.rs`
  - locator 规范化
  - stable/non-stable id 规则
  - `AssetUuid` / `AssetReference` roundtrip
  - runtime lease/refcount/unload
  - `ResourceId` display/parse roundtrip
  - typed handle 转换
  - registry rename/remove
  - manager last-good reload 语义
- `zircon_asset/src/tests/project/manager.rs`
  - 扫描 `assets/` 并生成 `library/`
  - 自动补写 `*.meta.toml`
- `zircon_asset/src/tests/pipeline/manager.rs`
  - 目录项目打开、资产状态和 watcher 重导入
  - `ResourceManager` status/revision/artifact locator 查询
  - reimport revision bump 和资源更新事件
- `zircon_asset/src/tests/editor/manager.rs`
  - editor catalog 构建
  - 直接引用图
  - preview dirty / visible refresh / meta 回写
- `zircon_scene/src/tests/asset_scene.rs`
  - `LevelManager` 创建/加载/保存 `LevelSystem`
  - `SceneAssetSerializer` 的 locator/handle roundtrip
- `zircon_scene/tests/viewport_packet.rs`
  - runtime world viewport packet 只保留基础 scene/preview packet
  - selection anchor / handle / grid / scene gizmo overlay 不再由 runtime world 直接生成
- `zircon_graphics/src/tests/project_render.rs`
  - headless project render
  - shader 驱动颜色回归
  - editor-composed gizmo overlay 非空帧断言
- `zircon_editor/src/tests/workbench/project/document_roundtrip.rs`
- `zircon_editor/src/tests/workbench/project/renderable_template.rs`
  - editor project/workspace roundtrip
- `zircon_editor/src/tests/editing/state.rs`
  - `EditorState` 将 catalog/detail/resource 同步为共享 `AssetWorkspaceSnapshot`
  - 资产引用跳转同时重定向 `Assets` 与 `AssetBrowser` 两个 surface
- `zircon_editor/src/tests/editing/import.rs`
  - editor import command 在 typed handle 模型下仍可 undo
- `zircon_editor/src/tests/host/resource_access/mod.rs`
  - host 侧按 `ResourceManager` 解析 ready typed handle
  - 非 ready 资源诊断透传

## Follow-up

- `zircon_graphics` 内部 shader locator 解析还在直接走 `AssetManager` 内部查询，后续可以继续收敛到更纯粹的 resource-only helper
- `mem://` 资源创建入口还未暴露给 editor/runtime
- 更完整的 PBR 扩展材质和 importer/transcoder 扩展仍需后续落地


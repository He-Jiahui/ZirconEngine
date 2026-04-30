---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_manager/src/records/asset.rs
  - zircon_manager/src/records/mod.rs
  - zircon_manager/src/lib.rs
  - zircon_manager/src/handles.rs
  - zircon_manager/src/traits.rs
  - zircon_manager/src/service_names.rs
  - zircon_manager/src/tests.rs
  - zircon_scene_protocol/Cargo.toml
  - zircon_scene_protocol/src/lib.rs
  - zircon_scene_protocol/src/world_handle.rs
  - zircon_scene_protocol/src/level_summary.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/level_system.rs
  - zircon_scene/src/render_extract/mod.rs
  - zircon_scene/src/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_scene/src/module/level_manager_lifecycle.rs
  - zircon_scene/src/module/level_manager_project_io.rs
  - zircon_scene/src/tests/boundary.rs
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_asset/src/pipeline/manager/records/asset_status_record.rs
  - zircon_asset/src/pipeline/manager/records/status_record.rs
  - zircon_asset/src/pipeline/manager/records/metadata_import_state.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/resource_manager_contract.rs
  - zircon_asset/src/pipeline/manager/registration/service_names.rs
  - zircon_asset/src/pipeline/manager/asset_manager/mod.rs
  - zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs
  - zircon_asset/src/tests/editor/boundary.rs
  - zircon_asset/src/tests/editor/manager.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_resource/src/marker.rs
  - zircon_resource/src/record/resource_event.rs
  - zircon_resource/src/record/resource_event_kind.rs
  - zircon_resource/src/record/resource_record.rs
  - zircon_framework/src/input/mod.rs
  - zircon_framework/src/input/input_button.rs
  - zircon_framework/src/input/input_event.rs
  - zircon_framework/src/input/input_event_record.rs
  - zircon_framework/src/input/input_snapshot.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/input/tests/boundary.rs
  - zircon_runtime/src/input/tests/input_manager.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/handles.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_manifest.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/graphics/host/module_host/rendering_manager/manager_backend_info.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/editor_state_asset_workspace.rs
  - zircon_editor/src/core/editor_event/runtime/execution/common.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/backend_refresh.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/ui/asset_surface_presentation.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_item_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_reference_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_selection_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/host/asset_manager_boundary/mod.rs
  - zircon_editor/src/tests/host/resource_access/mod.rs
  - zircon_editor/src/tests/host/slint_asset_refresh/mod.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/tests/mod.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - Cargo.toml
  - zircon_manager/src/records/asset.rs
  - zircon_manager/Cargo.toml
  - zircon_manager/src/records/mod.rs
  - zircon_manager/src/lib.rs
  - zircon_manager/src/handles.rs
  - zircon_manager/src/traits.rs
  - zircon_scene_protocol/Cargo.toml
  - zircon_scene_protocol/src/lib.rs
  - zircon_scene_protocol/src/world_handle.rs
  - zircon_scene_protocol/src/level_summary.rs
  - zircon_scene/Cargo.toml
  - zircon_scene/src/lib.rs
  - zircon_scene/src/level_system.rs
  - zircon_scene/src/render_extract/mod.rs
  - zircon_scene/src/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_scene/src/module/level_manager_lifecycle.rs
  - zircon_scene/src/module/level_manager_project_io.rs
  - zircon_scene/src/tests/boundary.rs
  - zircon_framework/Cargo.toml
  - zircon_framework/src/input/mod.rs
  - zircon_framework/src/input/input_button.rs
  - zircon_framework/src/input/input_event.rs
  - zircon_framework/src/input/input_event_record.rs
  - zircon_framework/src/input/input_snapshot.rs
  - zircon_runtime/Cargo.toml
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_asset/src/pipeline/manager/records/asset_status_record.rs
  - zircon_asset/src/pipeline/manager/records/status_record.rs
  - zircon_asset/src/pipeline/manager/records/metadata_import_state.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/resource_manager_contract.rs
  - zircon_asset/src/pipeline/manager/registration/service_names.rs
  - zircon_asset/src/pipeline/manager/asset_manager/mod.rs
  - zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/handles.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_manifest.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/graphics/host/module_host/rendering_manager/manager_backend_info.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/editor_state_asset_workspace.rs
  - zircon_editor/src/core/editor_event/runtime/execution/common.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/backend_refresh.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/ui/asset_surface_presentation.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_item_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_reference_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_selection_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs
  - zircon_resource/src/record/resource_event.rs
  - zircon_resource/src/record/resource_event_kind.rs
plan_sources:
  - user: 2026-04-19 先根据Runtime吸收层 Editor吸收的规则迁移，外部目录干净化
  - user: 2026-04-18 直接转去处理 zircon_graphics 红测，再开一轮更严格的包边界审计标准
  - user: 2026-04-18 按审计文档里的顺序处理 AssetRecordKind 和 PreviewStateRecord 这组边界收口
  - user: 2026-04-18 继续严格包边界审计，就需要重新找新的“证据充分”候选，而不是再回头动这两组
  - user: 2026-04-18 按审计顺序继续落代码，最该先处理的就是这组输入协议边界
  - user: 2026-04-18 继续整理代码迁移，寻找不合理
  - user: 2026-04-18 下一步先做 ResourceStateRecord -> zircon_resource::ResourceState，再处理 ResourceStatusRecord 的 canonical 收口
  - user: 2026-04-18 继续下一步
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model --locked -- --nocapture
  - cargo test -p zircon_graphics --locked
  - cargo check --workspace --locked
  - cargo test -p zircon_manager manager_public_surface_excludes_asset_display_taxonomy --offline
  - cargo test -p zircon_manager --offline
  - cargo test -p zircon_asset asset_kind_and_preview_taxonomy_live_in_resource_and_asset_crates --offline
  - cargo test -p zircon_asset --offline
  - cargo check -p zircon_editor --lib --offline
  - `Select-String` 扫描 `zircon_manager` / `zircon_framework` / `zircon_runtime` / `zircon_app` / `zircon_editor` 对 `InputButton` / `InputEvent` / `InputSnapshot` / `InputEventRecord` 的定义与调用面
  - cargo test -p zircon_manager manager_public_surface_excludes_input_protocol_types --offline
  - cargo test -p zircon_runtime --lib input_protocol_types_live_in_runtime_input_surface --offline
  - cargo test -p zircon_runtime --lib --offline
  - cargo check -p zircon_framework --lib --offline
  - cargo test -p zircon_app runtime_input_protocol_is_owned_by_input_subsystem --offline
  - cargo test -p zircon_manager manager_public_surface_excludes_vm_plugin_protocol_types --offline
  - cargo test -p zircon_runtime script_subsystem_is_physically_absorbed_into_runtime_crate --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime --lib --locked --target-dir target/codex-shared-b
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b
  - cargo check --workspace --locked --target-dir target/codex-shared-b
  - cargo test -p zircon_manager tests::manager_public_surface_excludes_resource_state_mirror --offline --target-dir target/tdd-red-manager -- --exact
  - cargo test -p zircon_asset tests::editor::boundary::resource_state_protocol_lives_in_resource_crate --offline --target-dir target/tdd-red-asset -- --exact
  - cargo test -p zircon_asset resource_server_reports_resource_records_for_project_assets --offline --target-dir target/tdd-red-asset
  - cargo test -p zircon_editor asset_manager_boundary --offline --target-dir target/tdd-red-editor
  - cargo test -p zircon_editor resolve_ready_handle_returns_typed_handle_from_resource_server --offline --target-dir target/tdd-red-editor --quiet
  - cargo test -p zircon_manager tests::manager_public_surface_excludes_resource_status_wrapper --offline --target-dir target/tdd-status-manager -- --exact
  - cargo test -p zircon_asset tests::editor::boundary::resource_status_protocol_lives_in_resource_crate --offline --target-dir target/tdd-status-asset -- --exact
  - cargo test -p zircon_asset resource_server_reports_resource_records_for_project_assets --offline --target-dir target/tdd-status-asset
  - cargo test -p zircon_manager --offline --target-dir target/tdd-status-manager
  - cargo test -p zircon_asset --offline --target-dir target/tdd-status-asset
  - cargo test -p zircon_editor tests::host::asset_manager_boundary::editor_asset_workspace_uses_canonical_resource_record --offline --target-dir target/tdd-red-editor -- --exact --quiet
  - cargo test -p zircon_editor asset_manager_boundary --offline --target-dir target/tdd-red-editor --quiet
  - cargo fmt -p zircon_manager -p zircon_scene -p zircon_scene_protocol
  - cargo test -p zircon_scene_protocol --offline --target-dir target/tdd-scene-protocol-protocol
  - cargo test -p zircon_manager --offline --target-dir target/tdd-scene-protocol-manager
  - cargo test -p zircon_scene --offline --target-dir target/tdd-scene-protocol-scene
  - cargo check -p zircon_manager -p zircon_scene --offline --target-dir target/tdd-scene-protocol-check --message-format short
  - cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b asset_project_api_moves_under_project_module_namespace -- --nocapture
  - cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b asset_watch_api_moves_under_watch_module_namespace -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b legacy_template_compat_api_moves_under_template_namespace -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b template_selector_api_moves_under_template_namespace -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b template_binding_model_api_moves_under_template_namespace -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b runtime_asset_surface_keeps_project_and_watch_under_namespaces -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b runtime_ui_surface_keeps_template_and_layout_specialists_under_namespaces -- --nocapture
  - git grep -n "zircon_asset::\\|zircon_scene::\\|zircon_ui::\\|zircon_runtime/crates/zircon_" -- ':(exclude)zircon_asset' ':(exclude)zircon_scene' ':(exclude)zircon_ui'
  - cargo test -p zircon_runtime --lib --offline
  - cargo test -p zircon_editor --lib --offline
  - cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_scene --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_graphics --lib --no-run --locked --offline --target-dir target/codex-shared-b
  - cargo test -p zircon_editor --lib --no-run --locked --offline --target-dir target/codex-shared-b
doc_type: module-detail
---

# Crate Boundary Audit Round 2

## Purpose

这轮文档做两件事：

- 记录 `zircon_graphics` 红测的真实根因和最小修复，避免把一次 compile-path 断点误判成 Virtual Geometry 行为回归
- 把“明显错包”升级成一组更严格、可重复执行的 crate boundary 审计标准，并据此给出下一批迁移候选

## Graphics Red Test Root Cause

目标红测是：

- `tests::virtual_geometry_unified_indirect::virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model`

这次真正卡住它的并不是 indirect segment 计数逻辑，而是 `SceneRendererCore::render_scene(...)` 还停留在旧的 `Vec<MeshDraw>` 消费路径：

- `mesh/build_mesh_draws/build/build.rs` 已经把返回值升级成 `BuiltMeshDraws { draws, indirect_segment_count }`
- `render_scene(...)` 仍把整个 `BuiltMeshDraws` 直接传给 overlay renderer
- overlay renderer 仍然要求 `&[MeshDraw]`

因此，lib test 一开始先死在类型不匹配上，根本没有进入 unified indirect 的断言。

最小修复就是在 [render_scene.rs](../../zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs) 里按新接口解包：

- `let built_mesh_draws = build_mesh_draws(...)`
- `let mesh_draws = built_mesh_draws.draws`

这个修复恢复了 non-virtual-geometry 场景路径对新 `BuiltMeshDraws` 结构的兼容；它没有修改 indirect segment dedup 逻辑本身。修复后：

- 单个目标测试转绿
- `cargo test -p zircon_graphics --locked` 全绿

## Stricter Audit Standard

第二轮审计不再只看“名字像不像错包”，而是按下面的规则判定：

1. **Canonical enum rule**
   同一领域的公共枚举只能有一个权威拥有者；别的 crate 不应重复定义语义等价的镜像枚举再到处转换。

2. **Editor-only semantics rule**
   只服务 editor catalog、preview、authoring、interaction 的记录和状态，必须归 editor 或 asset/editor-owned 协议层，不应挂在 generic manager service contract 上。

3. **Contract minimality rule**
   契约 crate 可以定义稳定入口 trait/handle，但不应顺手拥有实现域里的展示语义、preview 状态、authoring 细节和重复 type taxonomy。

4. **Dependency direction rule**
   底层 crate 不能反向依赖 `zircon_editor`；渲染 crate 不能再承载 editor interaction；runtime entry 不能复用 editor-only controller。

5. **Root-surface rule**
   crate 根和 `mod.rs` 只应导出结构入口；如果某个 public surface 需要解释 editor preview、asset catalog 或 viewport 操作语义，那大概率已经越过 crate 责任边界。

## Audit Results

### Confirmed Passes

- `zircon_graphics` 不再拥有 `ViewportController`、`ViewportState`、`ViewportInput`、`ViewportFeedback`、`GizmoAxis`
- `zircon_editor` 已接管 viewport interaction 类型和 scene gizmo/handle 交互
- `zircon_app` runtime 已切到 crate-private camera controller，不再复用 editor controller
- `EditorAssetManager` trait、resolver、handle、records 已从 `zircon_manager` 迁回 `zircon_asset`
- 当前扫描下，没有发现底层 crate 反向依赖 `zircon_editor`

### Strong Candidate 1: `AssetRecordKind`

当前 [zircon_manager/src/records/asset.rs](../../zircon_manager/src/records/asset.rs) 里的 `AssetRecordKind` 与 [zircon_resource/src/marker.rs](../../zircon_resource/src/marker.rs) 里的 `ResourceKind` 是一组语义重复的公共枚举：

- 两者成员集合一致：`Model / Material / Texture / Shader / Scene / UiLayout / UiWidget / UiStyle`
- 迁移前旧的 pipeline-manager records 投影链每次投影 `AssetMetadata` 时都要把 `AssetKind` 手动映射成 `AssetRecordKind`
- `zircon_editor` 的 asset workspace、resource access、Slint asset surface 继续消费的是 `AssetRecordKind`

按更严格标准，这属于典型的 “canonical enum duplication”：

- 权威种类枚举已经在 `zircon_resource::ResourceKind`
- `zircon_manager` 现在只是重复定义一份 façade 版本，并让 asset/editor 两侧继续做镜像转换

下一批迁移建议：

- 优先把 `AssetRecordKind` 收敛到 `zircon_resource::ResourceKind`
- `AssetStatusRecord` / `ResourceStatusRecord` 直接使用 canonical `ResourceKind`
- editor asset workspace、resource access、UI projection 一并改到 canonical kind

### Strong Candidate 2: `PreviewStateRecord`

当前 `PreviewStateRecord` 仍在 [zircon_manager/src/records/asset.rs](../../zircon_manager/src/records/asset.rs)，但它的实际消费面已经明显偏向 asset/editor：

- `zircon_asset/src/editor/manager.rs` 负责把 `PreviewState` 投影成 `PreviewStateRecord`
- `zircon_editor` 的 asset workspace 和 host tests 通过这组 preview 状态来投影 catalog/details
- `zircon_manager::traits::AssetManager` / `ResourceManager` 并不直接暴露 preview 生命周期

按 “editor-only semantics rule”，preview 状态不属于 generic manager service contract。它描述的是：

- editor preview cache 是否 dirty
- asset catalog 中的缩略图是否 ready/error

这组语义应该归 `zircon_asset`，更准确地说应归 `zircon_asset::editor` 这条 editor-facing 资产协议。

下一批迁移建议：

- `PreviewStateRecord` 从 `zircon_manager` 移到 `zircon_asset`
- 与 `EditorAssetDetailsRecord` / `AssetCatalogRecord` / preview cache 协议一起收口
- `zircon_editor` 跟随切换 imports

### Watchlist: `zircon_manager::service_names`

[zircon_manager/src/service_names.rs](../../zircon_manager/src/service_names.rs) 现在仍然持有：

- `ASSET_MANAGER_NAME = "AssetModule.Manager.AssetManager"`
- `RENDERING_MANAGER_NAME = "GraphicsModule.Manager.RenderingManager"`
- `LEVEL_MANAGER_NAME = "SceneModule.Manager.LevelManager"`

从“名字拥有权”看，这些常量更像实现域信息，而不是 manager service contract 自己的领域模型。

但这轮我把它列为 watchlist，而不是立即迁移，原因是：

- `ManagerResolver` 需要一个稳定 façade surface
- 这些常量当前承担的是跨模块解析协议，不是 editor-only 语义
- 现阶段证据还不足以证明“移动常量所有权”能换来明确收益，而不会只是换个 import 路径

因此暂不作为下一批主候选。

### Watchlist Review: Keep `service_names` In `zircon_manager`

这次继续往 watchlist 深挖后，我把 `zircon_manager::service_names` 从“待观察”降到了“明确保留”。

原因不是它看起来更像某个实现 crate 的字符串，而是它现在已经承担了 façade 契约的一部分：

- [zircon_manager/src/resolver.rs](../../zircon_manager/src/resolver.rs) 的 `resolve_asset_manager` / `resolve_resource_manager` / `resolve_rendering_manager` / `resolve_level_manager` / `resolve_input_manager` 直接把这些常量作为解析协议使用
- `ManagerResolver::{asset, resource, rendering, level, input}` 这些稳定入口也直接绑定到同一组常量
- [zircon_asset/src/pipeline/manager/registration/service_names.rs](../../zircon_asset/src/pipeline/manager/registration/service_names.rs)、[zircon_scene/src/module/service_names.rs](../../zircon_scene/src/module/service_names.rs)、[zircon_runtime/src/graphics/host/module_host/module_registration/service_names.rs](../../zircon_runtime/src/graphics/host/module_host/module_registration/service_names.rs) 当前都不是自定义一份新名字，而是主动 alias `zircon_manager::*_MANAGER_NAME`
- 上层调用方如 [zircon_app/src/entry/tests/mod.rs](../../zircon_app/src/entry/tests/mod.rs) 直接通过 `zircon_manager::resolve_*` / `ManagerResolver` 消费这组 façade 入口，而不是绕到实现 crate 私有名字上

这意味着如果现在把这些常量“迁回实现域”，真正要改的不是 import 路径，而是：

- `zircon_manager` resolver 的公开契约
- 各实现 crate 对 façade manager 名称的 alias 关系
- 上层启动与测试对 `resolve_*` / `ManagerResolver` 的假设

在当前证据下，这组名字已经更接近“façade registry contract”，而不是“实现细节误挂在 manager crate”。所以严格边界标准下，它们应该留在 `zircon_manager`，不再列为下一批迁移候选。

### Implemented Batch: Input Protocol Types

这一组输入协议类型已经按审计顺序完成收口，不再停留在候选阶段：

- `InputButton`
- `InputEvent`
- `InputEventRecord`
- `InputSnapshot`

原先这些类型挂在 `zircon_manager/src/records/input.rs`，同时被输入子系统运行态实现和 `zircon_app` 的 runtime 输入桥直接消费。它们满足“子系统协议类型错挂在 façade 层”的全部证据门槛：

1. 它们不是 generic manager service contract 自己的抽象，而是纯输入语义
2. 当前输入协议 owner 已收束到 `zircon_framework::input`，而 `zircon_runtime::input` 是唯一真实拥有输入模块生命周期和运行态状态机的实现表面
3. `zircon_manager` 一边定义 `InputManager` trait，一边顺手拥有整套输入协议模型，导致实现 crate 反向依赖 façade 类型

这批最终没有继续保留独立 `zircon_input` / `zircon_input_protocol` crate，而是进一步收束为 `zircon_framework::input` 契约 + `zircon_runtime::input` 实现：

- [zircon_framework/src/input/mod.rs](../../zircon_framework/src/input/mod.rs)、[zircon_framework/src/input/input_button.rs](../../zircon_framework/src/input/input_button.rs)、[zircon_framework/src/input/input_event.rs](../../zircon_framework/src/input/input_event.rs)、[zircon_framework/src/input/input_event_record.rs](../../zircon_framework/src/input/input_event_record.rs)、[zircon_framework/src/input/input_snapshot.rs](../../zircon_framework/src/input/input_snapshot.rs) 成为这组类型的唯一 owner
- [zircon_manager/src/traits.rs](../../zircon_manager/src/traits.rs) 只保留 `InputManager` façade trait，本身改为直接依赖 `zircon_framework`
- `zircon_manager` 根和 `records/mod.rs` 不再 re-export 输入协议类型，旧的 `src/records/input.rs` 已删除
- [zircon_runtime/src/input/mod.rs](../../zircon_runtime/src/input/mod.rs) 继续作为 runtime 输入子系统的公共入口 re-export 这组协议；[zircon_runtime/src/input/runtime/default_input_manager.rs](../../zircon_runtime/src/input/runtime/default_input_manager.rs) 与 [zircon_runtime/src/input/runtime/input_state.rs](../../zircon_runtime/src/input/runtime/input_state.rs) 改为消费 crate-local re-export
- [zircon_app/src/entry/runtime_entry_app/application_handler.rs](../../zircon_app/src/entry/runtime_entry_app/application_handler.rs) 改成从 `zircon_runtime::input` 导入 `InputButton` / `InputEvent`，runtime 不再从 `zircon_manager` 获取输入协议类型

这样 `service_names` 仍保留在 façade contract 层，而输入协议则下沉为真正的共享协议层，两者边界不再混淆。

### Implemented Batch: Script VM Private Protocol And Handles

这轮继续往下扫后，新一组证据充分的错包是原先挂在 `zircon_manager` 的脚本 VM 私有协议/handle：

- `CapabilitySet`
- `HostHandle`
- `PluginSlotId`

它们的问题比上一轮输入协议更直接：

1. 这组类型完全不经过任何 `ManagerResolver`、manager trait 或 façade 返回记录
2. 当前真实消费面只在 `zircon_runtime::script`：
   - [zircon_runtime/src/script/vm/plugin/vm_plugin_manifest.rs](../../zircon_runtime/src/script/vm/plugin/vm_plugin_manifest.rs) 的 `VmPluginManifest.capabilities`
   - [zircon_runtime/src/script/vm/host/host_registry.rs](../../zircon_runtime/src/script/vm/host/host_registry.rs) 的 host capability handle 注册
   - [zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs](../../zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs) 的 plugin slot 生命周期
3. `zircon_manager` 只是顺手持有了脚本 VM 的私有协议和句柄，并没有任何 façade 入口真正需要它们

因此这批的 canonical owner 很明确，就是 `zircon_runtime::script` 本身，而不是 `zircon_manager`。

实现结果：

- 新增 [zircon_runtime/src/script/vm/capability_set.rs](../../zircon_runtime/src/script/vm/capability_set.rs) 和 [zircon_runtime/src/script/vm/handles.rs](../../zircon_runtime/src/script/vm/handles.rs)
- [zircon_runtime/src/script/vm/mod.rs](../../zircon_runtime/src/script/vm/mod.rs) 与 [zircon_runtime/src/script/mod.rs](../../zircon_runtime/src/script/mod.rs) 改为由脚本子系统根级 re-export `CapabilitySet` / `HostHandle` / `PluginSlotId`
- [zircon_manager/src/lib.rs](../../zircon_manager/src/lib.rs)、[zircon_manager/src/records/mod.rs](../../zircon_manager/src/records/mod.rs)、[zircon_manager/src/handles.rs](../../zircon_manager/src/handles.rs) 已删除这组 public surface；旧的 `src/records/capability_set.rs` 已删除
- `zircon_runtime::script` 子树内部实现不再从 `zircon_manager` 反向取脚本私有类型；随着独立 `zircon_script` package 删除，这组能力现在只经 runtime script surface 暴露

这批收口以后，脚本 VM 的 manifest/state/hot-reload 协议终于回到了脚本子系统自己名下；`zircon_manager` 继续只保留真正的 manager service contract surface。

### Implemented Batch: AssetManager Protocol Boundary

继续往下扫后，`AssetPipelineInfo` / `ProjectInfo` 并不是“仍应保留在 façade 层的剩余 record”；真正的强候选其实是整条 generic `AssetManager` protocol boundary：

- `AssetManager` trait 只有 `zircon_asset` 一处真实实现
- `AssetManagerHandle` 与 `resolve_asset_manager` 都只是 asset 子系统公共入口；而 `ASSET_MANAGER_NAME` 属于后续继续下沉到 `zircon_runtime::asset` 的 module-registration surface，并不属于 generic manager service contract 自己的领域模型
- `AssetChangeRecord` / `AssetChangeKind` 只是 `zircon_asset::watch::AssetChange` 的镜像投影，增加了无意义 DTO 层

因此这批最终不是只搬 DTO，而是整条协议线一起迁回 `zircon_asset`：

- 新增 [zircon_asset/src/pipeline/manager/asset_manager/mod.rs](../../zircon_asset/src/pipeline/manager/asset_manager/mod.rs)，并把 trait/handle/resolver 继续下沉到 [zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs](../../zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs)、[zircon_asset/src/pipeline/manager/asset_manager/asset_manager_handle.rs](../../zircon_asset/src/pipeline/manager/asset_manager/asset_manager_handle.rs) 与 [zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs](../../zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs)
- [zircon_asset/src/pipeline/manager/mod.rs](../../zircon_asset/src/pipeline/manager/mod.rs) 与 [zircon_asset/src/lib.rs](../../zircon_asset/src/lib.rs) 现在直接拥有 `AssetManager`、`AssetManagerHandle`、`resolve_asset_manager`、`AssetPipelineInfo` 与 `ProjectInfo`
- [zircon_runtime/src/asset/mod.rs](../../zircon_runtime/src/asset/mod.rs) 与 [zircon_runtime/src/asset/module.rs](../../zircon_runtime/src/asset/module.rs) 继续持有 `AssetModule`、`ASSET_MODULE_NAME`、`ASSET_MANAGER_NAME`、`RESOURCE_MANAGER_NAME`、`PROJECT_ASSET_MANAGER_NAME` 与 `EDITOR_ASSET_MANAGER_NAME`
- [zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs](../../zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs) 的 `subscribe_asset_changes()` 已统一返回 `ChannelReceiver<zircon_asset::watch::AssetChange>`
- `zircon_manager` 已删除 `AssetManager` trait、asset/project records、handle、resolver 和 service-name surface；旧的 [zircon_manager/src/records/asset.rs](../../zircon_manager/src/records/asset.rs) 与 [zircon_manager/src/records/project.rs](../../zircon_manager/src/records/project.rs) 已删除

这说明 `AssetPipelineInfo` / `ProjectInfo` 已经随 canonical owner 一起回到 asset 子系统，不再属于剩余 watchlist。

### Implemented Batch: Scene Level Protocol Types

本轮继续重扫 `zircon_manager` 现存 façade surface 后，新的高证据候选不是 `RenderingBackendInfo`，而是 scene level 协议自己的 handle/summary：

- `WorldHandle`
- `LevelSummary`

它们的问题和上一轮 `InputButton` / `InputEvent` 很像：

1. 这组类型只服务 `LevelManager` 协议与 `zircon_scene` 运行时实现
2. 当前真实生产/消费面只在 [zircon_scene/src/module/default_level_manager.rs](../../zircon_runtime/src/scene/module/default_level_manager.rs)、[zircon_runtime/src/scene/module/level_manager_contract.rs](../../zircon_runtime/src/scene/module/level_manager_contract.rs)、[zircon_scene/src/level_system.rs](../../zircon_runtime/src/scene/level_system.rs) 与 [zircon_scene/src/render_extract/mod.rs](../../zircon_scene/src/render_extract/mod.rs)
3. `zircon_manager` 只是顺手持有了 scene 专属 handle 和 summary，本身并不是这组协议的实现 owner

但它们不能直接迁到 `zircon_scene`，否则会形成依赖环：

- `zircon_scene` 依赖 `zircon_manager::LevelManager`
- 如果 `zircon_manager` 再反向依赖 `zircon_scene` 来拿 `WorldHandle` / `LevelSummary`，就会出现 cycle

因此最终方案是新增独立协议层 [zircon_scene_protocol/src/lib.rs](../../zircon_scene_protocol/src/lib.rs)：

- [zircon_scene_protocol/src/world_handle.rs](../../zircon_scene_protocol/src/world_handle.rs) 成为 `WorldHandle` 的唯一 owner
- [zircon_scene_protocol/src/level_summary.rs](../../zircon_scene_protocol/src/level_summary.rs) 成为 `LevelSummary` 的唯一 owner
- [zircon_manager/src/lib.rs](../../zircon_manager/src/lib.rs) 不再 re-export `WorldHandle` / `LevelSummary`；旧的 [zircon_manager/src/handles.rs](../../zircon_manager/src/handles.rs) 与 [zircon_manager/src/records/level.rs](../../zircon_manager/src/records/level.rs) 已删除
- [zircon_manager/src/traits.rs](../../zircon_manager/src/traits.rs) 改为直接依赖 `zircon_scene_protocol`
- [zircon_scene/src/lib.rs](../../zircon_scene/src/lib.rs) 继续从 scene 子系统根级 re-export `WorldHandle` / `LevelSummary`，运行时实现不再从 `zircon_manager` 反向取 scene 协议类型

这样 `zircon_manager` 继续只保留 `LevelManager` façade trait，自身不再错误持有 scene 专属协议模型。

同一轮 runtime absorption 又继续推进了 scene/asset module-registration owner：

- [zircon_runtime/src/scene/mod.rs](../../zircon_runtime/src/scene/mod.rs) 与 [zircon_runtime/src/scene/module/mod.rs](../../zircon_runtime/src/scene/module/mod.rs) 现在持有 `SceneModule`、`SCENE_MODULE_NAME`、`DEFAULT_LEVEL_MANAGER_NAME`、`LEVEL_MANAGER_NAME`、`create_default_level()` 与 `load_level_asset()`
- [zircon_runtime/src/asset/mod.rs](../../zircon_runtime/src/asset/mod.rs) 与 [zircon_runtime/src/asset/module.rs](../../zircon_runtime/src/asset/module.rs) 现在持有 `AssetModule`、`module_descriptor()` 与整组 asset/resource manager service names
- [zircon_scene/src/lib.rs](../../zircon_scene/src/lib.rs) 与 [zircon_asset/src/lib.rs](../../zircon_asset/src/lib.rs) 则继续收窄到 world authority / asset API 本体，不再把 module owner、service-name 和 bootstrap helper 混在领域 root surface

### Not Promoted: `zircon_ui` Binding / Reflection / Template Protocols

这轮我也重新扫了 `zircon_ui::binding::{UiBindingValue, UiBindingCall, UiEventKind, UiEventPath}`、`zircon_ui::event_ui::{UiControlRequest, UiControlResponse, UiReflectionSnapshot, UiNodeDescriptor, UiPropertyDescriptor}` 与 `zircon_ui::template::{UiTemplateLoader, UiDocumentCompiler}` 这组高频 surface。

它们在 editor/editor_ui 里消费面很大，但当前证据还不足以把它们升级成“明显错包”，原因是：

- [zircon_ui/src/binding/mod.rs](../../zircon_ui/src/binding/mod.rs) 与 [zircon_ui/src/event_ui/mod.rs](../../zircon_ui/src/event_ui/mod.rs) 自己就拥有 binding parser、event manager、reflection store 和 invocation 协议
- [zircon_ui/src/template/mod.rs](../../zircon_ui/src/template/mod.rs) 自己拥有 template loader、validator、asset loader、document compiler 和 surface builder
- `zircon_ui` crate 内部测试直接覆盖这组能力：[zircon_ui/src/tests/binding.rs](../../zircon_ui/src/tests/binding.rs)、[zircon_ui/src/tests/event_manager.rs](../../zircon_ui/src/tests/event_manager.rs)、[zircon_ui/src/tests/template.rs](../../zircon_ui/src/tests/template.rs)、[zircon_ui/src/tests/asset.rs](../../zircon_ui/src/tests/asset.rs)

也就是说，虽然 editor 是这组协议的重度调用方，但权威实现和验证目前仍然在 `zircon_ui` 子系统内部。这更像“UI runtime/authoring 共用协议”而不是“editor-only 语义误挂在 runtime crate”。因此这轮先不升级它，继续留在 watchlist。

这轮再把 watchlist 收窄到 historical template fixture conversion 后，结论已经从“继续保留 adapter surface”更新为“formal public surface 不保留 adapter”：

- [zircon_runtime/src/ui/template/document.rs](../../zircon_runtime/src/ui/template/document.rs) 里的 `UiTemplateDocument` 仍服务 shared template fixture/test coverage；production editor/runtime path 已经固定到 tree-shaped `UiAssetDocument`
- [zircon_editor/src/ui/template/registry.rs](../../zircon_editor/src/ui/template/registry.rs) 生产态只托管 compiled asset documents，不再托管 legacy template document authority
- [zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs](../../zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs) 生产态只接受 tree `UiAssetDocument` source，不再以 `UiTemplateLoader` fallback 解析旧模板输入
- historical template / flat fixture conversion 只保留在 runtime test support 与 editor test support，用来把旧测试夹具转成 canonical tree TOML

因此 `UiTemplateDocument` / `UiTemplateLoader` 当前只是 shared fixture/test support 输入面，不再构成 production owner 迁移候选或 formal public adapter surface。

### Keep In `zircon_manager`

这轮严格审计后，下面几组我判断仍应留在 façade 层：

- `RenderingBackendInfo`
  原因：它是 `RenderingManager` façade 的能力摘要，不携带 editor-specific 或 preview-specific 语义；而且这轮已经补齐了“它不是空心壳”的证据链：
  - [zircon_runtime/src/graphics/host/module_host/rendering_manager/manager_backend_info.rs](../../zircon_runtime/src/graphics/host/module_host/rendering_manager/manager_backend_info.rs) 为 `WgpuRenderingManager` 提供了实际 `backend_info()` 实现
  - [zircon_runtime/src/graphics/host/module_host/module_registration/module_descriptor.rs](../../zircon_runtime/src/graphics/host/module_host/module_registration/module_descriptor.rs) 把它注册成 `GraphicsModule.Manager.RenderingManager`
  - [zircon_app/src/entry/tests/builtin_engine_entry.rs](../../zircon_app/src/entry/tests/builtin_engine_entry.rs) 仍通过 `resolve_rendering_manager(...).backend_info()` 验证 bootstrap 合同
  - [docs/assets-and-rendering/srp-rhi-render-server-architecture.md](../assets-and-rendering/srp-rhi-render-server-architecture.md) 与 [docs/assets-and-rendering/index.md](../assets-and-rendering/index.md) 也已经把它放在“`RenderingManager` 向纯兼容桥继续收束”的语境里

  现阶段它更像“仍被消费的 façade compat contract”，不是应该立即下沉到 graphics 内部的孤立 DTO。

### New Strong Candidate: `zircon_asset` Root Foreign Re-Exports Of `zircon_resource`

这轮避开 `zircon_editor` / `zircon_graphics` 热区继续重扫时，新的高证据候选不是再搬一组实现类型，而是清理 `zircon_asset` 根级错误代持的 foreign surface：

- [zircon_asset/src/lib.rs](../../zircon_asset/src/lib.rs) 当前直接 `pub use zircon_resource::{...}`，把 `ResourceLocator`、`ResourceKind`、`ResourceRecord`、`ResourceState`、`ResourceManager`、marker/type-descriptor/runtime-info 等一整批 raw resource 基础类型重新挂到了 asset crate 根下
- 但这批 raw surface 的 canonical owner 明确还是 `zircon_resource`；`zircon_asset` 自己真正拥有的是 `AssetManager`、project/import/watch/editor asset API，以及基于 resource 基础层再包装出来的 asset 语义
- 这轮对 workspace 和 `docs/` 的源码扫描里，没有发现任何 `zircon_asset::ResourceLocator`、`zircon_asset::ResourceKind`、`zircon_asset::ResourceRecord`、`zircon_asset::ResourceManager` 等 raw resource 外部消费面；后续补扫确认 `zircon_asset::AssetReference` 仍被 scene/editor/graphics 测试与 helper 代码当作 asset 语义别名消费，因此它应保留为 asset-named alias，而不是与 raw resource surface 一起删除
- 相反，当前实际代码已经直接使用 canonical owner：
  - [zircon_asset/src/assets/ui.rs](../../zircon_asset/src/assets/ui.rs) 直接从 `zircon_resource` 取 `AssetReference` / `ResourceLocator`
  - [zircon_scene/src/serializer/mod.rs](../../zircon_scene/src/serializer/mod.rs) 与 scene module 子树直接从 `zircon_resource` 取 `ResourceLocator`
  - `zircon_editor` / `zircon_app` 当前也没有通过 `zircon_asset` 根去拿任何 raw resource 类型

这符合一条新的高证据模式：

1. foreign subsystem 的 raw contract 已经有清晰 canonical owner
2. 代持 crate 根只是在重复暴露同一组类型
3. workspace 内外部消费已经全部绕过这层代持 surface

因此这条线更像“crate root 边界泄漏”，而不是必须保留的兼容入口。下一批最合理的收口不是搬模块，而是：

- 从 `zircon_asset` 根去掉对 `zircon_resource` raw surface 的大面积 re-export
- 只保留真正有 asset 语义的根级入口：`AssetId` / `AssetKind` / `AssetUri` / `AssetReference` / `AssetUuid`
- 把 `AssetMetadata` / `AssetRegistry` / `AssetUriError` / `AssetUriScheme` 这类 raw resource helper 从 `zircon_asset` 根移除，直接回到 `zircon_resource`
- `ProjectManager` / `ProjectManifest` / `ProjectPaths` 统一通过 `zircon_asset::project::*` 访问，不再继续由 asset root 平铺
- 让 raw resource contract 只从 `zircon_resource` 获取，避免 `zircon_asset` 根继续伪装成 resource foundation owner

## Implemented Batch

这一批已经按审计顺序完成收口，不再停留在候选状态：

1. `zircon_manager::AssetRecordKind` 已删除，`AssetStatusRecord.kind` / `ResourceStatusRecord.kind` 统一改成 `zircon_resource::ResourceKind`
2. `zircon_manager::PreviewStateRecord` 已删除，editor-facing catalog/details/reference records 直接使用 `zircon_asset::project::PreviewState`
3. `zircon_asset` 不再做 façade taxonomy 投影；`editor/records.rs` 与 `pipeline/manager/records/mod.rs` 下的 records 子树都直接传递 canonical kind / preview state
4. `zircon_editor` asset workspace、resource access、event filter parser、Slint asset surface、snapshot structs 和对应测试全部切到 `ResourceKind`
5. `zircon_framework::input` 成为 `InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot` 的独立协议 owner
6. `zircon_manager` 删除输入协议 re-export 与 `src/records/input.rs`，只保留 `InputManager` façade trait
7. `zircon_runtime::input` 根级 re-export 输入协议，运行态实现不再从 `zircon_manager` 反向取输入类型
8. `zircon_app` runtime 输入桥改为从 `zircon_runtime::input` 导入输入协议类型
9. `CapabilitySet` / `HostHandle` / `PluginSlotId` 已从 `zircon_manager` 迁回 `zircon_runtime::script`
10. `zircon_runtime::script` 不再经 `zircon_manager` 暴露脚本 VM 私有协议和热重载句柄；独立 `zircon_script` package 已删除
11. `zircon_manager::ResourceStateRecord` 已删除，`ResourceStatusRecord.state` 统一改成 `zircon_resource::ResourceState`
12. `zircon_runtime` asset pipeline service contract、`zircon_editor` resource access / asset snapshots / asset surface 和对应测试已切到 canonical `ResourceState`
13. `zircon_manager::ResourceStatusRecord` 已删除，`ResourceManager::{resource_status,list_resources}` 统一改成 `zircon_resource::ResourceRecord`
14. `zircon_asset` 不再把 `AssetMetadata`/registry record 投影成字符串化 contract DTO；resource contract 直接返回 cloned `ResourceRecord`
15. `zircon_editor` asset workspace / resource access / host tests 已切到 typed `ResourceRecord`，不再把 `id` / `locator` / `artifact_locator` / `diagnostics` 降级成字符串 record
16. `zircon_manager::ResourceChangeKind` / `ResourceChangeRecord` 已删除，`ResourceManager::subscribe_resource_changes` 统一改成 `ChannelReceiver<zircon_resource::ResourceEvent>`
17. `zircon_asset` 不再把 `ResourceEvent` 桥接成字符串化 contract DTO；resource contract 直接转发资源子系统原生事件流
18. `zircon_editor` host refresh planner、Slint asset refresh 测试和边界测试已切到 typed `ResourceEvent` / `ResourceEventKind`
19. generic `AssetManager` trait、`AssetManagerHandle` 与 `resolve_asset_manager` 已从 `zircon_manager` 迁回 `zircon_asset`；对应的 `AssetModule` / `ASSET_MANAGER_NAME` module-registration surface 则继续收口到 `zircon_runtime::asset`
20. `AssetPipelineInfo` / `ProjectInfo` 已随 `AssetManager` 协议线回到 `zircon_asset`，不再作为 façade 残留 record 保留在 `zircon_manager`
21. `AssetManager::subscribe_asset_changes` 已统一改成 `ChannelReceiver<zircon_asset::watch::AssetChange>`，`AssetChangeRecord` / `AssetChangeKind` 镜像已删除
22. `zircon_editor`、`zircon_app` 与 `zircon_graphics` 相关消费者已切到 asset-owned handle / resolver / change stream
23. 新增 `zircon_scene_protocol` 作为 `WorldHandle` / `LevelSummary` 的独立协议 owner
24. `zircon_manager` 删除 `WorldHandle` / `LevelSummary` re-export，`src/handles.rs` 与 `src/records/level.rs` 已删除
25. `zircon_scene` 根级 re-export scene protocol 类型，`level_system` / `render_extract` / `DefaultLevelManager` / `LevelManagerContract` 不再从 `zircon_manager` 反向取 scene 协议
26. `zircon_scene` 根级对 `WorldHandle` / `LevelSummary` 的 framework-owned re-export 已删除；crate 内部统一改成直接依赖 `zircon_framework::scene`
27. `zircon_asset` 根级只保留 asset 语义 alias；`AssetMetadata` / `AssetRegistry` / `AssetUriError` / `AssetUriScheme` 已从 root 移除，未接线的 `src/registry/**` 与 `src/uri.rs` 也已删除
28. `zircon_asset` 的 project public API 已收束到 `zircon_asset::project::{ProjectManager, ProjectManifest, ProjectPaths}`；workspace 调用点不再从 asset root 平铺取这组三元组
29. `zircon_runtime::ui` root 不再平铺 `UiTemplateDocument` / `UiTemplateLoader`；historical template fixture conversion 只保留在 runtime/editor test support，production editor/runtime path 统一走 tree `UiAssetDocument`
30. `zircon_asset` 的 watch public API 也已收束到 `zircon_asset::watch::{AssetChange, AssetChangeKind, AssetWatchEvent, AssetWatcher}`；asset root 不再继续平铺 watch 子域
31. `zircon_ui` root 不再平铺 compiler/layout/surface specialist surface；`UiCompiledDocument` / `UiDocumentCompiler` / `UiStyleResolver` 统一走 `zircon_ui::template::*`，`compute_layout_tree` / `compute_virtual_list_window` / `solve_axis_constraints` 统一走 `zircon_ui::layout::*`，`UiRenderExtract` / `UiRenderCommand*` / `UiResolvedStyle` / `UiVisualAssetRef` 统一走 `zircon_ui::surface::*`
32. `zircon_runtime::ui` root 继续把 template asset component-schema / reflection surface 收回 `zircon_runtime::ui::template::{UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema, UiStyleScope}`；editor `ui_asset` core 与 runtime asset compiler 调用点不再从 root 取这组 schema 类型
33. `zircon_ui` root 也继续把 template asset selector/parser surface 收回 `zircon_ui::template::{UiSelector, UiSelectorToken}`；selector stylesheet 的 parse/match model 现在只从 template namespace 获取，crate 内部 style compiler 也不再经 root 绕行
34. `zircon_ui` root 继续把 template document/binding model 收回 `zircon_ui::template::{UiActionRef, UiBindingRef, UiComponentTemplate, UiSlotTemplate}`；`zircon_editor` 的 binding inspector / command / document diff / template adapter 现在显式从 template namespace 取这组类型，`zircon_ui` 自己的 tree metadata 与 asset document 也不再经 crate root 绕行
35. `zircon_runtime::{asset,ui}` 吸收层入口也已回到 namespace-first 结构：runtime asset 只保留 `project` / `watch` namespace，runtime ui 只保留 `layout` / `surface` / `template` / `tree` namespace，不再在吸收层重新扁平化这些子域 surface；上述 template component-schema / reflection / selector-parser / binding-model surface 也不会在 runtime ui root 重新出现
36. `zircon_ui` root 不再平铺 binding / event-ui specialist surface；`UiBindingValue` / `UiBindingCall` / `UiEventKind` / `UiEventPath` 统一走 `zircon_ui::binding::*`，`UiControlRequest` / `UiControlResponse` / `UiReflectionSnapshot` / `UiNodeDescriptor` / `UiPropertyDescriptor` / `UiRouteId` / `UiTreeId` 统一走 `zircon_ui::event_ui::*`，`zircon_runtime::ui` 也只保留 `binding` / `event_ui` namespace 而不再重新拍平这两簇 DTO

这样 `zircon_manager` 继续保留 façade record/trait 本身，但不再重复拥有 asset/editor 展示语义，也不再错误持有输入子系统、资源子系统或 scene 子系统的专属协议模型。

## 2026-04-19 Runtime/Editor Absorption Cleanup

这轮补的是 owner cutover 后留下的“外部目录不干净”问题，而不是继续往上叠功能。

- `zircon_runtime/src/asset/tests/editor/manager.rs`、`zircon_runtime/src/scene/tests/component_structure.rs`、`zircon_runtime/src/ui/tests/asset.rs` 已切到 absorbed subtree 的真实根路径，边界测试不再继续引用旧的 `src/editor/manager`、`src/components`、`src/template/asset/compiler`
- `zircon_runtime/src/ui/mod.rs` 的 runtime UI host re-export 文本已重新对齐，`runtime_ui_host_surface_is_absorbed_into_runtime_ui_surface` 重新回到绿色
- `zircon_runtime/src/graphics/**` owner tree 已恢复到 runtime 目录，`graphics_module_host_is_absorbed_into_runtime_graphics_surface` 与 `graphics_runtime_host_no_longer_owns_legacy_preview_or_render_service_wiring` 重新转绿
- 针对工作树的代码级扫描已经看不到新的 `zircon_asset::`、`zircon_scene::`、`zircon_ui::` 实际消费点；这些旧 owner 名称现在主要只剩在历史文档，而不是编译链路
- `cargo test -p zircon_runtime --lib --offline` 当前结果是 `175 passed / 6 failed`，剩余 6 个失败全部集中在 animation binary asset 的 bincode 解析链，不属于这轮 Runtime/Editor 吸收边界
- `cargo test -p zircon_editor --lib --offline` 当前结果是 `596 passed / 0 failed`，证明 editor 侧已经稳定消费 absorbed runtime asset/scene/ui surface

graphics public owner cutover 在入口侧已经继续收口：

- `zircon_runtime::builtin_runtime_modules()` 现在直接持有 `GraphicsModule`
- `zircon_app/src/entry/builtin_modules.rs` 不再手工插入 `GraphicsModule`
- `asset -> graphics -> scene` 这段 builtin module 顺序现在完全由 runtime 持有，app 只保留 editor profile 附加模块

当前继续需要观察的是更深层 graphics 内部 helper/public surface，而不是入口 bootstrap 的 owner 关系。

## Remaining Tasks

当前已经从候选里划掉的批次包括：

- viewport interaction
- `AssetRecordKind` / `PreviewStateRecord`
- 输入协议 `InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot`
- `CapabilitySet` / `HostHandle` / `PluginSlotId`
- `ResourceStateRecord` / `ResourceStatusRecord` / `ResourceChangeRecord`
- generic `AssetManager` 协议线
- `WorldHandle` / `LevelSummary`

继续往下做时，当前剩余 TODO tasks 是：

1. 继续审计 runtime graphics 内部较深的 helper/public surface，确认 graphics owner cutover 在入口之外也没有遗留 root-surface 泄漏；`GraphicsModule` 本身已经稳定由 `zircon_runtime::graphics` 与 `zircon_runtime::builtin_runtime_modules()` 持有。
2. `RenderingBackendInfo` 目前已有充分 keep 证据；后续只需在 `RenderingManager` façade 真正删除或出现第二实现时再重开审计。
3. 继续细分 `zircon_runtime::ui` watchlist；当前 production path 已经固定 tree authority，下一步若要再升级，先要证明 `UiTemplateDocument` / `UiTemplateLoader` 已经脱离 runtime 自己的 validator/instance/surface fixture coverage。
4. 持续扫 live `docs/` 中对旧 asset/scene owner 的残留描述；当前仓库没有 `docs/source/` 目录，因此文档清扫目标以总览型文档为准。
5. `zircon_asset` 根级 raw-resource foreign re-export、project flatten 与 watch flatten 已完成，`zircon_runtime::{asset,ui}` 也已经回到 namespace-first surface；在 binding / event_ui 这两簇收口闭环后，下一批更合理的非热区候选转向 `zircon_ui` 其它仍高扇出的 root surface，以及 repo 内仍假定 runtime world 会生成 editor overlay 的残留测试/文档。
6. 在不碰 `zircon_editor` / `zircon_graphics` 热区的前提下，继续找新的“证据充分”候选；除这条 root-surface 泄漏外，本轮还没有发现新的高证据错包。

## Validation

这轮实际执行并确认了：

- `cargo test -p zircon_graphics virtual_geometry_prepare_uses_one_visibility_owned_indirect_segment_across_multi_primitive_model --locked -- --nocapture`
- `cargo test -p zircon_graphics --locked`
- `cargo check --workspace --locked`
- `cargo test -p zircon_manager manager_public_surface_excludes_asset_display_taxonomy --offline`
- `cargo test -p zircon_manager --offline`
- `cargo test -p zircon_asset asset_kind_and_preview_taxonomy_live_in_resource_and_asset_crates --offline`
- `cargo test -p zircon_asset --offline`
- `cargo check -p zircon_editor --lib --offline`
- `Select-String` 扫描 `zircon_manager` / `zircon_asset` / `zircon_scene` / `zircon_graphics` / `zircon_app` 对 `*_MANAGER_NAME` 和 `resolve_*` 的调用面
- `Select-String` 扫描 `zircon_manager` / `zircon_framework` / `zircon_runtime` / `zircon_app` / `zircon_editor` 对 `InputButton` / `InputEvent` / `InputSnapshot` / `InputEventRecord` 的定义与调用面
- `cargo test -p zircon_manager manager_public_surface_excludes_input_protocol_types --offline`
- `cargo test -p zircon_runtime --lib input_protocol_types_live_in_runtime_input_surface --offline`
- `cargo test -p zircon_runtime --lib --offline`
- `cargo check -p zircon_framework --lib --offline`
- `cargo test -p zircon_app runtime_input_protocol_is_owned_by_input_subsystem --offline`
- `cargo test -p zircon_manager manager_public_surface_excludes_vm_plugin_protocol_types --offline`
- `cargo test -p zircon_runtime script_subsystem_is_physically_absorbed_into_runtime_crate --offline --target-dir target/codex-shared-b -- --nocapture`
- `cargo test -p zircon_runtime --lib --locked --target-dir target/codex-shared-b`
- `cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b`
- `cargo check --workspace --locked --target-dir target/codex-shared-b`
- `cargo test -p zircon_manager tests::manager_public_surface_excludes_resource_state_mirror --offline --target-dir target/tdd-red-manager -- --exact`
- `cargo test -p zircon_asset tests::editor::boundary::resource_state_protocol_lives_in_resource_crate --offline --target-dir target/tdd-red-asset -- --exact`
- `cargo test -p zircon_asset resource_server_reports_resource_records_for_project_assets --offline --target-dir target/tdd-red-asset`
- `cargo test -p zircon_editor asset_manager_boundary --offline --target-dir target/tdd-red-editor`
- `cargo test -p zircon_editor resolve_ready_handle_returns_typed_handle_from_resource_server --offline --target-dir target/tdd-red-editor --quiet`
- `cargo test -p zircon_manager tests::manager_public_surface_excludes_resource_status_wrapper --offline --target-dir target/tdd-status-manager -- --exact`
- `cargo test -p zircon_asset tests::editor::boundary::resource_status_protocol_lives_in_resource_crate --offline --target-dir target/tdd-status-asset -- --exact`
- `cargo test -p zircon_asset resource_server_reports_resource_records_for_project_assets --offline --target-dir target/tdd-status-asset`
- `cargo test -p zircon_manager --offline --target-dir target/tdd-status-manager`
- `cargo test -p zircon_asset --offline --target-dir target/tdd-status-asset`
- `cargo test -p zircon_editor tests::host::asset_manager_boundary::editor_asset_workspace_uses_canonical_resource_record --offline --target-dir target/tdd-red-editor -- --exact --quiet`
- `cargo test -p zircon_editor asset_manager_boundary --offline --target-dir target/tdd-red-editor --quiet`
- `cargo check -p zircon_editor --lib --offline --target-dir target/tdd-red-editor --message-format short`
- `cargo fmt -p zircon_manager -p zircon_scene -p zircon_scene_protocol`
- `cargo test -p zircon_scene_protocol --offline --target-dir target/tdd-scene-protocol-protocol`
- `cargo test -p zircon_manager --offline --target-dir target/tdd-scene-protocol-manager`
- `cargo test -p zircon_scene --offline --target-dir target/tdd-scene-protocol-scene`
- `cargo check -p zircon_manager -p zircon_scene --offline --target-dir target/tdd-scene-protocol-check --message-format short`
- `cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b asset_project_api_moves_under_project_module_namespace -- --nocapture`
- `cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b asset_watch_api_moves_under_watch_module_namespace -- --nocapture`
- `cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b legacy_template_compat_api_moves_under_template_namespace -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b runtime_asset_surface_keeps_project_and_watch_under_namespaces -- --nocapture`
- `cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b runtime_ui_surface_keeps_template_and_layout_specialists_under_namespaces -- --nocapture`
- `git grep -n "zircon_asset::\|zircon_scene::\|zircon_ui::\|zircon_runtime/crates/zircon_" -- ':(exclude)zircon_asset' ':(exclude)zircon_scene' ':(exclude)zircon_ui'`
- `cargo test -p zircon_runtime --lib --offline`
- `cargo test -p zircon_editor --lib --offline`
- `cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b -- --nocapture`
- `cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b -- --nocapture`
- `cargo test -p zircon_scene --locked --offline --target-dir target/codex-shared-b -- --nocapture`
- `cargo test -p zircon_graphics --lib --no-run --locked --offline --target-dir target/codex-shared-b`
- `cargo test -p zircon_editor --lib --no-run --locked --offline --target-dir target/codex-shared-b`

结论：

- `zircon_graphics` 红测已修复，根因是 non-VG render path 没有消费 `BuiltMeshDraws.draws`
- `AssetRecordKind` / `PreviewStateRecord` 这组边界已完成收口
- `zircon_manager::service_names` 经二次审计后判定应保留在 façade 层，不再作为迁移候选
- `InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot` 已完成协议层收口：`zircon_manager` 不再拥有这组类型，`zircon_framework::input` 成为唯一 owner
- `zircon_runtime::input` / `zircon_app` 的输入协议导入边界已经切换到 `zircon_framework::input` / `zircon_runtime::input`
- `CapabilitySet` / `HostHandle` / `PluginSlotId` 已完成脚本子系统收口：`zircon_manager` 不再拥有这组 VM 私有类型，`zircon_runtime::script` 成为唯一 owner
- `zircon_runtime::script` 子树已经不再经 `zircon_manager` 暴露脚本 VM 私有协议；独立 `zircon_script` package 删除后，这组 surface 也不再保留历史 package 旁路
- `ResourceStateRecord` 已完成 canonical enum 收口：`zircon_manager` 不再拥有资源状态镜像，`zircon_asset` / `zircon_editor` 已统一使用 `zircon_resource::ResourceState`
- `ResourceStatusRecord` 已完成 canonical record 收口：`zircon_manager` 不再拥有资源状态 DTO，`zircon_asset` / `zircon_editor` 已统一使用 `zircon_resource::ResourceRecord`
- `ResourceChangeKind` / `ResourceChangeRecord` 已完成 canonical event 收口：`zircon_manager` 不再拥有资源变化镜像，`zircon_asset` / `zircon_editor` 已统一使用 `zircon_resource::ResourceEvent`
- generic `AssetManager` 协议线已完成收口：`zircon_manager` 不再拥有 asset manager trait/handle/resolver/service-name，也不再镜像 `AssetChange`
- `AssetPipelineInfo` / `ProjectInfo` 已随 `AssetManager` 协议线回到 `zircon_asset`；这一组不再属于剩余 watchlist
- `WorldHandle` / `LevelSummary` 已完成 scene 协议层收口：`zircon_scene_protocol` 成为唯一 owner，`zircon_manager` 不再 re-export scene handle/summary
- `RenderingBackendInfo` 当前应保留在 `zircon_manager`：它有实际 graphics 实现、模块注册和 bootstrap 消费，当前定位是 compat façade contract
- `zircon_runtime::ui` binding / reflection / template protocols 当前仍停留在 watchlist，因为 runtime UI 自身仍是这组协议的实现和测试 owner；historical fixture conversion 不属于 production public surface
- `zircon_runtime::ui` 的 root-surface 收口已扩展到 compiler/layout/template-runtime/tree specialist surface：workspace 调用点统一经 `zircon_runtime::ui::template::{UiTemplateDocument, UiTemplateLoader, UiCompiledDocument, UiDocumentCompiler, UiStyleResolver, UiTemplateBuildError, UiTemplateError, UiTemplateSurfaceBuilder, UiTemplateTreeBuilder, UiTemplateValidator, UiTemplateInstance, UiTemplateNode}`、`zircon_runtime::ui::layout::{compute_layout_tree, compute_virtual_list_window, solve_axis_constraints}` 与 `zircon_runtime::ui::tree::{UiTemplateNodeMetadata, UiTreeError, UiDirtyFlags, UiLayoutCache, UiHitTestIndex, UiHitTestResult}` 访问，而不是继续依赖 root flat exports
- `zircon_ui` 的 root-surface 收口也已经覆盖 binding / event_ui specialist：workspace 调用点统一经 `zircon_ui::binding::*` 与 `zircon_ui::event_ui::*` 访问，`zircon_runtime::ui` 只保留 `binding` / `event_ui` namespace，不再在吸收层根入口重新拍平这两簇协议 DTO
- `zircon_asset` 根级 raw `zircon_resource` foreign re-export 已进入收口实现：raw `Resource*` surface 应回到 `zircon_resource`，asset crate 根只保留 asset-named 语义 alias；其中 `AssetReference` 这类仍有消费证据的 asset 别名不在删除范围
- `zircon_asset` 的 project/watch public API 都已从“候选”转为“已落地”：`ProjectManager` / `ProjectManifest` / `ProjectPaths` / `AssetMetaDocument` / `PreviewState` 现在只经 `zircon_asset::project::*` 暴露，`AssetChange` / `AssetChangeKind` / `AssetWatchEvent` / `AssetWatcher` 现在只经 `zircon_asset::watch::*` 暴露
- `zircon_runtime::asset` / `zircon_runtime::ui` 也已同步保持结构性 namespace surface，不再在吸收层重新扁平化 `project/watch/template/layout/surface/tree` 子域
- `cargo test -p zircon_editor --lib --no-run --locked --offline --target-dir target/codex-shared-b` 本轮没有命中新的 namespace 编译错误，但会在本机现有 Skia 链接环境上失败于 `skparagraph.lib` 缺失；因此这次对 editor-side root-surface 切换的编译证据以 `cargo check --workspace --locked --offline --target-dir target/codex-shared-b` 和该 no-run 命令到 link 阶段为准
- 当前仓库不存在 `docs/source/` 目录；这轮文档清扫已转为更新 `docs/` 下的 live 总览文档，避免继续沿用过期的 manager-owned asset/scene 表述
- 本轮没有重新跑 full workspace `--locked` 联测；原因是工作树里的 `Cargo.lock` 已脏，这不在本批边界收口范围内
- `cargo check -p zircon_editor --lib --offline --target-dir target/tdd-red-editor --message-format short` 目前仍被 crate 内现有无关红阻塞；失败点落在 [`inspector_semantics.rs`](../../zircon_editor/src/core/editing/ui_asset/inspector_semantics.rs) 里 `UiAssetStructuredLayoutSemanticFields` 缺少 `box_gap` 字段初始化，不是这轮 `ResourceEvent` 迁移直接引入的断点

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
  - zircon_scene/src/render_extract.rs
  - zircon_scene/src/module/default_level_manager.rs
  - zircon_scene/src/module/level_manager_facade.rs
  - zircon_scene/src/module/level_manager_lifecycle.rs
  - zircon_scene/src/module/level_manager_project_io.rs
  - zircon_scene/src/tests/boundary.rs
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_asset/src/pipeline/manager/records/asset_status_record.rs
  - zircon_asset/src/pipeline/manager/records/status_record.rs
  - zircon_asset/src/pipeline/manager/records/metadata_import_state.rs
  - zircon_asset/src/pipeline/manager/facades/resource_manager_facade.rs
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
  - zircon_input_protocol/src/lib.rs
  - zircon_input_protocol/src/input_button.rs
  - zircon_input_protocol/src/input_event.rs
  - zircon_input_protocol/src/input_event_record.rs
  - zircon_input_protocol/src/input_snapshot.rs
  - zircon_input/src/module/descriptor.rs
  - zircon_input/src/lib.rs
  - zircon_input/src/runtime/default_input_manager.rs
  - zircon_input/src/runtime/input_state.rs
  - zircon_input/src/tests/boundary.rs
  - zircon_input/src/tests/input_manager.rs
  - zircon_script/Cargo.toml
  - zircon_script/src/lib.rs
  - zircon_script/src/vm/mod.rs
  - zircon_script/src/vm/capability_set.rs
  - zircon_script/src/vm/handles.rs
  - zircon_script/src/vm/host/host_registry.rs
  - zircon_script/src/vm/plugin/vm_plugin_manifest.rs
  - zircon_script/src/vm/runtime/hot_reload_coordinator.rs
  - zircon_script/src/vm/tests.rs
  - zircon_editor/src/editing/asset_workspace.rs
  - zircon_editor/src/editing/state/editor_state_asset_workspace.rs
  - zircon_editor/src/editor_event/runtime/execution/common.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/app/backend_refresh.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/host/slint_host/ui/asset_surface_presentation.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_item_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_reference_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_selection_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_workspace_snapshot.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/host/asset_manager_boundary.rs
  - zircon_editor/src/tests/host/resource_access.rs
  - zircon_editor/src/tests/host/slint_asset_refresh.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_entry/src/entry/runtime_entry_app/application_handler.rs
  - zircon_entry/src/entry/tests.rs
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
  - zircon_scene/src/render_extract.rs
  - zircon_scene/src/module/default_level_manager.rs
  - zircon_scene/src/module/level_manager_facade.rs
  - zircon_scene/src/module/level_manager_lifecycle.rs
  - zircon_scene/src/module/level_manager_project_io.rs
  - zircon_scene/src/tests/boundary.rs
  - zircon_input_protocol/Cargo.toml
  - zircon_input_protocol/src/lib.rs
  - zircon_input_protocol/src/input_button.rs
  - zircon_input_protocol/src/input_event.rs
  - zircon_input_protocol/src/input_event_record.rs
  - zircon_input_protocol/src/input_snapshot.rs
  - zircon_input/Cargo.toml
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager/records/mod.rs
  - zircon_asset/src/pipeline/manager/records/asset_status_record.rs
  - zircon_asset/src/pipeline/manager/records/status_record.rs
  - zircon_asset/src/pipeline/manager/records/metadata_import_state.rs
  - zircon_asset/src/pipeline/manager/facades/resource_manager_facade.rs
  - zircon_asset/src/pipeline/manager/registration/service_names.rs
  - zircon_asset/src/pipeline/manager/asset_manager/mod.rs
  - zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs
  - zircon_input/src/lib.rs
  - zircon_input/src/module/descriptor.rs
  - zircon_input/src/runtime/default_input_manager.rs
  - zircon_input/src/runtime/input_state.rs
  - zircon_script/Cargo.toml
  - zircon_script/src/lib.rs
  - zircon_script/src/vm/mod.rs
  - zircon_script/src/vm/capability_set.rs
  - zircon_script/src/vm/handles.rs
  - zircon_script/src/vm/host/host_registry.rs
  - zircon_script/src/vm/plugin/vm_plugin_manifest.rs
  - zircon_script/src/vm/runtime/hot_reload_coordinator.rs
  - zircon_script/src/vm/tests.rs
  - zircon_entry/src/entry/runtime_entry_app/application_handler.rs
  - zircon_editor/src/editing/asset_workspace.rs
  - zircon_editor/src/editing/state/editor_state_asset_workspace.rs
  - zircon_editor/src/editor_event/runtime/execution/common.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/app/backend_refresh.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/host/slint_host/ui/asset_surface_presentation.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_item_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_reference_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_selection_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_workspace_snapshot.rs
  - zircon_resource/src/record/resource_event.rs
  - zircon_resource/src/record/resource_event_kind.rs
plan_sources:
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
  - `Select-String` 扫描 `zircon_manager` / `zircon_input` / `zircon_entry` / `zircon_editor` 对 `InputButton` / `InputEvent` / `InputSnapshot` / `InputEventRecord` 的定义与调用面
  - cargo test -p zircon_manager manager_public_surface_excludes_input_protocol_types --offline
  - cargo test -p zircon_input input_protocol_types_live_in_input_subsystem --offline
  - cargo test -p zircon_input --offline
  - cargo test -p zircon_input_protocol --offline
  - cargo test -p zircon_entry runtime_input_protocol_is_owned_by_input_subsystem --offline
  - cargo test -p zircon_manager manager_public_surface_excludes_vm_plugin_protocol_types --offline
  - cargo test -p zircon_script vm_plugin_protocol_types_live_in_script_subsystem --offline
  - cargo test -p zircon_script --offline
  - cargo check -p zircon_manager -p zircon_script --offline
  - cargo check -p zircon_script --offline
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
   只服务 editor catalog、preview、authoring、interaction 的记录和状态，必须归 editor 或 asset/editor-owned 协议层，不应挂在 generic manager façade 上。

3. **Facade minimality rule**
   façade crate 可以定义稳定入口 trait/handle，但不应顺手拥有实现域里的展示语义、preview 状态、authoring 细节和重复 type taxonomy。

4. **Dependency direction rule**
   底层 crate 不能反向依赖 `zircon_editor`；渲染 crate 不能再承载 editor interaction；runtime entry 不能复用 editor-only controller。

5. **Root-surface rule**
   crate 根和 `mod.rs` 只应导出结构入口；如果某个 public surface 需要解释 editor preview、asset catalog 或 viewport 操作语义，那大概率已经越过 crate 责任边界。

## Audit Results

### Confirmed Passes

- `zircon_graphics` 不再拥有 `ViewportController`、`ViewportState`、`ViewportInput`、`ViewportFeedback`、`GizmoAxis`
- `zircon_editor` 已接管 viewport interaction 类型和 scene gizmo/handle 交互
- `zircon_entry` runtime 已切到 crate-private camera controller，不再复用 editor controller
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

按 “editor-only semantics rule”，preview 状态不属于 generic manager façade。它描述的是：

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

从“名字拥有权”看，这些常量更像实现域信息，而不是 manager façade 自己的领域模型。

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
- [zircon_asset/src/pipeline/manager/registration/service_names.rs](../../zircon_asset/src/pipeline/manager/registration/service_names.rs)、[zircon_scene/src/module/service_names.rs](../../zircon_scene/src/module/service_names.rs)、[zircon_graphics/src/host/module_host/module_registration/service_names.rs](../../zircon_graphics/src/host/module_host/module_registration/service_names.rs) 当前都不是自定义一份新名字，而是主动 alias `zircon_manager::*_MANAGER_NAME`
- 上层调用方如 [zircon_entry/src/entry/tests.rs](../../zircon_entry/src/entry/tests.rs) 直接通过 `zircon_manager::resolve_*` / `ManagerResolver` 消费这组 façade 入口，而不是绕到实现 crate 私有名字上

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

原先这些类型挂在 `zircon_manager/src/records/input.rs`，同时被 `zircon_input` 的运行态实现和 `zircon_entry` 的 runtime 输入桥直接消费。它们满足“子系统协议类型错挂在 façade 层”的全部证据门槛：

1. 它们不是 generic manager façade 自己的抽象，而是纯输入语义
2. `zircon_input` 已经是唯一真实拥有输入模块生命周期和运行态状态机的实现 crate
3. `zircon_manager` 一边定义 `InputManager` trait，一边顺手拥有整套输入协议模型，导致实现 crate 反向依赖 façade 类型

这批不能直接平移到 `zircon_input`，因为当前依赖方向会形成环：

- `zircon_input` 依赖 `zircon_manager::InputManager`
- 如果 `zircon_manager` 再反向依赖 `zircon_input` 来拿输入协议类型，就会出现 cycle

因此最终方案是新增独立协议层 [zircon_input_protocol/src/lib.rs](../../zircon_input_protocol/src/lib.rs)：

- [zircon_input_protocol/src/input_button.rs](../../zircon_input_protocol/src/input_button.rs)、[zircon_input_protocol/src/input_event.rs](../../zircon_input_protocol/src/input_event.rs)、[zircon_input_protocol/src/input_event_record.rs](../../zircon_input_protocol/src/input_event_record.rs)、[zircon_input_protocol/src/input_snapshot.rs](../../zircon_input_protocol/src/input_snapshot.rs) 成为这组类型的唯一 owner
- [zircon_manager/src/traits.rs](../../zircon_manager/src/traits.rs) 只保留 `InputManager` façade trait，本身改为直接依赖 `zircon_input_protocol`
- `zircon_manager` 根和 `records/mod.rs` 不再 re-export 输入协议类型，旧的 `src/records/input.rs` 已删除
- [zircon_input/src/lib.rs](../../zircon_input/src/lib.rs) 继续作为输入子系统的公共入口 re-export 这组协议；[zircon_input/src/runtime/default_input_manager.rs](../../zircon_input/src/runtime/default_input_manager.rs) 与 [zircon_input/src/runtime/input_state.rs](../../zircon_input/src/runtime/input_state.rs) 改为消费 crate-local re-export
- [zircon_entry/src/entry/runtime_entry_app/application_handler.rs](../../zircon_entry/src/entry/runtime_entry_app/application_handler.rs) 改成从 `zircon_input` 导入 `InputButton` / `InputEvent`，runtime 不再从 `zircon_manager` 获取输入协议类型

这样 `service_names` 仍保留在 façade contract 层，而输入协议则下沉为真正的共享协议层，两者边界不再混淆。

### Implemented Batch: Script VM Private Protocol And Handles

这轮继续往下扫后，新一组证据充分的错包是原先挂在 `zircon_manager` 的脚本 VM 私有协议/handle：

- `CapabilitySet`
- `HostHandle`
- `PluginSlotId`

它们的问题比上一轮输入协议更直接：

1. 这组类型完全不经过任何 `ManagerResolver`、manager trait 或 façade 返回记录
2. 当前真实消费面只在 `zircon_script`：
   - [zircon_script/src/vm/plugin/vm_plugin_manifest.rs](../../zircon_script/src/vm/plugin/vm_plugin_manifest.rs) 的 `VmPluginManifest.capabilities`
   - [zircon_script/src/vm/host/host_registry.rs](../../zircon_script/src/vm/host/host_registry.rs) 的 host capability handle 注册
   - [zircon_script/src/vm/runtime/hot_reload_coordinator.rs](../../zircon_script/src/vm/runtime/hot_reload_coordinator.rs) 的 plugin slot 生命周期
3. `zircon_manager` 只是顺手持有了脚本 VM 的私有协议和句柄，并没有任何 façade 入口真正需要它们

因此这批的 canonical owner 很明确，就是 `zircon_script` 本身，而不是 `zircon_manager`。

实现结果：

- 新增 [zircon_script/src/vm/capability_set.rs](../../zircon_script/src/vm/capability_set.rs) 和 [zircon_script/src/vm/handles.rs](../../zircon_script/src/vm/handles.rs)
- [zircon_script/src/vm/mod.rs](../../zircon_script/src/vm/mod.rs) 与 [zircon_script/src/lib.rs](../../zircon_script/src/lib.rs) 改为由脚本子系统根级 re-export `CapabilitySet` / `HostHandle` / `PluginSlotId`
- [zircon_manager/src/lib.rs](../../zircon_manager/src/lib.rs)、[zircon_manager/src/records/mod.rs](../../zircon_manager/src/records/mod.rs)、[zircon_manager/src/handles.rs](../../zircon_manager/src/handles.rs) 已删除这组 public surface；旧的 `src/records/capability_set.rs` 已删除
- `zircon_script` 内部实现不再从 `zircon_manager` 反向取脚本私有类型，`Cargo.toml` 也已去掉对 `zircon_manager` 的依赖

这批收口以后，脚本 VM 的 manifest/state/hot-reload 协议终于回到了脚本子系统自己名下；`zircon_manager` 继续只保留真正的 manager façade surface。

### Implemented Batch: AssetManager Protocol Boundary

继续往下扫后，`AssetPipelineInfo` / `ProjectInfo` 并不是“仍应保留在 façade 层的剩余 record”；真正的强候选其实是整条 generic `AssetManager` protocol boundary：

- `AssetManager` trait 只有 `zircon_asset` 一处真实实现
- `AssetManagerHandle`、`resolve_asset_manager`、`ASSET_MANAGER_NAME` 都只是 asset 子系统公共入口，不属于 generic manager façade 自己的领域模型
- `AssetChangeRecord` / `AssetChangeKind` 只是 `zircon_asset::AssetChange` 的镜像投影，增加了无意义 DTO 层

因此这批最终不是只搬 DTO，而是整条协议线一起迁回 `zircon_asset`：

- 新增 [zircon_asset/src/pipeline/manager/asset_manager/mod.rs](../../zircon_asset/src/pipeline/manager/asset_manager/mod.rs)，并把 trait/handle/resolver 继续下沉到 [zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs](../../zircon_asset/src/pipeline/manager/asset_manager/asset_manager.rs)、[zircon_asset/src/pipeline/manager/asset_manager/asset_manager_handle.rs](../../zircon_asset/src/pipeline/manager/asset_manager/asset_manager_handle.rs) 与 [zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs](../../zircon_asset/src/pipeline/manager/asset_manager/resolve_asset_manager.rs)
- [zircon_asset/src/pipeline/manager.rs](../../zircon_asset/src/pipeline/manager.rs) 与 [zircon_asset/src/lib.rs](../../zircon_asset/src/lib.rs) 现在直接拥有 `AssetManager`、`AssetManagerHandle`、`resolve_asset_manager`、`AssetPipelineInfo`、`ProjectInfo`、`ASSET_MANAGER_NAME`
- [zircon_asset/src/pipeline/manager/facades/asset_manager_facade.rs](../../zircon_asset/src/pipeline/manager/facades/asset_manager_facade.rs) 的 `subscribe_asset_changes()` 已统一返回 `ChannelReceiver<zircon_asset::AssetChange>`
- `zircon_manager` 已删除 `AssetManager` trait、asset/project records、handle、resolver 和 service-name surface；旧的 [zircon_manager/src/records/asset.rs](../../zircon_manager/src/records/asset.rs) 与 [zircon_manager/src/records/project.rs](../../zircon_manager/src/records/project.rs) 已删除

这说明 `AssetPipelineInfo` / `ProjectInfo` 已经随 canonical owner 一起回到 asset 子系统，不再属于剩余 watchlist。

### Implemented Batch: Scene Level Protocol Types

本轮继续重扫 `zircon_manager` 现存 façade surface 后，新的高证据候选不是 `RenderingBackendInfo`，而是 scene level 协议自己的 handle/summary：

- `WorldHandle`
- `LevelSummary`

它们的问题和上一轮 `InputButton` / `InputEvent` 很像：

1. 这组类型只服务 `LevelManager` 协议与 `zircon_scene` 运行时实现
2. 当前真实生产/消费面只在 [zircon_scene/src/module/default_level_manager.rs](../../zircon_scene/src/module/default_level_manager.rs)、[zircon_scene/src/module/level_manager_facade.rs](../../zircon_scene/src/module/level_manager_facade.rs)、[zircon_scene/src/level_system.rs](../../zircon_scene/src/level_system.rs) 与 [zircon_scene/src/render_extract.rs](../../zircon_scene/src/render_extract.rs)
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

### Not Promoted: `zircon_ui` Binding / Reflection / Template Protocols

这轮我也重新扫了 `zircon_ui::{UiBindingValue, UiBindingCall, UiEventKind, UiEventPath, UiControlRequest, UiControlResponse, UiReflectionSnapshot, UiNodeDescriptor, UiPropertyDescriptor, UiTemplateLoader, UiDocumentCompiler}` 这组高频 surface。

它们在 editor/editor_ui 里消费面很大，但当前证据还不足以把它们升级成“明显错包”，原因是：

- [zircon_ui/src/binding/mod.rs](../../zircon_ui/src/binding/mod.rs) 与 [zircon_ui/src/event_ui/mod.rs](../../zircon_ui/src/event_ui/mod.rs) 自己就拥有 binding parser、event manager、reflection store 和 invocation 协议
- [zircon_ui/src/template/mod.rs](../../zircon_ui/src/template/mod.rs) 自己拥有 template loader、validator、asset loader、document compiler 和 surface builder
- `zircon_ui` crate 内部测试直接覆盖这组能力：[zircon_ui/src/tests/binding.rs](../../zircon_ui/src/tests/binding.rs)、[zircon_ui/src/tests/event_manager.rs](../../zircon_ui/src/tests/event_manager.rs)、[zircon_ui/src/tests/template.rs](../../zircon_ui/src/tests/template.rs)、[zircon_ui/src/tests/asset.rs](../../zircon_ui/src/tests/asset.rs)

也就是说，虽然 editor 是这组协议的重度调用方，但权威实现和验证目前仍然在 `zircon_ui` 子系统内部。这更像“UI runtime/authoring 共用协议”而不是“editor-only 语义误挂在 runtime crate”。因此这轮先不升级它，继续留在 watchlist。

这轮再把 watchlist 收窄到 legacy template compat 链后，结论仍然是“先不搬”：

- [zircon_ui/src/template/document.rs](../../zircon_ui/src/template/document.rs) 里的 `UiTemplateDocument` 不是孤立 editor DTO；[zircon_ui/src/template/validate.rs](../../zircon_ui/src/template/validate.rs)、[zircon_ui/src/template/instance.rs](../../zircon_ui/src/template/instance.rs) 和 [zircon_ui/src/template/loader.rs](../../zircon_ui/src/template/loader.rs) 共同组成 `zircon_ui` 自己的 template runtime 真源
- [zircon_editor_ui/src/template/registry.rs](../../zircon_editor_ui/src/template/registry.rs) 当前虽然直接托管 `UiTemplateDocument`，但实例化仍然回到 `UiTemplateInstance::from_document(...)` 这条 shared runtime 合同，而不是 editor 私有实例树
- [zircon_editor/src/host/template_runtime/runtime/runtime_host.rs](../../zircon_editor/src/host/template_runtime/runtime/runtime_host.rs) 对 `UiTemplateLoader::load_toml_str(...)` 的生产态使用，本质上也是 editor runtime 对 shared legacy template 输入格式的 fallback 兼容，而不是 editor 自己拥有 parser
- [zircon_ui/src/template/asset/legacy.rs](../../zircon_ui/src/template/asset/legacy.rs) 的 `UiLegacyTemplateAdapter` 确实更接近 migration/authoring compat，但它桥接的是 `UiTemplateDocument -> UiAssetDocument` 这两种都由 `zircon_ui` 自己拥有的 shared model；现在把 adapter 迁到 editor 侧，会让 legacy->canonical 的共享合同脱离 model owner

因此 `UiTemplateDocument` / `UiTemplateLoader` / `UiLegacyTemplateAdapter` 当前最多只能算 `zircon_ui::template` 下的 legacy compat watchlist，还没有达到 crate 迁移门槛。

### Keep In `zircon_manager`

这轮严格审计后，下面几组我判断仍应留在 façade 层：

- `RenderingBackendInfo`
  原因：它是 `RenderingManager` façade 的能力摘要，不携带 editor-specific 或 preview-specific 语义；而且这轮已经补齐了“它不是空心壳”的证据链：
  - [zircon_graphics/src/host/module_host/rendering_manager/manager_backend_info.rs](../../zircon_graphics/src/host/module_host/rendering_manager/manager_backend_info.rs) 为 `WgpuRenderingManager` 提供了实际 `backend_info()` 实现
  - [zircon_graphics/src/host/module_host/module_registration/module_descriptor.rs](../../zircon_graphics/src/host/module_host/module_registration/module_descriptor.rs) 把它注册成 `GraphicsModule.Manager.RenderingManager`
  - [zircon_entry/src/entry/tests/profile_bootstrap.rs](../../zircon_entry/src/entry/tests/profile_bootstrap.rs) 仍通过 `resolve_rendering_manager(...).backend_info()` 验证 bootstrap 合同
  - [docs/assets-and-rendering/srp-rhi-render-server-architecture.md](../assets-and-rendering/srp-rhi-render-server-architecture.md) 与 [docs/assets-and-rendering/index.md](../assets-and-rendering/index.md) 也已经把它放在“`RenderingManager` 向纯兼容桥继续收束”的语境里

  现阶段它更像“仍被消费的 façade compat contract”，不是应该立即下沉到 graphics 内部的孤立 DTO。

### New Strong Candidate: `zircon_asset` Root Foreign Re-Exports Of `zircon_resource`

这轮避开 `zircon_editor` / `zircon_graphics` 热区继续重扫时，新的高证据候选不是再搬一组实现类型，而是清理 `zircon_asset` 根级错误代持的 foreign surface：

- [zircon_asset/src/lib.rs](../../zircon_asset/src/lib.rs) 当前直接 `pub use zircon_resource::{...}`，把 `ResourceLocator`、`ResourceKind`、`ResourceRecord`、`ResourceState`、`ResourceManager`、marker/type-descriptor/runtime-info 等一整批 raw resource 基础类型重新挂到了 asset crate 根下
- 但这批 raw surface 的 canonical owner 明确还是 `zircon_resource`；`zircon_asset` 自己真正拥有的是 `AssetManager`、project/import/watch/editor asset API，以及基于 resource 基础层再包装出来的 asset 语义
- 这轮对 workspace 和 `docs/` 的源码扫描里，没有发现任何 `zircon_asset::ResourceLocator`、`zircon_asset::ResourceKind`、`zircon_asset::ResourceRecord`、`zircon_asset::ResourceManager` 等 raw resource 外部消费面；后续补扫确认 `zircon_asset::AssetReference` 仍被 scene/editor/graphics 测试与 helper 代码当作 asset 语义别名消费，因此它应保留为 asset-named alias，而不是与 raw resource surface 一起删除
- 相反，当前实际代码已经直接使用 canonical owner：
  - [zircon_asset/src/assets/ui.rs](../../zircon_asset/src/assets/ui.rs) 直接从 `zircon_resource` 取 `AssetReference` / `ResourceLocator`
  - [zircon_scene/src/serializer.rs](../../zircon_scene/src/serializer.rs) 与 scene module 子树直接从 `zircon_resource` 取 `ResourceLocator`
  - `zircon_editor` / `zircon_entry` 当前也没有通过 `zircon_asset` 根去拿任何 raw resource 类型

这符合一条新的高证据模式：

1. foreign subsystem 的 raw contract 已经有清晰 canonical owner
2. 代持 crate 根只是在重复暴露同一组类型
3. workspace 内外部消费已经全部绕过这层代持 surface

因此这条线更像“crate root 边界泄漏”，而不是必须保留的兼容入口。下一批最合理的收口不是搬模块，而是：

- 从 `zircon_asset` 根去掉对 `zircon_resource` raw surface 的大面积 re-export
- 保留 asset-owned 的别名/语义入口，如 `AssetId` / `AssetKind` / `AssetUri` / `AssetMetadata`，以及仍有消费证据的 `AssetReference` / `AssetUuid` / `AssetRegistry`
- 让 raw resource contract 只从 `zircon_resource` 获取，避免 `zircon_asset` 根继续伪装成 resource foundation owner

## Implemented Batch

这一批已经按审计顺序完成收口，不再停留在候选状态：

1. `zircon_manager::AssetRecordKind` 已删除，`AssetStatusRecord.kind` / `ResourceStatusRecord.kind` 统一改成 `zircon_resource::ResourceKind`
2. `zircon_manager::PreviewStateRecord` 已删除，editor-facing catalog/details/reference records 直接使用 `zircon_asset::PreviewState`
3. `zircon_asset` 不再做 façade taxonomy 投影；`editor/records.rs` 与 `pipeline/manager/records/mod.rs` 下的 records 子树都直接传递 canonical kind / preview state
4. `zircon_editor` asset workspace、resource access、event filter parser、Slint asset surface、snapshot structs 和对应测试全部切到 `ResourceKind`
5. 新增 `zircon_input_protocol` 作为 `InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot` 的独立协议 owner
6. `zircon_manager` 删除输入协议 re-export 与 `src/records/input.rs`，只保留 `InputManager` façade trait
7. `zircon_input` 根级 re-export 输入协议，运行态实现不再从 `zircon_manager` 反向取输入类型
8. `zircon_entry` runtime 输入桥改为从 `zircon_input` 导入输入协议类型
9. `CapabilitySet` / `HostHandle` / `PluginSlotId` 已从 `zircon_manager` 迁回 `zircon_script`
10. `zircon_script` 不再依赖 `zircon_manager`；脚本 VM 私有协议和热重载句柄改由脚本子系统自己拥有
11. `zircon_manager::ResourceStateRecord` 已删除，`ResourceStatusRecord.state` 统一改成 `zircon_resource::ResourceState`
12. `zircon_asset` pipeline facade、`zircon_editor` resource access / asset snapshots / asset surface 和对应测试已切到 canonical `ResourceState`
13. `zircon_manager::ResourceStatusRecord` 已删除，`ResourceManager::{resource_status,list_resources}` 统一改成 `zircon_resource::ResourceRecord`
14. `zircon_asset` 不再把 `AssetMetadata`/registry record 投影成字符串化 façade DTO；resource facade 直接返回 cloned `ResourceRecord`
15. `zircon_editor` asset workspace / resource access / host tests 已切到 typed `ResourceRecord`，不再把 `id` / `locator` / `artifact_locator` / `diagnostics` 降级成字符串 record
16. `zircon_manager::ResourceChangeKind` / `ResourceChangeRecord` 已删除，`ResourceManager::subscribe_resource_changes` 统一改成 `ChannelReceiver<zircon_resource::ResourceEvent>`
17. `zircon_asset` 不再把 `ResourceEvent` 桥接成字符串化 façade DTO；resource facade 直接转发资源子系统原生事件流
18. `zircon_editor` host refresh planner、Slint asset refresh 测试和边界测试已切到 typed `ResourceEvent` / `ResourceEventKind`
19. generic `AssetManager` trait、`AssetManagerHandle`、`resolve_asset_manager`、`ASSET_MANAGER_NAME` 已从 `zircon_manager` 迁回 `zircon_asset`
20. `AssetPipelineInfo` / `ProjectInfo` 已随 `AssetManager` 协议线回到 `zircon_asset`，不再作为 façade 残留 record 保留在 `zircon_manager`
21. `AssetManager::subscribe_asset_changes` 已统一改成 `ChannelReceiver<zircon_asset::AssetChange>`，`AssetChangeRecord` / `AssetChangeKind` 镜像已删除
22. `zircon_editor`、`zircon_entry` 与 `zircon_graphics` 相关消费者已切到 asset-owned handle / resolver / change stream
23. 新增 `zircon_scene_protocol` 作为 `WorldHandle` / `LevelSummary` 的独立协议 owner
24. `zircon_manager` 删除 `WorldHandle` / `LevelSummary` re-export，`src/handles.rs` 与 `src/records/level.rs` 已删除
25. `zircon_scene` 根级 re-export scene protocol 类型，`level_system` / `render_extract` / `DefaultLevelManager` / `LevelManagerFacade` 不再从 `zircon_manager` 反向取 scene 协议

这样 `zircon_manager` 继续保留 façade record/trait 本身，但不再重复拥有 asset/editor 展示语义，也不再错误持有输入子系统、资源子系统或 scene 子系统的专属协议模型。

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

1. `RenderingBackendInfo` 目前已有充分 keep 证据；后续只需在 `RenderingManager` façade 真正删除或出现第二实现时再重开审计。
2. 继续细分 `zircon_ui` watchlist，但当前只保留 legacy template compat 链：`UiTemplateDocument` / `UiTemplateLoader` / `UiLegacyTemplateAdapter`；若要升级，先要证明它们已经脱离 `zircon_ui` 自己的 validator/instance/surface runtime。
3. 持续扫 live `docs/` 中对旧 asset/scene owner 的残留描述；当前仓库没有 `docs/source/` 目录，因此文档清扫目标以总览型文档为准。
4. 下一批最强候选已经转到非热区的 crate-root boundary cleanup：`zircon_asset` 根级对 `zircon_resource` raw surface 的 foreign re-export 收口。
5. 在不碰 `zircon_editor` / `zircon_graphics` 热区的前提下，继续找新的“证据充分”候选；除这条 root-surface 泄漏外，本轮还没有发现新的高证据错包。

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
- `Select-String` 扫描 `zircon_manager` / `zircon_asset` / `zircon_scene` / `zircon_graphics` / `zircon_entry` 对 `*_MANAGER_NAME` 和 `resolve_*` 的调用面
- `Select-String` 扫描 `zircon_manager` / `zircon_input` / `zircon_entry` / `zircon_editor` 对 `InputButton` / `InputEvent` / `InputSnapshot` / `InputEventRecord` 的定义与调用面
- `cargo test -p zircon_manager manager_public_surface_excludes_input_protocol_types --offline`
- `cargo test -p zircon_input input_protocol_types_live_in_input_subsystem --offline`
- `cargo test -p zircon_input --offline`
- `cargo test -p zircon_input_protocol --offline`
- `cargo test -p zircon_entry runtime_input_protocol_is_owned_by_input_subsystem --offline`
- `cargo test -p zircon_manager manager_public_surface_excludes_vm_plugin_protocol_types --offline`
- `cargo test -p zircon_script vm_plugin_protocol_types_live_in_script_subsystem --offline`
- `cargo test -p zircon_script --offline`
- `cargo check -p zircon_manager -p zircon_script --offline`
- `cargo check -p zircon_script --offline`
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

结论：

- `zircon_graphics` 红测已修复，根因是 non-VG render path 没有消费 `BuiltMeshDraws.draws`
- `AssetRecordKind` / `PreviewStateRecord` 这组边界已完成收口
- `zircon_manager::service_names` 经二次审计后判定应保留在 façade 层，不再作为迁移候选
- `InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot` 已完成协议层收口：`zircon_manager` 不再拥有这组类型，`zircon_input_protocol` 成为唯一 owner
- `zircon_input` / `zircon_entry` 的输入协议导入边界已经切换到 `zircon_input_protocol` / `zircon_input`
- `CapabilitySet` / `HostHandle` / `PluginSlotId` 已完成脚本子系统收口：`zircon_manager` 不再拥有这组 VM 私有类型，`zircon_script` 成为唯一 owner
- `zircon_script` 已去掉对 `zircon_manager` 的依赖，说明这组迁移还顺带消除了原先不必要的 façade 反向依赖
- `ResourceStateRecord` 已完成 canonical enum 收口：`zircon_manager` 不再拥有资源状态镜像，`zircon_asset` / `zircon_editor` 已统一使用 `zircon_resource::ResourceState`
- `ResourceStatusRecord` 已完成 canonical record 收口：`zircon_manager` 不再拥有资源状态 DTO，`zircon_asset` / `zircon_editor` 已统一使用 `zircon_resource::ResourceRecord`
- `ResourceChangeKind` / `ResourceChangeRecord` 已完成 canonical event 收口：`zircon_manager` 不再拥有资源变化镜像，`zircon_asset` / `zircon_editor` 已统一使用 `zircon_resource::ResourceEvent`
- generic `AssetManager` 协议线已完成收口：`zircon_manager` 不再拥有 asset manager trait/handle/resolver/service-name，也不再镜像 `AssetChange`
- `AssetPipelineInfo` / `ProjectInfo` 已随 `AssetManager` 协议线回到 `zircon_asset`；这一组不再属于剩余 watchlist
- `WorldHandle` / `LevelSummary` 已完成 scene 协议层收口：`zircon_scene_protocol` 成为唯一 owner，`zircon_manager` 不再 re-export scene handle/summary
- `RenderingBackendInfo` 当前应保留在 `zircon_manager`：它有实际 graphics 实现、模块注册和 bootstrap 消费，当前定位是 compat façade contract
- `zircon_ui` binding / reflection / template protocols 当前仍停留在 watchlist，因为 `zircon_ui` 自身仍是这组协议的实现和测试 owner；其中 legacy template compat 链也还没达到 crate 迁移门槛
- `zircon_asset` 根级 raw `zircon_resource` foreign re-export 已进入收口实现：raw `Resource*` surface 应回到 `zircon_resource`，asset crate 根只保留 asset-named 语义 alias；其中 `AssetReference` 这类仍有消费证据的 asset 别名不在删除范围
- 当前仓库不存在 `docs/source/` 目录；这轮文档清扫已转为更新 `docs/` 下的 live 总览文档，避免继续沿用过期的 manager-owned asset/scene 表述
- 本轮没有重新跑 full workspace `--locked` 联测；原因是工作树里的 `Cargo.lock` 已脏，这不在本批边界收口范围内
- `cargo check -p zircon_editor --lib --offline --target-dir target/tdd-red-editor --message-format short` 目前仍被 crate 内现有无关红阻塞；失败点落在 [`inspector_semantics.rs`](../../zircon_editor/src/editing/ui_asset/inspector_semantics.rs) 里 `UiAssetStructuredLayoutSemanticFields` 缺少 `box_gap` 字段初始化，不是这轮 `ResourceEvent` 迁移直接引入的断点

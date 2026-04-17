---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_manager/src/records/asset.rs
  - zircon_manager/src/records/resource.rs
  - zircon_manager/src/records/mod.rs
  - zircon_manager/src/lib.rs
  - zircon_manager/src/handles.rs
  - zircon_manager/src/traits.rs
  - zircon_manager/src/service_names.rs
  - zircon_manager/src/tests.rs
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager/records.rs
  - zircon_asset/src/tests/editor/boundary.rs
  - zircon_asset/src/tests/editor/manager.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_resource/src/marker.rs
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
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_entry/src/entry/runtime_entry_app/application_handler.rs
  - zircon_entry/src/entry/tests.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - Cargo.toml
  - zircon_manager/src/records/asset.rs
  - zircon_manager/src/records/resource.rs
  - zircon_manager/Cargo.toml
  - zircon_manager/src/records/mod.rs
  - zircon_manager/src/lib.rs
  - zircon_manager/src/handles.rs
  - zircon_manager/src/traits.rs
  - zircon_input_protocol/Cargo.toml
  - zircon_input_protocol/src/lib.rs
  - zircon_input_protocol/src/input_button.rs
  - zircon_input_protocol/src/input_event.rs
  - zircon_input_protocol/src/input_event_record.rs
  - zircon_input_protocol/src/input_snapshot.rs
  - zircon_input/Cargo.toml
  - zircon_asset/src/editor/records.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager/records.rs
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
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/host/slint_host/ui/asset_surface_presentation.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_item_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_reference_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_selection_snapshot.rs
  - zircon_editor/src/workbench/snapshot/asset/asset_workspace_snapshot.rs
plan_sources:
  - user: 2026-04-18 直接转去处理 zircon_graphics 红测，再开一轮更严格的包边界审计标准
  - user: 2026-04-18 按审计文档里的顺序处理 AssetRecordKind 和 PreviewStateRecord 这组边界收口
  - user: 2026-04-18 继续严格包边界审计，就需要重新找新的“证据充分”候选，而不是再回头动这两组
  - user: 2026-04-18 按审计顺序继续落代码，最该先处理的就是这组输入协议边界
  - user: 2026-04-18 继续整理代码迁移，寻找不合理
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
  - cargo test -p zircon_manager --offline
  - cargo test -p zircon_script --offline
  - cargo check -p zircon_manager -p zircon_script --offline
  - cargo check -p zircon_script --offline
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
- `zircon_asset/src/pipeline/manager/records.rs` 每次投影 `AssetMetadata` 时都要把 `AssetKind` 手动映射成 `AssetRecordKind`
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
- [zircon_asset/src/pipeline/manager/service_names.rs](../../zircon_asset/src/pipeline/manager/service_names.rs)、[zircon_scene/src/module/service_names.rs](../../zircon_scene/src/module/service_names.rs)、[zircon_graphics/src/host/module_host/service_names.rs](../../zircon_graphics/src/host/module_host/service_names.rs) 当前都不是自定义一份新名字，而是主动 alias `zircon_manager::*_MANAGER_NAME`
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

### Not Promoted: `AssetPipelineInfo` / `ProjectInfo`

这次顺带复核了 `AssetPipelineInfo`、`ProjectInfo`、`RenderingBackendInfo` 这组剩余 façade records。

目前我没有把它们升级成下一批候选，原因是：

- 它们没有第二份 canonical duplicate
- 它们直接作为 façade trait 返回值存在：[zircon_manager/src/traits.rs](../../zircon_manager/src/traits.rs)
- 当前实现 crate 只是投影这些值，并没有出现像 `zircon_input` 那样“实现状态机反向依赖 façade 协议模型”的形状

所以在这轮更严格标准下，它们暂时仍属于“可留在 façade 层的稳定返回记录”，还没达到必须迁移的证据门槛。

### Not Promoted: `zircon_ui` Binding / Reflection / Template Protocols

这轮我也重新扫了 `zircon_ui::{UiBindingValue, UiBindingCall, UiEventKind, UiEventPath, UiControlRequest, UiControlResponse, UiReflectionSnapshot, UiNodeDescriptor, UiPropertyDescriptor, UiTemplateLoader, UiDocumentCompiler}` 这组高频 surface。

它们在 editor/editor_ui 里消费面很大，但当前证据还不足以把它们升级成“明显错包”，原因是：

- [zircon_ui/src/binding/mod.rs](../../zircon_ui/src/binding/mod.rs) 与 [zircon_ui/src/event_ui/mod.rs](../../zircon_ui/src/event_ui/mod.rs) 自己就拥有 binding parser、event manager、reflection store 和 invocation 协议
- [zircon_ui/src/template/mod.rs](../../zircon_ui/src/template/mod.rs) 自己拥有 template loader、validator、asset loader、document compiler 和 surface builder
- `zircon_ui` crate 内部测试直接覆盖这组能力：[zircon_ui/src/tests/binding.rs](../../zircon_ui/src/tests/binding.rs)、[zircon_ui/src/tests/event_manager.rs](../../zircon_ui/src/tests/event_manager.rs)、[zircon_ui/src/tests/template.rs](../../zircon_ui/src/tests/template.rs)、[zircon_ui/src/tests/asset.rs](../../zircon_ui/src/tests/asset.rs)

也就是说，虽然 editor 是这组协议的重度调用方，但权威实现和验证目前仍然在 `zircon_ui` 子系统内部。这更像“UI runtime/authoring 共用协议”而不是“editor-only 语义误挂在 runtime crate”。因此这轮先不升级它，继续留在 watchlist。

### Keep In `zircon_manager`

这轮严格审计后，下面几组我判断仍应留在 façade 层：

- `LevelSummary`
  原因：它是 `LevelManager` 稳定 façade 的返回值，scene crate 是实现方，不是消费协议的唯一拥有者。

- `RenderingBackendInfo`
  原因：它是 `RenderingManager` façade 的能力摘要，不携带 editor-specific 或 preview-specific 语义。

## Implemented Batch

这一批已经按审计顺序完成收口，不再停留在候选状态：

1. `zircon_manager::AssetRecordKind` 已删除，`AssetStatusRecord.kind` / `ResourceStatusRecord.kind` 统一改成 `zircon_resource::ResourceKind`
2. `zircon_manager::PreviewStateRecord` 已删除，editor-facing catalog/details/reference records 直接使用 `zircon_asset::PreviewState`
3. `zircon_asset` 不再做 façade taxonomy 投影；`editor/records.rs` 与 `pipeline/manager/records.rs` 都直接传递 canonical kind / preview state
4. `zircon_editor` asset workspace、resource access、event filter parser、Slint asset surface、snapshot structs 和对应测试全部切到 `ResourceKind`
5. 新增 `zircon_input_protocol` 作为 `InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot` 的独立协议 owner
6. `zircon_manager` 删除输入协议 re-export 与 `src/records/input.rs`，只保留 `InputManager` façade trait
7. `zircon_input` 根级 re-export 输入协议，运行态实现不再从 `zircon_manager` 反向取输入类型
8. `zircon_entry` runtime 输入桥改为从 `zircon_input` 导入输入协议类型
9. `CapabilitySet` / `HostHandle` / `PluginSlotId` 已从 `zircon_manager` 迁回 `zircon_script`
10. `zircon_script` 不再依赖 `zircon_manager`；脚本 VM 私有协议和热重载句柄改由脚本子系统自己拥有

这样 `zircon_manager` 继续保留 façade record/trait 本身，但不再重复拥有 asset/editor 展示语义，也不再错误持有输入子系统协议模型。

## Remaining Watchlist

`InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot` 这组已经从候选里划掉。

如果继续推进下一批边界收紧，当前需要做的是重新扫描新的“证据充分”候选，而不是回头再动已收口批次。现阶段仍未升级为下一批主候选的项目包括：

- `AssetPipelineInfo`
- `ProjectInfo`
- `RenderingBackendInfo`
- `zircon_ui` binding / reflection / template protocols
- 其他还未达到“canonical owner 明确且迁移收益清晰”门槛的 watchlist 项

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

结论：

- `zircon_graphics` 红测已修复，根因是 non-VG render path 没有消费 `BuiltMeshDraws.draws`
- `AssetRecordKind` / `PreviewStateRecord` 这组边界已完成收口
- `zircon_manager::service_names` 经二次审计后判定应保留在 façade 层，不再作为迁移候选
- `InputButton` / `InputEvent` / `InputEventRecord` / `InputSnapshot` 已完成协议层收口：`zircon_manager` 不再拥有这组类型，`zircon_input_protocol` 成为唯一 owner
- `zircon_input` / `zircon_entry` 的输入协议导入边界已经切换到 `zircon_input_protocol` / `zircon_input`
- `CapabilitySet` / `HostHandle` / `PluginSlotId` 已完成脚本子系统收口：`zircon_manager` 不再拥有这组 VM 私有类型，`zircon_script` 成为唯一 owner
- `zircon_script` 已去掉对 `zircon_manager` 的依赖，说明这组迁移还顺带消除了原先不必要的 façade 反向依赖
- `AssetPipelineInfo` / `ProjectInfo` / `RenderingBackendInfo` 当前仍未达到迁移门槛
- `zircon_ui` binding / reflection / template protocols 当前仍停留在 watchlist，因为 `zircon_ui` 自身仍是这组协议的实现和测试 owner
- 本轮没有重新跑 full workspace `--locked` 联测；原因是工作树里的 `Cargo.lock` 已脏，这不在本批边界收口范围内

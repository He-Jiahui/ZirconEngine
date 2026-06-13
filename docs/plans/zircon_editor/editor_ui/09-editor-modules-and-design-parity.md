---
related_code:
  - zircon_editor/src/ui/host/editor_runtime_client.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/view/view_registry.rs
  - zircon_editor/src/ui/asset_editor/command.rs
  - zircon_editor/src/ui/asset_editor/session/command_entry.rs
  - zircon_editor/src/ui/asset_editor/replay_workspace.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
plan_sources:
  - .codex/plans/Zircon Editor UI 回迁 + 树形 TOML Cutover 实施计划.md
  - .codex/plans/ZirconEngine Unity 式编辑器优先补齐计划.md
  - .codex/plans/编辑器资源管理器与运行时资源分层重构计划.md
  - .codex/plans/编辑器启动最近工程与 Welcome 新建工程计划.md
  - .codex/plans/Editor Event Decoupling And Replay Plan.md
design_references:
  - docs/ui-and-layout/editor-workbench-designs/scene-workbench.png
  - docs/ui-and-layout/editor-workbench-designs/hierarchy-drawer-content-spec.png
  - docs/ui-and-layout/editor-workbench-designs/inspector-drawer-content-spec.png
  - docs/ui-and-layout/editor-workbench-designs/asset-grid-drawer-content-spec.png
  - docs/ui-and-layout/editor-workbench-designs/console-drawer-content-spec.png
  - docs/ui-and-layout/editor-workbench-designs/timeline-drawer-content-spec.png
  - docs/ui-and-layout/ai-workbench-style/ai-scene-editor-layout.png
  - docs/ui-and-layout/ai-workbench-style/ai-material-editor-layout.png
  - docs/ui-and-layout/ai-workbench-style/ai-asset-browser-layout.png
status: planned
---

# 09 编辑器模块与设计图结构对齐

## 1. 目标

在切换完成的 runtime UI 承载（08）之上，把核心编辑器模块从「模板投影」升级为「数据接线完整、可交互的真实模块」，最终编辑器主界面达到 `editor-workbench-designs` / `ai-workbench-style` 截图的画面组织结构。验收口径：区域结构、组件使用、交互语义与设计图一致；密度与配色走 token；不逐像素。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| runtime 通道 | `zircon_editor/src/ui/host/editor_runtime_client.rs` | `EditorRuntimeClient` trait：`session_handle`（:10）、`handle_event(ZrRuntimeEventV1)`（:12）、`capture_frame`（:14）、`profile_control`（:20）+ `DetachedEditorRuntimeClient`（:31）——**数据接线走事件词汇，不是逐能力方法** |
| 事件基础设施 | `zircon_editor/src/ui/host/` | editor_event_dispatch、editor_event_execution/、editor_event_runtime_reflection、editor_event_control_requests、editor_event_listener_control（事件解耦/重放计划已落） |
| EditorState 族 | `zircon_editor/src/ui/workbench/state/` | editor_state、**editor_state_apply_intent**、editor_state_selection、editor_state_play_mode、editor_state_viewport、editor_state_render、editor_world_slot、no_project_open |
| 反射契约 | `zircon_runtime_interface/src/ui/event_ui/reflection.rs` | `UiReflectionSnapshot`（:492）、`UiReflectionDiff`（:509）；runtime 侧 `surface/{reflection_snapshot, property_mutation}.rs`、`UiSurface::mutate_property` |
| 命令/重放模型（资产编辑器先行） | `zircon_editor/src/ui/asset_editor/` | command.rs、contract.rs、session/command_entry.rs、replay_workspace.rs——undo/重放形态可作场景命令样板 |
| view registry | `zircon_editor/src/ui/workbench/view/` | view_descriptor(+builder)、view_registry(+instance mutation)、dock_policy、pane_template_spec（08 已核实） |
| core module `.zui` workspace | `assets/ui/editor/components/workbench/modules/core/` | index（module_workspace、additional_module_workspaces）、assets（assets_workspace）、ui（hud_workspace）、gameplay（ability/effect/tags）、rendering（material/render/vfx）、ai——共 11 个 |
| 批次 1 面板表面 | `assets/.../workbench/shell/` | scene_tree_panel、inspector_panel、viewport_panel（08 区域承载） |
| pane 数据转换 | `zircon_editor/src/ui/retained_host/ui/pane_data_conversion.rs` | hierarchy/inspector/console/asset browser 的基础数据投影 |

### 2.2 真实缺口

多数模块内容是静态模板或最小投影，缺真实数据流（双向）：选中→inspector 编辑→场景写回、资产操作、console 过滤、诊断实时流；`ZrRuntimeEventV1` 词汇需为批次 1 补齐场景数据事件；场景编辑命令（undo/redo）未建（只有 asset_editor 命令模型）；More Editors 类扩展模块全部 prototype-only。

## 3. 模块分批与内容定义

每个模块的结构以对应 layout-spec / content-spec PNG 为权威，组件只用计划 06 清单。

**批次 1 —— 场景编辑核心环（Unity 式最小可用编辑器）**

| 模块 | 结构（对照设计图） | 数据接线 |
|------|--------------------|---------|
| Scene | 中央 viewport + viewport toolbar（变换工具/视图模式/snap）+ 左 SceneTree + 右 Inspector + 底 Console（`scene-workbench.png`） | runtime client：场景树快照/选中同步、gizmo 操作回写、play mode 控制（editor_state_play_mode 既有） |
| Hierarchy | 搜索 + 树（图标、可见性/锁定列、拖拽重排、context menu）（`hierarchy-drawer-content-spec.png`） | 场景树双向：重命名/重排/删除/新建（06 TreeView 命令 → editor command） |
| Inspector | 对象头（名称/启用/tag/layer）+ 组件分节（Transform VectorRow、材质槽、组件增删）（`inspector-drawer-content-spec.png`） | 反射驱动属性编辑（UiReflectionSnapshot/Diff）→ property mutation → 场景写回；undo/redo |
| Asset Browser | 左 folder tree + 右网格/列表切换 + 过滤 + 缩略图 + 导入入口（`asset-grid-drawer-content-spec.png`） | 目录式 project 资产层；导入走 importer；双击打开对应编辑器 |
| Console | 级别过滤 + 搜索 + 虚拟滚动日志 + 详情区（`console-drawer-content-spec.png`） | runtime/editor 日志流实时接入（03 M5 富文本 span） |

**批次 2 —— 资产型编辑器（每个 = main tab 页 + 专属 drawer 组）**

Material Editor（图画布走 Canvas 容器 + 节点 = 组件组合）、UI Asset Editor（asset_editor session 已有命令模型，接 widget 树 + 属性 + 预览）、Animation/Montage（计划 07 M4 时间轴）、Texture/Mesh 预览检查器。

**批次 3 —— 工具与诊断**

Project Overview、Runtime Diagnostics（editor_manager_runtime_diagnostics 既有数据源）、Performance Timeline（Tracy）、Plugin Manager、Build/Export（editor_manager_plugins_export 既有）、Widget Tree Debugger（02 M5 debug packet 数据面）、Theme Token 预览（04 M6）。

**扩展模块**：More Editors 的 44 个 prototype-only 模块维持 web-native handoff matrix 的推进秩序，按需逐个晋升，不在本计划批量承诺。

## 4. 数据接线架构

- editor ↔ runtime 经 `EditorRuntimeClient` / runtime interface 契约（`ZrRuntimeEventV1` 事件、帧捕获、反射快照、property mutation），不引入新私有通道。批次 1 所需事件词汇扩展集中一次定稿（§5）。
- 模块 UI 状态走 workbench view 模型 + 数据绑定（route id）；编辑操作统一进 editor command（undo/redo、event replay 既有计划约束沿用）。
- 模块面板皆为 view descriptor 注册的可停靠 view，自动获得 docking/浮窗/持久化能力（计划 08）。

## 5. 接口与数据结构草案

```rust
// 场景编辑命令栈（新增 zircon_editor/src/core/command/{mod.rs, stack.rs, scene_commands.rs}）
// 形态对齐 asset_editor/session/command_entry.rs 既有命令模型；业务逻辑归 core/（边界约束）
pub trait EditorSceneCommand {
    fn apply(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorCommandError>;
    fn revert(&mut self, ctx: &mut SceneCommandContext) -> Result<(), EditorCommandError>;
    fn label(&self) -> &str;                       // undo 菜单显示
}
pub struct EditorCommandStack { undo: Vec<Box<dyn EditorSceneCommand>>, redo: Vec<Box<dyn EditorSceneCommand>> }
// 批次 1 命令清单：SpawnNode、DeleteNode、RenameNode、ReparentNode（拖拽重排）、
//   SetTransform（gizmo/VectorRow）、SetProperty（反射路径+旧值新值）、ImportAsset、DeleteAsset

// ZrRuntimeEventV1 词汇扩展（interface，集中一次定稿；既有事件枚举追加 variant）
//   SceneTreeSnapshotRequest / SceneTreeChanged（增量 diff）
//   SelectionSync { selected: Vec<SceneNodeId> }（双向）
//   PropertyMutationRequest { path, value } / PropertyChanged（diff 载体复用 UiReflectionDiff）
//   PlayModeControl { Play | Pause | Step | Stop }
//   LogStreamEvent { level, target, message, timestamp }（Console 接入）

// Hierarchy 完整链（样板，其余模块同构）：
// view_descriptor 注册（view/view_descriptor_builder.rs）
//   → pane 模板 = shell/workbench_scene_tree_panel.zui（06 TreeView 组合）
//   → 数据投影：SceneTreeChanged → pane_data_conversion → mutate_property 刷新树行
//   → 交互：TreeView reducer 发 UiTreeViewCommand（06）→ route_intent（01 M5）
//   → editor_state_apply_intent → EditorSceneCommand 入栈执行
//   → handle_event(ZrRuntimeEventV1) 写回 runtime → SceneTreeChanged diff 回流闭环
pub struct HierarchyViewModel {                    // 新增（workbench/model/ 旁）
    pub rows: Vec<HierarchyRowProjection>,         // 由 SceneTreeChanged 增量维护
    pub filter: String,
}
```

## 6. 模块与文件落点

**新增**：`zircon_editor/src/core/command/{mod.rs, stack.rs, scene_commands.rs}`、`workbench/model/` 下各模块 view model（hierarchy_view_model.rs、inspector_view_model.rs、console_view_model.rs、asset_browser_view_model.rs）、interface 的 ZrRuntimeEventV1 词汇扩展（集中一次）

**修改**：

| 路径 | 改什么 |
|------|--------|
| `host/editor_runtime_client.rs` 消费方 | 新事件词汇收发 |
| `workbench/state/editor_state_apply_intent.rs` | intent → command 入栈（不再直改状态） |
| `retained_host/ui/pane_data_conversion.rs` | 投影改增量（diff 驱动），接 mutate_property |
| `workbench/view/` 注册表 | 批次 1–3 模块 view descriptor 全量注册 |
| `asset_editor/{session, command}.rs` | 与场景命令栈共享 undo 入口（壳级 Edit 菜单） |

**删除（硬切换义务）**：pane_data_conversion 中全量重建式投影路径（增量接通后同变更删除）；模块模板中的静态假数据段（每模块接线切片删除）。

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | 事件词汇定稿：批次 1 所需 ZrRuntimeEventV1 扩展集中一次 + serde 兼容测试 | interface 事件枚举 | `cargo test -p zircon_runtime_interface --locked` | 无删除 |
| M1.S2 | 场景命令栈：EditorSceneCommand + stack + 批次 1 命令清单；Edit 菜单 undo/redo 接 08 M4 命令注册表 | core/command/ | `cargo test -p zircon_editor --lib command --locked` | apply_intent 直改路径删除 |
| M1.S3 | Hierarchy 完整链（§5 样板）：树双向（选中/重命名/重排/删除/新建） | hierarchy_view_model、pane_data_conversion | `cargo test -p zircon_editor --lib hierarchy --locked` | 静态树模板数据删除 |
| M1.S4 | Inspector 反射链：UiReflectionSnapshot → 分节投影 → 属性编辑 → SetProperty 命令 → diff 回流；Transform 走 VectorRow | inspector_view_model | `cargo test -p zircon_editor --lib inspector --locked` | 静态 inspector 模板删除 |
| M1.S5 | Scene viewport 回路（选中同步 + gizmo 写回 SetTransform）+ Console 日志流（LogStreamEvent + 虚拟滚动 + 级别过滤） | console_view_model、viewport 接缝 | `cargo test -p zircon_editor --lib console --locked` + 实机 | 静态 console 数据删除 |
| M1.S6 | **实机回路验收脚本**：新建工程 → 放置对象 → Hierarchy 选中 → Inspector 改 Transform → viewport 即时更新 → Console 出日志 → Ctrl+Z 撤销 | 实机 | editor 实机逐步执行 | 无删除 |
| M2.S1 | Asset Browser：目录树 + 网格/列表 + 过滤 + 缩略图（05 M3 数据面） | asset_browser_view_model | `cargo test -p zircon_editor --lib asset_browser --locked` | 静态资产格删除 |
| M2.S2 | 导入环：导入入口 → importer → 浏览出现 → 双击打开对应编辑器 → 保存 | editor_asset_manager 接缝 | `cargo test -p zircon_editor --lib --locked` + 实机 | 无删除 |
| M3.S1 | UI Asset Editor 可用（asset_editor session 既有命令模型 + widget 树 + 属性 + 热重载预览） | asset_editor/ | `cargo test -p zircon_editor --lib asset_editor --locked` + 实机 | 无删除 |
| M3.S2 | Material Editor 可用（Canvas 图画布 + 节点组合；材质接口走渲染计划 08） | 材质模块 | 实机：改参数 → viewport 反映 | 无删除 |
| M4.S1 | Runtime Diagnostics + Project Overview 真实数据（editor_manager_runtime_diagnostics 既有源） | 批次 3 模块 | `cargo test -p zircon_editor --lib diagnostics --locked` | 静态诊断模板删除 |
| M4.S2 | Widget Tree Debugger（02 M5 packet）+ Theme Token 预览（04）+ Build/Export 面板 | 批次 3 模块 | 实机 + 集成契约 | 无删除 |
| M5.S1 | 逐模块对照设计图 PNG 出差异清单（结构/组件/交互三维度） | 审查文档 | 评审 | 无删除 |
| M5.S2 | 差异收敛 + 剩余差异显式记录落 `docs/zircon_editor/ui/` | 各模块 + 文档 | 实机复查 | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`scene_command_undo_restores_transform`、`hierarchy_rename_round_trips_to_runtime`、`hierarchy_drag_reparent_emits_command`、`inspector_reflection_diff_updates_rows_incrementally`、`console_filters_by_level_and_query`、`selection_sync_is_bidirectional`
- **M2**：`asset_import_appears_in_browser`、`double_click_opens_matching_editor`
- **M3**：`ui_asset_edit_hot_reloads_preview`、`material_param_change_reaches_viewport`
- **M4**：`diagnostics_panel_streams_live_counters`、`widget_tree_debugger_consumes_layout_packet`
- **M5**：逐模块结构对齐清单（人工评审 + 截图存档）

落点：editor `src/tests/` 既有结构 + 模块内 `#[cfg(test)]`；实机脚本步骤写入审查文档。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| 事件词汇反复扩（每模块加一批 variant，ABI 抖动） | M1.S1 批次 1 词汇集中一次定稿；后续批次各自集中一次，禁止零散追加 |
| undo/redo 与 runtime 状态漂移（命令 revert 后场景不一致） | 命令带旧值快照；diff 回流校验测试；event replay 既有设施做回归 |
| 反射快照体量大（大场景 inspector 卡顿） | 增量 diff（UiReflectionDiff）驱动；只订阅选中对象子树 |
| 模块接线对 08 区域切换的依赖排队 | 依赖表显式 gating（§10）；批次 1 只需 08 M3/M4，批次 2/3 部分模块需 08 M5 浮窗 |
| 「结构对齐」验收主观漂移 | M5 差异清单三维度模板化（区域结构/组件使用/交互语义），剩余差异显式记录而非默默搁置 |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 08 M3（全壳承载）、08 M4（命令/快捷键）、06 M2（Tree/Table/PropertyRow）、03 M4/M5（重命名/Console 文本） | 09 M2–M5；**E1 门槛** |
| M2 | 09 M1、05 M3（资产数据面） | 09 M3；E1 门槛 |
| M3 | 09 M2、07 M4（动画模块，部分）、渲染计划 08（材质接口） | 09 M5；E2 门槛 |
| M4 | 09 M1、02 M5（debug packet）、04 M6（theme 预览）、08 M5（浮窗承载） | 09 M5 |
| M5 | 09 M1–M4 | E3 门槛 |

## 11. 完成定义

- M1 实机回路脚本（§7 M1.S6）全步骤通过且可重复执行。
- 批次 1 五模块、批次 2 至少 Material + UI Asset、批次 3 工具面板全部真实数据接线；undo/redo 覆盖全部编辑操作。
- 逐模块设计图对照审查文档落 `docs/zircon_editor/ui/`，剩余差异显式列表。
- 验收命令组：`cargo test -p zircon_editor --lib --locked`、`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked`、实机回路脚本。

## 12. 边界约束

- 模块业务逻辑（场景操作、资产操作）归 editor `core/`/`scene/`，UI 层只做投影与意图转发。
- 不为单个模块发明专属控件——缺组件先回计划 06 补清单。
- 数据接线只经 EditorRuntimeClient / interface 契约；禁止新私有通道。
- 验收始终是「可见、可布局、可点击、可聚焦、可输入、数据真实」，像素级修正只允许出现在 M5 审查后的明确差异项里。

## 13. 参考实现对照（dev/ 源码锚点）

实现模块接线前先读对应锚点，不确定的交互语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 模块/设计点 | 主参考 | 次参考 | 参考什么 |
|------------|--------|--------|---------|
| Inspector 反射属性编辑 | `dev/Fyrox/fyrox-ui/src/inspector/{mod.rs, editors}` | `dev/Fyrox/editor`（inspector 面板接线） | 反射驱动的 property editor 注册表、值变更消息回流——与 09 的 UiReflectionSnapshot/Diff 链直接对应 |
| 场景树/Hierarchy | `dev/Fyrox/editor`（world viewer/scene graph 面板） | `dev/godot/editor`（scene tree dock） | 树与选中同步、重排/重命名命令化、多选语义 |
| 资产浏览器 | `dev/godot/editor`（filesystem dock） | `dev/Fyrox/editor`（asset browser） | 目录树 + 网格/列表 + 缩略图生成 + 双击打开编辑器的组织 |
| undo/redo 命令栈 | `dev/godot/editor`（undo_redo 用法） | `zircon_editor/src/ui/asset_editor/session/command_entry.rs`（仓内样板） | 命令带旧值快照、合并策略（拖拽连续改值合并为一条） |
| Console/日志面板 | `dev/godot/editor`（log/output 面板） | `dev/Fyrox/fyrox-ui/src/log.rs` | 级别过滤、详情区、虚拟滚动日志的交互组织 |
| 时间轴模块 | `dev/theatre/packages/studio` | `dev/godot/editor`（animation 面板） | keyframe 编辑交互（协同 07 M4） |
| 图编辑画布（Material Editor） | `dev/godot/scene/gui/{graph_edit.cpp, graph_edit_arranger.cpp}` | — | 节点图画布的平移/缩放/连线/框选语义（Canvas 容器 + 节点组合的行为标准） |

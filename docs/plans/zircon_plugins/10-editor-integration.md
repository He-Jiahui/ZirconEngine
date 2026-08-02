# 10 · 插件编辑器集成规范（Editor Plugin 约定与 AI Workbench 风格）

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "plugins-10-editor-integration",
  "goal": "维护插件 Editor 与 Runtime UI 之间的共享集成合同、受管验证和跨计划回传。",
  "milestones": [
    {"id": "M1", "title": "Shared UI integration support", "depends_on": []}
  ]
}
```

<!-- Workflow topology is maintained independently from milestone output records. -->
<!-- M1 closeout is submitted through the shared UI integration owner session. -->

> 状态：工程化细化版 v2 · 优先级：P1（横切规范，随各插件 Editor 里程碑落地）
> 关联：`zircon_editor/src/core/editor_plugin.rs`（`EditorPlugin` trait）、`zircon_editor/src/core/editor_extension.rs`（`EditorExtensionRegistry`）
> UI 参考：`docs/ui-and-layout/ai-workbench-style/`（STYLE-NOTES.md + 62 张 workbench 布局图）

- open failure：[terrain-tilemap-scene-mode-factories-missing](10/failure-2026-08-01-terrain-tilemap-scene-mode-factories-missing.md)

## 1. 目标

为所有插件的 Editor 部分确立统一的注册约定、面板布局语言与调试设施规范，使每个 runtime 插件的编辑器体验一致、可被 capability gate 管理，并为"editor 部分多为骨架"的现状给出共同验收标准。

## 2. 现状基线（实查）

- `EditorPlugin` trait（`editor_plugin.rs:226`）：`descriptor()` / `package_manifest()` / `editor_capabilities()` / `register_editor_extensions(&mut EditorExtensionRegistry)` / `on_lifecycle_event(&EditorPluginLifecycleEvent)`——对接 01 §3.4 后与 RuntimePlugin 四阶段对称（lifecycle event 即对称化挂点，01-M4 范围）。
- `EditorExtensionRegistry`（`editor_extension.rs`）的扩展点包含 `register_view` / `register_drawer` / `register_menu_item` / `register_component_drawer` / `register_ui_template` / `register_asset_importer` / `register_asset_editor` / `register_asset_creation_template` / `register_scene_mode(SceneModeRegistration)` / `register_viewport_overlay_provider` / `register_graph_editor` / `register_graph_node_palette` / `register_timeline_editor` / `register_timeline_track_type` / `register_operation`；scene mode 必须携带可执行 factory，不接受仅元数据注册。
- capability gate 原语已在：`EditorExtensionRegistration::with_required_capabilities` / `is_enabled_by`（`editor_extension.rs:271-316`）；缺统一验证管线（01-M4 交付）。
- undo/redo 现状：`core/editing/history.rs`（EditorHistory）+ `ui/host/editor_event_execution/undo_policy.rs` + workbench `editor_state.rs` 各持局部历史——E2 的收口对象；`core/editor_operation.rs` 已有 operation stack 雏形。
- 诊断：`zircon_plugins/runtime_diagnostics` 插件存在（rolling store 宿主）。

## 3. 扩展点使用约定（签名级）

插件 Editor 部分一律实现 `EditorPlugin` trait，经 `EditorExtensionRegistry` 注册，禁止旁路接线编辑器内部状态。各扩展点的使用规则（描述符均为 `editor_extension.rs` 现有类型）：

| 扩展点（现有签名） | 用途 | 命名规则 |
|--------|------|---------|
| `register_view(ViewDescriptor::new(id, display_name, category))` | 插件主面板（Mixer Console、Navigation Bake、BT Debugger 等） | `view.<plugin>.<name>` |
| `register_component_drawer(ComponentDrawerDescriptor::new(component_type, ui_document, controller))` | 插件组件 Inspector 绘制器 | 组件全名 `<Plugin>.Component.<Type>` |
| `register_asset_editor` / `register_asset_creation_template` | 插件资产（.btree.toml、.ragdoll.toml、.avatar_mask.toml、.znavmesh…）打开/新建 | 按资产扩展名路由 |
| `register_graph_editor` / `register_graph_node_palette` | 节点图类编辑（动画状态机、行为树、未来 shader graph） | 共享图编辑基座，插件只提供 node palette + 语义校验 |
| `register_timeline_editor` / `register_timeline_track_type(TimelineTrackDescriptor)` | 时间轴类（animation sequence、sound automation） | track type 全名 `<plugin>.track.<name>` |
| `register_scene_mode(SceneModeRegistration)` | 可执行视口场景模式（navmesh 烘焙范围、碰撞体编辑手柄） | descriptor 与 factory 原子注册；overlay 绘制走共享 gizmos 通道 |
| `register_operation` → `EditorOperationRegistry` | 一切可撤销编辑动作 | `XXX.YYY.ZZZ` 命名（既定规则），进统一 EditorOperationStack（E2） |
| `register_menu_item(EditorMenuItemDescriptor::new(path, operation))` | 菜单入口 | Unity 风格路径 `Tools/<Plugin>/<Action>`；debug overlay 一律 `View/Debug Overlays/<Plugin>` |

对接 [01 计划](01-plugin-architecture-core.md) M4 的新增要求：

- Editor 注册同样走 capability gate：缺失 capability 产出 `RegistrationDiagnostic`（`zircon_runtime_interface/src/plugin_diagnostics.rs`，01-M4-T1）而非静默禁用；`EditorPluginDescriptor` 的 capability 从 `plugin.toml` 单源派生（01-M4-T2）。
- 组件 drawer 默认实现由反射描述自动生成（E1，见 §6），插件只为需定制的字段注册覆盖绘制器。

## 4. AI Workbench 风格对位表

各插件主面板以 `docs/ui-and-layout/ai-workbench-style/` 对应布局图为版式基准（三栏 workbench：左结构树 / 中主编辑区 / 右属性检查器，遵循 STYLE-NOTES.md 间距与层级 token）：

| 插件面板 | 参考布局图 | 交付里程碑 |
|----------|-----------|-----------|
| Sound Mixer Console | 沿用现有 `mixer_console.zui`，token 对齐 STYLE-NOTES | [02](02-sound.md) M5 |
| Physics 碰撞调试 / Ragdoll 编辑 | `ai-physics-collision-layout.png` | [03](03-physics.md) M6 |
| Animation 状态机 / Blend Space / Sequencer | `ai-blend-space-layout.png`、`ai-sequencer-layout.png`、`ai-montage-editor-layout.png` | [04](04-animation.md) M6 |
| Navigation 烘焙与 Agent 调试 | `ai-navmesh-ai-layout.png` | [05](05-navigation.md) M6 |
| AI 行为树编辑/调试、Perception | `ai-behavior-tree-layout.png`、`ai-ai-perception-layout.png` | [06](06-ai.md) M5 |
| Net 诊断 / Replication 配置 | `ai-console-diagnostics-layout.png` | [07](07-net.md) M7 |
| 导出向导 | `ai-build-export-layout.png` | [09](09-export-publishing.md) M6 |

面板实现走 retained host 的 `.zui` 模板体系（与 `docs/zircon_editor/ui/retained_host/` 契约一致），经 `register_ui_template(EditorUiTemplateDescriptor)` 注册；`.ui.toml` / `.v2.ui.toml` 后缀已退役，不作为当前插件 editor view/layout 文档口径；不引入 Slint 旁路（zircon_hub 独占 Slint 的边界维持）。

## 5. 运行时调试设施规范（横切，每插件 Editor 里程碑必交付）

1. **Viewport overlay**：空间型插件（physics/navigation/sound volume/ai perception）提供可开关视口叠加——gizmos 通道绘制，菜单统一 `View/Debug Overlays/<Plugin>`（`register_menu_item` + overlay 开关 operation）；overlay 注册清单进契约快照测试。
2. **诊断接入**：运行时指标（physics step 耗时、GC 暂停、net 带宽、audio underrun）进 rolling diagnostics store——插件在 runtime 侧注册 `DiagnosticPath`（`runtime_diagnostics` 插件的路径目录），editor 侧面板自动呈现，插件 editor 不写自有图表。
3. **Play-in-editor 状态镜像**：行为树节点状态、状态机当前状态、agent 路径等运行态经 editor 运行时客户端**只读镜像**到对应面板——数据走类型化事件（`BtNodeResultEvent` 等）或反射读请求（`ReflectReadRequest`，`zircon_runtime_interface::reflect` 现有 DTO），编辑器不直接持有 runtime world 引用（维持 ABI 边界）。

## 6. 前置基建两项（单列工程项）

### E1 反射驱动默认 drawer（前置：[08 ZrVM](08-zr-vm.md) M1 derive 宏）

- 裁决：默认 drawer 采用**运行期构建**（由 `ReflectTypeInfo` + `ReflectEditorHint` 即时生成控件树），不走静态模板投影——理由：组件 schema 随插件加载动态变化，静态模板生成会引入第二份需同步的产物；定制 drawer 仍走 `.zui`（`register_component_drawer` 覆盖）。
- 落点：`zircon_editor/src/ui/inspector/reflect_drawer.rs` [新增]——`ReflectTypeKind` → 控件映射表（数值带 `ReflectNumericRange` 滑条、枚举带 `ReflectEnumOption` 下拉、嵌套 struct 折叠组）；写回经 `ReflectWriteRequest` + `register_operation` 包装（自动可撤销）。
- 验收测试：`reflect_drawer_renders_all_reflect_type_kinds`、`reflect_drawer_write_is_undoable`。

### E2 EditorOperationStack 收口（前置：01-M4）

- 统一 undo/redo 栈替代分散历史。迁移表（实查）：

| 现状 | 动作 |
|------|------|
| `core/editing/history.rs`（EditorHistory） | 收编为 `EditorOperationStack` 内核，迁移至 `core/editing/operation_stack.rs` |
| `ui/host/editor_event_execution/undo_policy.rs` | 改为消费统一栈的 policy 层 |
| `ui/workbench/state/editor_state.rs` / `editor_state_play_mode.rs` 局部历史 | 删除局部栈，经 operation 提交 |
| `core/editor_operation.rs` 雏形 | 扩展为栈宿主：`push(op)` / `undo()` / `redo()` / `transaction(label)`（合并复合编辑） |

- 验收测试：`all_registered_operations_are_undoable`（契约：注册表内每个 operation 必须实现 inverse 或显式声明 non-undoable 并给理由）、`transaction_undoes_atomically`。

## 7. 验收标准（每插件 Editor 里程碑通用）

- `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked` 全绿。
- 插件 editor crate 契约测试覆盖：扩展点注册清单快照、operation 可撤销性、capability 缺失时的降级行为（`RegistrationDiagnostic` 产出断言）。
- §5 三项调试设施各有对应测试（overlay 快照 / DiagnosticPath 注册 / 镜像通道契约）。
- `docs/zircon_plugins/<plugin>/editor.md` 与实现同步更新。

## 8. 里程碑

本文档为规范而非独立工程项；落地节奏跟随各插件计划的 Editor 里程碑（02-M5、03-M6、04-M6、05-M6、06-M5、07-M7、09-M6）。前置基建单列：

| 项 | 内容 | 前置 | 新增测试 |
|----|------|------|---------|
| E1 | 反射驱动默认 drawer（运行期构建） | 08-M1 | `reflect_drawer_renders_all_reflect_type_kinds`、`reflect_drawer_write_is_undoable` |
| E2 | EditorOperationStack 收口（迁移表见 §6） | 01-M4 | `all_registered_operations_are_undoable`、`transaction_undoes_atomically` |

## 9. 验收命令

```bash
cargo test -p zircon_editor --lib --locked
cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked
```

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`10/2026-07-09-editor-integration-output-records.md`](10/2026-07-09-editor-integration-output-records.md)

## 10. 风险

- E2 触及 workbench 状态层的局部历史，与编辑器既有事件执行路径（editor_event_execution）耦合深；迁移按“先栈后策略”两步走，undo_policy 行为以现有测试为回归网。
- E1 的运行期控件树在大组件（几十字段）上的布局性能需要 retained host 虚拟化支持；首版以折叠组惰性展开规避。

## 11. 附录 · dev 参考源码对位

实现各项前**必须先读对应参考实现**，交互语义与视觉基准对照真实代码核对，禁止凭空实现：

| 设计点 | 参考源码（已核验存在） | 看什么 |
|--------|----------------------|--------|
| 反射驱动 Inspector（E1） | `dev/godot/editor/`（inspector 相关：editor_inspector/property editor 族）与 `dev/Fyrox/editor/`（reflect-based Inspector） | 类型 → 控件映射表、嵌套折叠/数组项编辑、撤销集成点 |
| undo/redo 栈（E2） | `dev/godot/`（core/object 下 undo_redo 实现 + editor 侧封装） | action 合并（merge mode）、对象生命期安全引用、transaction 嵌套 |
| 编辑器控件视觉/交互基准 | `dev/material-ui/`、`dev/material-components/`、`dev/bevy/crates/bevy_feathers/`、`bevy_ui_widgets/` | 组件状态层（hover/focus/disabled）、密度 token——与 `ai-workbench-style/STYLE-NOTES.md` 互补 |
| 时间轴/sequencer 交互 | `dev/theatre/`（Theatre.js studio 的 sequence 编辑） | 关键帧选择/拖拽/吸附、轨道折叠交互形态 |
| 声明式 UI 模板体系 | `dev/slint/` | 模板编译与属性绑定（zircon_hub 一侧的边界参照，editor 不引入） |

---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/presentation_cache.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/retained_host/root_shell_projection.rs
  - zircon_editor/src/ui/retained_host/floating_window_projection.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/view/view_registry.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/dock_policy.rs
  - zircon_editor/src/ui/workbench/window_registry
  - zircon_editor/src/ui/workbench/preset/shell_preset.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/window/mod.rs
plan_sources:
  - .codex/plans/Zircon Editor Workbench Shell V1.md
  - .codex/plans/Zircon Editor Workbench Shell VNext.md
  - .codex/plans/JetBrains Hybrid Workbench Shell Spec Implementation Plan.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - .codex/plans/Drawer_Window_Menu Slate 化推进计划.md
design_references:
  - docs/ui-and-layout/ai-workbench-style/ai-workbench-web-framework.png
  - docs/ui-and-layout/editor-workbench-designs/main-tabs-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/tool-drawers-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/drawer-collapsed-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/floating-tool-window-state-spec.png
status: planned
---

# 08 Workbench Shell 全面切到 Runtime UI

## 1. 目标

宿主编辑器窗口从「editor 自管 presentation/painter」切到「runtime UI surface 承载」：workbench shell（top toolbar + main tabs、activity rail、左/右/底 drawer、中央 document workspace、status bar）全部以计划 06 的 L4 组件拼装，布局走 runtime Taffy + docking 接缝（02 M4），输入走统一 input manager（01 M5），样式走 selector（04），渲染继续走 GPU command stream。editor 只保留 workbench/docking/windowing 语义与编辑器业务状态。同时补齐壳级缺口：浮动窗口、完整菜单栏、快捷键、布局持久化、context menu、toast 触发、status bar 扩展。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| runtime `UiSurface` 宿主能力 | `zircon_runtime/src/ui/surface/surface.rs` | `hit_test`（:156）、`surface_frame`（:165）、`mutate_property`（:223）、`reflector_snapshot`（:299）、`focus_path`（:307）、`capture_pointer`（:315）、`apply_dispatch_reply`（:338）、`dispatch_input_event(_with_manager)`（:354/:363）、`dispatch_window_input_pump_event(_with_manager)`（:371/:380）——**宿主接缝 API 基本齐备** |
| L4 shell `.zui` 资产（8 件） | `zircon_editor/assets/ui/editor/components/workbench/shell/` | workbench_top_toolbar、workbench_main_band、workbench_activity_rail、workbench_status_bar、workbench_component_drawer、workbench_scene_tree_panel、workbench_inspector_panel、workbench_viewport_panel |
| view registry | `zircon_editor/src/ui/workbench/view/` | view_descriptor(+builder/id)、view_registry(+descriptor/instance access、instance_mutation)、view_kind、dock_policy、pane_template_spec、pane_route_namespace、preferred_host 共 20 文件 |
| 窗口注册表 | `zircon_editor/src/ui/workbench/window_registry/` + `src/tests/workbench/registry/window_registry.rs` | EditorWindowRegistry 已有骨架与测试 |
| 布局 preset | `zircon_editor/src/ui/workbench/preset/` | default_layout、default_registry、shell_preset、panel_preset、functional_window、design_stack |
| shell 几何 | `zircon_editor/src/ui/workbench/autolayout/` | workbench_shell_geometry、region、constraints（02 M4 接缝对象） |
| 旧投影双轨（待退役） | `zircon_editor/src/ui/retained_host/` | `app/presentation_cache.rs`（HostPresentationCache）、`root_shell_projection.rs`、`floating_window_projection.rs`、`host_contract/painter/` 全 template_* 族、presenter/ |

### 2.2 真实缺口

1. **双轨投影**：editor host contract painter 与 runtime render extract 是两套投影；shell 像素当前由 painter 路径产出。
2. **菜单栏只有快速按钮**；无命令注册表（grep 无 CommandRegistry/keymap 命中）、无快捷键表、无 CommandPalette 数据源。
3. **浮窗仅模板投影**（floating_window_projection.rs），window_registry 未接独立 `UiSurface` + 原生子窗口。
4. **布局持久化不全**：`host/layout_persistence.rs` 与 preset 模块已有骨架（default/builtin 级），project workspace 与 global default 序列化恢复缺失。
5. 全局 context menu、toast 触发链、status bar 实时状态、split tabs 未完成。

## 3. 设计

### 3.1 承载切换（核心硬切换）

- editor 主窗口持有一个 runtime `UiSurface`（新增 `retained_host/shell_surface_host.rs` 作 owner）：shell 树由 L4 组件实例化（§2.1 的 8 件 `.zui`），`UiSurfaceFrame` 驱动布局/命中/提取，GPU command stream 消费 `UiRenderExtract`。
- **区域硬切顺序与删除清单**（每区域同变更删除旧投影）：

| 区域 | 承载 `.zui` | 同变更删除 |
|------|------------|-----------|
| status bar（M1） | workbench_status_bar.zui | painter 中 status bar 投影段 + presentation_cache 对应区段 |
| activity rail（M2） | workbench_activity_rail.zui | activity_rail_pointer 桥（与 01 M5 协同）+ painter rail 段 |
| main tabs（M2） | workbench_main_band.zui | document_tab_pointer、tab_drag 命中态 + painter tab 段 |
| drawers（M2） | workbench_component_drawer.zui + scene_tree/inspector panel | drawer_header_pointer、drawer_resize 命中态 + painter drawer 段 |
| 全 shell（M3） | workbench_top_toolbar.zui + viewport_panel.zui | `root_shell_projection.rs`、`app/presentation_cache.rs`、`host_contract/painter/` 投影族、presenter/ 残余 |

- workbench 模型（`WorkbenchLayout`、view registry、EditorState）保留为业务状态层，经数据绑定（route id / `UiSurface::mutate_property`）与 surface 同步——editor 改状态、runtime 改像素。

### 3.2 Docking 与窗口

- docking 拓扑沿用 Shell V1 定稿：固定壳 + 受控 docking 树；只有中心 document workspace 与浮窗允许递归 split；6 个固定 drawer 槽；dock_policy.rs 既有语义沿用。
- FloatingWindow：window_registry 管实例；每个浮窗一个独立 `UiSurface` + 原生子窗口（复用 runtime window 抽象）；drawer ↔ 浮窗互转。
- 布局持久化：preset > project workspace > global default > builtin fallback 四级恢复；序列化 docking 树 + drawer extent + 活动 view。

### 3.3 壳级功能补齐

- **菜单栏**：File/Edit/View/Window/Help 真实菜单树（PopupMenu 多级已有路由），菜单项 = command id + 快捷键标注 + enabled 谓词。
- **命令与快捷键**：editor command registry（id、标题、类别、默认键位）；keymap 资产（TOML）可改绑；input manager 焦点链未消费的按键进 keymap 解析；CommandPalette（06 M3 骨架）按命令注册表搜索执行。
- **Context menu**：右键经 hit path 取最近声明 context-menu provider 的节点，editor 按节点语义出菜单。
- **Toast/通知**：editor 事件（构建完成、导入失败等）→ Toast 队列（06 行为）+ NotificationCenter 历史。
- **Status bar**：左侧状态消息/警告计数，右侧 grid/snap/zoom 等 chips + 任务进度槽，数据绑定实时刷新。

### 3.4 Viewport 接缝

- ViewportPanel 作为 UI 节点持有 runtime 场景纹理（GPU command stream 已支持 surface 合成）；指针事件经 01 的路由进入 viewport 节点后转交 scene 交互路径（picking/gizmo/camera controller），UI 不解释 3D 语义。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_editor/src/ui/retained_host/shell_surface_host.rs
pub struct EditorShellSurfaceHost {
    surface: UiSurface,                        // 现有类型（runtime）
    intent_map: EditorRouteIntentMap,          // 01 M5 类型
    binding_sync: ShellBindingSync,            // workbench 状态 → mutate_property 批
}
impl EditorShellSurfaceHost {
    pub fn instantiate_shell(&mut self, /* prototype store 句柄 */) -> Result<(), EditorShellHostError>;
    pub fn pump_input(&mut self, batch: UiWindowInputPumpBatch) -> Vec<EditorIntent>;   // 01 outcome → intent
    pub fn sync_workbench_state(&mut self, layout: &WorkbenchLayout /* 现有 */);         // 状态差量 → property mutation
}

// 新增 zircon_editor/src/ui/host/commands/{mod.rs, registry.rs, keymap.rs}
pub struct EditorCommandRegistry { commands: Vec<EditorCommandDescriptor> }
pub struct EditorCommandDescriptor {
    pub id: EditorCommandId,                   // "editor.scene.delete" 等稳定 id
    pub title: String,
    pub category: EditorCommandCategory,       // File | Edit | View | Window | Help | Scene | Asset
    pub default_binding: Option<EditorKeyChord>,
    pub enabled_route: Option<String>,         // enabled 谓词的数据绑定路径
}
pub struct EditorKeymap { bindings: Vec<(EditorKeyChord, EditorCommandId)> }
// keymap 资产 TOML（zircon_editor/assets/ui/editor/keymap/default.keymap.toml）：
// [bindings]
// "editor.command_palette" = "Ctrl+Shift+P"
// "editor.scene.delete"    = "Delete"
// "editor.scene.rename"    = "F2"
impl EditorKeymap {
    pub fn resolve(&self, chord: EditorKeyChord) -> Option<EditorCommandId>;   // 焦点链未消费按键进此
}

// 布局持久化（扩展既有 zircon_editor/src/ui/host/layout_persistence.rs）
pub struct WorkbenchLayoutSnapshot {
    pub docking_tree: /* dock_policy 序列化形态 */,
    pub drawer_extents: Vec<(ShellRegionId, f32)>,   // 现有 ShellRegionId
    pub active_views: Vec<ViewInstanceId>,           // 现有类型（view/）
}
pub enum WorkbenchLayoutSource { Preset(String), ProjectWorkspace, GlobalDefault, Builtin }
pub fn restore_layout(/* 四级查找 */) -> (WorkbenchLayoutSnapshot, WorkbenchLayoutSource);

// 浮窗（扩展 workbench/window_registry/）
pub struct EditorFloatingWindow {
    pub view: ViewInstanceId,
    pub surface: UiSurface,                    // 每浮窗独立
    pub native_window: /* runtime window 抽象句柄 */,
}
```

## 5. 模块与文件落点

**新增**：`retained_host/shell_surface_host.rs`、`host/commands/{mod.rs, registry.rs, keymap.rs}`、`assets/ui/editor/keymap/default.keymap.toml`、菜单树 `.zui`/模板（File/Edit/View/Window/Help 内容声明）

**修改**：

| 路径 | 改什么 |
|------|--------|
| `retained_host/app.rs`、`app/host_lifecycle.rs` | 主窗口生命周期挂 EditorShellSurfaceHost；按区域逐步把渲染来源切到 surface extract |
| `workbench/window_registry/` | 浮窗实例持独立 UiSurface + 原生子窗口 |
| `host/layout_persistence.rs`、`workbench/preset/{default_layout, shell_preset}.rs` | 扩展为四级恢复链（序列化 docking 树 + drawer extent + 活动 view） |
| `workbench/view/view_registry*.rs` | 视图激活状态经 binding_sync 同步 |
| `host/module.rs` | EditorModule 接线命令注册表与 keymap 加载 |

**删除（硬切换义务，按区域分批）**：§3.1 删除清单全部条目；M3 末 `host_contract/painter/` 投影族、`presentation_cache.rs`、`root_shell_projection.rs`、presenter/ 旧路径删除确认（验收项）。

## 6. 管线时序（切换后）

```
winit（editor EventLoop）→ 01 platform_input 翻译 → batch
→ EditorShellSurfaceHost.pump_input → UiSurface dispatch（01 七阶段路由）
→ component events → route_intent → EditorIntent → editor command（undo/redo）
→ workbench 状态变更 → sync_workbench_state → mutate_property → dirty
→ runtime 帧管线（state→motion→layout→text→extract）→ GPU command stream → present
键盘未消费 → EditorKeymap.resolve → command 执行
```

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | EditorShellSurfaceHost 骨架：主窗口挂 UiSurface，实例化最小 shell 树（仅 status bar 区） | shell_surface_host.rs、app.rs | `cargo check -p zircon_editor --lib --locked` | 无删除 |
| M1.S2 | status bar 区域首迁：workbench_status_bar.zui 承载 + 数据绑定（状态消息/chips） | shell_surface_host.rs、status_bar.zui | `cargo test -p zircon_editor --lib status_bar --locked` | 删 painter status bar 段 |
| M1.S3 | 实机：status bar 由 runtime 路径渲染/命中（GPU stream 验收 software_fallback_count=0 沿用） | 实机 | editor 实机 | 删 presentation_cache 对应区段 |
| M2.S1 | activity rail 迁移（与 01 M5.S2 协同删桥） | activity_rail.zui、shell_surface_host | `cargo test -p zircon_editor --lib activity_rail --locked` | 删 rail 桥命中态 + painter 段 |
| M2.S2 | main tabs 迁移：tab 切换/拖拽重排走 runtime（06 TabStrip） | main_band.zui | `cargo test -p zircon_editor --lib document_tab --locked` | 删 tab 桥命中态 + painter 段 |
| M2.S3 | drawers 框架迁移 + docking 接缝接通（02 M4 PaneContentRootConstraint）：开合/改宽/切 tab 全走新路径 | component_drawer.zui、autolayout | `cargo test -p zircon_editor --lib drawer --locked` + 实机 | 删 drawer 桥命中态 + painter 段 |
| M3.S1 | 剩余区域（top toolbar、document workspace、viewport 挂点）迁移 | top_toolbar.zui、viewport_panel.zui | `cargo test -p zircon_editor --lib --locked` | 删 root_shell_projection.rs |
| M3.S2 | 旧路径总删除：painter 投影族、presentation_cache、presenter 残余；全壳实机交互回归 | retained_host/ | `cargo test -p zircon_editor --lib --locked` + `--test integration_contracts --features integration-contracts` + 实机 | **删除确认清单出文档** |
| M4.S1 | EditorCommandRegistry + 默认命令集（File/Edit/View/Window/Help 全菜单项） | host/commands/ | `cargo test -p zircon_editor --lib commands --locked` | 快速按钮旧实现删除 |
| M4.S2 | keymap 资产 + 焦点链未消费按键解析（01 路由 default-action 后接） | keymap.rs、keymap.toml | `cargo test -p zircon_editor --lib keymap --locked` | 无删除 |
| M4.S3 | 菜单栏真实菜单树 + CommandPalette 接命令源（06 M3 骨架）；命令矩阵测试 + 实机快捷键 | 菜单模板、palette | 同上 + 实机 | 无删除 |
| M5.S1 | 浮窗：window_registry 接独立 UiSurface + 原生子窗口；drawer ↔ 浮窗互转 | window_registry/ | `cargo test -p zircon_editor --lib window_registry --locked` | 删 floating_window_projection.rs |
| M5.S2 | 布局持久化四级恢复：序列化 + 启动恢复 + preset 切换 | layout_persistence.rs、preset/ | `cargo test -p zircon_editor --lib layout_persistence --locked` + 重启实机 | 无删除 |
| M6.S1 | context menu：hit path → provider 查找 → 节点语义菜单 | shell_surface_host、菜单 | `cargo test -p zircon_editor --lib context_menu --locked` | 无删除 |
| M6.S2 | toast 触发链 + NotificationCenter 历史（06 M3 组件） | editor 事件 → toast 队列 | `cargo test -p zircon_editor --lib toast --locked` | 无删除 |
| M6.S3 | status bar 实时状态（任务进度槽）+ 实机验收 | binding_sync | 实机 + focused tests | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`status_bar_renders_via_runtime_extract`、`status_bar_chip_click_routes_intent`
- **M2**：`drawer_resize_reflows_content_via_taffy`、`tab_drag_reorders_documents`、`rail_button_toggles_drawer`
- **M3**：`no_painter_projection_paths_remain`（编译期：旧模块不存在）、集成契约全量回归
- **M4**：`keymap_resolves_unconsumed_chord_to_command`、`menu_item_enabled_predicate_follows_state`、`palette_filters_commands_by_query`
- **M5**：`floating_window_owns_independent_surface`、`layout_restores_from_project_workspace_over_global`
- **M6**：`right_click_opens_nearest_provider_menu`、`toast_fires_on_import_failure_event`

落点：editor `src/tests/` 既有结构（host/、workbench/ 子目录惯例）+ 模块内 `#[cfg(test)]`。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| 区域双路径过渡期视觉/交互不一致 | 区域硬切：同区域单帧只有一个来源；切前后截图对比纳入切片验收 |
| 旧 painter 删除牵连未盘点调用方 | M3.S2 出删除确认清单；编译期断言（模块不存在即红）|
| 浮窗多 surface 的输入焦点/IME 归属混乱 | 每窗口独立 batch 与 focus path；跨窗口焦点切换测试 |
| 布局持久化格式将随 docking 演进破坏兼容 | snapshot 带版本号；不可读时按四级链降级并记录诊断 |
| keymap 与文本编辑焦点冲突（输入框里按 Delete） | 严格按 01 路由次序：focus-path 未消费才进 keymap；测试覆盖输入框场景 |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 01 M3、02 M1、04 M4、05 M2（shell 资产热重载）、06 M1 | 08 M2 |
| M2 | 08 M1、01 M5（对应区域桥收编）、02 M4（docking 接缝）、06 M4（L4 组合） | 08 M3 |
| M3 | 08 M2 | 09 全部（E0 门槛）、08 M4–M6 |
| M4 | 08 M3、06 M3（CommandPalette 骨架） | 09 M1（编辑命令入口） |
| M5 | 08 M3 | 09 批次 2/3（浮窗承载编辑器） |
| M6 | 08 M3、06 M3（toast/context menu 组件）、07 M2（过渡动画，弱） | 09 M1（console/状态反馈） |

## 11. 完成定义

- editor 主窗口像素全部来自 runtime render extract → GPU command stream；painter 投影族物理删除。
- 实机全壳交互：tabs/drawer/rail/status bar/菜单/快捷键/浮窗/布局恢复全部可用。
- `cargo test -p zircon_editor --lib --locked` 与集成契约全绿；software_fallback_count=0 持续成立。
- 验收命令组：`cargo test -p zircon_editor --lib --locked`、`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked`、实机 `cargo run -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor`。

## 12. 边界约束

- 切换以区域为单位硬切，禁止同区域双路径长期并存；GPU command stream 验收标准（software_fallback_count=0）持续有效。
- workbench/docking/windowing 语义不下沉 runtime；runtime 不出现 editor page/drawer/window 概念。
- 视觉验收对照 `ai-workbench-web-framework.png` 的结构与配色方向：近黑 chrome、teal 激活态、左窄 activity rail、左 scene/assets 树、右 inspector、底 console/timeline、薄 status bar——结构正确即可，不逐像素。
- 命令 id 是稳定契约：改名走废弃别名表，不直接断链。

## 13. 参考实现对照（dev/ 源码锚点）

实现切片前先读对应锚点，不确定的行为语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 设计点 | 主参考 | 次参考 | 参考什么 |
|--------|--------|--------|---------|
| docking/tab manager | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking`（FTabManager/SDockTab） | `dev/Fyrox/fyrox-ui/src/dock/{tile.rs, config.rs}` | tab 拖出/合回、layout 序列化恢复（FTabManager::PersistLayout 对应布局持久化四级链）；Fyrox dock config 是 Rust 端序列化样板 |
| 菜单栏/工具栏装配 | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/MultiBox` | — | MultiBox：菜单项 = command + 可扩展插槽的装配模式（菜单树声明的架构参照） |
| 命令注册表/快捷键 | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands`（UICommandList/InputBindingManager） | — | command id、默认键位、上下文绑定链、按键未消费时的命令解析（EditorKeymap.resolve 对照） |
| 通知/Toast | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Notifications` | — | NotificationManager 的队列与生命周期 |
| 编辑器壳整体组织 | `dev/Fyrox/editor` | `dev/godot/editor`（editor_node 等） | Rust 引擎编辑器的 docking 壳 + 面板注册组织；Godot 的 dock 槽位/主题化壳（结构参考） |
| 多窗口/浮窗 | `dev/bevy/crates/bevy_winit`（多窗口生命周期） | `dev/Fyrox/fyrox-ui/src/window.rs` | 每窗口独立 surface 的事件归属与关闭语义；Fyrox 的 UI 内浮动窗口（拖动/缩放/置顶） |

---
related_code:
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/component/descriptor/mod.rs
  - zircon_runtime/src/ui/component/descriptor/validation.rs
  - zircon_runtime/src/ui/component/state_reducer.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/component/data_binding
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/descriptor
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives
  - docs/ui-and-layout/ai-workbench-style/component-prototype/src/components
  - dev/material-ui/packages/mui-material/src
plan_sources:
  - .codex/plans/Material UI 全组件原型陈列与渲染实现计划.md
  - .codex/plans/Material UI 全组件样式设计与验证计划.md
  - .codex/plans/Runtime UI 组件库与 Slint Material Showcase Cutover 计划.md
  - .codex/plans/Zircon UI .zui 组件资产与 Unreal 风格入口重构计划.md
design_references:
  - docs/ui-and-layout/ai-workbench-style/component-prototype/index.html
  - https://mui.com/material-ui/all-components/
status: planned
---

# 06 MUI 式组件库落地

## 1. 目标

按 MUI 的自底向上组件分类，把编辑器需要的全部组件做成「runtime 路径完整」的真实组件：每个组件 = `.zui` 资产 + component descriptor + 语义状态机（state reducer）+ Taffy 布局描述 + selector 化样式 + render extract + 输入行为 + 焦点/可访问性语义。组件是统一描述、可拼装的——多个相似布局一种声明，禁止逐组件像素特调。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| 组件契约 | `zircon_runtime_interface/src/ui/component/` | `UiComponentFlags`（state.rs:9）、`UiComponentState`（state.rs:25）、`UiComponentEventKind/UiComponentEvent`（event.rs:7/:43）、descriptor/、data_binding/、drag.rs、catalog/、category.rs、validation.rs |
| catalog | `zircon_runtime/src/ui/component/catalog/material_foundation/` | 25 个子模块文件：inputs、button_inputs、selection_inputs、text_inputs、form_controls、data_display（+editor/subcomponents/table）、feedback、layout（+editor/mui/transitions/utilities）、navigation（+editor/secondary/subcomponents）、surfaces、surface_subcomponents、mui_x、lab_subcomponents、shared |
| 状态机入口 | `zircon_runtime/src/ui/component/state_reducer.rs` | `apply_component_event`（:8）——单入口，未按 family 分表 |
| descriptor 校验 | `zircon_runtime/src/ui/component/descriptor/{mod,validation}.rs` | 已有 |
| `.zui` 原语资产（白名单 4 目录） | `zircon_editor/assets/ui/editor/components/workbench/primitives/` | **inputs（10）**：button、checkbox、dropdown、field、icon_button、radio、segmented_control、slider、tab、toggle；**data（5）**：list_row、tree_row、table_row、property_row、component_property_row；**feedback（4）**：popup_menu、tooltip、toast、status_item；**chrome（4）**：rail_button、chip、section_title、axis_value_field |
| 状态选择器 | `zircon_runtime_interface/src/ui/style.rs` | `UiPainterStyleSelector` 按 family 折叠（04 已核实） |

### 2.2 真实缺口

1. 「covered」≠「行为完整」：现有原语多数有渲染与基本点击，但键盘交互、焦点语义、编辑行为、popup 行为完整度参差，且无逐组件的 7 维盘点记录。
2. `.zui` 缺口（对照 §3 清单）：L1 缺 Label、Icon、ProgressBar、Divider、Skeleton、NumberField、SearchInput、RangeSlider、TabStrip 独立资产；L3 缺 ContextMenu、DropdownPopup、Alert、Dialog/ConfirmDialog、CommandPalette、NotificationCenter、DragOverlay。
3. `state_reducer` 是单入口函数，组件 family 行为未分表；部分行为仍在 editor pointer bridge / template bindings（依赖计划 01 M5 收编）。
4. 复杂组件（Dropdown 弹出定位与键盘导航、Slider 拖拽与步进、TextField 全编辑链、TreeView 虚拟化+展开+多选、Table 列宽/排序、PopupMenu 多级）未定稿。

## 3. 组件清单与分层（验收以此清单逐项打勾）

按依赖序分四层，上层只组合下层。**现状标注**：✓=有 `.zui` 原语，△=catalog 有定义无独立资产或行为不全，✗=缺。

**L1 原子（atoms）**
Label(△)、Icon(✗，依赖 05 M4 图标通道)、Button(✓)、IconButton(✓)、TextField/Field(✓，含 SearchInput(✗)、NumberField/拖拽改值(△ axis_value_field 雏形))、Checkbox(✓)、Radio(✓)、Toggle/Switch(✓)、Slider(✓，RangeSlider ✗)、Tab(✓)/TabStrip(✗)、SegmentedControl(✓)、Dropdown 触发器(✓)、ProgressBar(✗)、Badge/Tag(△ chip)、Divider(✗)、Skeleton(✗)。

**L2 容器与集合（collections）**
Container/Box、HorizontalBox、VerticalBox、GridBox、WrapBox、Overlay、ScrollBox、SizeBox、SplitView（容器类走 02 的 `UiLayoutStyle`，不需逐个 `.zui`，需要 descriptor 注册）；ListView/ListRow(✓ row)、TreeView/TreeRow(✓ row，树容器行为 △)、TableView/TableHeader/TableRow(✓ row，表容器 △)、PropertyRow(✓)、VectorRow(△ axis_value_field)、Toolbar(✗)、PanelGroup(✗)。

**L3 瞬态层（overlays & feedback）**
PopupMenu/MenuRow(✓ 多级已有路由)、ContextMenu(✗)、DropdownPopup(△)、Tooltip(✓)、Toast(✓)、Alert(✗)、Dialog/ConfirmDialog(✗)、CommandPalette(✗)、NotificationCenter(✗)、DragOverlay(✗，interface drag.rs 契约已有)。

**L4 工作台表面（surfaces，归 editor 资产，全用 L1–L3 组合）**
WindowChrome、TopToolbar、MainTabStrip、ActivityRail、DrawerSurface（left/right/bottom）、StatusBar、SceneTreePanel、InspectorPanel、AssetBrowserPanel、ConsolePanel、TimelinePanel、ViewportPanel、FloatingWindow、WelcomeSurface——shell 模板已有对应区域，M4 重组为纯组合。

每个组件的「完整」定义（Definition of Done，逐条对应检查手段）：

| # | DoD | 检查手段 |
|---|-----|---------|
| 1 | `.zui` 资产在白名单目录、过治理测试 | `zui_asset_governance` Rust 测试 + node 脚本 |
| 2 | descriptor 声明 props/slots/事件/状态 | descriptor/validation.rs 校验测试 |
| 3 | 布局只用计划 02 的 `UiLayoutStyle` 属性集 | 布局属性扫描测试（禁私有布局字段） |
| 4 | 视觉全状态过计划 04 的 selector 矩阵 | 状态矩阵快照（04 M3 设施） |
| 5 | 鼠标 + 键盘 + 焦点行为齐备（Tab 可达、Enter/Space 激活、Escape 关闭、方向键导航适用者） | family 行为测试（state_reducer 分表） |
| 6 | render extract 与 native painter 双路对拍 | 04 双路对拍测试 |
| 7 | focused test + showcase 陈列页可见 | showcase 模块 + 契约脚本 |

## 4. 关键行为定稿（挑大头）

- **Dropdown/PopupMenu**：anchor 定位 + 屏幕边界翻转/夹取（Overlay 容器）、外点关闭、Escape 关闭、方向键/Home/End/输入首字母导航、多级子菜单 hover 展开延时（01 timers）。
- **TextField**：计划 03 编辑链全量；NumberField 附加拖拽改值、步进、min/max/精度、表达式留接口。
- **TreeView**：虚拟化（02 M3）、展开状态、单/多选（Ctrl/Shift）、键盘展开折叠、行内重命名、拖拽重排（reply drag）。
- **TableView**：列定义、列宽拖拽、排序指示、行选择、虚拟化滚动。
- **Slider**：轨道点击跳值、拖拽 capture、键盘步进、双滑块范围。
- **Tooltip/Toast**：input manager 计时（01 M3）、Toast 队列与超时归 runtime 组件状态。

## 5. 接口与数据结构草案（以 Button 与 TreeView 为样板）

```rust
// state_reducer.rs 升级为 state_reducer/ 目录（mod.rs 薄声明 + 按 family 分表）
// zircon_runtime/src/ui/component/state_reducer/{mod.rs, button.rs, text_field.rs,
//   selection.rs, slider.rs, dropdown.rs, tree_view.rs, table_view.rs, overlay.rs}
pub fn apply_component_event(/* 现有签名保持 */);   // mod.rs 按 family 分派

// button.rs（简单样板）
pub fn reduce_button_event(
    state: &mut UiComponentState,          // 现有类型
    event: &UiComponentEvent,              // 现有类型
) -> UiButtonReduceOutcome;                // 新增：{ activated: bool, state_changed: bool }
// 转移表：PointerPress→pressed=true(+capture 经 reply)；PointerRelease(in)→activated；
//        KeyDown(Enter|Space)→activated；Disabled 由 interaction_gate 先拦截

// tree_view.rs（复杂样板）
pub struct UiTreeViewState {               // 新增：组件局部语义态
    pub expanded: BTreeSet<UiTreeRowId>,
    pub selection: UiTreeSelection,        // Single(id) | Multi(BTreeSet)
    pub anchor: Option<UiTreeRowId>,       // Shift 范围选择锚点
    pub editing: Option<UiTreeRowId>,      // 行内重命名
}
pub fn reduce_tree_view_event(
    tree: &mut UiTreeViewState, event: &UiComponentEvent,
) -> UiTreeViewReduceOutcome;              // 新增：{ commands: Vec<UiTreeViewCommand> }
pub enum UiTreeViewCommand {               // editor 经 route id 收到的语义事件载荷
    SelectionChanged, RowExpanded(UiTreeRowId), RowCollapsed(UiTreeRowId),
    RenameRequested(UiTreeRowId), RenameCommitted { row: UiTreeRowId, text: String },
    ReorderRequested { row: UiTreeRowId, target: UiTreeDropTarget },   // 经 reply begin_drag
}
// 键盘表：Up/Down 移焦、Left/Right 折叠/展开、Enter 激活、F2 重命名、
//        Ctrl+Click 增选、Shift+Click 范围、Escape 取消重命名

// Button 四件套落点（每组件同构）：
//   .zui       primitives/inputs/workbench_button.zui（已有）
//   descriptor catalog/material_foundation/button_inputs.rs（已有，补 props/slots/事件全量声明）
//   reducer    state_reducer/button.rs（新增分表）
//   测试       state_reducer/button.rs 内 #[cfg(test)] + showcase 陈列
```

## 6. 模块与文件落点

**新增**：`state_reducer/` 目录分表（§5 文件清单）；缺失 `.zui` 原语（L1：label、icon、progress_bar、divider、skeleton、number_field、search_input、range_slider、tab_strip 进 primitives/inputs|data；L3：context_menu、dropdown_popup、alert、dialog、command_palette、notification_center、drag_overlay 进 primitives/feedback）；L2 容器 descriptor 注册（catalog/material_foundation/layout.rs 扩展）

**修改**：catalog/material_foundation 各 family 文件（descriptor 全量声明）、`zui_asset_governance` 白名单（新增资产登记）、showcase 模块工作区

**删除（硬切换义务）**：editor pointer bridge / template bindings 中残存的组件行为逻辑（随 01 M5 与各 family 切片同变更删除）；catalog 中与统一布局属性集重复的私有布局字段（02 M1.S4 协同）。

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | L1 盘点矩阵：16 个原子按 7 条 DoD 打分，差距清单落 `docs/zircon_runtime/ui/components.md` | 盘点文档 | 评审 | 无删除 |
| M1.S2 | Button/IconButton 四件套补满（衔接 04 M4 selector 切换），作为同构样板 | state_reducer/button.rs 等 | `cargo test -p zircon_runtime --lib state_reducer --locked` | template bindings 内按钮行为删除 |
| M1.S3 | 缺失 `.zui` 原语补齐（9 个，含治理登记） | primitives/ 各目录 | `cargo test -p zircon_editor --lib zui_asset_governance --locked` | 无删除 |
| M1.S4 | 键盘/焦点行为补齐：Tab 可达、Enter/Space、Escape、方向键（按 family 分表落 reducer） | state_reducer/ 各文件 | `cargo test -p zircon_runtime --lib state_reducer --locked` | 无删除 |
| M1.S5 | L1 全量状态矩阵 + showcase 陈列实机 | showcase | 实机 + 契约脚本 | 无删除 |
| M2.S1 | ListView/ListRow 接 02 M3 虚拟化 | list 容器路径 | `cargo test -p zircon_runtime --lib list --locked` | 无删除 |
| M2.S2 | TreeView 全行为（§4 键盘表 + 多选 + 重命名 + reply drag 重排） | state_reducer/tree_view.rs | `cargo test -p zircon_runtime --lib tree_view --locked` | editor hierarchy bridge 行为删除（与 01 M5.S3 协同） |
| M2.S3 | TableView：列宽拖拽、排序指示、行选择、虚拟滚动 | state_reducer/table_view.rs | `cargo test -p zircon_runtime --lib table_view --locked` | 无删除 |
| M2.S4 | PropertyRow/VectorRow + ScrollBox/SplitView/Toolbar/PanelGroup descriptor 注册 | catalog layout 族 | `cargo test -p zircon_runtime --lib catalog --locked` | 无删除 |
| M3.S1 | PopupMenu/ContextMenu/DropdownPopup：anchor 定位、边界翻转、键盘导航、多级延时 | state_reducer/dropdown.rs、overlay.rs | `cargo test -p zircon_runtime --lib popup --locked` | 无删除 |
| M3.S2 | Tooltip/Toast：计时接 01 M3，Toast 队列归 runtime 状态 | overlay.rs | `cargo test -p zircon_runtime --lib toast --locked` | 无删除 |
| M3.S3 | Dialog/Alert/ConfirmDialog + CommandPalette 骨架（命令源接 08 M4） | feedback `.zui` + reducer | `cargo test -p zircon_runtime --lib dialog --locked` | 无删除 |
| M3.S4 | DragOverlay：拖拽影子 + drop 指示（interface drag.rs 契约） | overlay.rs | `cargo test -p zircon_runtime --lib drag --locked` | 无删除 |
| M4.S1 | L4 表面盘点：shell 模板内联重复 L1–L3 结构清单 | 盘点 | 评审 | 无删除 |
| M4.S2 | L4 重组为纯组合（治理测试把关「不得内联重复」） | workbench/shell `.zui` | `cargo test -p zircon_editor --lib zui_asset_governance --locked` | 内联结构删除 |
| M4.S3 | shell 实机验收（结构不变、组合化，重组前后结构快照零 diff） | 实机 | editor 实机 | 无删除 |
| M5.S1 | showcase 模块对齐 component-prototype 的 Component Lab 结构 | showcase 工作区 | 契约脚本 | 无删除 |
| M5.S2 | 截图陈列 + 全组件打勾表更新 | 文档 | `verify-native-component-contract.mjs` | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`button_enter_space_activates`、`button_disabled_blocks_via_interaction_gate`、`toggle_checked_state_round_trips`、`tab_focus_order_follows_declaration`
- **M2**：`tree_view_shift_click_selects_range`、`tree_view_f2_begins_rename_escape_cancels`、`tree_view_drag_reorder_emits_command`、`table_column_resize_persists_width`、`virtual_tree_1k_rows_renders_window_only`
- **M3**：`dropdown_popup_flips_at_screen_edge`、`menu_first_letter_navigation`、`submenu_opens_after_hover_delay`、`toast_queue_expires_in_order`、`dialog_escape_dismisses_confirm_requires_action`
- **M4**：`l4_surfaces_contain_no_inline_primitive_structures`（治理）
- **M5**：showcase 契约脚本全绿

落点：runtime `state_reducer/` 各文件 `#[cfg(test)]`；治理测试沿 `zui_asset_governance` 既有位置。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| 16+ 组件 × 7 DoD 工程量大、容易烂尾在「部分完整」 | M1.S1 盘点先行，打勾表进文档；每 family 切片整列交付，不留半行为组件 |
| 行为收编与 01 M5 时序耦合（bridge 删早了行为缺失） | family 切片与对应 bridge 删除同变更；先 reducer 后删 bridge |
| TreeView/Table 虚拟化与选中/展开状态交互复杂 | 语义态全量、渲染窗口化分离；1k 行测试守恒 |
| L4 重组动 shell 模板，影响 08 切换基线 | M4 在 08 M2 之前完成；重组前后 shell 结构快照零 diff |
| CommandPalette 依赖 08 M4 命令注册表 | M3.S3 只做骨架（输入+列表+过滤），命令源后接 |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 01 M3（reply）、02 M1（布局属性）、04 M4（Button selector）、05 M1（资产 DoD） | 06 M2–M5、08 M2 |
| M2 | 06 M1、02 M3（虚拟化）、03 M4（行内重命名编辑链） | 09 M1（Hierarchy/AssetBrowser） |
| M3 | 06 M1、01 M2（popup 路由）、01 M3（计时） | 08 M4（CommandPalette）、08 M6（toast/context menu） |
| M4 | 06 M2、06 M3 | 08 M2/M3（shell 区域用 L4 组合） |
| M5 | 06 M4 | 09 M5（结构对齐审查参照） |

## 11. 完成定义

- L1–L3 打勾表全✓（7 条 DoD 全过）；L4 表面零内联重复结构。
- 实机 showcase：全组件可见、可聚焦、可键盘操作；多级菜单/拖拽/Toast 队列行为正确。
- 验收命令组：`cargo test -p zircon_runtime --lib --locked`（state_reducer/catalog/popup/tree_view/table_view 过滤）、`cargo test -p zircon_editor --lib zui_asset_governance --locked`、`node .../verify-native-component-contract.mjs`、showcase 实机。

## 12. 边界约束

- 组件行为状态机归 runtime state reducer；editor 只接收语义事件（route id），不实现组件行为。
- 不追求 MUI 全集（DatePicker/Charts 等 mui_x 高级件按需后置）；以编辑器设计图实际用到的集合为版图。
- L4 表面资产在 `zircon_editor/assets`，但其实现不得内联重复 L1–L3 结构（治理测试把关）。
- L2 容器类不造 `.zui` 壳——容器即 `UiLayoutStyle` 声明 + descriptor 注册，避免无意义资产文件。

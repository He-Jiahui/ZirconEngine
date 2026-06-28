---
related_code:
  - zircon_runtime_interface/src/ui/event_ui.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_editor/src/ui/retained_host/host_contract
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/20-style-cascade-and-computed-style.md
status: planned
---
# 19 焦点与导航模型(焦点作用域 / Tab 顺序 / 方向导航 / 焦点环)

## 1. 目标

把"哪个节点持有键盘焦点、Tab/方向键/手柄如何在节点间移动焦点、焦点何时可视"沉淀为一份**焦点与导航规范**。当前代码仅有一个 `navigation_dispatcher` 参数名(`editor_ui/01`),**没有焦点作用域、Tab 顺序、方向导航几何求解、焦点环**——这是 UI 成熟度的明显短板(键盘可达性、手柄导航、模态焦点陷阱全缺)。本计划定语义契约,运行时派发归 `editor_ui/01`。

> 工程化硬目标(接 `index` §4.0):一个工程化编辑器 UI 必须**全键盘可达**、焦点可见、模态可陷焦。这不是可选项。

## 2. 现状(按代码核实)

- `editor_ui/01` 路由参数里有 `navigation_dispatcher`,但无 `EUINavigation` 式方向枚举、无 `FNavigationReply` 式边界规则、无方向几何求解、无焦点作用域/Tab 链。
- `event_ui.rs` 无 `focusable`/`tab-index`/焦点态字段;焦点态目前只在个别 retained 控件局部表达。
- 18 产出 hover/press,但**focus 是独立维度**(键盘焦点 ≠ 指针 hover),需本计划补。

## 3. 设计

### 3.1 可聚焦性与 Tab 顺序

每节点声明可聚焦性,对标 UE `SupportsKeyboardFocus`(`SWidget.h`)、Unity `focusable`+`tabIndex`、Godot `FOCUS_NONE/CLICK/ALL`(`control.h:66-71`)、CSS `tabindex`:

| 声明 | 语义 | 对标 |
| --- | --- | --- |
| `focus: none` | 不可聚焦 | Godot `FOCUS_NONE`;Unity `focusable=false` |
| `focus: click` | 仅指针可聚焦,不进 Tab 链 | Godot `FOCUS_CLICK` |
| `focus: all`(默认可交互件) | 指针 + Tab/方向可聚焦 | Godot `FOCUS_ALL`;Unity `focusable=true` |
| `tab-index: i` | Tab 链顺序权重 | CSS `tabindex`;Unity `tabIndex` |

Tab/Shift+Tab = `Next`/`Previous`,沿 Tab 链遍历(对标 UE `EUINavigation::Next/Previous` `SlateEnums.h:107/108`、Slint `default_next_in_local_focus_chain` `item_focus.rs:29`、Godot `find_next_valid_focus` `control.cpp:2943`)。默认链 = 树前序遍历可聚焦节点,`tab-index` 覆盖默认序。

### 3.2 方向导航(方向键 / 手柄)

方向移动 = `Left/Right/Up/Down`(对标 UE `EUINavigation` `SlateEnums.h:101-104`、Unity `NavigationMoveEvent.Direction`),求"当前焦点几何沿方向最近的可聚焦节点":

- 几何求解:沿方向投射,按几何打分取最近可聚焦节点,对标 UE `FHittestGrid::FindNextFocusableWidget`(`HittestGrid.h:72`)、Godot `_get_focus_neighbor` 距离搜索(`control.cpp:3171`)。
- 显式邻居覆盖:节点可声明四向显式目标(对标 Godot `focus_neighbor[4]` `control.h:287`、UE `FNavigationMetaData::SetNavigationExplicit`),优先于几何求解。
- 来源:键盘/手柄(对标 UE `ENavigationGenesis::Keyboard/Controller`),手柄模拟量映射方向(UE `GetNavigationDirectionFromAnalog`)。

### 3.3 导航边界规则(治"焦点跑出模态")

到达容器边界时的行为,对标 UE `FNavigationReply`/`EUINavigationRule`(`NavigationReply.h:14`):

| 规则 | 语义 | 对标 |
| --- | --- | --- |
| `escape`(默认) | 逃出容器,系统找下一个可聚焦 | UE `Escape`(`NavigationReply.h:17`) |
| `wrap` | 环绕到对侧(列表/网格循环) | UE `Wrap`(`NavigationReply.h:24`) |
| `stop` | 停在边界 | UE `Stop`(`NavigationReply.h:26`) |
| `explicit(target)` | 跳到显式目标 | UE `Explicit`(`NavigationReply.h:19`) |
| `trap`(模态) | 焦点陷在作用域内(对话框/弹层) | UE `Stop` + 焦点作用域;Unity 模态 panel |

### 3.4 焦点作用域(focus scope)

规范:浮层/对话框/抽屉是**焦点作用域**;打开模态作用域 → 焦点进入并 `trap`,关闭 → 焦点还原到打开前节点(对标 UE `HasUserFocusedDescendants` `SWidget.h`、Unity panel focus、DOM focus-trap)。作用域用 `focus-within`(=子孙持焦)状态,喂 20 的样式。

### 3.5 焦点动作:Accept / Back

`Accept`(Enter/Space/手柄 A)= 激活当前焦点(≈ 点击);`Back`(Esc/手柄 B)= 关闭作用域/取消,对标 UE `EUINavigationAction::Accept/Back`(`SlateEnums.h:126/129`)、Unity `NavigationSubmitEvent`/`NavigationCancelEvent`。键映射可配(对标 UE `FNavigationConfig`)。

### 3.6 焦点环(focus-visible,治"无焦点反馈")

规范:焦点态产出 `:focus` 与 `:focus-visible` 两态——`:focus-visible` 仅在**键盘/方向**导致聚焦时为真(指针点击聚焦不显示焦点环),对标 CSS `:focus-visible`、UE `OnQueryShowFocus(EFocusCause)`(由 `EFocusCause::Navigation` vs `Mouse` 区分,`Events.h`)、Unity USS `:focus`。焦点环视觉走 20 样式(accent 1-2px outline,遵 STYLE-NOTES,不发光)。

### 3.7 与 18/20/11 衔接

18 命中 → 指针聚焦(`EFocusCause::Mouse`);19 Tab/方向 → 键盘聚焦(`Navigation`);焦点态 + `focus-within` 喂 20 伪状态;焦点变更走事件→命令(单向受控,11),view 不直接 set 焦点视觉。可达性(a11y)名称/角色挂节点,对标 Bevy `bevy_a11y`、Unity a11y,留接口。

## 4. 接口与数据结构草案(Rust)

```rust
pub enum UiFocusMode { None, Click, All }
pub enum UiNavDirection { Left, Right, Up, Down, Next, Previous }
pub enum UiNavBoundary { Escape, Wrap, Stop, Explicit(UiNodeId), Trap }
pub enum UiFocusCause { Pointer, Navigation, Programmatic, Restore }

pub struct UiFocusable { pub mode: UiFocusMode, pub tab_index: i32,
    pub neighbors: [Option<UiNodeId>; 4], pub boundary: UiNavBoundary }

// 方向导航几何求解(单源)
pub fn find_next_focus(arranged: &ArrangedTree, from: UiNodeId, dir: UiNavDirection) -> Option<UiNodeId>;
// Tab 链
pub fn focus_chain(tree: &UiTree) -> Vec<UiNodeId>;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增(契约) | `docs/ui-and-layout/focus-navigation-contract.md` | 可聚焦/Tab/方向/边界/作用域/焦点环 |
| DTO | `event_ui.rs` | `UiFocusable`/方向/边界/cause + `:focus(-within/-visible)` 态 |
| 运行时 | `editor_ui/01` navigation dispatcher | Tab 链 + 方向几何求解 + 焦点作用域 + 还原(不在本计划) |

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | focusable/tab-index/边界 DTO + Tab 链遍历 + focus 态 | `cargo test -p zircon_runtime_interface --lib focus_nav --locked` |
| S2 | 方向导航几何求解(显式邻居 + 几何打分) | `cargo test -p zircon_editor --lib directional_nav --locked` |
| S3 | 焦点作用域 trap/还原 + focus-visible 焦点环(接 20) | `cargo test -p zircon_editor --lib focus_scope --locked` |

## 7. 测试矩阵

- Tab/Shift+Tab 沿链前后移动;`tab-index` 覆盖默认序;`focus:none` 跳过。
- 方向键取几何最近可聚焦;显式邻居优先于几何。
- 边界:`wrap` 环绕、`stop` 停、`escape` 逃出、`trap` 陷焦。
- 模态作用域打开陷焦、关闭还原到原节点。
- `:focus-visible` 仅键盘/方向聚焦为真,指针点击不显示焦点环。
- 手柄方向/Accept/Back 等价键盘。

## 8. 风险与对策

- 风险:方向几何打分边界 case(重叠/不规则布局)。对策:S2 打分函数 + 大量几何用例测试,显式邻居兜底。
- 风险:焦点还原栈与多浮层叠加。对策:S3 作用域栈 + 还原目标弱引用(节点销毁安全)。

## 9. 完成定义

可聚焦性/Tab 顺序/方向导航/边界规则/焦点作用域/焦点环成文且落 DTO;方向几何求解单源;模态陷焦与还原成立;`:focus-visible` 区分键盘/指针;全键盘可达验收通过。

## 10. 边界约束

运行时导航派发归 `editor_ui/01`;指针命中/捕获归 18;伪状态样式(焦点环视觉)归 20;布局几何归 13;缩放遵 16;单向受控遵 11;a11y 细化另立。

## 11. 参考实现对照(dev/ 源码锚点,已核实)

- **Unreal**:`SlateCore/.../Types/SlateEnums.h:98`(EUINavigation)`:101-104`(Left/Right/Up/Down)`:107/108`(Next/Previous)`:123`(EUINavigationAction)`:126/129`(Accept/Back);`Input/NavigationReply.h:14`(EUINavigationRule)`:17/19/24/26`(Escape/Explicit/Wrap/Stop);`Input/HittestGrid.h:72`(FindNextFocusableWidget);`Widgets/SWidget.h`(SupportsKeyboardFocus/OnNavigation/OnQueryShowFocus);`Types/NavigationMetaData.h`(SetNavigationExplicit/Wrap/Stop)。
- **Unity UI Toolkit**(`dev/ui-toolkit-manual-code-examples` + UIElements docs):`FocusController`、`Focusable.focusable`/`tabIndex`、`NavigationMoveEvent`(Left/Right/Up/Down/Next/Previous)、`NavigationSubmitEvent`/`NavigationCancelEvent`、USS `:focus`;`create-a-tabbed-menu-for-runtime` 示例可作 Tab 导航验证参照;`slide-toggle/SlideToggle.uss:54`(`.slide-toggle:focus` 焦点态选择器)。
- **Godot**:`scene/gui/control.h:66-71`(FOCUS_NONE/CLICK/ALL)`:287`(focus_neighbor[4]);`control.cpp:2943`(find_next_valid_focus)`:3171`(_get_focus_neighbor)`:3291`(find_valid_focus_neighbor)。
- **Slint**:`internal/core/item_focus.rs:29-38`(next focus chain)`:51-60`(previous);`input.rs:904/907`(FocusAccepted/Ignored)`:914-918`(FocusIn/FocusOut + FocusReason)。
- **Bevy**:`bevy_ui` `AutoDirectionalNavigation`(auto_directional_navigation.rs:34-49)、`bevy_input_focus` `DirectionalNavigation`。

## 12. 状态与产出记录

planned。后续项:S1 focusable/tab-index/边界 DTO + Tab 链 + focus 态。

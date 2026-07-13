---
related_code:
  - zircon_runtime_interface/src/ui/event_ui/mod.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_editor/src/ui/retained_host/host_contract
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_layout/12-widget-slot-componentization.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md
status: planned
---
# 18 输入响应与命中测试模型(事件相位 / pointer-events / 指针捕获 / 拖拽)

## 1. 目标

把"指针/键盘/触摸事件如何命中节点、沿树传播、被消费、被捕获"沉淀为一份**声明式输入响应规范**,使编辑器界面的交互像 CSS/DOM 与 Unity UI Toolkit 一样**可声明、可预测、单源**,而不是由 11 个手写 pointer bridge 各自实现命中与状态机(`editor_ui/01` §2.2.5 现状)。本计划定**输入响应的语义契约**(相位、命中、捕获、pointer-events、拖拽阈值、cursor),运行时派发实现归 `editor_ui/01`(Slate 式内核)。

> 工程化硬目标(接 `index` §4.0):交互不在 Rust 壳代码里逐控件写命中矩形 + 手写 hover/press 状态机;而是节点声明 `pointer-events`/`focusable`/`cursor`,由统一命中测试 + 相位派发驱动。手写命中是"简单实现"反模式,本规范要消除它。

## 2. 现状(按代码核实)

- `editor_ui/01` 已有事件归一化(对应 `GenericApplicationMessageHandler`)、路由策略枚举、`FReply` 式回复,但:**路由次序未单点固化**(capture→popup→preview→direct→bubble→focus-path 散在 dispatch.rs/route_policy.rs,`editor_ui/01` §2.2.4);**11 个 pointer bridge 自带命中/hover/press**(`editor_ui/01` §2.2.5);**无统一命中测试单源**(无 hit-test grid / 命中树)。
- `event_ui.rs` 有 `UiNodeId` 与事件 DTO,但缺**节点级 `pointer-events`/命中可见性**位与**相位(capture/target/bubble)**契约。
- 触摸/多指针、双击/tooltip 计时、指针捕获无统一 owner(`editor_ui/01` §2.2.2/2.2.3)。

## 3. 设计

### 3.1 命中测试单源(治"手写命中")

规范:**命中测试只有一处实现**,输入 = 已排布树(13 Taffy 几何)+ 节点命中可见性,输出 = 命中路径(从最深命中节点到根)。任何 pointer bridge 不得再自带命中矩形。

- 命中路径 = `Vec<UiNodeId>`(deepest→root),对标 UE `FHittestGrid::GetBubblePath()`(`HittestGrid.h:37`)与 DOM `composedPath()`、Unity UI Toolkit `panel.Pick()`。
- 命中加速可用网格分桶(对标 `FHittestGrid::AddWidget` `HittestGrid.h:82`),但 MVP 可直接逆 `draw_order` 命中(对标 Godot `Viewport::_gui_find_control_at_pos` `viewport.cpp:1853`、Bevy `UiStack` 逆序)。
- 命中复用渲染的 z/layer 序(接 21):上层先命中。
- **裁剪与滚动感知(2026-07-02 评审收口)**:hit_test 输入包含**裁剪栈**(与 `21` `UiClipStack` 同语义)与**滚动偏移**——被 `overflow:hidden` 裁掉的子节点区域**不命中**(命中区 = 节点 frame ∩ 祖先裁剪交集);滚动容器内子节点命中坐标先加滚动偏移再比较。测试补 `clipped_child_not_hit`、`scrolled_content_hit_at_offset`。
- **坐标空间(2026-07-02 评审收口,对应 `16` §3.4-4)**:hit_test 输入为**逻辑坐标**;物理→逻辑换算(`÷ scale_factor`)在**输入边界一次完成**,命中与后续派发全程逻辑空间。

### 3.2 pointer-events / 命中可见性(节点位)

每个节点声明命中行为,对标 CSS `pointer-events`、UE `EVisibility`、Unity `picking-mode`、Godot `mouse_filter`:

| 声明值 | 语义 | 对标 |
| --- | --- | --- |
| `auto`(默认) | 自身+子孙都可命中 | CSS `pointer-events:auto`;UE `Visible`(`Visibility.h:14`);Unity `picking-mode: Position` |
| `none` | 自身+子孙都不命中(事件穿透) | CSS `pointer-events:none`;UE `HitTestInvisible`(`Visibility.h:23`);Unity `picking-mode: Ignore`;Godot `MOUSE_FILTER_IGNORE`(`control.h:90-92`) |
| `self-none` | 自身不命中、子孙可命中 | UE `SelfHitTestInvisible`(`Visibility.h:26`) |
| `pass`(容器) | 自身不消费、向父冒泡(透传) | Godot `MOUSE_FILTER_PASS`;Bevy `FocusPolicy::Pass`(`focus.rs:109-114`) |

与显示态分离:`display:none`(布局不占位+不命中,对标 UE `Collapsed` `Visibility.h:17`)、`visibility:hidden`(占位+不命中,UE `Hidden` `Visibility.h:20`)归 13/布局,不在本表。

### 3.3 事件相位:capture → target → bubble(治"路由次序未固化")

规范:指针/键盘事件沿命中路径走**三相**,单点权威实现,对标 DOM、Unity UI Toolkit `TrickleDown`/`BubbleUp`、Slint `InputEventFilterResult`/`InputEventResult`:

```
事件 →（沿命中路径 root→deepest）CAPTURE / TrickleDown  ——父可预拦截/过滤
      →（deepest 节点）           TARGET                 ——目标处理
      →（deepest→root）           BUBBLE / BubbleUp      ——未消费则冒泡，父接管
```

- 每节点处理返回 `Reply`:`Handled`(停传,对标 `FReply::Handled` `Reply.h:233`、Slint `EventAccepted` `input.rs:211`、DOM `stopPropagation`)或 `Unhandled`(继传,Slint `EventIgnored` `input.rs:214`)。
- 父预拦截:对标 Slint `InputEventFilterResult::Intercept`(`input.rs:239`)/`ForwardAndIgnore`(`input.rs:233`)、UE `FEventRouter::FTunnelPolicy`(SlateApplication.cpp 路由)。
- 全链权威次序(单实现,封 `editor_ui/01`):`capture(grab) → popup/overlay → preview → 命中路径 capture → target → bubble → focus-path(键盘)`。外点关闭(popup dismiss)在此次序内判定。

### 3.3a 滚轮与滚动事件(2026-07-02 评审收口)

wheel 是编辑器最高频输入之一(滚动区/树/日志/可缩放视口),此前未进相位模型,在此补齐:

- **wheel 进相位模型**:wheel 事件沿命中路径走同一 capture → target → bubble 三相;默认消费者=命中路径上**最近的可滚动祖先**(`overflow: scroll` 且对应轴有可滚余量)。
- **按轴部分消费**:消费按 x/y 轴独立判定——内层横向可滚但纵向到底时,wheel 的 y 分量继续冒泡给外层滚动容器,x 分量被内层消费。
- **修饰键升级**:`Ctrl+wheel`(缩放)不走滚动默认消费,由声明了缩放能力的视口组件在 target 阶段显式消费;未消费则冒泡(不触发滚动)。
- **滚动只触发 paint/提取,不触发 relayout**:滚动偏移不进 taffy 求解(遵 `13` §3.7-4 滚动线 owner 注记);滚动偏移参与命中换算见 §3.1。
- 滚动条视觉规范归 `20`(伪状态)+ `15`(组件);虚拟化契约归 `editor_ui/02` M3。

### 3.3b 双击/tooltip 计时与多指针(2026-07-02 评审收口)

§2 点名的"双击/tooltip 计时无统一 owner"在此定归属:

- **click-count 判定**:单 owner = `ui/dispatch/input_manager` 的 timers(已在码);双击=同节点同键在阈值时间与阈值距离内的连续 press,阈值 token 化(时间/距离走 01 design token,不硬编码)。三击(整行选择)同机制。
- **tooltip 计时**:hover 驻留计时归同一 timers owner;进入节点起表、离开/press 取消、显示后跟随命中路径;延时 token 化。
- **多指针实例表**:登记为后续扩展(触摸/笔),V1 仅单指针,不阻塞本计划切片。

### 3.4 指针捕获(grab)

规范:节点可在 target/bubble 阶段请求**捕获指针**,后续指针事件直送捕获者直至释放,对标 UE `FReply::CaptureMouse`(`Reply.h:28-32`)/DOM `setPointerCapture`/Slint `GrabMouse`(`input.rs:216`)/Unity `MouseCaptureController`。捕获路径用弱引用(对标 UE `FWeakWidgetPath`),节点销毁自动释放。拖动分隔条、滑块、画布平移必须走捕获,不得轮询全局指针。

### 3.5 拖拽阈值与拖放(治散落 tab_drag/drawer_resize)

规范:`press → 超过阈值 → drag-start → drag-move(每帧)→ drop`,统一拖拽机,对标 UE `DetectDrag`→`OnDragDetected`→`BeginDragDrop`(`Reply.h:129`、`DragAndDrop.h`)、DOM dragstart/drop、Unity `PointerManipulator`/`DragAndDropUtility`。拖放载荷 = ABI 安全 DTO(对标 `DataTransfer`/UE `FDragDropOperation`),不传 Rust trait 对象。阈值是逻辑单位(× scale,接 16)。

### 3.6 cursor 声明

规范:节点声明 `cursor`(default/pointer/text/resize-ew/resize-ns/grab/…),命中时由命中路径最近的有声明者决定,对标 CSS `cursor`、UE `FCursorReply`/`OnQueryCursor`(`CursorReply.h`)、Unity USS `cursor`。`Unhandled` 时父决定(对标 `FCursorReply::Unhandled` `CursorReply.h:24`)。

### 3.7 与 Reply/状态的衔接

命中/相位/捕获产出的是**语义状态**(hover/press/active/focus-within),喂给 20 的样式系统(`:hover`/`:active` 伪状态)与 19 的焦点模型;输入层不直接改视觉(单向受控,接 11)。

**文本命中交接(2026-07-02 评审收口)**:本计划的命中单源只到**节点级**。命中进入文本节点后,二级字形级 hit-test **委托 runtime text**(cluster 反查光标位,`zircon_runtime/src/ui/text/hit_test.rs`,契约见 `docs/plans/zircon_runtime/text/03`,以 UE `GetGlyphAtOffset` 为样板):点击定位光标、拖拽选区、双击选词(UAX#29 word 边界)、三击整行由文本编辑链承接,本计划只保证把命中点(逻辑坐标、已扣除滚动偏移与节点原点)交给文本节点的 target 处理。

## 4. 接口与数据结构草案(Rust,规范形态)

```rust
// 节点命中位(进 event_ui 或 layout 节点)
pub enum UiPointerEvents { Auto, None, SelfNone, Pass }
pub enum UiCursor { Default, Pointer, Text, ResizeEw, ResizeNs, Grab, Grabbing, /* … */ }

// 命中测试单源:已排布树 → 命中路径(deepest→root)
pub fn hit_test(arranged: &ArrangedTree, point: UiPoint) -> Vec<UiNodeId>;

// 三相派发结果
pub enum UiEventPhase { Capture, Target, Bubble }
pub enum UiEventReply { Handled, Unhandled }
pub struct UiPointerCaptureRequest { pub node: UiNodeId }
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增(契约) | `docs/ui-and-layout/input-response-contract.md` | 命中单源 + 三相 + pointer-events + 捕获 + 拖拽 + cursor |
| 运行时实现 | `editor_ui/01` 的 dispatch/route owner(不在本计划) | 把全链次序固化为单实现 + 命中单源,迁出 11 个 bridge 的命中 |
| DTO | `zircon_runtime_interface/src/ui/event_ui/mod.rs` | 增 `UiPointerEvents`/`UiCursor`/相位/Reply 契约 |

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | pointer-events/cursor DTO + 命中单源契约 + 三相次序文档 | `cargo test -p zircon_runtime_interface --lib input_response --locked` |
| S2 | 11 个 pointer bridge 命中迁到命中单源(衔接 editor_ui/01) | `cargo test -p zircon_editor --lib --locked` |
| S3 | 指针捕获 + 拖拽阈值统一机(收编 tab_drag/drawer_resize) | `cargo test -p zircon_editor --lib drag_capture --locked` |

## 7. 测试矩阵

- `pointer-events:none` 节点事件穿透到下层;`self-none` 自身不命中子孙命中。
- capture→target→bubble 次序正确;`Handled` 停传、`Unhandled` 冒泡。
- 父 `Intercept` 预拦截阻止子收事件。
- 指针捕获后移出节点仍收事件;捕获者销毁自动释放。
- 拖拽:位移 < 阈值不触发 drag-start;阈值随 scale 变化(接 16)。
- cursor 由命中路径最近声明者决定。
- `clipped_child_not_hit`:被祖先 `overflow:hidden` 裁掉的子节点区域不命中(2026-07-02 评审收口)。
- `scrolled_content_hit_at_offset`:滚动容器内子节点按滚动偏移换算后正确命中(2026-07-02 评审收口)。
- wheel:最近可滚动祖先消费;内层纵向到底时 y 分量冒泡外层(按轴部分消费);`Ctrl+wheel` 不触发滚动、由缩放视口消费(2026-07-02 评审收口)。
- 双击:阈值时间/距离内同节点连续 press 判定 click-count=2;超阈值重新计数(2026-07-02 评审收口)。
- 文本节点命中:命中点交接到字形级 hit-test(runtime text),点击定位光标 offset 正确(2026-07-02 评审收口)。

## 8. 风险与对策

- 风险:命中单源改动波及 11 个 bridge 回归。对策:S2 逐 bridge 迁移 + 命中快照测试,保留行为等价。
- 风险:三相次序与现有 popup/preview 行为冲突。对策:S1 先把现状次序写成测试基线,再固化为单实现。

## 9. 完成定义

命中测试单源成文且 bridge 命中迁入;pointer-events/cursor 声明落 DTO;capture/target/bubble 三相次序单点固化;指针捕获与拖拽阈值统一;语义状态喂 19/20,不直接改视觉。

## 10. 边界约束

运行时事件内核(归一化/派发/Reply)归 `editor_ui/01`,本计划只定语义契约;焦点/导航归 19;伪状态样式归 20;布局几何(命中输入)归 13;阈值/缩放遵 16;单向受控遵 11。

## 11. 参考实现对照(dev/ 源码锚点,已核实)

- **Unreal**:`SlateCore/.../Input/HittestGrid.h:37`(GetBubblePath)`:72`(FindNextFocusableWidget)`:82`(AddWidget);`Input/Reply.h:28-32`(CaptureMouse)`:129-134`(DetectDrag)`:233-244`(Handled/Unhandled);`Layout/Visibility.h:14/17/20/23/26`(Visible/Collapsed/Hidden/HitTestInvisible/SelfHitTestInvisible);`Input/CursorReply.h:24/33`(Unhandled/Cursor);`Input/DragAndDrop.h`(FDragDropOperation)。
- **Unity UI Toolkit**(`dev/ui-toolkit-manual-code-examples` + docs.unity.cn UIElements):事件 `TrickleDown`/`BubbleUp` 三相;`picking-mode: Position/Ignore`(≈ pointer-events);`MouseCaptureController`(指针捕获);USS `cursor`;`PointerManipulator`/拖放。可作为后续验证参照。
- **Slint**:`internal/core/input.rs:211/214/216`(EventAccepted/Ignored/GrabMouse)`:226-247`(InputEventFilterResult:ForwardEvent/ForwardAndIgnore/Intercept/DelayForwarding)。
- **Godot**:`scene/gui/control.h:90-92`(MOUSE_FILTER_STOP/PASS/IGNORE);`scene/main/viewport.cpp:1853`(_gui_find_control_at_pos)`:1929`(_gui_input_event)。
- **Bevy**:`bevy_ui/src/focus.rs:52-61`(Interaction)`:86-92`(RelativeCursorPosition)`:109-114`(FocusPolicy Block/Pass)`:149-366`(ui_focus_system)。

## 12. 状态与产出记录

planned。后续项:S1 pointer-events/cursor DTO + 命中单源契约 + 三相次序文档。

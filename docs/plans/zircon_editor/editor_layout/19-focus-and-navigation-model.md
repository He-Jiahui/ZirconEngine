---
related_code:
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_editor/src/ui/retained_host/host_contract
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/20-style-cascade-and-computed-style.md
status: in_progress
---
# 19 焦点与导航模型(焦点作用域 / Tab 顺序 / 方向导航 / 焦点环)

## 1. 目标

把"哪个节点持有键盘焦点、Tab/方向键/手柄如何在节点间移动焦点、焦点何时可视"沉淀为一份**焦点与导航规范**。当前代码仅有一个 `navigation_dispatcher` 参数名(`editor_ui/01`),**没有焦点作用域、Tab 顺序、方向导航几何求解、焦点环**——这是 UI 成熟度的明显短板(键盘可达性、手柄导航、模态焦点陷阱全缺)。本计划定语义契约,运行时派发归 `editor_ui/01`。

> 工程化硬目标(接 `index` §4.0):一个工程化编辑器 UI 必须**全键盘可达**、焦点可见、模态可陷焦。这不是可选项。

## 2. 现状(按代码核实)

- runtime tree 已通过 `UiRuntimeTreeFocusExt::next_navigation_target` 实现 Tab、显式邻居、空间方向候选与 modal scope 过滤，相关契约由 interface 的 focus/navigation 模块持有；当前计划不再以“能力全缺”为前提。
- interface 已在 `ui/focus.rs` 落 `UiFocusContract`、`UiFocusMode`、`UiFocusCause`、`focus_chain`，在 `ui/navigation.rs` 落 `UiTabIndex`、`UiNavigationContract` 与 `UiNavigationBoundary`；runtime 已把 MUI overlay 与非 MUI `modal` navigation group 收敛到同一 focus scope 语义：子树继承 group、打开后 autofocus、Tab/方向/程序化焦点共用 trap 判定，关闭时先按节点 id、再按稳定 `UiNodePath` 还原。`UiComponentFlags` 现持有独立的 `focus_visible` 状态，并经运行时样式投影为 `:focus-visible`；仅导航原因置位，指针、程序化、autofocus、restore 和 a11y 聚焦均保持隐藏。编辑器端完整键盘/手柄可达性验收仍待完成。
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
- 导航键判定次序(2026-07-02 评审收口):导航键先作为普通键事件递交**焦点节点 target**处理,返回 `Handled` 则**不进入** Tab/方向导航求解;文本输入类控件默认消费 `Left/Right/Home/End/Enter`(caret 移动/换行),`Esc` 不消费、交还 `Back` 语义;Tab 在单行文本框默认跳格(不消费),多行编辑器可声明捕获 Tab(插入制表符)。字形级 caret 移动语义本身归 runtime text(见 §3.7),本条只定判定次序。
- scroll-into-view(2026-07-02 评审收口):导航目标集合**包含视口外可聚焦节点**(几何打分不因裁剪剔除候选);焦点落到视口外节点后,触发其**最近可滚动祖先**执行 scroll-into-view(滚动只触发 paint/提取,不触发 relayout,遵 18 滚动线裁决)。

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

补(2026-07-02 评审收口):Accept/Back 同样遵 §3.2 判定次序——焦点节点 target 先处理,文本输入类控件消费 `Enter` 时不触发 `Accept` 语义;`Esc` 一律不被文本控件消费,进入 `Back`(IME 组合期例外,见 §3.7)。

### 3.6 焦点环(focus-visible,治"无焦点反馈")

规范:焦点态产出 `:focus` 与 `:focus-visible` 两态——`:focus-visible` 仅在**键盘/方向**导致聚焦时为真(指针点击聚焦不显示焦点环),对标 CSS `:focus-visible`、UE `OnQueryShowFocus(EFocusCause)`(由 `EFocusCause::Navigation` vs `Mouse` 区分,`Events.h`)、Unity USS `:focus`。焦点环视觉走 20 样式(accent 1-2px outline,遵 STYLE-NOTES,不发光)。

### 3.7 与 18/20/11 衔接

18 命中 → 指针聚焦(`EFocusCause::Mouse`);19 Tab/方向 → 键盘聚焦(`Navigation`);焦点态 + `focus-within` 喂 20 伪状态;焦点变更走事件→命令(单向受控,11),view 不直接 set 焦点视觉。可达性(a11y)名称/角色挂节点,对标 Bevy `bevy_a11y`、Unity a11y,留接口。

IME 收尾交接条款(2026-07-02 评审收口):字形级 hit-test、caret/选区、IME 组合的实现契约权威归 `docs/plans/zircon_runtime/text/`(03/08),本文只持交接条款——(a)焦点**离开可编辑节点**时(Tab/指针/程序化任一 cause),必须先 commit 当前 preedit 再完成焦点转移(commit 语义归 runtime text/08);(b)popup 抢焦点期间若源可编辑节点仍有组合中文本,`Esc` 次序 = **先取消组合、再关 popup**(即组合中 `Esc` 被 IME 收尾消费,不进入 §3.5 `Back`)。

## 4. 接口与数据结构草案(Rust)

```rust
pub enum UiFocusMode { None, Click, All }
pub enum UiFocusCause { Pointer, Navigation, Programmatic, Restore }
pub struct UiFocusContract { pub focusable: bool, pub mode: UiFocusMode, /* … */ }

pub enum UiNavigationBoundary { Escape, Wrap, Stop, Explicit(UiNodeId), Trap }
pub struct UiNavigationContract {
    pub tab_index: Option<UiTabIndex>,
    pub group: Option<UiNavigationGroup>,
    pub directional: Option<UiDirectionalNavigation>,
    pub boundary: UiNavigationBoundary,
}

// 当前 runtime 单源（zircon_runtime/src/ui/tree/node/focus.rs）
pub trait UiRuntimeTreeFocusExt {
    fn next_navigation_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError>;
}
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增(契约) | `docs/ui-and-layout/focus-navigation-contract.md` | 可聚焦/Tab/方向/边界/作用域/焦点环 |
| DTO | `zircon_runtime_interface/src/ui/focus.rs`、`ui/navigation.rs` | `UiFocusContract`/方向/`UiNavigationBoundary`/cause + `:focus(-within/-visible)` 态 |
| 运行时 | `zircon_runtime/src/ui/tree/node/focus.rs` 与 `editor_ui/01` navigation dispatcher | 维护 Tab 链、方向几何求解和 modal scope；补齐焦点还原与编辑器接线 |

## 6. 里程碑切片化

| # | 切片 | 验证命令 |
| -- | --- | --- |
| S1 | focusable/tab-index/边界 DTO + Tab 链遍历 + focus 态 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime_interface -SkipBuild -LibTests -TestFilter focus_nav` |
| S2 | 方向导航几何求解(显式邻居 + 几何打分) | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter directional_nav` |
| S3 | 复核/推广焦点作用域 trap/还原 + focus-visible 焦点环(接 20)，覆盖非 MUI 通用 scope 与编辑器组件 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter focus_scope` |

## 7. 测试矩阵

- Tab/Shift+Tab 沿链前后移动;`tab-index` 覆盖默认序;`focus:none` 跳过。
- 方向键取几何最近可聚焦;显式邻居优先于几何。
- 边界:`wrap` 环绕、`stop` 停、`escape` 逃出、`trap` 陷焦。
- 模态作用域打开陷焦、关闭还原到原节点。
- `:focus-visible` 仅键盘/方向聚焦为真,指针点击不显示焦点环。
- 手柄方向/Accept/Back 等价键盘。
- (2026-07-02 评审收口)文本输入类控件消费 Left/Right/Home/End/Enter 后不触发导航;单行 Tab 跳格、多行捕获 Tab;焦点落视口外节点触发最近可滚动祖先 scroll-into-view。
- (2026-07-02 评审收口)抽屉折叠(15e tier 驱动)时焦点还原到对应 rail 图标(作用域还原路径,回挂 15e §2.3);还原逻辑标识解析失败回退作用域首个可聚焦。

## 8. 风险与对策

- 风险:方向几何打分边界 case(重叠/不规则布局)。对策:S2 打分函数 + 大量几何用例测试,显式邻居兜底。
- 风险:焦点还原栈与多浮层叠加。对策:S3 作用域栈 + 还原目标弱引用(节点销毁安全)。
- 补(2026-07-02 评审收口):还原目标键改用**稳定逻辑标识**(节点名字/路径),不以 `UiNodeId` 为持久键——重建后 id 失效;还原时按逻辑标识重解析,解析失败(节点已销毁/改名)则回退到**作用域内首个可聚焦节点**,不静默丢焦。

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

in_progress。S1 的 focus/tab-index/边界 DTO、Tab 链与 S2 的显式/空间方向求解已有当前源码 owner；S3 的非 MUI 通用 scope、继承式 modal group、稳定路径还原、全局 z-order trap 与任意关闭顺序下的恢复链重接已完成实现。2026-08-10 已补独立 `focus_visible` 组件态，并将其投影为通用 `:focus-visible` 伪状态：导航置位，指针、程序化、autofocus、restore 和 a11y 聚焦隐藏；`UiPointerDispatchEffect` 同批硬切为不携带 visibility 的焦点请求，防止指针路径重新越过该规则。随后已将该状态接入 painter selector 与 retained-host 按钮：`selected`/`checked` 保留活动表面，不再冒充 `Focused` 或获得 focus outline；显式 `focus_visible` 才解析为焦点态。二次独立复审发现的两项 P1 已前向修复：命令按钮的旧 selection/checked focus-outline 断言已改为活动表面断言；V2/component-state 的 `focusVisible` 驼峰别名现统一投影为 `:focus` 与 `:focus-visible`，并有同一原子状态命中两个伪选择器的集成覆盖。shared `UiSurface` 的 runtime focus state 已进一步投影到 host model 与 workbench `TemplatePaneNodeData`：`focus_visible_known` 把 pointer/programmatic/导航等运行时原因与静态作者态分开，已知隐藏焦点只保留语义 focus，不再让 native workbench painter 画 focus outline；静态组件展示继续保留其作者态。直接合成、未从 shared surface 得到原因的 retained 节点仍按原兼容映射处理，故它们仍是 P2 后续项，不能据此宣称所有 native painter 都有完整原因区分。静态格式检查通过。Windows 托管编译/聚焦测试仍在 Cargo 启动前被 coordinator 以 `unmanaged_artifacts_detected` 拒绝，外部路径为 `E:\ZirconBuilds\mvp-supplemental-20260810-184017`，因此不记录测试通过。编辑器键盘/手柄可达性与截图验收也仍待完成，因此不宣称本计划完成。

2026-08-10 复审追加的 retained-host P1 已前向修复：主题选择器生成的 `ButtonInteractionState::Focused` 不再覆盖 `focus_visible_known=true` 的 runtime 决策。已覆盖程序化焦点的隐藏环、导航焦点的可见环、焦点切换后旧节点清理，以及静态 `focused=true` 模板在 full/surface-backed projection 中保持 unknown 作者态。同阶段已删除 TextField painter 按 `WorkbenchInputFocused` control id 强制绘制焦点环的特例，改为统一来源状态；未知作者态保持兼容的 `Focused`，已知隐藏 runtime 焦点解析为 `Normal`，独立复审 P0-P2 为 0。上述测试和改动已通过 `rustfmt --check` 与 scoped `git diff --check`；Cargo 与截图验收仍为待验证状态。

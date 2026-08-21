---
title: Runtime UI Component Catalog、Widget Behavior、State Reducer、Interaction Semantics、Accessibility 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime75
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/ui/component
  - zircon_runtime_interface/src/ui/widget.rs
  - zircon_runtime/src/ui/component
  - zircon_runtime/src/ui/v2
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/accessibility
  - zircon_editor/src/ui/component_registry
  - zircon_editor/src/ui/asset_editor/palette
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/template_runtime/component_adapter
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives
  - zircon_editor/assets/ui
  - examples/woc/assets/ui
tests:
  - zircon_runtime/src/ui/tests/component_catalog
  - zircon_runtime/src/ui/tests/accessibility_widget_actions.rs
  - zircon_runtime/src/ui/tests/widget_menu_behavior.rs
  - zircon_runtime/src/ui/tests/widget_radio_behavior.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/zircon_editor/editor_ui/06/failure-2026-07-18-runtime-ui-component-catalog-deep-clone.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/Button.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Components/Button.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Blueprint/UserWidgetPool.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Components/ListView.h
  - dev/bevy/crates/bevy_ui_widgets/src/button.rs
  - dev/bevy/crates/bevy_ui_widgets/src/checkbox.rs
  - dev/bevy/crates/bevy_ui_widgets/src/slider.rs
  - dev/bevy/crates/bevy_ui_widgets/src/text_input.rs
  - dev/Fyrox/fyrox-ui/src/button.rs
  - dev/Fyrox/fyrox-ui/src/check_box.rs
  - dev/Fyrox/fyrox-ui/src/list_view.rs
  - dev/Fyrox/fyrox-ui/src/tree.rs
  - dev/godot/scene/gui/base_button.cpp
  - dev/godot/scene/gui/line_edit.cpp
  - dev/godot/scene/gui/tree.cpp
  - dev/godot/tests/scene/test_button.cpp
  - dev/godot/tests/scene/test_option_button.cpp
  - dev/godot/tests/scene/test_tab_bar.cpp
  - dev/godot/tests/scene/test_text_edit.cpp
  - dev/godot/tests/scene/test_tree.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 75 · Runtime UI Component Catalog、Widget Behavior、State Reducer、Interaction Semantics、Accessibility 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前并非没有组件系统。公开接口已经定义 `UiComponentDescriptor`、prop/state/slot/event schema、host/render capability、fallback policy 与 `UiWidgetContract`；Runtime 内有 69 项 editor-showcase 目录、211 项 Material 目录、20 余个 state reducer 模块、独立 retained `UiSurface` 默认交互、渲染提取和 AccessKit 投影；Editor 与 WOC 也已经形成真实产品资产。应保留这些类型、测试和局部算法，不应回退到临时 hard-code widget。

真正的工程级缺口是这些底座没有形成一条权威执行链。当前物理源码存在至少五份组件事实：69 项 showcase、211 项 Material、Editor 将两者后注册覆盖后的 merged registry、Editor-local builtin/retained component kind，以及 `UiWidgetBehavior::infer_from_component`、v2 `infer_interaction`、renderer 与 accessibility 各自维护的字符串分类表。v2 compiler 只拒绝空 component id；找不到 prototype 的任意字符串都会继续作为 native component。随后 builder 将 descriptor 的 prop/state/capability/fallback/a11y/widget contract 全部绕开，`a11y` 与 `widget` 被写成默认值。

同时，目录下 20 余个 reducer 和 107 个 catalog tests 并不是产品控件的执行引擎。生产侧 `apply_component_event` 只接入 Editor showcase demo；真实 `UiSurface` 走另一套 `UiWidgetBehavior + mutate_tree_property + default_interactions`。surface 发出的 component event effect 只检查 node 存在便回报 `delivered: true`，不会调用 reducer。live mutation 对除少量 tree 内建字段外的任意属性和值类型都直接写入 TOML metadata；即使直接调用 reducer，未知 prop 也会被接受。这使 catalog schema 目前更接近“编辑器清单与测试样例”，而不是可执行 ABI。

本轮登记 **6 项 Runtime75 独有 P0、48 项 P1、12 项 P2 与 48 项资格门**。Runtime11A继续拥有总体 tree/layout/input/focus/OS accessibility host、通用 virtualization 与产品 tick；Runtime73拥有 style/theme/cascade/pseudo-state/transition；Runtime74拥有 template/binding/model/event command 与 hot-reload generation；Editor01/23拥有 retained host 和 UI authoring workflow。Runtime75只拥有“component id 解析到唯一 descriptor、typed behavior、state transaction、interaction、render 与 component-specific accessibility”的 vertical contract。

在单一 component authority、v2 admission、reducer/live-surface convergence、schema-safe mutation、controlled/uncontrolled transaction、per-component accessibility、产品资产迁移和真实 conformance/scale evidence 全部通过前，不得把当前目录中的 258 个唯一 id 称为 258 个已工程化控件，也不得宣称 UI 组件性能或表现达到或超过当前 Unreal。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime75 责任 | 不重复登记 |
|---|---|---|---|
| UI tree/layout/input/focus/a11y host | Runtime11A | 每个 component 的行为、role/state/action/relationship 投影及命中资格来源 | tree、layout、focus manager、AccessKit host、通用导航与通用 virtualization |
| Style/theme/transition | Runtime73 | descriptor/state transaction 产生 typed pseudo-state 与 invalidation | selector、cascade、token、theme、transition runner |
| Template/binding/model/reload | Runtime74 | 控件 controlled proposal、external commit 与 component state migration hook | expression、target executor、model provider、command、整体 reload transaction |
| Resource/dynamic session | Runtime43/64 | component artifact id、generation 与 capability admission 的消费点 | 通用资源 handle/cache/lease 与 gameplay session 生命周期 |
| Editor retained host/authoring | Editor01/23 | Editor 消费同一 component authority，不再维护覆盖式 merge 和本地行为表 | palette UX、asset editing、undo/save/cook、native window host |
| Style-specific painter | Editor01/Runtime73 | painter 必须由 resolved component implementation handle 驱动 | Material visual design 与 renderer 通用架构 |

本篇不把 Runtime11A 已登记的布尔 input dispatch、产品 tick 或总体 a11y schema 再算一次 P0；也不把 Runtime74 的 binding executor、hot reload tree replacement 再算一次。Runtime75只登记即使这些父层能力补齐，当前 component catalog/behavior 实现仍会失败的差距。

### 2.2 Zircon 物理冻结

本轮核心冻结 447 个 production/cross-product Rust 文件、61,461 行、2,218,453 bytes，manifest fingerprint 为 `91c12aac743c24ca5586c33a077cb418f15a5d6605856513850868b3354bb018`；聚焦 69 个测试文件、17,374 行、619,616 bytes，fingerprint 为 `1edbb0d2f1bfee550a35ac1287fe43fa6ad6fed124c7134acd689e6e391aaccd`。算法为对排序后的 `relative-path=per-file-SHA-256` 以 LF 连接、末尾不附加 LF，再做 SHA-256。结论绑定当前共享 working copy，而不只绑定 baseline HEAD；并行 Session 已使物理文件数高于 Runtime74 时的快照，因此实施前必须重取指纹。

| 范围 | 文件 / 行 / bytes | fingerprint / 本轮证据 |
|---|---:|---|
| Public component/widget contracts | 26 / 2,004 / 68,249 | `bacb562fc5574533a65bd6a0a9a610493594e7d9ab1d04264cea4981466cf79a`；descriptor、schema、event、state、fallback、widget contract |
| Runtime catalog/reducer/v2/surface/a11y/render | 189 / 45,363 / 1,639,983 | `34e305da2da7713d71ac72cd0860f97718743e60c6e3595f88c8bcea4397322c`；两套 catalog、reducers、v2 admission、live behavior、mutation、render 与 a11y |
| Editor registry/palette/adapter/material painter | 232 / 14,094 / 510,221 | `2bf367658c5788b33b3f805f62c5eba6986df40cc50a891822028787678ee526`；merged registry、showcase-only palette、local builtin、retained kind 与 painter 分流 |
| 聚焦测试 | 69 / 17,374 / 619,616 | `1edbb0d2f1bfee550a35ac1287fe43fa6ad6fed124c7134acd689e6e391aaccd`；107 catalog tests 与 widget/a11y live-surface tests |
| Editor 与 WOC 产品 `.zui` | 282 / 46,657 / 3,131,228 | `0d529fa03d5d866b36101afd3f5ae477c80a3a649da9b76915b1bfdf346d4ba7`；267 Editor + 15 WOC 资产 |

目录与产品统计按当前物理源码重新计算：

| 事实 | 当前值 | 结论 |
|---|---:|---|
| editor-showcase descriptor | 69 | `UiDocumentCompiler::default`、v2 interaction inference 与 Editor palette 仍主要依赖它 |
| Material descriptor | 211 | 其中包含 web utility、MUI/MUI X 与 Editor shell 类声明 |
| 两目录重合 / 并集 | 22 / 258 | 同 id 在 Editor merge 中由 Material 后注册覆盖，没有显式冲突政策 |
| `UiWidgetBehavior` string alias | 42 | 其中 22 命中 showcase、21 命中 Material、11 项只存在于 Material |
| catalog reducer tests | 52 files / 107 tests | 23 个文件直接执行 206 次 `.apply_event(...)`，但不是 live surface 路径 |
| live widget behavior tests | 28 files / 179 tests | 测试另一套 `UiWidgetBehavior/default_interactions`，没有证明 descriptor/reducer 合流 |
| Editor `.zui` | 267 files / 4,684 refs / 351 unique refs | 246 definitions、244 unique definitions；144 unique refs 命中 Runtime catalog，198 命中全局 custom definition |
| Editor catalog 外 component | 9 | `ActivityRail/DocumentHost/DocumentTabs/PaneSurface/StatusBar/UiHostToolbar/UiHostWindow` 属于 retained adapter；`InspectorSurfaceControls/SceneViewportToolbar` 属于 Editor-local builtin |
| WOC `.zui` | 15 files / 498 refs / 12 unique refs | 12 项都在 258 项并集中，但仍未证明每项走同一 live behavior contract |

9 个 Editor catalog 外 id 并非简单“找不到实现”；它们证明 owner 分裂：七项由 `RetainedUiHostComponentKind::from_component` 识别，两项由 Editor builtin descriptor 安装。报告要求把这些 owner 变成显式 namespace/provider，而不是把它们粗暴塞进 Runtime Material 清单。

本轮只做 review，不修改 production/tests/assets，不运行 Cargo、Editor、WOC、真实窗口、fault、soak 或 benchmark。共享工作树有其他 Session 修改；后续实施必须先重验 source、测试和产品指纹，再按 owner 取得租约。

### 2.3 参考物理冻结

参考侧冻结 21 个文件、25,395 行、1,097,043 bytes，manifest fingerprint 为 `bd196e639e2ab70f1f63a6aaa470159feaa2f6ebdc40cb052db7effc6cd28886`。

| 参考 | 采用的工程事实 | 对 Zircon 的约束 | 不外推内容 |
|---|---|---|---|
| Unreal UMG/Slate | `UButton::RebuildWidget/SynchronizeProperties`绑定 click/press/release/hover/focus，区分 click/touch/press method并提供 accessibility widget；ListView/UserWidgetPool有 entry generation、refresh、active/inactive pooling | component 必须有真实 implementation、同步点、输入模态、生命周期、池化和 a11y，不是只有 descriptor 清单 | 不复制 UObject、Blueprint、Slate attribute 宏或 Unreal 全部 widget 数量 |
| Fyrox UI | Button/CheckBox/ListView/Tree 各自实现 `Control::handle_routed_message`，消息有 destination/direction/handled，builder/message/selection 语义和 widget 本体在同一模块闭合 | descriptor、typed message、state transition 与实际 control 必须是一条路径 | 不复制其单线程模型或现有视觉体系 |
| Bevy UI Widgets | Button/Checkbox/Slider/TextInput 使用 typed component/observer；required accessibility role 与 widget 一起声明；Checkbox 发 `ValueChange<bool>`，另有可选 `checkbox_self_update`；测试覆盖 disabled/focus/click | controlled proposal 与 self-update 必须分离；a11y、disabled、pointer/keyboard 与状态必须由同一 typed widget contract 驱动 | 不要求 Zircon 采用 ECS 存储布局 |
| Godot Control | BaseButton 将 GUI input、shortcut、action mode、button mask、toggle/group signal 与 accessibility update 合流；LineEdit 集成 IME、virtual keyboard、selection、undo/redo、clipboard/menu；Tree 有长期成熟的数据/选择/编辑语义和专项测试 | 基础控件要覆盖完整输入生命周期、平台文本服务、状态与可访问性，而不是只实现 click/value happy path | 不复制 SceneTree/ClassDB、Godot theme API 或全部 legacy 行为 |
| Unity Graphics | 本地仓库只有 SRP `DebugUI.Widget/IContainer/Value/Button`，有 flags、query path、getter/setter/validation 与 runtime/editor context | 可借鉴 value validation、container identity 与 runtime/editor可见性合同 | 该仓库不是 Unity UI Toolkit 源码，不能作为通用 Unity widget parity 证据 |

## 3. 当前可保留底座

| 底座 | 当前价值 | 重构要求 |
|---|---|---|
| `UiComponentDescriptor` 与 schema 类型 | 已经有 prop/state/slot/event、host/render capability 和 fallback 的统一承载位置 | 升级为 versioned executable contract，compiler、surface、render、a11y 全部通过 resolved handle 消费 |
| 两个 process-wide shared registry | `OnceLock` 避免每次查询重新构建，registry 有 deterministic id order 与 revision | 只保留一个 authority/view graph；owned clone 和覆盖式 merge 退出热路径 |
| 组件 reducer 模块 | Button、menu、numeric、slider、table、tree、toast、command palette 等状态算法已有较多测试 | 接入 live surface transaction，不再只服务 showcase/direct unit test |
| `UiWidgetContract` 与 default interactions | live surface 已覆盖 button/toggle/radio/range/scrollbar/text/menu/popup 的一部分 pointer/keyboard 语义 | behavior 必须由 descriptor implementation id 解析，不再靠 component string alias |
| `mutate_tree_property` 与 dirty report | 已有局部更新、dirty domain 与 binding report 入口 | 变成 schema-aware transaction，拒绝未知/错型/read-only prop并支持 rollback |
| v2 prototype expansion/tree build | 产品 `.zui` 已经能展开 component definition并构建 retained tree | 在实例化 native component 前执行 descriptor admission/default/state/widget/a11y projection |
| AccessKit extract/action | 已有 role/state/action snapshot 与 platform adapter | per-component contract 负责 role、required relation、value range、actions；heuristic 只作 legacy diagnostic |
| Editor material painter 与产品资产 | 已证明复杂 Editor 工作台可由 `.zui` 驱动并进行视觉测试 | painter 绑定 implementation handle/capability，不再成为另一份 component classifier |

## 4. 六项新增 P0

### RUW-P0-001 · Component authority 被拆成多套 registry 与本地 classifier，同一 id 没有唯一语义

showcase 69 项、Material 211 项只重合 22 项。Runtime old compiler、v2 interaction、Editor palette 分别固定读取 showcase；Editor retained registry先 clone showcase，再遍历 owned Material clone并以 `register` 替换同 id。`register`的公开语义就是“register or replace”，没有 provider、namespace、priority、conflict diagnostic 或 descriptor compatibility check。Editor-local builtin 与 retained component kind又在 registry 外增加两套事实。

影响：同一 `.zui` 在 palette、old compiler、v2 surface、Editor retained host 中可获得不同 category/events/default/schema/behavior；cache key 和 artifact 也无法证明编译时、实例化时、执行时使用同一代 descriptor。必须建立单一 `UiComponentAuthority`：component id 解析到 provider-qualified/versioned descriptor 与 implementation handle；重复 id默认 fail close，只有显式 override policy 可替换并产生 receipt。

关闭门：同一 artifact 在 Runtime、Editor、WOC 的 component resolution hash一致；重复 id、provider unload、generation change与 override 都有 deterministic diagnostic；palette、compiler、surface、render、a11y 查询同一 authority snapshot。

### RUW-P0-002 · v2 产品 compiler 绕过 descriptor schema/capability/fallback，任意非空字符串都可伪装 native component

`UiV2DocumentCompiler`只检查 component 非空、root/graph/cycle/repeat等结构；没有 prototype 时直接保留为 native node。`build_tree_from_arena`复制 attributes/layout后把 `a11y` 与 `widget`写成 default，不应用 descriptor default prop/state schema、slot/event contract、required host/render capability 或 widget fallback。`required_render_capabilities`在 production 只被 descriptor validation 读取；host capability只用于 registry/palette query；component fallback policy没有 runtime consumer。

影响：资产可通过 compile/cook并在缺少 renderer、virtualization、text input 或 host feature 的环境中静默退化；拼错 component id也会成为 generic native node。必须让 v2 compile/instantiate执行 provider resolution、schema/default normalization、capability negotiation和明确 fallback；未知 id、缺 capability 与 fallback 结果都进入 artifact/receipt。

关闭门：unknown、missing provider、missing host/render capability、unsupported fallback 都有确定结果；`Reject/Placeholder/Omit/DisableInteractions`在 Runtime 与 Editor 有一致测试；native component 不能靠任意字符串隐式创建。

### RUW-P0-003 · 目录 reducer 与真实 `UiSurface` 控件执行链断开，206 次 reducer 测试调用不能证明产品行为

`apply_component_event`的生产调用只存在于 Editor showcase adapter/state。真实 surface 使用独立的 `UiWidgetBehavior`、`default_interactions`、editable-text/table/tree专用分支和 `mutate_tree_property`。input effect 的 `EmitComponentEvent`只 `require_node`，随后无条件生成 `delivered: true` report，不调用 descriptor reducer，也不验证 event 是否属于该 component。catalog tests直接在 `UiComponentState` 上 `.apply_event(...)`；live widget tests测试另一套 surface行为。

影响：Button/Slider/Tree/Table/CommandPalette/NotificationCenter/Toast等可能在 direct reducer test里正确，在产品 surface中完全不执行、执行不同算法或产生不同 state/event。必须把 reducer改造成 live component implementation 的纯 transition core，由 pointer/keyboard/a11y/programmatic action共同进入同一 transaction；showcase不再拥有特殊执行路径。

关闭门：每个支持的 widget event 从真实 v2 asset经 hit-test/keyboard/a11y 进入同一 reducer；state、event、dirty、binding receipt一致；没有 reducer consumer的 catalog component不能标记为 behavior supported。

### RUW-P0-004 · Input eligibility、widget behavior、special interaction、render 与 a11y 使用互不一致的字符串分类器

v2 `infer_interaction`只查询 showcase registry；`UiWidgetBehavior::infer_from_component`另有 42 个 alias，其中 11 个只在 Material；table/tree、popup、render dirty、text field painter和accessibility又各有自己的 component/role匹配。Material-only `Slider/Switch/Menu/ContextMenu/DropdownPopup/RadioGroup/Scrollbar/SearchField`可被 behavior classifier识别，却可能因 v2 showcase inference没有 authored binding而从未成为输入目标。`ButtonGroup`被硬映射成 `RadioGroup`，`ToggleButtonGroup`反而没有 live behavior。

影响：控件能画但不能点、能点但无 reducer、能更新但错误 dirty、能交互但报错 role/action；添加别名或重命名时没有编译期完整性检查。必须让 descriptor解析出 typed `ComponentImplementationId`，由 implementation一次性声明 hit-test、behavior、render adapter、a11y adapter 与 dirty mapping；禁止下游再按字符串猜种类。

关闭门：生成式 conformance test遍历所有 resolved component，证明 input/behavior/render/a11y handler集合相容；所有 string classifier 收敛或只保留 migration diagnostic；未知 implementation fail close。

### RUW-P0-005 · Live property mutation与 reducer 都允许 schema 外写入，descriptor无法保护状态不变量

`mutate_tree_property`只对 visibility/enabled/visible/clickable/hoverable/focusable/pressed/checked/input_policy等少量字段做类型检查；其余 property把任意 `UiValue`转成 TOML后直接插入 `metadata.attributes`。dirty domain再由独立 component/property string table推断。即使直接走 `apply_component_event`，`apply_value`在 `descriptor.prop(property)`为 `None` 时仍直接 `set_value`并返回成功。

影响：拼错 prop、错类型、越界、read-only/state-only字段与恶意大值都可能进入 retained state，renderer/a11y/binding看到不同解释；transaction没有 prepare/rollback，部分更新可留下破坏性中间态。必须由 resolved descriptor产生 typed property handle、access mode、normalizer、validation、dirty impact与transaction policy；未知或不允许的写入默认拒绝。

关闭门：所有 pointer/keyboard/a11y/binding/programmatic mutation走同一 schema transaction；unknown/wrong-kind/out-of-range/read-only写入无副作用；多字段 reducer原子提交或回滚并生成old/new/dirty receipt。

### RUW-P0-006 · Per-component accessibility contract在 v2 build时丢失，最终靠 behavior与字符串启发式猜 role/action

v2 node的 `a11y` 与 `widget`都初始化为 default。accessibility extract先看非Generic authored role，否则按 `UiWidgetBehavior`和另一份字符串表推断；无法识别但 interactive 的组件被宣布为 Button，非interactive则是 Generic。descriptor自身的 role/category/state/event、required relation与range语义没有进入a11y构建。Scrollbar behavior甚至在 behavior-to-role阶段返回 None，依赖后续其他条件补救。

影响：Tabs、DataGrid、Tree、combobox、range、menu、dialog、tooltip、复合 label/description关系可能被宣布为Generic或Button，并暴露错误 action/state；可视行为与辅助技术行为会分叉。必须让 component implementation提供 typed a11y contract，并由同一 state transaction更新checked/selected/expanded/value/range/disabled/relations。

关闭门：每个交互组件都有显式 role/state/action/relationship schema；键盘、pointer、AccessKit action产生相同 reducer transition；unsupported或缺 label/relation在compile/CI fail close，而不是运行时猜 Button。

## 5. P1 工程化重构清单

### 5.1 Authority、descriptor 与 artifact

| ID | 差距 | 必须收敛到的合同 |
|---|---|---|
| RUW-P1-001 | `register`默认静默替换同 id descriptor | provider-qualified id、冲突诊断、显式override policy与兼容性检查 |
| RUW-P1-002 | `editor_showcase()`与`material_editor_foundation()`仍深 clone 完整 registry | immutable authority snapshot/Arc view；owned clone仅允许离线编辑副本 |
| RUW-P1-003 | registry `revision`只是本地饱和计数，不进入artifact/cache/runtime handle | stable schema hash、provider generation与artifact dependency |
| RUW-P1-004 | Editor palette/native slots只读showcase，retained host读merged registry | palette、compiler、preview与host共享同一capability-filtered snapshot |
| RUW-P1-005 | Editor-local builtin与retained kind在Runtime catalog外隐式存在 | 显式 `editor://` provider/namespace和可卸载lifetime |
| RUW-P1-006 | 9个产品host/local id没有统一resolution receipt | product asset compile记录provider、implementation、generation与fallback |
| RUW-P1-007 | Material shared defaults给大量无关组件统一注入hovered/pressed/selected/error/open等state | 每个implementation只声明真实拥有的state；复用通过typed trait fragment组合 |
| RUW-P1-008 | `NoSsr/UseMediaQuery/CssBaseline/Portal/ClickAwayListener/InitColorSchemeScript`等web utility以普通widget展示 | 明确分为compile utility、host service、layout adapter或真实widget；不伪装成同级component |
| RUW-P1-009 | host/render capability与fallback只参与query/validation | compile+instantiate+render执行同一capability negotiation与fallback receipt |
| RUW-P1-010 | descriptor没有implementation ABI/version | `ComponentImplementationId + AbiVersion + SchemaHash + ProviderGeneration` |
| RUW-P1-011 | category/role是宽泛enum/string，不能决定行为组合 | typed capability facets：pressable/checkable/selectable/range/editable/collection/popup |
| RUW-P1-012 | component event只有kind，没有per-component payload schema与routing policy | typed event id、payload schema、bubble/capture/default-action/cancel policy |

### 5.2 v2 实例化、状态与 mutation

| ID | 差距 | 必须收敛到的合同 |
|---|---|---|
| RUW-P1-013 | v2不应用descriptor default props | compile-time canonical default expansion，artifact中可审计 |
| RUW-P1-014 | `state_schema`默认值不实例化到live component state | generation-qualified component state block与deterministic initialization |
| RUW-P1-015 | slot schema在v2 prototype/native child结构上不强制 | slot name/cardinality/allowed-child/layout-role compile validation |
| RUW-P1-016 | event schema不校验asset authored event是否受组件支持 | typed event endpoint resolution，unknown event阻断compile |
| RUW-P1-017 | prop required/enum/range/type只在old compiler局部执行 | v2成为唯一schema normalizer，old path迁移后删除 |
| RUW-P1-018 | widget/a11y contract不能从asset/descriptor合成 | descriptor defaults + asset override + host policy的typed merge |
| RUW-P1-019 | 没有controlled/uncontrolled模式 | proposal event、optional self-update、external commit/reject与generation reconciliation |
| RUW-P1-020 | live surface先改本地state再发binding，没有authoritative external state协议 | prepare/propose/commit/rollback transaction与stale response拒绝 |
| RUW-P1-021 | disabled、readOnly、inert、focusable语义散落在不同模块 | 单一interaction gate并由component facet声明允许的programmatic/a11y action |
| RUW-P1-022 | state/property ownership不区分input prop、derived state、ephemeral state | read/write/derived/ephemeral/persisted access mode与migration policy |
| RUW-P1-023 | component state、tree flags、metadata attributes可能三份存储同一值 | canonical state cell与只读projection；禁止双向镜像无generation |
| RUW-P1-024 | dirty impact由独立string table猜测 | property descriptor预编译Layout/Render/A11y/Interaction/Schedule impact set |

### 5.3 基础交互控件

| ID | 差距 | 必须收敛到的合同 |
|---|---|---|
| RUW-P1-025 | Button只有通用Button behavior，没有click/touch/press activation method | pointer kind/button mask/press-vs-release/touch/keyboard/drag-cancel policy |
| RUW-P1-026 | Toggle/Checkbox缺少统一checked/indeterminate/group/state proposal合同 | typed check state、tri-state policy、controlled update与a11y checked state |
| RUW-P1-027 | `ButtonGroup`被当作`RadioGroup`，普通按钮组被错误赋予互斥语义 | visual grouping与selection grouping分离；membership由typed owner建立 |
| RUW-P1-028 | `ToggleButtonGroup`在catalog有声明但live behavior未识别 | single/multiple selection、roving focus、disabled item与controlled value |
| RUW-P1-029 | Tabs/TabList/TabPanel没有统一live reducer与selection/focus/activation policy | tab owner、selected id、manual/automatic activation、orientation、panel relation |
| RUW-P1-030 | Menu/MenuItem/ContextMenu/DropdownPopup依赖多份alias和metadata | typed menu owner/item/submenu、roving focus、typeahead、escape与outside-click transaction |
| RUW-P1-031 | Range/Slider live算法与catalog numeric/slider reducer分离 | min/max/step/orientation/inversion/rounding/drag capture统一transition |
| RUW-P1-032 | NumberField被分类为TextInput，numeric step/clamp reducer未接入live surface | text edit buffer与committed numeric value分离，locale parse/step/spin/a11y range统一 |
| RUW-P1-033 | Text input surface editor与descriptor reducer各自处理value/validation | 单一edit session，IME/composition/selection/undo/clipboard/validation/commit合同 |
| RUW-P1-034 | Scrollbar/track/thumb依赖字符串alias和metadata target id | typed scroll owner、axis、range、viewport/content extent、thumb drag/capture与a11y |
| RUW-P1-035 | Popup/Disclosure open属性通过`open/popup_open`别名同步 | typed open state、anchor/owner/focus restore/dismiss reason与modal scope |
| RUW-P1-036 | focus/navigation能力从category/event猜测 | component facet显式声明focus target、roving owner、navigation axis与activation |

### 5.4 Collection、复合控件与产品能力

| ID | 差距 | 必须收敛到的合同 |
|---|---|---|
| RUW-P1-037 | Table/DataGrid live行为读取TOML metadata数组/map和字符串column/row | typed row model/provider、stable row/column id、sort/filter/edit transaction与selection model |
| RUW-P1-038 | Tree live行为扫描surface tree/metadata，catalog tree reducer另有一套状态 | typed hierarchical data source、stable item id、lazy load、expand/edit/drag transaction |
| RUW-P1-039 | List/VirtualList capability声明不进入实例化，entry lifecycle不是组件合同 | provider-backed collection view、pool/recycle generation、selection与focus restoration |
| RUW-P1-040 | CommandPalette reducer只在showcase/direct test链生效 | query generation、async provider、ranking/cancellation、selection/commit与live popup合流 |
| RUW-P1-041 | NotificationCenter/Toast reducer与surface timer各走一套状态 | typed notification model、dedupe、priority、expiry/pause/dismiss/announcement transaction |
| RUW-P1-042 | Dialog/Modal/ConfirmDialog声明与live popup/focus scope没有单一owner | modal stack、return value、cancel/default action、focus trap/restore与a11y dialog relation |
| RUW-P1-043 | Portal/Popper/ClickAway/Transition等没有清晰host service implementation | overlay host service、anchor generation、clip/window transform、lifecycle/cancellation能力 |
| RUW-P1-044 | drag payload schema与live drop admission没有由descriptor统一 | typed payload id/schema、source/target capability、preview/commit/cancel与security budget |

### 5.5 Accessibility、性能与证据

| ID | 差距 | 必须收敛到的合同 |
|---|---|---|
| RUW-P1-045 | component-specific label/description/controls/owns/active-descendant关系不做schema验证 | compile-time relation target/type/cardinality validation与generation-safe handle |
| RUW-P1-046 | 107个catalog tests与179个live behavior tests没有同资产端到端交叉 | descriptor-driven conformance harness：compile→surface→input/a11y→state/event/render receipt |
| RUW-P1-047 | registry clone、string match、TOML conversion、tree scan没有component workload benchmark | 10/1k/100k widget instantiate/update/input/a11y/collection benchmark与allocation budget |
| RUW-P1-048 | component resolution/fallback/state transition没有统一diagnostic/telemetry | provider/id/generation/event/old-new/dirty/fallback/error的结构化receipt与采样策略 |

## 6. P2 收敛项

| ID | 问题 | 收敛方向 |
|---|---|---|
| RUW-P2-001 | `Checkbox/CheckBox`、`Scrollbar/ScrollBar`等alias散落 | canonical id + deprecated alias table + migration diagnostic |
| RUW-P2-002 | snake_case与camelCase prop成对重复进入schema | canonical property id与import alias，不双份持久化state |
| RUW-P2-003 | `UiComponentCategory`同时承担palette分组与行为推断 | presentation category与behavior facet分离 |
| RUW-P2-004 | `role`是自由字符串 | versioned semantic role id/enum与extension namespace |
| RUW-P2-005 | `revision.saturating_add(1)`在溢出后失去变化语义 | opaque generation/hash，禁止静默饱和 |
| RUW-P2-006 | `delivered: true`只表示node存在 | 拆分accepted/queued/applied/rejected/cancelled状态 |
| RUW-P2-007 | v2缺descriptor时input policy退到`Inherit`，没有unknown diagnostic | compile admission失败或显式placeholder/inert policy |
| RUW-P2-008 | fallback与resource fallback同名但语义不相同 | component capability fallback使用专用命名与receipt |
| RUW-P2-009 | Material web export inventory tests容易被误读为runtime parity | 测试名明确`catalog_inventory_only`并增加implementation parity test |
| RUW-P2-010 | showcase `Vec::with_capacity(70)`与精确69项清单分离维护 | 由静态descriptor table/codegen产生容量和inventory |
| RUW-P2-011 | default prop/state/class通过重复builder链构造 | immutable schema fragments与deterministic canonicalization |
| RUW-P2-012 | 多处component/role/property string match缺少集中审计 | CI生成classifier inventory，在最终迁移阶段归零生产字符串分类 |

## 7. 组件族差距矩阵

| 组件族 | Zircon当前真实路径 | 参考成熟能力 | Runtime75要求 |
|---|---|---|---|
| Button/IconButton | showcase descriptor + string behavior + surface default interaction；reducer另存 | Unreal/Godot区分按下/释放、鼠标mask、touch/shortcut/focus/a11y；Bevy覆盖press/release/drag cancel/disabled | typed pressable facet、activation policy、统一pointer/keyboard/a11y transition |
| Checkbox/Toggle | direct reducer与surface toggle各自写checked | Bevy显式`ValueChange`与可选self-update；Godot group/toggle signal与a11y同步 | controlled proposal、tri-state、group owner、same-path a11y |
| Radio/Group | component字符串与祖先metadata推断group；ButtonGroup误归类 | 成熟实现用显式group/selection owner和roving focus | stable membership、single selection transaction、orientation/navigation |
| Slider/NumberField | slider reducer、range live behavior、text-input NumberField三路 | Bevy Slider typed range/value/drag；Godot控件有完整step/range/action | 统一range facet、numeric editor session、rounding与external commit |
| TextField | surface editable text已有较多能力，但descriptor reducer/validation另一路 | Godot LineEdit覆盖IME、virtual keyboard、undo/redo、clipboard/menu/a11y；Bevy用typed TextInput state | 单一edit session与平台服务，schema只生成contract不复制算法 |
| Menu/Popup/Dialog | popup stack、menu default interaction、overlay reducer和Material descriptors分裂 | Unreal/Fyrox/Godot有明确owner/routing/focus lifecycle | typed overlay owner、submenu/modal/focus restore/dismiss transaction |
| List/Table/Tree | surface tree/metadata特化 + catalog reducer；capability只在palette过滤 | Unreal ListView池化entry；Fyrox ListView/Tree typed message/model；Godot Tree长期覆盖选择/编辑 | provider-backed data model、stable id、pool generation、selection/edit transaction |
| Command/Notification | reducer测试丰富但产品surface未调用 | 成熟引擎把async source、lifecycle与presentation state区分 | live implementation、cancellation/generation、timer/announcement统一 |
| A11y | v2 default a11y，extract靠字符串/behavior推断 | Unreal/Godot控件更新自身可访问state；Bevy widget required role与behavior同模块 | descriptor implementation生成role/state/action/relation，无启发式猜Button |

## 8. 目标架构

### 8.1 单一 Component Authority

```text
UiComponentProviderRegistry
  -> UiComponentAuthoritySnapshot(generation, schema_hash)
      -> resolve(ComponentQualifiedId, host_caps, render_caps)
          -> UiResolvedComponent
              descriptor_handle
              implementation_handle
              behavior_facets
              render_adapter
              accessibility_adapter
              fallback_decision
```

Runtime builtin、Material、Editor-local、plugin component都作为provider注册；id必须带稳定namespace或由import scope解析。snapshot immutable且可共享，compiler artifact记录provider generation/schema hash。Editor palette、v2 compiler、surface、renderer与a11y只能持有resolved handle，不能重新按字符串查另一个registry。

### 8.2 Component 实例与状态事务

```text
input / keyboard / a11y / binding / programmatic command
  -> UiComponentAction(typed id + payload + generation)
  -> implementation.reduce(readonly state, action)
  -> UiComponentTransition
       proposed property writes
       emitted typed events
       focus/popup/capture intents
       dirty impact
  -> schema validate + controlled policy
  -> atomic commit / external proposal / rollback
  -> render + a11y + binding receipts
```

`UiComponentState`只保留一份canonical typed state。tree flags、metadata、render extract与AccessKit snapshot都是同generation projection。controlled component默认只发proposal；只有uncontrolled或显式self-update策略才立即commit。外部commit必须携带generation/sequence，旧响应拒绝。

### 8.3 Behavior Facet 而非组件名 switch

首批facet建议为 `Pressable`、`Checkable`、`Selectable`、`RangeValue`、`EditableText`、`Scrollable`、`PopupOwner`、`CollectionView`、`HierarchicalView`、`DragSource/DropTarget`。facet是实现组合，不是公开继承树；每项同时提供action/event、state schema、input admission、dirty impact和a11y contract。具体组件如Button、Checkbox、Slider、Tabs、DataGrid只组合所需facet并添加自己的约束。

### 8.4 产品 provider 边界

Editor retained chrome的七个host component和两个Editor-local builtin进入 `editor://` provider；Material/MUI utility明确分类为widget、compile utility或host service；WOC只依赖runtime provider。plugin unload/reload必须先retire provider generation，再让旧surface完成quiescence或fallback，禁止悬空implementation handle。

## 9. 分层实施里程碑

### M0 · Authority 与 admission 硬收敛

建立provider/namespace/snapshot/schema hash；合并showcase/Material/Editor merge的查询路径；v2必须解析component并执行host/render/fallback admission。先提供只读兼容alias与diagnostic，随后删除生产字符串classifier入口。

### M1 · Live behavior 合流

将现有 reducer重构为纯transition core并接入 `UiSurface`。先迁移Button、Toggle/Checkbox、Radio、Range、TextInput、Popup/Menu；pointer、keyboard、a11y与programmatic action共享同一入口。showcase adapter只发送action，不直接调用另一套state runtime。

### M2 · Schema-safe state transaction

引入typed property handle、access mode、default/state initialization、normalizer、dirty impact与atomic transaction；禁止unknown prop写入。实现controlled/uncontrolled proposal/commit/reject、sequence/generation与rollback。

### M3 · Render 与 accessibility convergence

renderer和AccessKit从resolved implementation adapter读取，不按component字符串猜。补齐role/state/action/relation contract；移除interactive→Button兜底作为正常路径，仅保留legacy diagnostic。

### M4 · Collection 与复合控件

以typed provider/model迁移List/VirtualList/Table/DataGrid/Tree；补齐Tabs、Menu、Dialog、CommandPalette、Notification/Toast、Portal/Popper与drag/drop owner。entry pooling、stable id、selection/edit transaction必须有代际测试。

### M5 · 产品迁移与证据闭环

迁移267个Editor资产、15个WOC资产及Editor-local provider；运行component conformance矩阵、真实窗口、a11y、reload、fault、soak和10/1k/100k规模benchmark。完成后删除owned registry clone、覆盖式merge与残余字符串classifier。

## 10. 资格门

### 10.1 Authority 与编译门

| Gate | 必须证明 |
|---|---|
| RUW-GATE-001 | showcase、Material、Editor、plugin provider进入同一authority snapshot |
| RUW-GATE-002 | duplicate id默认失败并报告两个provider/source span |
| RUW-GATE-003 | 显式override验证ABI/schema兼容并记录priority/receipt |
| RUW-GATE-004 | artifact记录component qualified id、provider generation、schema hash |
| RUW-GATE-005 | palette、compiler、preview、surface、render、a11y解析hash一致 |
| RUW-GATE-006 | unknown component在无fallback时阻断compile/instantiate |
| RUW-GATE-007 | missing host capability覆盖Reject/Placeholder/Omit/DisableInteractions |
| RUW-GATE-008 | missing render capability不静默绘制generic控件 |
| RUW-GATE-009 | required/default/enum/range prop在v2 canonicalize |
| RUW-GATE-010 | state defaults只初始化一次且deterministic |
| RUW-GATE-011 | slot cardinality/allowed child/layout role在compile验证 |
| RUW-GATE-012 | unsupported event及payload mismatch阻断compile |

### 10.2 行为与状态门

| Gate | 必须证明 |
|---|---|
| RUW-GATE-013 | Button pointer/keyboard/a11y/programmatic action进入同一reducer |
| RUW-GATE-014 | press/release/cancel/drag-out/disabled序列状态和event一致 |
| RUW-GATE-015 | Checkbox controlled模式只proposal，不擅自写checked |
| RUW-GATE-016 | Checkbox uncontrolled/self-update模式原子提交并更新a11y |
| RUW-GATE-017 | RadioGroup stable membership与single-selection invariant成立 |
| RUW-GATE-018 | ButtonGroup不再隐式拥有RadioGroup语义 |
| RUW-GATE-019 | ToggleButtonGroup single/multiple模式与roving focus通过 |
| RUW-GATE-020 | Tabs manual/automatic activation及TabPanel relation通过 |
| RUW-GATE-021 | Slider drag/keyboard/a11y value在min/max/step上输出相同结果 |
| RUW-GATE-022 | NumberField edit buffer、parse、clamp、commit/reject可回滚 |
| RUW-GATE-023 | TextInput IME/selection/undo/clipboard/validation使用同一session |
| RUW-GATE-024 | Popup/Menu escape/outside-click/submenu/focus restore有单一owner |

### 10.3 Mutation、render 与 a11y 门

| Gate | 必须证明 |
|---|---|
| RUW-GATE-025 | unknown property写入被拒且state/tree/metadata均无变化 |
| RUW-GATE-026 | wrong kind、out-of-range、read-only/state-only写入fail close |
| RUW-GATE-027 | 多字段transition全部提交或全部rollback |
| RUW-GATE-028 | stale generation/sequence external commit被拒绝 |
| RUW-GATE-029 | 每个property的dirty impact由schema产生并覆盖全部domain |
| RUW-GATE-030 | component state只有一个canonical owner，无无代际镜像 |
| RUW-GATE-031 | resolved implementation决定renderer，不再按字符串分流 |
| RUW-GATE-032 | unsupported painter走声明fallback并生成receipt |
| RUW-GATE-033 | 每个交互component有显式a11y role/state/action合同 |
| RUW-GATE-034 | label/description/controls/owns/active-descendant关系验证通过 |
| RUW-GATE-035 | pointer、keyboard与AccessKit action产生同一state/event |
| RUW-GATE-036 | 正常产品路径不再使用interactive→Button启发式兜底 |

### 10.4 Collection、产品与规模门

| Gate | 必须证明 |
|---|---|
| RUW-GATE-037 | List/VirtualList用stable item id与generation-safe pooled entry |
| RUW-GATE-038 | Table/DataGrid typed row/column/sort/filter/edit transaction通过 |
| RUW-GATE-039 | Tree lazy load/expand/select/edit/drag在provider刷新后保持identity |
| RUW-GATE-040 | CommandPalette async query取消旧generation且commit唯一 |
| RUW-GATE-041 | Toast/Notification timer、pause、dismiss与a11y announce同源 |
| RUW-GATE-042 | Dialog modal stack、default/cancel action与focus restore通过 |
| RUW-GATE-043 | 267个Editor与15个WOC资产零隐式unknown native component |
| RUW-GATE-044 | Editor-local九项component全部通过显式provider解析 |
| RUW-GATE-045 | 258项catalog inventory逐项标记implemented/utility/unsupported，不虚报 |
| RUW-GATE-046 | conformance harness逐项执行compile→surface→input/a11y→receipt |
| RUW-GATE-047 | 10/1k/100k instantiate/update/input/a11y benchmark满足预算且无registry clone |
| RUW-GATE-048 | provider reload/unload、fault、soak证明无旧handle、重复event或state泄漏 |

## 11. 测试与证据缺口

当前测试数量不能直接折算为工程成熟度。107个catalog tests主要证明descriptor inventory、schema字段和direct reducer transition；179个live widget tests主要证明另一套surface string behavior。两类测试都值得保留，但必须由第三层conformance harness把它们接到同一真实v2 asset和resolved component implementation上。

最低测试分层应为：

1. descriptor/schema纯测试：唯一性、default、payload、slot、capability与ABI hash。
2. reducer transition table：给定typed state/action，验证transition、event、dirty与拒绝原因。
3. v2 artifact测试：component resolution、default expansion、capability/fallback、provider generation。
4. live surface测试：pointer、keyboard、a11y、programmatic action走同一implementation。
5. product fixture：真实Editor/WOC `.zui`，不是手写最小node。
6. collection model测试：refresh、reorder、pool reuse、selection/focus/edit identity。
7. platform测试：IME、clipboard、virtual keyboard、AccessKit、native window focus。
8. reload/fault/soak/benchmark：provider generation、state migration、cancellation、allocation和延迟分位。

当前没有资格证据包括：全258项implementation parity、产品component resolution receipt、controlled rejection、schema rollback、renderer capability fallback、component-specific a11y relationship、provider unload、100k widget benchmark、长时间collection churn和真实Windows IME/辅助技术验证。

## 12. Owner 与状态

| 工作项 | Owner | 状态 |
|---|---|---|
| component provider/authority/schema hash/admission | `zircon_runtime` + `zircon_runtime_interface` | pending |
| reducer/live surface/typed mutation convergence | `zircon_runtime` | pending |
| component-specific render/a11y adapter | `zircon_runtime`，Editor painter协作 | pending |
| Editor-local provider与palette/retained host迁移 | `zircon_editor` | pending |
| WOC与Editor产品资产迁移 | `zircon_app`/`zircon_editor`/示例owner | pending |
| conformance/fault/soak/benchmark evidence | Runtime owner + validation owner | pending |

历史 `failure-2026-07-18-runtime-ui-component-catalog-deep-clone.md` 只完成了部分shared accessor替换；owned `material_editor_foundation()`、owned `editor_showcase()`、old compiler持有owned registry和Editor merged registry仍在。该失败记录继续作为 RUW-P1-002 的继承证据，不另建重复failure。

本篇状态为 `review_complete / implementation_pending / source_recheck_required`。下一轮实现不得从“再加一个component字符串分支”开始，必须先完成 M0 authority/admission，再按 M1-M5逐层迁移并以48项资格门验收。

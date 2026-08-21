---
title: Runtime UI Accessibility、Semantic Tree、Name、Description、Relation、State、Action、Live Region、Platform Adapter 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime78
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime_interface/src/ui/event_ui
  - zircon_runtime_interface/src/ui/dispatch
  - zircon_runtime/src/ui/accessibility
  - zircon_runtime/src/ui/surface
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_app
  - zircon_editor
  - examples/woc
tests:
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs
  - zircon_runtime/src/ui/tests/accessibility_state_values.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/accessibility_widget_actions.rs
  - zircon_runtime/src/ui/tests/accesskit.rs
  - zircon_runtime/src/dynamic_api/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/Accessibility/GenericAccessibleInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/Accessibility/SlateAccessibleMessageHandler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/Accessibility/SlateAccessibleWidgetCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/Accessibility/SlateCoreAccessibleWidgets.cpp
  - dev/bevy/crates/bevy_a11y/src/lib.rs
  - dev/bevy/crates/bevy_ui/src/accessibility.rs
  - dev/bevy/crates/bevy_winit/src/accessibility.rs
  - dev/godot/servers/display/accessibility_server_enums.h
  - dev/godot/servers/display/accessibility_server.h
  - dev/godot/servers/display/accessibility_server.cpp
  - dev/godot/drivers/accesskit/accessibility_server_accesskit.h
  - dev/godot/drivers/accesskit/accessibility_server_accesskit.cpp
  - dev/Fyrox/fyrox-ui/src/lib.rs
  - dev/Fyrox/fyrox-ui/src/message.rs
  - dev/Fyrox/fyrox-ui/src/widget.rs
  - dev/Fyrox/fyrox-ui/src/text_box.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.Fields.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.Containers.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.UIState.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 78 · Runtime UI Accessibility、Semantic Tree、Name、Description、Relation、State、Action、Live Region、Platform Adapter 与 Product Integration 工程化差距

## 1. 结论

Zircon 已经有一套可保留的中立无障碍 DTO、retained tree 提取器、名称与关系解析、诊断、动作分派以及 AccessKit 转换器。它不是完全空白：公开合同能表达常见 role、checked/expanded/selected/value 状态和十一种动作；Runtime 会从 UI 节点、component、text、tooltip、relation、focus 与 geometry 构建快照；局部测试也覆盖名称、关系、动作和 AccessKit 映射。

但当前实现仍是“可序列化快照与测试转换器”，不是工程级产品无障碍系统。仓内 **317 份 tracked `.zui`、5,594 个 `[nodes.*]` section 中，没有一行显式 `a11y =`、`labelled_by =` 或 `label_for =` authoring**，产品语义全部依赖组件字符串、交互性、文本和 tooltip 启发式推断。`accesskit.rs` 没有任何 production `accesskit_winit::Adapter` consumer；App、Editor 与 WOC 也没有真实 screen reader publication/action path。当前唯一产品可达面是 Dynamic Runtime 手工 JSON capture/action ABI，且 capture 可修改 viewport，action 与 capture 又不绑定同一 tree generation。

中立 schema 同样不足以承载成熟引擎必须长期维护的语义：快照没有 window/surface/generation/locale/publication identity，动作没有 request/user/window/tree generation，关系只有单个 `labelled_by`/`label_for`，description 使用 `#id` 魔法字符串，缺 live region、announcement、range、scroll、collection/table/tree、placeholder、language、required/read-only/invalid/busy/modal 等合同。AccessKit 转换还丢失 `pressed` 和 outbound text selection，使用可碰撞的 `u64::MAX` synthetic root，并把 `ScrollIntoView` 映射为一个没有所需 payload、因此必然被 Runtime 拒绝的 `ScrollTo`。

本轮 **不新增 P0**。已观察到的半提交 input/text/component mutation、Dynamic Runtime 伪造 preview accessibility success、全量 snapshot、无真实 adapter、坐标代际和 authoring 父问题，分别继续由 Runtime77、Runtime11B、Runtime75、Runtime43、Runtime11A、Runtime76 与 Editor23 拥有。本报告登记 **48 项 Runtime78 独有 P1、12 项 P2 与 48 项资格门**，负责把 semantic source、compiled tree、incremental publication、platform adapter、action transaction、live announcement 与产品资格连成同一条可证链。

在显式产品语义、同代增量树、真实 per-window adapter、可回执动作队列、跨平台 screen reader 测试和规模/延迟证据完成前，不得用“JSON 能导出”“AccessKit converter 有测试”或“按钮能 Activate”宣称无障碍已经产品化，更没有证据支持其完备度、稳定性或性能达到并超越 Unreal Slate、Godot AccessibilityServer 或 Bevy 的 per-window AccessKit 集成。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime78 责任 | 本轮不重复登记 |
|---|---|---|---|
| UI service/tree/accessibility 总链 | Runtime11A | semantic publication、adapter 与 action lifecycle 的专项收口 | full snapshot rebuild、无 OS adapter、window publication、global ID、bool-only产品返回 |
| Component semantics | Runtime75 | 接收 typed component semantic facet | role/action 字符串推断、component event 假 delivered |
| Input/effect transaction | Runtime77 | a11y action request/receipt 接入统一 transaction | focus/capture/drag/component/text 前缀半提交 |
| Text/IME/privacy | Runtime11B | a11y text model与平台 range 映射 | secure field、grapheme/edit history、IME 隐私 |
| Geometry/DPI | Runtime76 | adapter 只发布同代可访问 geometry | logical/physical transform、clip 与 DPI 总 owner |
| Dynamic Runtime | Runtime43 | typed a11y product receipt 与 capability truth | 无 surface 时伪造 preview tree/OK 的既有 P0 |
| UI authoring | Editor23 | Runtime compiler/schema/diagnostic contract | Designer、Accessibility panel、asset save/import/export UX |

Runtime78 只登记这些父问题暴露出的无障碍专属合同和产品断点，不把相同根因再次升级或计数。实施时可以共用 `UiInputTransaction`、layout generation、component facet 和 Dynamic ABI，但资格 owner 必须保持唯一。

### 2.2 Zircon 物理冻结

指纹算法：对排序后的 `relative-path=per-file-SHA-256` 以 LF 连接，末尾不附加 LF，再做 SHA-256。统计绑定本次共享 working copy，而非仅绑定 baseline HEAD；实施前必须重取。

| 范围 | 文件 / 行 / bytes | fingerprint / 本轮证据 |
|---|---:|---|
| Public contracts | 14 / 3,685 / 117,725 | `d47542106fde5ee82d1ea64a86337c822b4df9224c1bba81a670de86f6796e35`；中立 tree/state/relation/action 与 dispatch DTO |
| Runtime execution | 58 / 10,344 / 362,174 | `0492895b4247066ec828968e6dfe232d8e69453f390afe5bc0223b9964141d99`；extract、validate、AccessKit、action、surface、Dynamic API；10 项 inline test |
| Product ZUI/reachability | 320 / 55,847 / 3,218,769 | `d451635f67c8c45271797632c82bf22b77072ef0d71129fa27a44e19e7fa8967`；317 份 ZUI、5,594 个 node section、0 项显式 a11y/relation authoring |
| Production union | 392 / 69,876 / 3,698,668 | `db4504abdbe7c9e2c2db19193fa695dbd15ece07adcdfcd12b734ece808d9576`；App/Editor/WOC 无 AccessKit adapter consumer |
| Focused tests | 21 / 8,387 / 296,580 | `cbdd167710a7c887e4dca92f19b2e8ef3e578d950265c1adbd442697ddc6dfa3`；123 项 test、0 ignored |

聚焦 production 中 `zircon_runtime/src/dynamic_api/session/state.rs` 与 `zircon_runtime/src/ui/surface/surface.rs` 已被其他工作修改，故 `source_recheck_required: true`。本轮只做静态 review，没有修改 production、tests、assets 或 Cargo，也没有运行真实 Windows UI Automation、macOS VoiceOver、Linux AT-SPI、screen reader、fault、soak 或 benchmark。

### 2.3 参考物理冻结

参考冻结 25 个文件、19,473 行、757,814 bytes，fingerprint 为 `55772c9ac14ab8c818331064b82b9ba0a55d642271abab9fecbe9f3a69e24f39`。

| 参考 | 文件 / 行 / bytes | 可吸收的工程不变量 | 局限与不照搬内容 |
|---|---:|---|---|
| Unreal | 9 / 2,745 / 103,881；`a508a9a2ea816803bcb2d0ac407941609d507b77482947ab42ccba179cbe5e8a` | message handler activation、per-user accessible focus、widget cache/dirty tick、event/announcement、task queue、window/widget lifecycle | 不复制宏、shared pointer 与具体平台层级 |
| Bevy | 3 / 850 / 28,325；`d3d1d8899c06d9602d318963f72069b5f3700730fe840a117a6dbe7f33e08c82` | `AccessibilityRequested`、per-window `accesskit_winit::Adapter`、changed-node update、action queue、window close teardown | 不把 ECS system ordering 当作 transaction 证明 |
| Godot | 5 / 2,580 / 122,384；`c6aab68d3f9f189a3078e94316b2ca42512ed8106467cad2d26d97d12792fbc8` | 独立 AccessibilityServer、per-window driver、activation/update/action callback、丰富 role/state/relation/live region | 不复制 Object/Variant 与平台 driver API |
| Fyrox | 4 / 9,548 / 360,239；`8348a1f354dd8f0cb2ef88c6615221c962072cdc26e1c5f2617a5f7768350bb1` | UI message/control/focus owner 可作底层生命周期参照 | 本地 snapshot 未找到原生 accessibility adapter，不把缺失源码冒充成熟实现 |
| Unity Graphics | 4 / 3,750 / 142,985；`619d52b42cbe5f09fcc44860e1a5940b5dc2f5d9c924ebe7c1e703455d969a30` | DebugUI control/state/enable-disable 生命周期可作下限 | 本地 Graphics snapshot 不含 UI Toolkit accessibility，不据此推断 Unity 全产品能力 |

参考共同不变量是：语义来自显式且可验证的控件/资产合同；每个 window 有长期 adapter owner；tree update 与 action callback 共享 generation；增量、移除、focus 和 live event 有明确 publication；平台关闭时 teardown；产品资格由真实辅助技术而非 DTO 单测证明。

## 3. 可保留底座

- `UiAccessibilityTreeSnapshot`、`UiAccessibilityNode`、`UiA11yState`、`UiAccessibilityActionRequest/Result` 已经建立 Runtime 与平台库之间的中立层，适合扩展而不是删除。
- `extract.rs` 的多阶段收集、隐藏节点处理、名称/description 解析和诊断框架可以重构为 compiled semantic tree；现有 `BTreeMap/BTreeSet` 结果也提供确定性测试基线。
- component、text、tooltip、relation、focus、bounds 与 action inference 已覆盖常见控件的最低行为，应迁移成 typed semantic contributor/facet。
- action dispatcher 已按 focus/activate/value/text/range/expanded/scroll/dismiss 分模块，是引入 preflight/transaction/typed receipt 的良好边界。
- AccessKit role/state/action converter 与文本 range 辅助函数可保留为 adapter codec 层，但必须由真实 per-window adapter owner 调用。
- 123 项聚焦测试可作为 regression corpus；应在其上增加 generation、graph integrity、Unicode、platform action、product 和规模层，而不是重新从零写起。

## 4. 新增 P0

**本轮无新增 P0。** 下列高危事实仍然开放，但已有唯一 owner，因此只作为 Runtime78 的前置阻塞引用：

| 事实 | Canonical owner | Runtime78 接口要求 |
|---|---|---|
| accessibility capture/action 每次重建完整 snapshot，产品无真实 OS adapter/window publication | Runtime11A | 接收其 incremental publication 与 adapter lifecycle，不重复计数 |
| component event 标记 delivered 而未真实交付 | Runtime75 | semantic action 不得继承假回执 |
| input/effect 多步 mutation 可部分提交 | Runtime77 | a11y action 必须进入同一原子 transaction |
| text mutation/selection/IME 隐私与历史不闭合 | Runtime11B | text action 仅调用其 qualified text operation |
| logical/physical geometry 与 DPI 代际断链 | Runtime76 | adapter publication 绑定 geometry generation |
| Dynamic Runtime 无 surface 时伪造单节点 preview accessibility success | Runtime43 | capability/capture 必须返回真实 unavailable/stale |
| Accessibility authoring 与资产无损保存 UX | Editor23 | Runtime 提供可编译 schema 与诊断，不接管 Editor UX |

## 5. P1 差距

### 5.1 Contract、identity 与 semantic vocabulary

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIA-P1-001 | snapshot 只有 `tree_id`，没有 window/surface/generation/locale/publication identity | 定义 qualified `UiAccessibilityPublicationId` 并贯穿 capture、update、action 与 receipt |
| RUIA-P1-002 | action request 只有 target/action/source/payload，没有 tree generation、request、user、window 身份 | 建立 bounded、generation-bound action envelope，缺关键身份即拒绝 |
| RUIA-P1-003 | Dynamic tree request 的 `generation_hint` 在 Runtime 无消费 | 实现 exact/since generation、not-modified、delta/resync 或明确拒绝 |
| RUIA-P1-004 | `UiAccessibilityActionResult` 在 production 不消费，状态降为 notes/string/bool | 统一 Accepted/Rejected/Unsupported/Stale/Deferred/Committed typed receipt |
| RUIA-P1-005 | request `source` 在 production action path 未读取 | source 参与 trust、focus-visible、policy、telemetry 与 user attribution |
| RUIA-P1-006 | public DTO 大量 `serde(default)` 允许缺失字段静默变成 Generic/Activate/0 | schema version + strict admission；兼容默认仅在显式 legacy reader 中存在 |
| RUIA-P1-007 | role 枚举缺 heading/link/document/form/table/tree/grid/progress/status 等产品角色 | 建立可演进 role taxonomy、platform fallback 与 unsupported diagnostics |
| RUIA-P1-008 | state 缺 required/read-only/invalid/busy/modal/multiselect/orientation 等语义 | 建立 typed state/property vocabulary 与 role-state legality matrix |
| RUIA-P1-009 | relation 只有单个 `labelled_by`/`label_for` | 支持多目标 labelled/described/controls/owns/flow/details/error 等 typed relations |
| RUIA-P1-010 | 没有 live region、announcement、politeness、atomic/relevant 事件合同 | 建立有界 event channel、coalescing、privacy、priority 与 delivery receipt |
| RUIA-P1-011 | value 只有字符串/数字 payload，节点没有 min/max/step/unit/scroll range | 定义 range、progress、scroll viewport/offset/extents 与合法动作参数 |
| RUIA-P1-012 | 缺 text run、line、selection direction、collection/table/tree index、set size 等模型 | 采用 versioned optional capability facets，避免把所有字段塞入单个节点结构 |

### 5.2 Authoring、extraction、name、relation 与 validation

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIA-P1-013 | 317 份产品 ZUI、5,594 个 node section 中零显式 a11y/relation authoring | 建立 typed asset schema、compiler、required semantic policy 与 migration |
| RUIA-P1-014 | name 不聚合 descendant text，也没有 role-specific name-from-content 规则 | 编译名称来源图，按 role/presentation/hidden/locale 规则聚合且有预算 |
| RUIA-P1-015 | 名称 precedence 固定为 explicit、单 label、own text、alt、tooltip | 用 typed name behavior 与 provenance，避免 tooltip/文本启发式成为无声权威 |
| RUIA-P1-016 | description 用 `#id` 魔法字符串引用节点并擦除 typed described-by relation | 将 description text 与 described-by references 分离并保留 provenance |
| RUIA-P1-017 | relation target 是裸 numeric node id、单目标、无 tree/generation/stable handle | 编译 stable semantic handle，publication 时解析为同代 node set |
| RUIA-P1-018 | 引用文本只取直接 target，自身关系链、多标签和 descendant text 不解析 | 建立 cycle-safe relation/name graph evaluator 与 deterministic order |
| RUIA-P1-019 | relation cycle validation 只识别直接二节点 cycle | 使用 SCC/DFS 检测任意长度 cycle，并报告完整 bounded path |
| RUIA-P1-020 | validator 不校验 dangling child、duplicate child、multiple parent、root reachability/order | 定义 semantic forest invariants，非法图不得 publication |
| RUIA-P1-021 | invalid Runtime focus 只在 snapshot 内改成 fallback，真实 focus owner 不变 | publication 与 focus transaction 共享 truth；stale 时拒绝或显式修复并回执 |
| RUIA-P1-022 | fallback 选首个可见非 disabled root，不保证 focusable/actionable，且非 per-user | 用 per-user accessible focus policy、focus proxy 与 stable restore path |
| RUIA-P1-023 | diagnostic 缺 property/path/fix/generation/locale/platform/privacy 分类 | 输出 bounded structured diagnostic，关联 source span、semantic provenance 与 suppression policy |
| RUIA-P1-024 | 缺失名称/角色只在 Runtime snapshot 警告，compile/cook/product gate 不消费 | 在 asset compile、Editor validation、cook 与 capability report 建立 required exit |

### 5.3 AccessKit conversion 与 platform publication

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIA-P1-025 | 多 root 使用 `NodeId(u64::MAX)` synthetic root，但 `UiNodeId` 未保留该值 | 建立独立 adapter identity namespace与碰撞测试，禁止 magic sentinel |
| RUIA-P1-026 | synthetic root 不在中立 tree，action 无法回映 Runtime，label 暴露 raw tree id | adapter-only node 必须有不可交互 policy、稳定 label 与 stale-action rejection |
| RUIA-P1-027 | outbound AccessKit node 丢弃 `pressed` | 建立 exhaustive state mapping matrix 与 compile-time/fixture coverage |
| RUIA-P1-028 | neutral text selection 没有写入 outbound AccessKit update | 映射同代 text document/selection，并对 secure text 应用 redaction |
| RUIA-P1-029 | schema 无 range、scroll、collection、table、tree、live region 等字段，converter 无法发出 | 先扩中立 facets，再由各平台 codec 做 capability-preserving projection |
| RUIA-P1-030 | adapter 输入只有 frame，缺 transform/clip/window coordinate generation | 继承 Runtime76 geometry owner，publication 绑定同代 screen-space projection |
| RUIA-P1-031 | converter 只生成整棵 `TreeUpdate`，没有 deletion/tombstone/delta contract | 定义 node revision、changed/removed sets、resync 与 bounded update batch |
| RUIA-P1-032 | AccessKit `ScrollIntoView` 无 payload，却映射到 Runtime 要求 payload 的 `ScrollTo` | 拆分 BringIntoView 与 SetScrollOffset，分别定义 target、ancestor chain 和 payload |
| RUIA-P1-033 | Dialog Dismiss 映射为 Blur，inbound Blur/HideTooltip 又统一映射 Dismiss | 建立逐平台 action legality table，禁止语义相近但效果不同的映射 |
| RUIA-P1-034 | CustomAction、ShowTooltip、方向 scroll、ScrollToPoint、context menu 等静默丢弃 | 返回 typed Unsupported/NotExposed，并记录 provider capability |
| RUIA-P1-035 | text selection 只支持 anchor/focus 同一节点，不能表达跨 run/line selection | 建立 semantic text document、run identity、range normalization 与 stale handling |
| RUIA-P1-036 | AccessKit character index 与 Unicode grapheme/byte 转换缺 combining/ZWJ/Bidi conformance | 明确平台 index unit，使用共享 text segmentation 并建立多语种 differential corpus |

### 5.4 Action execution、product integration 与 qualification

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUIA-P1-037 | finite `f64` 直接 cast `f32` 可溢出为 infinity，non-finite 又被改成 0 | checked conversion；malformed/out-of-range 返回 typed rejection，不得静默矫正 |
| RUIA-P1-038 | platform translator 构造 request 前不验证 target/action 属于已发布同代 snapshot | action callback 绑定 adapter publication generation 与 exposed-action set |
| RUIA-P1-039 | disabled node 仍可暴露 Focus，但 Runtime focus action 拒绝；virtual a11y focus 与 keyboard focus 混同 | 区分 accessible focus、keyboard focus、focus proxy 与 disabled policy |
| RUIA-P1-040 | focus action 忽略 source，固定 `Programmatic` 且隐藏 focus-visible | source/user/device 进入 focus transaction，视觉策略由输入 modality 决定 |
| RUIA-P1-041 | Dismiss dispatcher 未先验证节点 advertised actions 中含 Dismiss | 所有 action 经过同一 exposure/capability/precondition validator |
| RUIA-P1-042 | SetTextSelection 先标记 handled/accepted，后续多个 retained property mutation 可失败 | 继承 Runtime77/11B，返回最终 atomic text operation receipt |
| RUIA-P1-043 | validator 允许 Slider/Scrollbar range action，dispatcher 只接受 Slider | role-action legality 由单一 generated table 驱动 converter、validator 与 executor |
| RUIA-P1-044 | ScrollTo 只写目标 container 标量 offset，不做 descendant reveal/ancestor scroll chain | 建立 bring-into-view solver、两轴 range、alignment、partial visibility 与 final receipt |
| RUIA-P1-045 | read-like accessibility capture 会先 `resize_viewport(request.size)` 并触发布局状态变化 | capture 只读取 qualified publication；viewport resize 必须是独立有回执 operation |
| RUIA-P1-046 | capture 与 action 各自重建 snapshot，无共享 generation，存在 stale target/action race | action 只消费已发布 generation；stale 返回 resync hint 而非重新猜测 |
| RUIA-P1-047 | 未来 adapter action 没有有界 queue、dedup、deadline、idempotency、backpressure 与 teardown receipt | 建立 per-window action service，activation/deactivation/close 时幂等 drain/cancel |
| RUIA-P1-048 | Dynamic JSON 是唯一产品路径，App/Editor/WOC 无真实 screen reader 资格与 capability truth | 建立真实三平台 adapter、产品入口、AT smoke/soak/latency 矩阵与 evidence manifest |

## 6. P2 改进

| ID | 改进 | 收益 |
|---|---|---|
| RUIA-P2-001 | 为 semantic node、adapter node 与 source node 使用可打印的不同 typed id | 降低 synthetic/root/asset id 混用 |
| RUIA-P2-002 | snapshot 提供按 node id 与 source handle 的 immutable index | 消除 action lookup 线性扫描并方便 delta |
| RUIA-P2-003 | 对重复 role/name/property key 使用 interned/compact representation | 降低大型 UI publication 内存与拷贝 |
| RUIA-P2-004 | diagnostic code 生成稳定文档、默认严重级别和 remediation 模板 | 保持 Editor/cook/runtime 一致解释 |
| RUIA-P2-005 | 固化 CJK、Arabic、combining mark、emoji ZWJ 与 RTL 名称/selection corpus | 提升多语言可重复验证 |
| RUIA-P2-006 | Editor inspector 显示 semantic provenance、relations、exposed actions 和 platform projection | 缩短 authoring/debug 路径 |
| RUIA-P2-007 | 提供 publication diff 与 action trace 可视化 | 定位 stale、重复 announcement 与错误 focus |
| RUIA-P2-008 | capability manifest 记录 OS/provider/library/build/version | 防止“编译了 AccessKit”被误报为产品可用 |
| RUIA-P2-009 | 为 semantic forest、relation graph 与 action payload 增加 property/fuzz corpus | 提前发现 cycle、orphan、overflow 与 parser fault |
| RUIA-P2-010 | 保存可脱敏的 screen reader interaction transcript 与 expected event order | 让跨版本回归可审计 |
| RUIA-P2-011 | 清理 converter/DTO 中把局部支持描述成完整支持的注释和命名 | 让源码 capability truth 与产品事实一致 |
| RUIA-P2-012 | 固化 1K/10K/100K node、dirty burst、announcement storm、action flood workload | 使性能声明绑定可复现实验 |

## 7. 当前差距矩阵

| 子系统 | 当前可保留 | 关键断点 | 参考不变量 | 目标 owner |
|---|---|---|---|---|
| Semantic source | `UiAccessibilityContract`、component/text metadata | 产品资产零显式 authoring，语义靠启发式 | Unreal widget behavior、Godot rich properties | `UiSemanticSourceSchema` |
| Tree compile | 多阶段 extraction/diagnostics | 全量 rebuild、graph validator 不完整、无 generation | Unreal widget cache、Bevy changed nodes | `UiSemanticCompiler` |
| Name/relation | explicit/text/alt/tooltip 与两种 relation | 单 target、magic `#id`、无 descendant/chain/SCC | 平台 name/description/relation graph | `UiSemanticRelationGraph` |
| State/value/text | 常见 bool/value/selection | taxonomy、range、collection、text document 不足 | Godot role/flag/action/property breadth | capability facets |
| Publication | neutral snapshot 与 AccessKit conversion | synthetic id、字段丢失、无 delta/removal/window owner | Bevy per-window adapter、Godot `wd.update` | `UiAccessibilityPublication` |
| Action | 分模块 dispatcher | 无同代 binding、映射矛盾、回执失真 | action queue/callback 与 exposed action 一致 | `UiAccessibilityActionService` |
| Live events | 无 | 无 announcement/live/focus event publication | Unreal event/announcement、Godot live region | `UiAccessibilityEventStream` |
| Product | Dynamic JSON capture/action | App/Editor/WOC 无 screen reader adapter | 真实 window lifecycle 与 AT qualification | `UiAccessibilityWindowSession` |

## 8. 目标架构

### 8.1 Semantic source 与 compiled facets

ZUI、component descriptor、runtime-created node 和 text control 统一产生 versioned `UiSemanticSource`。角色、名称行为、description、relations、state、range、text、collection、live region 和 actions 使用可选 typed facets；compiler 校验 role-property/action legality、source span、locale 与 budget，生成稳定 source handle 和 provenance。启发式只能作为显式、可诊断、可关闭的 legacy migration policy。

### 8.2 Incremental semantic tree 与 publication

`UiSemanticCompiler` 订阅 tree/component/text/layout/focus generation，维护 immutable node store、parent/child/relation index 与 dirty graph。每次 publication 带 project/session/window/surface/tree/schema/locale/geometry generation，输出 changed/removed/focus/event set；budget 超限时返回 typed resync/partial-unavailable，不在 action callback 内临时全量重建。

### 8.3 Per-window platform adapter

`UiAccessibilityWindowSession` 拥有 AccessKit 或平台 driver、activation、adapter node namespace、update queue、action queue、capability manifest 和 teardown。adapter codec 只做中立 facets 到平台 schema 的 loss-aware projection；任何 dropped property/action 必须产生 bounded capability diagnostic。窗口 close/deactivate 时取消旧 generation actions 并发布终态 receipt。

### 8.4 Action transaction 与 focus/text/scroll service

平台 callback 生成含 publication generation、request id、user/source/deadline 的 action envelope。service 验证 target 和 exposed action，编译成 Runtime77 transaction；focus 调用 per-seat focus owner，text 调用 Runtime11B document operation，scroll 调用 ancestor reveal solver。Deferred/Committed/Rejected/Stale/Unsupported 都以 typed receipt 回到 adapter，并可关联产品日志。

### 8.5 Live event、privacy 与 product evidence

focus、selection、value、structure、announcement/live region 进入同代 bounded event stream，支持 coalescing、politeness、atomic/relevant、redaction 和 delivery metrics。App、Editor 与 WOC 共用 window session；资格档案保存 build/provider/OS/AT/version、workload、expected event order、latency、drop/resync 和脱敏 transcript，禁止仅凭 converter unit test 发布 Supported。

## 9. 分层实施里程碑

### M0 · Owner、schema 与 inherited blocker 对齐

冻结 semantic/source/publication/action/event schema 与 owner manifest；Runtime11A/11B/75/76/77、Runtime43 和 Editor23 同步 generation/operation 边界，禁止再增加裸 numeric relation、magic description reference 或 bool-only product result。

### M1 · Authoring source 与 compiler

建立 typed ZUI/component facets、migration 和 compile/cook diagnostics；先迁移核心控件与真实 Editor/WOC 页面，再扩大 required semantic coverage。

### M2 · Incremental tree 与 graph integrity

实现 stable handle、name/relation graph、SCC validation、parent/root invariants、immutable index、changed/removed sets 和 bounded resync；删除 action 内全量 snapshot rebuild。

### M3 · Rich state、text、range、collection 与 live events

补齐 role/property/action legality、text document/range、scroll/range/collection/table/tree facets，以及 announcement/live region event stream。

### M4 · Per-window AccessKit/platform adapter

建立 activation、publication、action queue、adapter id namespace、capability diagnostics 与 close teardown；先 Windows，再以相同合同接 macOS/Linux provider。

### M5 · Atomic action integration

action 接入 Runtime77 transaction、Runtime11B text operation、per-seat focus 与 scroll reveal；实现 generation rejection、deadline、dedup、idempotency 和 typed receipt。

### M6 · Product hard cutover

App、Editor、WOC 使用同一 window session；Dynamic API 改为同代 publication/delta/action receipt。删除 fake preview、bool-only、manual JSON-only capability 与未消费 converter 双轨。

### M7 · Qualification 与 observability

完成 screen reader smoke、multilingual、multi-window/user、window rebuild、fault/replay/soak、1K/10K/100K node、dirty burst/action flood 和 latency/memory benchmark。

### M8 · Performance comparison

在功能与正确性门全部通过后，使用相同硬件、OS、AT、build profile、tree/workload 和采样规则与参考实现比较 update latency、action latency、CPU、allocation、RSS 和 event loss；原始证据入 archive 后才能声明性能优势。

## 10. 资格门

### 10.1 Contract 与 authoring 门

| Gate | 必须证明 |
|---|---|
| RUIA-GATE-001 | publication 含 window/surface/tree/schema/locale/geometry generation，且跨重建不串代 |
| RUIA-GATE-002 | action 缺 request/user/source/publication generation 时在 mutation 前拒绝 |
| RUIA-GATE-003 | `generation_hint` 能得到 exact/not-modified/delta/resync，不被静默忽略 |
| RUIA-GATE-004 | product/ABI 返回 typed final action status，不再降为 bool 或自由文本 |
| RUIA-GATE-005 | strict reader 拒绝缺失关键字段，legacy default 只在显式版本内生效 |
| RUIA-GATE-006 | role/state/relation/action taxonomy 具有版本、fallback 和 legality matrix |
| RUIA-GATE-007 | range/text/collection/live facets 可独立协商且未知 facet 可诊断 |
| RUIA-GATE-008 | 关键产品 ZUI 具有显式 semantic source，启发式占比可度量并逐版本下降 |

### 10.2 Name、relation 与 tree integrity 门

| Gate | 必须证明 |
|---|---|
| RUIA-GATE-009 | button/link/heading/text input/image/dialog 等 role 的 name-from-content 规则正确 |
| RUIA-GATE-010 | 多 label、described-by、descendant text 与 locale change 结果确定且有 provenance |
| RUIA-GATE-011 | description text 不再用 `#id` 魔法字符串承载 relation |
| RUIA-GATE-012 | 任意长度 relation cycle 由 SCC/DFS 检出且诊断 path 有界 |
| RUIA-GATE-013 | dangling/duplicate child、multiple parent、orphan、bad root 阻止 publication |
| RUIA-GATE-014 | hidden/presentation/disabled 节点的 inclusion、name 与 focus 规则有矩阵测试 |
| RUIA-GATE-015 | accessible focus 与 Runtime focus 同代；fallback 不会指向不可聚焦 root |
| RUIA-GATE-016 | compile/cook/product gate 消费 structured diagnostics 和 source span |

### 10.3 Publication 与 platform adapter 门

| Gate | 必须证明 |
|---|---|
| RUIA-GATE-017 | adapter node namespace 与所有 Runtime node id 无碰撞，synthetic node 不可误 action |
| RUIA-GATE-018 | pressed、selection、range、scroll、collection、live properties 不被静默丢失 |
| RUIA-GATE-019 | screen-space bounds 与 clip/transform/window generation 一致 |
| RUIA-GATE-020 | changed/removed/focus event 以 bounded delta 发布，gap 可 resync |
| RUIA-GATE-021 | 1K/10K/100K node 单节点 dirty 不触发无说明的全树 rebuild |
| RUIA-GATE-022 | adapter activate/deactivate/window close 幂等且旧 action 全部终态化 |
| RUIA-GATE-023 | unsupported platform property/action 产生 typed capability diagnostic |
| RUIA-GATE-024 | Windows/macOS/Linux provider/build/version 出现在 capability manifest |

### 10.4 Action、focus、text 与 scroll 门

| Gate | 必须证明 |
|---|---|
| RUIA-GATE-025 | action target 与 exposed action 必须属于 callback 所见 publication generation |
| RUIA-GATE-026 | stale/duplicate/late action 不修改新 tree/window/document |
| RUIA-GATE-027 | oversized/non-finite numeric/point payload typed reject，不转换为 0/inf |
| RUIA-GATE-028 | ScrollIntoView 与 SetScrollOffset 语义分离并完成 ancestor reveal |
| RUIA-GATE-029 | Blur、Dismiss、HideTooltip、ShowTooltip、ContextMenu 映射不再互相冒充 |
| RUIA-GATE-030 | disabled/virtual/keyboard focus policy 一致并正确驱动 focus-visible |
| RUIA-GATE-031 | Slider 与 Scrollbar 的 exposed/validated/executed actions 来自同一 legality table |
| RUIA-GATE-032 | text set/replace/selection 要么完整 commit，要么无 mutation 并返回最终 receipt |

### 10.5 Text、Unicode、live event 与 privacy 门

| Gate | 必须证明 |
|---|---|
| RUIA-GATE-033 | platform character index、UTF-8 byte、grapheme 和 text run 转换定义明确 |
| RUIA-GATE-034 | combining mark、emoji ZWJ、CJK、Arabic、Bidi selection differential corpus 通过 |
| RUIA-GATE-035 | cross-run/line selection、caret、direction 与 stale document 可正确处理 |
| RUIA-GATE-036 | secure text 不泄漏 value、selection surrounding content 或 diagnostic detail |
| RUIA-GATE-037 | polite/assertive live region 的 coalescing、ordering 与 atomic/relevant 规则可测 |
| RUIA-GATE-038 | announcement storm 在预算下不会饿死 focus/value/structure 事件 |
| RUIA-GATE-039 | locale/theme/layout change 只发布必要 semantic/event delta |
| RUIA-GATE-040 | event/action trace 可关联 request/publication/commit 且默认脱敏 |

### 10.6 Product、fault 与 performance 门

| Gate | 必须证明 |
|---|---|
| RUIA-GATE-041 | App、Editor 与 WOC 都由真实 per-window adapter 发布并接收 action |
| RUIA-GATE-042 | Dynamic capture 不修改 viewport/layout，action 与 capture 共享 generation |
| RUIA-GATE-043 | no-surface/unavailable provider 返回真实 unavailable，不伪造 preview success |
| RUIA-GATE-044 | window recreate、device lost、adapter panic、queue overflow 均有 bounded recovery/receipt |
| RUIA-GATE-045 | 多 window、多 user、modal、popup、virtualized list 的 focus/action 不串扰 |
| RUIA-GATE-046 | 真实 screen reader smoke/soak 保存 expected event order 与脱敏 transcript |
| RUIA-GATE-047 | update/action p50/p95/p99、CPU、allocation、RSS、drop/resync 指标绑定原始证据 |
| RUIA-GATE-048 | “优于 Unreal/参考实现”只可在同硬件/OS/AT/build/workload 下由可复现实验发布 |

## 11. 实施顺序与声明边界

1. 先关闭 Runtime11A/43/75/77、Runtime11B/76 与 Editor23 的依赖边，冻结 generation、source、publication、operation 与 diagnostic schema。
2. 再做显式 authoring、compiler 与 graph integrity；没有可靠 semantic source 时，不应先扩大量平台 mapping。
3. 在 incremental publication 与同代 action contract 稳定后接 per-window AccessKit/platform adapter，并硬切产品入口。
4. 最后进行真实辅助技术、fault/soak 和性能资格；不得以 DTO 数量、unit test 数量或 reference 路径存在替代产品证据。

本报告是 review/refactor plan，不是 implementation completion。所有 P1/P2、gate 和 milestone 均为待实施；`review_complete` 只表示本次冻结范围已完成源码与参考纵向审查。

## 12. 最终结论

Zircon 的无障碍代码已经具备中立 DTO、快照提取、局部诊断、动作分派和 AccessKit codec 的真实底座，但距离工程级系统的主要差异并不是再加几个 role，而是缺少显式产品语义、同代增量 publication、完整 relation/text/range/live 模型、per-window adapter owner、原子 action receipt 与真实 screen reader 资格。当前 317 份产品 ZUI 的零显式 authoring 和零 production AccessKit adapter consumer，是“局部功能实现”与“产品能力”之间最直接的证据。

目标架构应把 source schema、semantic compiler、publication、platform adapter、action transaction、live event 和 evidence manifest 建成一条长期可演进的 authority chain。完成 48 项资格门之前，只能称为可测试的无障碍快照原型，不能称为完整、可发布或性能领先的引擎无障碍系统。

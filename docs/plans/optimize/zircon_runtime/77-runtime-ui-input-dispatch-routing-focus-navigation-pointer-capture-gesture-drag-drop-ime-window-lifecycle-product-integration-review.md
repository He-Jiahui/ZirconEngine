---
title: Runtime UI Input、Dispatch、Routing、Focus、Navigation、Pointer Capture、Gesture、Drag-Drop、IME、Window Lifecycle 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime77
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/ui/dispatch
  - zircon_runtime_interface/src/ui/window
  - zircon_runtime_interface/src/ui/focus
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/accessibility.rs
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime/src/ui/platform_input
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/focus
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/accessibility/action
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_editor/src/ui/retained_host
  - zircon_editor/src/ui/workbench
  - examples/woc/native/apps/woc_client/src/input/touch.rs
tests:
  - zircon_runtime/src/ui/tests/event_routing
  - zircon_runtime/src/ui/tests/focus_navigation
  - zircon_runtime/src/ui/tests/runtime_input_manager
  - zircon_runtime/src/ui/tests/runtime_input_ownership
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes
  - zircon_runtime/src/ui/tests/runtime_window_input_pump
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context
  - zircon_runtime/src/ui/tests/accessibility
  - zircon_runtime/src/ui/tests/accessibility_widget_actions
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateUser.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/bevy/crates/bevy_input_focus/src/directional_navigation.rs
  - dev/bevy/crates/bevy_input_focus/src/tab_navigation.rs
  - dev/bevy/crates/bevy_ui/src/focus.rs
  - dev/bevy/crates/bevy_ui/src/picking_backend.rs
  - dev/bevy/crates/bevy_picking/src/events.rs
  - dev/bevy/crates/bevy_winit/src/accessibility.rs
  - dev/Fyrox/fyrox-ui/src/lib.rs
  - dev/Fyrox/fyrox-ui/src/message.rs
  - dev/Fyrox/fyrox-ui/src/widget.rs
  - dev/Fyrox/fyrox-ui/src/text_box.rs
  - dev/godot/scene/main/viewport.cpp
  - dev/godot/scene/gui/control.cpp
  - dev/godot/scene/main/window.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.Input.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.UIState.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 77 · Runtime UI Input、Dispatch、Routing、Focus、Navigation、Pointer Capture、Gesture、Drag-Drop、IME、Window Lifecycle 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前并非只有临时按钮回调。公开接口已经覆盖 pointer、keyboard、text、IME、navigation、accessibility、focus、capture、pointer lock、drag/drop、popup、tooltip、clipboard和host request；Runtime拥有hit test、preview/bubble路由、默认interaction、焦点恢复、tab/spatial navigation、capture、double click、timer、drag state、IME surrounding text和AccessKit中性树；96个聚焦测试文件内有460项测试。这些类型、局部状态机和测试应保留，不能退回产品侧直接匹配Winit事件。

真正的工程级缺口是“输入身份、窗口入口、路由、focus/capture/gesture seat、effect commit、host交互、产品回执”没有形成单一权威链。Dynamic Runtime、Editor retained host与WOC touch各自转换和分派输入；Dynamic Runtime把非pointer事件依次克隆给每个surface，Editor绕过Runtime window pump和`UiInputManager`，WOC又局部实现long press、double tap和pinch。公开metadata允许user/device/window/surface/pointer/timestamp全部缺省，产品路径又经常填零；因此已有路由测试无法证明真实多窗口、多用户、多设备或窗口重建后的所有权。

本轮确认一项独立于父报告的新P0：`apply_dispatch_reply_core`按effects顺序直接修改focus、capture、drag和component状态，后续effect拒绝时不回滚已经提交的前缀；drag begin/complete内部又由多个可失败步骤构成。当前`UiInputDispatchResult`只能列出applied/rejected，不能证明原子提交、补偿或host确认。一个回复因此可能同时报告失败并留下半更新的交互状态。

本轮登记 **1项Runtime77独有P0、48项P1、12项P2与48项资格门**。Runtime11A继续拥有UI service/tree、multi-surface焦点仲裁、window pump、tick、bool-only产品返回、handler lifetime、global ID和AccessKit产品适配；Runtime11B拥有secure text、编辑历史和IME隐私；Runtime56拥有“UI先吞事件导致core physical state不更新”；Runtime75拥有component事件假投递；Runtime76拥有logical/physical geometry与DPI barrier；Editor01拥有Editor丢弃完整IME事件。上述问题在本轮证据中仍成立，但不重复计数。

在同代input identity、单一window/session入口、原子effect transaction、per-seat focus/capture/gesture、异步host request/result、window teardown、真实Editor/WOC迁移以及fault/soak/performance receipt完成前，不得用“事件类型已定义”“局部测试通过”或“dispatch result里有applied列表”宣称输入系统达到Unreal Slate、Godot GUI、Bevy focus/picking/accessibility或Fyrox UI的产品完备度，更没有证据支持性能和表现优于当前Unreal。

## 2. 审查边界与物理冻结

### 2.1 Owner边界

| 领域 | Canonical owner | Runtime77责任 | 不重复登记 |
|---|---|---|---|
| UI service/tree/window总链 | Runtime11A | 输入transaction、seat与product adapter的细化 | multi-surface order、bool-only返回、window pump、tick、global ID、handler token、AccessKit adapter |
| Text editing/IME/privacy | Runtime11B | IME session与host ack的输入侧协议 | secure field、grapheme/edit history、composition geometry与隐私 |
| Physical input/action map | Runtime56 | UI route与core state的明确交接receipt | UI swallow导致core pressed/released缺失、device/action/trigger/remap |
| Component behavior | Runtime75 | effect transaction调用typed component endpoint | 字符串classifier、schema外mutation、component event假投递 |
| Layout/DPI geometry | Runtime76 | input只接受同代logical geometry | logical/physical size、pointer transform、scroll/virtual layout |
| Dynamic session | Runtime43 | typed input receipt和host result桥 | FFI/session/event总生命周期与host request总transport |
| Editor host | Editor01/23 | Editor迁移到同一window/input/IME/a11y adapter | Editor retained架构、authoring UX和UI资产工作流 |

`RUII-P0-001`只拥有“一个已接受dispatch reply内的effect前缀可部分提交”这一独立线性化故障；它不重新登记Runtime11A的host request丢失或Runtime75的component event假投递。实施时应由同一个`UiInputTransaction`关闭这些边界，但计数和验收owner保持唯一。

### 2.2 Zircon物理冻结

算法为对排序后的`relative-path=per-file-SHA-256`以LF连接、末尾不附加LF，再做SHA-256。结论绑定当前共享working copy而非只绑定baseline HEAD；实施前必须重取指纹。

| 范围 | 文件 / 行 / bytes | fingerprint / 本轮证据 |
|---|---:|---|
| Public input contracts | 34 / 4,049 / 128,430 | `610d10f846940b613492a8b88095c802a4cc0ab6a9a1b95edbeaa9056bd33eb3`；dispatch/window/focus/navigation/a11y合同，6项内联测试 |
| Runtime execution | 123 / 18,611 / 640,793 | `742116397b8276ee3ee3f9f739be8871918107f5812f4853cb5df2d93634fc67`；dispatch、platform_input、surface input/focus、hit、a11y、dynamic session，44项内联测试 |
| Product consumers | 25 / 3,415 / 123,466 | `3e1c9f2797ddf6f932ca4998be427e4d7f0484f321c9ef87522823f823c0c4a4`；Editor Winit/retained shell与WOC touch/HUD，48项内联测试 |
| Production union | 182 / 26,075 / 892,689 | `693bdf3f437d18a69d888c740991fd249f0fde3ebb74e450541099ad6dc9ed8e`；98项内联测试，0项ignored |
| 聚焦测试 | 96 / 30,613 / 1,063,518 | `8d1389c4a57d49ca6a1c182bf93c1edb57aaec53e31cbe922dce5412c42dcd4a`；460项test，0项ignored |

聚焦测试足以证明局部pointer preview/bubble、focus restoration、navigation repeat、IME context、accessibility action和window translation已有行为底座；但没有“第二个effect失败后第一个effect回滚”的fault test，没有真实多window/seat/device/generation矩阵，也没有Dynamic Runtime、Editor与WOC消费同一transaction receipt的产品测试。`UiInputManager::tick`、UI manager的IME host request drain与通用window pump在production中没有形成同一个caller链。

本轮只做review，不修改production/tests/assets，不运行Cargo、Editor、WOC、真实窗口、fuzz、fault、soak或benchmark。共享工作树有其他Session修改，因此保持`source_recheck_required: true`。

### 2.3 参考物理冻结

参考冻结23个文件、49,605行、1,838,600 bytes，fingerprint为`d232693ad4a926afe37386bdb7cb4bfbe13a67803681727bb09b97db46837c94`。

| 参考 | 文件 | 可吸收的工程不变量 | 不照搬内容 |
|---|---:|---|---|
| Unreal Slate | 7 | `SlateApplication`集中入口和reply应用；`SlateUser`按user保存focus path、capture与drag；editable text持有text input method context | 不复制宏、shared pointer或平台抽象层级 |
| Bevy | 7 | focus event bubbling、directional/tab graph、window/camera/pointer identity、clip-aware picking、每窗口AccessKit adapter/action queue/close teardown | 不把ECS system顺序直接当Zircon transaction |
| Fyrox | 4 | `UserInterface`统一picked/captured/focused/drag/clipboard/double-click；`UiMessage`有方向与handled状态；单一OS事件入口 | 不复制具体message enum和arena布局 |
| Godot | 3 | `Viewport`统一GUI input、mouse/touch capture清理、focus navigation、drag/drop与tooltip；`Window`按window桥接IME | 不复制Node通知编号与DisplayServer API |
| Unity Graphics | 2 | Debug UI input action、activation gesture和enable/disable生命周期 | 仅有DebugManager，不冒充Unity UI Toolkit源码 |

参考共同不变量不是“拥有更多event variant”，而是：一个长期owner接收带身份的窗口事件；route只产生proposal；focus/capture/drag/IME等状态在确定的提交点更新；window失活/销毁有完整清理；产品和可访问性适配器消费同一代状态。

## 3. 可保留底座

- `UiInputEvent`、`UiDispatchReply`、`UiDispatchEffect`与`UiInputDispatchResult`已经把事件、处置、局部副作用和结果从回调返回值中分离，是建立transaction的可用起点。
- pointer preview/bubble、keyboard focus route、default interaction与accessibility action都已有显式路径；不需要重写全部widget，只需统一proposal/commit。
- focus state已有scope、modal stack、restore policy和navigation request；应升级为per-seat owner与persistent focus path，而不是删除。
- pointer capture、pointer lock、高精度mouse、double click、timer和drag state已有局部模型；应补身份、generation、取消和transaction。
- IME surrounding text window、clause、cursor/composition geometry与Enable/Disable请求已有接口；应接到per-window text input session和host结果。
- Winit translator、window metrics/state与AccessKit中性转换可继续作为平台adapter底座，但不能再由Editor和Dynamic Runtime分别绕过。
- 460项聚焦测试应迁移为新transaction的regression corpus，并新增产品、fault和并发资格层。

## 4. 新增P0

### RUII-P0-001 · Dispatch effects顺序直写，失败时可留下半提交focus/capture/drag/component状态

**现状。** `apply_dispatch_reply_core`遍历`reply.effects`，逐项直接修改surface；若某项因目标、capture、drag session或component条件拒绝，先前成功项仍保留。pointer/navigation路径又会在下层直接改dirty、focus或capture，然后把effect记为applied；drag Begin/Complete内部包含drag state、capture、component flag和dirty等多个步骤。`UiDispatchHostRequest`在本地变更后发出，但没有operation id、ack或reconcile。

**危险。** 一次widget reply可先取得capture或切换focus，再因drag/component/host尾项失败而返回rejected；调用者看到“部分失败”，系统却已改变输入所有权。重放、超时、窗口销毁和插件卸载会进一步放大不一致，且当前result无法证明写集合、commit generation或补偿结果。

**必须重构。** route只生成sealed `UiInputProposal`；`UiInputTransaction`先解析并验证完整write set、目标generation、互斥关系、host前置条件与预算，再在单一commit point更新focus/capture/drag/component状态。允许异步host操作时必须明确`Prepared/AwaitingHost/Committed/Aborted/Compensated`状态，并返回带input/effect generation的receipt。禁止继续用“前缀applied + 尾部rejected”冒充可接受的事务语义。

## 5. P1差距

### 5.1 Identity、admission与dispatch contract

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUII-P1-001 | metadata的user/device/window/surface/pointer均可缺省，产品常填零 | 建立generation-qualified `UiInputIdentity`，关键事件缺身份即拒绝 |
| RUII-P1-002 | timestamp/sequence没有统一monotonic clock domain | 由window input session分配clock、sequence、frame和source generation |
| RUII-P1-003 | batch、字符串key、text与payload无统一bytes/count预算 | compile/ingress前做typed bounds、checked allocation和诊断 |
| RUII-P1-004 | physical/logical key、control和popup id以字符串承载 | 使用versioned typed codes；未知值保留显式扩展域 |
| RUII-P1-005 | synthetic/platform/replay/a11y来源与信任级别不可证明 | envelope记录origin、trust、schema version和replay lineage |
| RUII-P1-006 | effect没有声明读写集合、冲突和前置条件 | proposal标准化为可preflight的typed write set |
| RUII-P1-007 | result没有input generation、commit generation和最终outcome | 生成不可歧义的Committed/Rejected/Deferred/Compensated receipt |
| RUII-P1-008 | host request只有effect index/request/reason | 增加operation id、owner identity、deadline、idempotency和request hash |
| RUII-P1-009 | clipboard/IME/popup等没有host result/ack/reconcile协议 | host结果返回原transaction并检查generation后提交或补偿 |
| RUII-P1-010 | pointer、navigation和default behavior走不同effect应用路径 | 全部route只能提交proposal，由同一committer应用 |
| RUII-P1-011 | Unhandled/Passthrough/effects组合语义容易丢effect或继续变更 | 定义handled、propagation、default prevention和effect validity正交状态 |
| RUII-P1-012 | reject原因多为自由文本，缺稳定diagnostic code与privacy policy | 提供bounded code、safe context和debug-only detail |

### 5.2 Routing、focus、navigation、capture与gesture

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUII-P1-013 | 每surface只有单一focus state，未落实metadata中的user/device | 建立per-seat/per-window focus owner与发布模型 |
| RUII-P1-014 | Dynamic Runtime跨surface顺序广播，无global focused/modal arbiter | 继承Runtime11A owner，建立compositor级ordered surface route |
| RUII-P1-015 | capture仅按pointer id，缺window/surface/user/device/generation | 使用qualified capture lease并在stale event前校验 |
| RUII-P1-016 | active pointer线性表和三键mask不足以承载multi-touch/tablet | typed contact/button/tool state、pressure/tilt/twist与bounded indexed store |
| RUII-P1-017 | focus变更重建候选，缺persistent focus path和严格通知顺序 | 缓存qualified path，原子发布lost/gained/within变化 |
| RUII-P1-018 | navigation每次递归收集、排序、全扫描候选 | 建立随tree/layout generation更新的navigation graph/index |
| RUII-P1-019 | tab无条件wrap，缺group、boundary、trap和restore策略 | 编译tab group/order/wrap policy并支持modal scope |
| RUII-P1-020 | spatial score只看斜率/距离，忽略beam、transform、clip和RTL | 引入可配置score、manual override、blocker与differential corpus |
| RUII-P1-021 | focus成功不产生bring-into-view/scroll chain transaction | focus commit联动同代reveal request并报告最终可见性 |
| RUII-P1-022 | Runtime没有gesture recognizer、arena、竞争和取消 | 建立per-pointer/team recognizer arena与deterministic resolution |
| RUII-P1-023 | long press/double tap/pinch仅在WOC局部算术实现 | 提供typed tap/hold/pan/pinch/rotate/fling及统一阈值/profile |
| RUII-P1-024 | callback registry缺unregister token/generation安全 | 继承Runtime11A owner，以RAII subscription和retire barrier硬切 |

### 5.3 Drag-drop、text/IME、clipboard、a11y与window

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUII-P1-025 | drag owner只靠pointer/session，缺source/target/window/surface/user generation | 建立qualified drag session和source/target lease |
| RUII-P1-026 | payload/accept没有typed MIME、operation和policy negotiation | 定义data offer、allowed/preferred operation与accepted subset |
| RUII-P1-027 | dynamic file drag只进入core input，UI收不到外部drag/drop | window adapter统一内部、跨surface和OS file/data offer |
| RUII-P1-028 | drag缺统一threshold、preview、autoscroll、cancel和rollback | recognizer启动，transaction管理preview/capture/target/complete |
| RUII-P1-029 | IME owner只有node，缺window/surface/user/document generation | 建立per-window `UiTextInputSession`与document revision |
| RUII-P1-030 | Enable/Disable/cursor update无host ack和stale拒绝 | platform adapter回传session-qualified result后发布状态 |
| RUII-P1-031 | product adapter未闭合preedit clauses、delete surrounding和candidate geometry | 用完整IME conformance矩阵覆盖Editor与Runtime产品 |
| RUII-P1-032 | clipboard只有text read/write且无异步correlation/MIME/selection | 建立data offer/request/result、selection和security policy |
| RUII-P1-033 | cut先改文本再发clipboard write，失败不能原子恢复 | 将clipboard outcome与edit transaction/history绑定 |
| RUII-P1-034 | 只有AccessKit转换，没有每窗口adapter/action queue/close teardown | 继承Runtime11A owner，建立真实platform accessibility session |
| RUII-P1-035 | Winit translator只被Editor局部使用，Dynamic Runtime手写另一条转换 | 单一`UiWindowInputSession`拥有翻译、metrics、route和publication |
| RUII-P1-036 | deactivate/occlude/close/destroy没有统一释放focus/capture/drag/IME顺序 | 定义幂等teardown transaction、deadline和completion receipt |

### 5.4 Product、performance、observability与evidence

| ID | 差距 | 工程化要求 |
|---|---|---|
| RUII-P1-037 | Dynamic Runtime输入只返回bool，丢失effect/host/reject细节 | 继承Runtime11A/43 owner，返回bounded typed receipt |
| RUII-P1-038 | UI manager tick、timer和IME host drain没有production scheduler owner | 继承Runtime11A owner，接入clock/deadline scheduler |
| RUII-P1-039 | UI可在core physical state更新前吞掉事件 | 继承Runtime56 owner，明确physical state与UI disposition顺序 |
| RUII-P1-040 | Dynamic Runtime、Editor、WOC各自翻译/分派/gesture authority | 迁移到同一adapter/router，删除旁路而非保留兼容双轨 |
| RUII-P1-041 | gesture、file drag、tablet、额外mouse button和lock state覆盖不全 | 建立platform capability matrix与unsupported硬诊断 |
| RUII-P1-042 | wheel translator使用(0,0)，Editor靠last pointer补偿 | window session绑定同代pointer location或明确无位置scroll语义 |
| RUII-P1-043 | hit/focus/nav route反复分配、克隆和全扫描 | generation index、route scratch、small-buffer与访问预算 |
| RUII-P1-044 | 跨surface事件和route context/payload多次clone | 借用/arena carrier与bounded publication，避免热路径深拷贝 |
| RUII-P1-045 | 没有产品input queue backpressure和per-frame事件预算 | admission queue按window/source分舱、合并move并保留边缘事件 |
| RUII-P1-046 | replay/journal缺完整identity、route、effects和host results | 记录canonical envelope、proposal、commit receipt与外部结果 |
| RUII-P1-047 | 缺route depth、reject、capture、gesture、host latency和stale指标 | 建立低开销metrics、trace correlation和privacy分层 |
| RUII-P1-048 | 没有真实Windows/macOS/Linux窗口及Editor/WOC长期资格 | 建立多窗口、多DPI、多seat、IME/a11y、fault/soak/perf矩阵 |

## 6. P2改进

| ID | 改进 | 收益 |
|---|---|---|
| RUII-P2-001 | 将pointer/nav路径里的“applied”命名改为proposal/committed真实状态 | 降低把镜像记录误认成交付的风险 |
| RUII-P2-002 | 把double click、hold、drag、repeat阈值集中到versioned input profile | 避免产品魔数漂移 |
| RUII-P2-003 | button/contact集合使用compact bitset/small vector | 减少热路径分配并扩展设备 |
| RUII-P2-004 | 为route path、effect write set和diagnostic提供可复用scratch arena | 降低每事件临时分配 |
| RUII-P2-005 | 从平台schema生成key/button/tool映射表和unknown coverage test | 减少手写映射遗漏 |
| RUII-P2-006 | 删除按component字符串猜modal/dialog的热路径 | 统一使用typed behavior/facet |
| RUII-P2-007 | input inspector显示identity、route、proposal、commit和host阶段 | 提高Editor诊断能力 |
| RUII-P2-008 | gesture arena与capture状态提供开发可视化 | 缩短竞争/取消问题定位时间 |
| RUII-P2-009 | capability report附provider/build/version和证据hash | 防止平台能力自报失真 |
| RUII-P2-010 | 统一structured diagnostic code、redaction与sampling | 支持遥测而不泄漏文本/剪贴板内容 |
| RUII-P2-011 | 清理“supported”注释和未接产品caller的误导命名 | 让源码声明与产品事实一致 |
| RUII-P2-012 | 固化pointer storm、deep route、nav grid、IME、drag和a11y workload catalog | 让优化声明可复现 |

## 7. 当前差距矩阵

| 子系统 | 当前可保留 | 关键断点 | 参考不变量 | 目标owner |
|---|---|---|---|---|
| Window ingress | Winit translator、window state/metrics | Editor/Dynamic双转换，生命周期不闭环 | Unreal application、Fyrox single OS entry、Godot Window | `UiWindowInputSession` |
| Identity | optional metadata | 缺seat/device/window/surface generation与clock | SlateUser、Bevy window/camera/pointer | `UiInputEnvelope` |
| Routing | preview/bubble/default behavior | 跨surface广播、不同路径直接写状态 | Slate reply、Fyrox UiMessage、Godot Viewport | `UiInputRouter` |
| Effect commit | typed effect/result | 顺序直写、部分提交、无host ack | centralized reply application | `UiInputTransaction` |
| Focus/navigation | scope/modal/restore/tab/spatial | 单focus、全扫描、无reveal/RTL/clip | SlateUser focus path、Bevy graph、Godot neighbors | `UiFocusSeat` |
| Capture/gesture | capture、double click、WOC局部touch | identity弱、无arena/cancel | SlateUser capture/drag、Godot touch focus | `UiPointerSeat`/`UiGestureArena` |
| Drag/drop | typed drag effect和局部state | 无payload negotiation、外部file route、rollback | Unreal/Godot/Fyrox完整drag lifecycle | `UiDragSession` |
| IME/clipboard | surrounding text、clauses、geometry、text clipboard | 无per-window session/ack，产品断链 | Slate text method context、Godot Window IME | `UiHostInteractionBridge` |
| Accessibility | AccessKit中性tree/action conversion | 无真实per-window adapter/lifecycle | Bevy AccessKit Winit adapter | `UiAccessibilitySession` |
| Product | 大量局部测试 | Dynamic/Editor/WOC不是同一authority | 参考产品入口消费统一owner | `UiInputPublication` |

## 8. 目标架构

### 8.1 `UiInputEnvelope`与identity

Window adapter把平台事件canonicalize为bounded `UiInputEnvelope`：schema/source/trust、monotonic timestamp/sequence、window/surface/user/device/pointer/tool及各自generation、logical/physical position和metrics generation。replay、a11y和synthetic event使用同一carrier但来源显式；关键identity不能靠default补零。

### 8.2 `UiWindowInputSession`与router

每个native/embedded window拥有input session，负责平台转换、metrics、lifecycle、queue budget、IME/a11y adapter与teardown。compositor级`UiInputRouter`根据modal/focus/capture/picking决定唯一ordered route；surface不再通过循环广播竞争同一keyboard/text事件。

### 8.3 `UiInputTransaction`

route和widget只产生immutable proposal。transaction compiler合并effect、解析目标、生成read/write set并检查generation、预算、互斥和host依赖；commit在单一点原子更新focus/capture/drag/component/dirty。异步host effect进入prepared state，result以operation id回流；所有终态生成可重放receipt。

### 8.4 Per-seat focus、pointer与gesture

`UiFocusSeat`维护per-user/device/window focus path、modal scope、navigation graph与restore；`UiPointerSeat`维护qualified contacts、buttons和capture leases；`UiGestureArena`承载tap/hold/pan/pinch/rotate/fling recognizer竞争、team、取消和阈值profile。窗口失活或owner retire通过同一个teardown transaction释放所有lease。

### 8.5 Drag、IME、clipboard与accessibility host bridge

`UiHostInteractionBridge`承载typed request/result。drag session使用data offer和operation negotiation；text input session绑定window/surface/user/document generation；clipboard支持MIME/selection/security；AccessKit adapter按window维护tree generation/action queue/focus。任何host callback都必须在generation校验后才能提交。

### 8.6 Product publication与replay

Dynamic Runtime、Editor和WOC只消费`UiInputReceipt`与同代UI publication；debug inspector、metrics和journal读取同一记录。旧bool callback、Editor自定义Winit分派和WOC局部gesture状态机在迁移完成后硬切删除，不保留双authority兼容层。

## 9. 分层实施里程碑

### M0 · Identity、owner与P0硬门

冻结event/effect/receipt schema、身份与预算；为partial commit建立可失败回归；引入proposal/preflight/commit并关闭`RUII-P0-001`。

### M1 · 单一window/router/transaction

建立`UiWindowInputSession`、ordered surface router和统一effect committer；pointer/navigation/default behavior迁移后删除直写路径。

### M2 · Focus、capture、navigation与gesture

实现per-seat focus path、qualified capture lease、incremental nav graph、bring-into-view与recognizer arena；WOC gesture迁移到Runtime。

### M3 · Drag、IME、clipboard、a11y与lifecycle

建立data offer、text input session、host request/result、per-window AccessKit adapter和幂等teardown transaction。

### M4 · 产品硬切

Dynamic Runtime、Editor retained host与WOC使用同一adapter/router/receipt；关闭bool-only、手写Winit和局部gesture旁路。

### M5 · 资格与性能

运行多窗口/seat/device/DPI、IME/a11y、file drag、disconnect/close、fault/replay/soak和热路径benchmark；性能声明绑定build、硬件、workload和原始证据。

## 10. 资格门

### 10.1 Identity、contract与atomicity门

| Gate | 必须证明 |
|---|---|
| RUII-GATE-001 | 关键事件缺window/surface/user/device generation时在route前拒绝 |
| RUII-GATE-002 | 每个window/source的timestamp与sequence单调且跨重建不串代 |
| RUII-GATE-003 | 超大batch/text/key/payload在allocation前被预算拒绝 |
| RUII-GATE-004 | key/button/tool unknown值可诊断且不被错误映射 |
| RUII-GATE-005 | platform/synthetic/replay/a11y来源与trust可在receipt追溯 |
| RUII-GATE-006 | proposal完整列出read/write set、generation和precondition |
| RUII-GATE-007 | 任一尾部effect失败时无focus/capture/drag/component前缀残留 |
| RUII-GATE-008 | drag composite任一步骤失败均原子abort或完整compensate |
| RUII-GATE-009 | host request具operation id、deadline、idempotency和owner identity |
| RUII-GATE-010 | stale/duplicate/late host result不能提交到新window/document |
| RUII-GATE-011 | pointer/nav/default/a11y全部经过同一committer |
| RUII-GATE-012 | disposition、propagation、default prevention和effect validity正交可测 |

### 10.2 Focus、capture、navigation与gesture门

| Gate | 必须证明 |
|---|---|
| RUII-GATE-013 | 两user/两device/两window焦点路径互不污染 |
| RUII-GATE-014 | modal surface阻断背景keyboard/pointer且恢复顺序确定 |
| RUII-GATE-015 | stale capture lease在node/surface/window重建后被拒绝 |
| RUII-GATE-016 | multi-touch、五键mouse和tablet pressure/tilt/twist状态正确 |
| RUII-GATE-017 | focus lost/gained/within通知顺序与published path一致 |
| RUII-GATE-018 | 单节点变化只更新navigation graph依赖闭包 |
| RUII-GATE-019 | tab group、wrap、trap、disabled/hidden skip矩阵通过 |
| RUII-GATE-020 | spatial navigation通过beam/transform/clip/RTL/manual override corpus |
| RUII-GATE-021 | focus reveal在nested scroll中到达可见目标并有receipt |
| RUII-GATE-022 | competing tap/pan/hold recognizer只有一个确定winner或team |
| RUII-GATE-023 | capture loss、window deactivate和pointer cancel终止全部gesture |
| RUII-GATE-024 | subscription retire后在途事件不能调用旧callback |

### 10.3 Drag、IME、clipboard、a11y与window门

| Gate | 必须证明 |
|---|---|
| RUII-GATE-025 | drag source/target/session跨node/window重建不串代 |
| RUII-GATE-026 | MIME/data offer与copy/move/link negotiation结果一致 |
| RUII-GATE-027 | internal、cross-surface、cross-window和OS file drag走同一生命周期 |
| RUII-GATE-028 | drag cancel/drop failure恢复capture、preview、target和component state |
| RUII-GATE-029 | 两window IME session和document revision互不串写 |
| RUII-GATE-030 | Enable/Disable/cursor update ack乱序时只有新代生效 |
| RUII-GATE-031 | preedit clauses、delete surrounding、selection和candidate geometry产品通过 |
| RUII-GATE-032 | secure text不向IME/clipboard/journal/metrics泄漏plaintext |
| RUII-GATE-033 | clipboard拒绝/超时不会留下已删除且不可恢复的cut内容 |
| RUII-GATE-034 | 每window AccessKit adapter tree/focus/action generation一致 |
| RUII-GATE-035 | WindowClosed删除adapter并清空action queue、focus和IME owner |
| RUII-GATE-036 | deactivate/occlude/close/destroy teardown幂等且有deadline receipt |

### 10.4 Product、performance、fault与evidence门

| Gate | 必须证明 |
|---|---|
| RUII-GATE-037 | Dynamic Runtime返回typed receipt而非只返回bool |
| RUII-GATE-038 | timer/double-click/repeat/IME request由production scheduler推进 |
| RUII-GATE-039 | UI consume不会让core physical pressed/released状态丢失 |
| RUII-GATE-040 | Dynamic Runtime、Editor和WOC消费同一adapter/router build hash |
| RUII-GATE-041 | gesture/file drag/tablet/extra buttons/lock states有平台能力矩阵 |
| RUII-GATE-042 | wheel命中使用同代last pointer或显式positionless语义，不落(0,0) |
| RUII-GATE-043 | 10万节点hit/nav workload访问量受index和route budget约束 |
| RUII-GATE-044 | pointer storm与深route下allocation/clone/p50/p95/max有receipt |
| RUII-GATE-045 | queue过载合并move但不丢press/release/cancel/focus边缘事件 |
| RUII-GATE-046 | canonical journal double-run得到相同route、proposal和commit digest |
| RUII-GATE-047 | late host result、close race、callback retire、OOM预算fault无partial state |
| RUII-GATE-048 | Windows/macOS/Linux真实窗口、Editor/WOC soak/perf结果绑定原始证据 |

## 11. 测试与证据缺口

| 缺口 | 当前证据 | 必须补齐 |
|---|---|---|
| Effect atomicity | 有applied/rejected列表和局部route测试 | 每个effect边界注入失败，验证完整rollback/abort/compensation |
| Identity | optional metadata单测 | 多window/surface/user/device/generation property与stale corpus |
| Route authority | preview/bubble与surface route测试 | Dynamic/Editor/WOC同一envelope到同一receipt differential |
| Focus/navigation | restore、tab、spatial局部测试 | persistent path、modal、RTL、clip、bring-into-view与大图增量测试 |
| Gesture/capture | double click/capture及WOC touch局部测试 | recognizer competition、cancel、multi-touch/tablet和lifecycle fault |
| Drag/drop | Begin/Update/Accept/Complete局部状态 | payload negotiation、外部file、cross-window、autoscroll和failure rollback |
| IME/clipboard | surrounding text、clauses、geometry局部测试 | 真实Windows/macOS/Linux IME、ack乱序、secure field和clipboard failure |
| Accessibility | neutral conversion/action dispatch测试 | per-window Adapter、OS inspector、WindowClosed与action race |
| Window lifecycle | translator与metrics测试 | deactivate/occlude/close/destroy全资源释放和deadline |
| Performance | 无统一产品receipt | pointer storm、deep route、large nav、gesture、IME与a11y workload基线 |

## 12. Owner、状态与硬约束

| 项目 | 状态 |
|---|---|
| Review | complete；绑定上述working-copy fingerprints |
| Implementation | pending；必须先关闭`RUII-P0-001`并重取source fingerprint |
| 新增严重度 | 1 P0 / 48 P1 / 12 P2 |
| Qualification | 48 gates；当前均未满足产品级证据 |
| Production/tests/assets修改 | none |
| 本轮运行 | 文档静态核对；未运行Cargo、Editor、WOC、真实窗口、fuzz、fault、soak或benchmark |

实施硬约束：

1. 不允许通过继续增加Editor/WOC局部Winit、gesture、clipboard或IME旁路来关闭本报告。
2. 不允许保留旧直写effect path与新transaction双authority；迁移后必须硬切删除旧路径。
3. 不允许把`applied`列表、bool handled、dirty flag、event variant数量或mock host当作原子提交和产品资格。
4. 不允许在输入identity缺失时用零值或当前focused surface猜owner。
5. 不允许把Unity Graphics DebugManager当作Unity UI Toolkit的完整输入参考。
6. Runtime77只拥有输入transaction与其细化子系统；父报告中的既有P0必须回到各自canonical owner关闭。

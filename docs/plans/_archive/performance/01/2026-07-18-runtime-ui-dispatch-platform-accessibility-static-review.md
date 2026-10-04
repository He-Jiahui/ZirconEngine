---
related_code:
  - zircon_runtime/src/ui/dispatch
  - zircon_runtime/src/ui/platform_input
  - zircon_runtime/src/ui/accessibility
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_matrix.rs
  - zircon_runtime/src/ui/tests/accessibility/activation_actions.rs
  - zircon_runtime/src/ui/tests/accessibility/focus_diagnostics.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/context.rs
  - zircon_runtime_interface/src/ui/dispatch/navigation/context.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/surface/navigation/route.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
  - dev/slint/internal/backends/winit/accesskit.rs
  - dev/bevy/crates/bevy_a11y/src/lib.rs
tests:
  - pointer context per-node source-level RED to GREEN guard passed
  - accessibility validator and action target clone source-level RED to GREEN guards passed
  - rustfmt check and scoped diff check passed
  - current-source Windows zircon_runtime UI tests pending
  - route/timer/accessibility/text-action scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI dispatch/platform/accessibility逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已逐文件完整阅读`zircon_runtime/src/ui/dispatch` 13/13、`platform_input` 3/3与`accessibility` 43/43，共59/59个Rust文件；三目录均无外部脏文件。连同上一批，`ui`累计生产文件87/783。定向阅读pointer/navigation route/context契约、surface/action产品调用、Runtime12输入回传与Slint/Bevy/UE参考。

## PERF-MVP-254/255：owned route与timer全扫

`UiPointerRoute`包含hit path及bubbled/stacked/entered/left/root等多组Vec。dispatch先clone进result，stacked候选再clone；每个命中node/phase的owned context又clone route。原实现同一node多个handler还逐handlerclone，本轮已用源码RED→GREEN把context移出handler循环，H次降为1次。navigation已有每node一次context，但仍有result/context route深copy。EditorUI01需让handler context借用或Arc共享单一route generation，并让候选遍历借用ordered slices。

`UiInputManager::tick`每次分别扫描typeahead/submenu/tooltip/toast四个BTreeMap，收集并clone所有due payload，再逐事件dispatch；idle成本随所有armed timer增长。统一deadline queue/timing wheel必须以`target+kind+generation`处理replace/cancel/stale，并给同帧大量due事件count/time budget。active pointer Vec按真实指针数线性查找规模很小，当前不单独立项。

## platform_input接入前风险

winit translator每个keyboard event物化physical/logical key String和可选text String；platform event normalize会move这些字段，未发现第二次clone。当前产品检索没有`translate_winit_window_event/translate_winit_modifiers`调用，EditorUI01 M1.S3仍计划接入并删除editor本地翻译，因此这里只登记接入前counter门：优先typed/compact key identity，诊断字符串按需生成；没有产品trace前不把它宣称为现有MVP瓶颈。Runtime12已回传frame input retention/coalescing修复，UI batch仍需单独验证pointer move频率不会绕过该语义。

## PERF-MVP-256/257：accessibility全树重建

`accessibility_snapshot`先两遍遍历全部nodes，两遍都逐node沿parent链求effective hidden；随后再做name、description、hidden relation target、children与diagnostics多轮处理。`filter_children`为每个tree node递归穿过excluded descendants。生产validator原先把所有含String/Vec的accessibility nodes深clone进BTreeMap，本轮已TDD改成`node_id→snapshot index`，保留duplicate/relation/focus diagnostics。AccessKit转换仍每次重建全部Node/children/String。

每个assistive action又同步调用`surface.accessibility_snapshot()`只为验证一个target，并在线性snapshot lookup后原先再深clone目标node；target clone本轮已改borrow。根因仍是没有generation-owned node index/action contract。Slint AccessKit adapter保存CachedNode与每node PropertyTracker，property变化只发送dirty nodes，结构变化才全树重建；action request通过node id直接映射ItemRc。Zircon应采用同一所有权边界，stable generation不构建snapshot，单target action不访问无关node。

## PERF-MVP-258：AT文本动作串行mutation fanout

`SetTextSelection`依次mutation caret、anchor、focus，再mutation composition start/end/text/restore text，共7次。`SetValue`和`ReplaceSelectedText`先mutation正文，再执行同一selection/composition同步，最多8次；每次都可能做property查找、binding report clone/format、dirty/invalidation与诊断String。EditorUI03需发布atomic typed text edit-state patch，一次校验和提交正文/selection/composition，只形成一份binding report、dirty union与component event；unchanged字段不写。

## 责任计划与验收

EditorUI01收到route/timer与accessibility generation/action两份failure，EditorUI03收到text mutation fanout failure。100/1k/10k route depth、armed timers、tree nodes、text chars规模记录route clone bytes、timer visited/due/age、tree/ancestor/child visits、node/String clone、AccessKit changed nodes、action snapshot build、mutation/binding/dirty count与CPU p50/p95。current-source Cargo、产品pointer/keyboard/assistive technology trace及AccessKit/IME行为矩阵完成前，59/59仍留pending。

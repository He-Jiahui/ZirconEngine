---
related_code:
  - zircon_runtime_interface/src/ui/event_ui
  - zircon_runtime/src/ui/event_ui/manager
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - zircon_runtime/src/ui/tests/event_manager.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - current-source Windows event/control tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI event/control 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/event_ui/**`当前源 **4/4** 个 Rust 文件、**732** 行已逐文件阅读，并反查runtime manager注册、invoke、query和broadcast；runtime 3条event-manager tests及interface control/reflection serde合同为当前功能基线。目录当前无工作区改动，本轮未修改源码。

## 性能结论

- `UiRouteId`/`UiSubscriptionId`是Copy句柄，`InvokeRoute`已经提供不依赖native String的产品入口，正向支持PERF-MVP-572的typed route hard cut。
- `UiInvocationContext`拥有完整binding和arguments；`UiInvocationResult`再次拥有binding与JSON Value，`UiNotification::Invocation`再包一层。runtime当前success/failure/broadcast会复制这些递归payload，继续归 **PERF-MVP-572**；订阅fanout的无界queue、死sender与每subscriber clone继续归 **PERF-MVP-252**。
- `UiControlResponse::{Tree,Node,Property}`返回全owned reflection DTO；node同时拥有class/display、children、properties/actions表，每property还可同时拥有resolved/authored/default递归`UiValue`。全树query/fanout不得进入普通输入帧，generation-owned snapshot/delta继续归 **PERF-MVP-278/456**。
- `UiBindingCodec`只是native format/parse薄包装，没有施加bytes/args/nodes/depth/string预算；其hard limits已纳入572，不能再由上层各自补一套不一致限制。
- `with_property`/`with_action`为BTreeMap key复制name/action id，属于snapshot构造期常数复制；在generation artifact只建一次的前提下不单独立项。

## 交接验收

1. Runtime09在route generation内共享binding/default arguments/result payload；normal invoke只传`UiRouteId`/handle，binding/JSON owner=1，broadcast采用有界shared notification。
2. EditorUI01的input/route-intent直接消费route id；unknown/native codec只在外部边界物化并执行572预算。
3. EditorUI08/Runtime09的Tree/Node/Property query读取同generation reflection artifact；stable query不重建全树，diff只携changed/removed rows，订阅count/bytes/age/drop有界。
4. routes/nodes/subscribers 1/100/10k、payload 0/1KiB/1MiB、events 1M记录binding/JSON/String clone bytes、snapshot builds、fanout copies、queue bytes/age与p95；保留3条event-manager、serde、error与remote-callable合同。

current-source Cargo、规模counter与F4 control/diagnostics产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。

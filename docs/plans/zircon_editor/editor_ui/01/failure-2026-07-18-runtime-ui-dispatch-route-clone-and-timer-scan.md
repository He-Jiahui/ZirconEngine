---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-dispatch-route-clone-and-timer-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
  - zircon_runtime/src/ui/dispatch/input_manager/timers.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/surface/navigation/route.rs
---

# Runtime UI owned route clone与timer全扫

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/dispatch` 13/13与`platform_input` 3/3
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：Runtime12已拥有跨系统input retention/coalescing；EditorUI01仍需验证UI batch不绕过频率语义。
- 交接原因：统一input manager、route context与timer owner属于EditorUI01 M2/M4。

## 失败现象与复现证据

PERF-MVP-254：pointer/navigation route包含多组node Vec，dispatch会在result、candidate与逐node context间深copy。pointer同node多handler原先还逐handlerclone，本轮已TDD降到每node/phase一次。PERF-MVP-255：每tick独立全扫四个timer BTreeMap并clone due String，再逐项dispatch。

## 最低共享层根因

route/result/context都要求owned DTO，没有一份event-lifetime共享route；timer按功能拆成四份以target排序的表，没有统一deadline authority、generation取消和frame budget。

## 架构修复验收

- handler context借用或Arc共享单一route，result move/共享同一payload；候选遍历不得clone stacked/bubbled/root slices。
- 1/10/100 depth×1/4 handlers记录route clone count/bytes、Vec alloc、visited/candidate copy与CPU p95；handler数不增加clone bytes。
- timer使用统一deadline queue/wheel，`target+kind+generation`支持replace/cancel；tick无due为O(1)，due近O(K log T)。
- due dispatch有count/time budget、age/fairness与deferred计数；同deadline次序稳定，stale entry不触发。
- capture/preview/direct/bubble/passthrough、tooltip/submenu/typeahead/toast/double-click、saturation与current-source Cargo/产品trace通过。

## 禁止临时方案

- 不得只reserve route Vec或把BTreeMap改HashMap而保留深copy/全扫。
- 不得每timer kind各建私有heap；deadline与预算必须统一。
- 不得通过丢弃pointer edge或延迟所有timer破坏输入/弹窗语义。

## 修复结果与回传

Open state: `等待EditorUI01回传shared route context、unified deadline timer、规模counter、current-source Cargo与产品input trace`。

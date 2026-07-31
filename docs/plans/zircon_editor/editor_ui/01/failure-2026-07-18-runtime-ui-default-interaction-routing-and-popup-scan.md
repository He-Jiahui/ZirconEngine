---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-default-interaction-routing-and-popup-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/popup.rs
  - zircon_runtime/src/ui/surface/popup_stack.rs
tests:
  - route depth by behavior-count dispatch probe test
  - popup outside-click arranged-node visit test
  - current-source pointer routing Cargo and product trace
---

# Runtime UI default interaction route探测与popup全扫

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface default interactions与event routing 18/18
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：EditorUI06提供compiled behavior descriptor；EditorUI02提供indexed arranged/hit path。
- 交接原因：event-lifetime route ownership、input dispatch与popup routing属于EditorUI01。

## 失败现象与复现证据

PERF-MVP-283：default interaction按多个控件种类串行探测，并重复用字符串识别binding/component event；popup primary release逆序扫描全部arranged nodes。本轮已删除routing前完全未使用的一份hover path clone，但主要分发和popup成本未变。

## 最低共享层根因

route没有event-lifetime共享authority，owner也没有compiled behavior mask；popup stack与hit path没有合成可直接判定outside-click的索引契约。

## 架构修复验收

- 一次event只构造/共享一份route payload，按owner behavior mask直接dispatch，handler数不增加route clone bytes。
- popup outside-click使用popup stack与indexed hit/ancestor path，不遍历全部arranged nodes。
- 1/16/64 route depth、1/8 behaviors、1/100 popups连续100k events记录clone bytes、behavior/string probes、arranged visits与CPU p95。
- capture/preview/bubble/passthrough/modal/nested popup、窗口失焦与current-source Cargo/产品trace通过。

## 禁止临时方案

- 不得只reserve route Vec或把串行probe换成HashMap而继续每event字符串匹配。
- 不得在popup模块建立第二份arranged索引；必须消费layout/hit authority。

## 修复结果与回传

Open state: `等待EditorUI01回传shared route、compiled behavior dispatch、popup indexed outside-click与规模证据`。

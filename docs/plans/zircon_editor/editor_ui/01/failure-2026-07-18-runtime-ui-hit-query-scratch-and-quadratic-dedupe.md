---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-hit-query-scratch-and-quadratic-dedupe
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/arranged.rs
tests:
  - exact-query zero-intermediate-candidate-Vec guard
  - dense-radius near-linear dedupe counter
  - 120-Hz pointer hit-query allocation and CPU trace
---

# Runtime UI hit query临时分配与二次去重

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`hit_grid.rs`与`tree/hit_test.rs`命中调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：EditorUI02提供dense arranged/input-policy index。
- 交接原因：hit query scratch、cursor-radius策略和pointer frame budget属于input dispatch；arranged结构索引不能在input层私建。

## 失败现象与复现证据

PERF-MVP-316：每query原先分配五个Vec；radius跨cell以`Vec::contains`去重，密集候选最坏O(K²)。本轮已TDD让radius=0借用单cell，删除四个中间candidate Vec；radius query、linear arranged get和ancestor input-policy仍未解决。

## 最低共享层根因

hit grid没有generation-owned query scratch/dense visit mark，arranged tree也没有O(1) node index与继承input-policy；每次高频pointer move只能重新收集、去重、排序和走祖先。

## 架构修复验收

- radius query用generation-stamped dense mark/scratch或等价近线性去重，不以全grid clear替代。
- frame-local scratch受entry/byte预算且不跨并发query泄漏；exact path保持零中间candidate Vec。
- EditorUI02 index使arranged get近O(1)、input policy不逐candidate走祖先。
- 1/100/1k/10k overlapping entries、radius 0/8/64、120 Hz记录probes/sort/alloc/CPU p95；route/z/paint/scope/world parity与Cargo/F4 trace通过。

## 禁止临时方案

- 不得用每query新建HashSet替代Vec而不记录allocation/byte预算。
- 不得关闭cursor-radius fallback或改变exact-hit优先级来降低成本。

## 修复结果与回传

Open state: `exact borrowed-cell止损已实现，等待EditorUI01/02回传radius scratch、dense arranged index与产品规模证据`。

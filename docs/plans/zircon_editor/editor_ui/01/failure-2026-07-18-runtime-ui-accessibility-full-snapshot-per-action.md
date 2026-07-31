---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-accessibility-full-snapshot-per-action
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/state.rs
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/accesskit.rs
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility
  - zircon_runtime/src/ui/tests/accessibility_widget_actions.rs
  - zircon_runtime/src/ui/tests/accessibility_widget_actions
  - zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs
  - zircon_runtime/src/ui/tests/accessibility_state_values.rs
  - zircon_runtime/src/ui/tests/accesskit.rs
---

# Runtime UI accessibility全树snapshot与逐action重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/accessibility` 43/43及surface产品调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：EditorUI02提供tree/layout generation；EditorUI03接收text edit-state atomic patch。
- 交接原因：accessibility event、action route、focus与platform update属于EditorUI01统一输入边界。

## 失败现象与复现证据

PERF-MVP-256：每snapshot两遍全tree且两次逐node祖先回溯，之后多轮relation/name/child/diagnostic/AccessKit重建。生产validator深clone全部node DTO已直接改为index。PERF-MVP-257：每个assistive action为单target重建/校验完整snapshot，原target node深clone已直接删除。测试切片51个用例中仍有44次`accessibility_snapshot()`调用，尚无stable generation零构建或action无关节点零访问计数。

## 最低共享层根因

accessibility projection不是tree/layout/component/focus generation拥有的缓存，而是即时全量函数；action validation与platform TreeUpdate也没有共享node id index、included/hidden/action contract或changed set。

## 架构修复验收

- generation提交时维护`node_id→accessible node/action contract`、effective hidden、relations和included child adjacency；stable generation snapshot/update=0。
- property/focus变化只重建changed nodes；结构变化限制到affected subtree。AccessKit只发送changed nodes，node id稳定。
- action按node id近O(1)查同generation contract，不构建snapshot、不访问无关node；generation mismatch显式拒绝或重试。
- 100/1k/10k nodes、depth 1/16/64记录tree passes、ancestor/child visits、DTO/String/Vec clone、validator copies、AccessKit update nodes、action lookup与CPU p95。
- relation/description/name、hidden focus fallback、duplicate/dangling diagnostics、stale/excluded/disabled action、multi-root、current-source Cargo与真实AT通过。

## 禁止临时方案

- 不得缓存全snapshot但每action仍clone整份或线性查找。
- 不得在release关闭diagnostics以掩盖全树复制；validator必须消费索引/changed set。
- 不得用不受tree/layout/component/focus generation失效约束的全局cache。

## 修复结果与回传

Open state: `等待EditorUI01联动EditorUI02回传generation-owned accessibility projection、dirty TreeUpdate、O(1) action contract、规模counter与current-source Cargo`。

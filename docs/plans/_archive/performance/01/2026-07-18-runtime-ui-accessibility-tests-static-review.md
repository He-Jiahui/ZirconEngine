---
related_code:
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
tests:
  - 7 tracked Rust files and 51 test definitions statically reviewed
  - 44 accessibility snapshot calls and one 8-report/16-update text action identified
  - current-source Cargo, large-tree counters, and real AT validation pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI accessibility tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`tests/accessibility.rs`与`tests/accessibility/**` 7/7个tracked Rust文件、2,280行、51个测试。累计UI tracked source从505/783增至512/783。覆盖extraction、name/description relations、focus diagnostics、activate/value/text actions、hidden/excluded/stale/disabled状态与cycle诊断。

测试源码共调用44次`accessibility_snapshot()`；单节点action通常在action前后再次全量snapshot验证。现有源码守卫确认validator用`node_id→index`而不深clone node，action借用snapshot target而不clone，但没有证明stable generation snapshot为零、action近O(1)或AccessKit update只含changed nodes。

`accessibility_set_value_updates_editable_text_property`明确显示单次文本SetValue产生8份binding reports、合计16个updates，并为text、caret、selection和四个composition字段分别生成diagnostic/dirty mutation。这是PERF-MVP-258 atomic text patch handoff的可执行基线；当前render-only range/toggle action断言也应保留。

## 责任与验收

PERF-MVP-256/257与EditorUI01负责generation-owned accessibility projection、changed-node TreeUpdate和O(1) action contract；PERF-MVP-258与EditorUI03负责单次typed text-state transaction。100/1k/10k nodes、depth 1/16/64与连续1k actions记录tree passes、snapshot/action lookup、relation visits、DTO/String clone、AccessKit update nodes、mutation/report/update/dirty counts及CPU p95：stable snapshot/update=0、action不扫无关node、text action transaction/report/dirty commit各≤1。current-source Cargo与真实AT通过前保持pending。

---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - interaction priority and asset/content surface style tests
  - current-source Windows Cargo pending
  - 1/100/10000 state-resolution/theme-access trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template style逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_style.rs`、`template_style/**`与`template_style_tests/**`共 **18/18** 个Rust文件、**795** 行已逐文件阅读。覆盖surface/border/text color、interaction/severity/variant、dimension/elevation/overlay/surface-role与完整priority/asset-surface tests。当前源Cargo与规模trace未完成，因此仍留在`pending.md`。

## 判定与热点

所有函数均为有界match、布尔判断或几何算术，无分配、I/O、队列或无界算法。重复成本来自调用组合：同一node在surface、border、text、border width与elevation路径多次进入`button_interaction_state -> style_selector::resolved_state_for_node`；border helper的asset exceptions还可再次求state。Typed background/tone/style-role helper会各自取得palette，放大PERF-MVP-161的per-node theme访问。

PERF-MVP-161要求frame只取得一次immutable theme snapshot；PERF-MVP-178要求changed node一次解析resolved style并写入compiled paint segment，surface/border/text/dimensions/elevation共用该结果。不得给每个color helper建立独立cache或跳过shared state priority。

## 动态验收

在1/100/10,000 normal/hover/pressed/focused/loading/disabled与asset/content nodes上记录resolved-state calls、palette/metrics accesses、compiled-style builds和command count。局部changed-node gate要求interaction resolve≤1；最终stable generation这些计数为0且frame theme lock近常数。保持disabled>loading>pressed/focused>hover>normal priority、declared typed colors、asset thumbnail/content/preview exceptions、border width/elevation与pixels一致。

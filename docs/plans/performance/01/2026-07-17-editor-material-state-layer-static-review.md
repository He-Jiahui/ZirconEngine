---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - material state priority and ripple geometry/command tests
  - idle no-theme-read regression test pending
  - current-source Windows Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor material state layer逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`material_state_layer.rs`、`material_state_layer/**`与`material_state_layer_tests/**`共 **9/9** 个Rust文件、**425** 行已逐文件阅读。State priority与ripple几何均为有界算术，无队列、I/O或容器增长；当前源Cargo与idle/theme counter未完成，因此仍留在`pending.md`。

## 热点与直接修复门

`push_state_layer_commands`首先调用`state_layer_color(node)`，其fallback会取得全局palette `RwLock`；之后才判断`state_layer_opacity`是否存在以及ripple是否enabled/pressed。普通idle node、禁用state-layer且无ripple node因此会读取主题但不生成任何命令。该入口被template button和generic node surface调用，成本按可见节点、按帧放大。

PERF-MVP-189是低风险直接修复：先计算overlay opacity与ripple eligibility；两者均无工作时立即返回，只有实际将生成overlay/ripple command时才取得一次color。测试需冻结idle/disabled/no-ripple零command且零theme read、overlay-only/ripple-only/both各一次theme read及现有state priority/ripple geometry。最终stable generation仍由PERF-MVP-178/182 compiled style消除整个调用。

## 动态验收

1/100/10,000 idle buttons/nodes的state-layer theme reads与commands均为0；hover/focus/press/drag/ripple changed node theme read<=1。保持disabled/pressed/drag/focus/hover priority、ripple clip/origin/diameter、opacity、order及GPU/Softbuffer pixels一致。Current-source Cargo通过前不进入`review.md`。

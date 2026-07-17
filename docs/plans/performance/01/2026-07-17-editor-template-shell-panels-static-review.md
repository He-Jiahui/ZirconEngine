---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels_tests/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - shell panel identity/state/frame/separator/pixel tests
  - current-source Windows Cargo baseline failed on unrelated source guards
  - stable shell theme/metrics/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template shell panels逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_shell_panels*`与tests共 **16/16** 个Rust文件、**683** 行已逐文件阅读。覆盖shell kind identity、chrome style、content frame metrics、surface/directional separators及identity/state/pixel tests。Current-source baseline的本组测试未失败，但全批次有2个无关source-guard漂移，stable产品trace尚未完成，因此仍留在`pending.md`。

## 结论

未发现新的独立性能瓶颈。每节点只执行一次chrome selector；非content panel不读取frame metrics，content panel仅分别取得border width和radius两次metrics projection。命令数量由kind固定：通常为surface加0–2条separator，算法和分支均为O(1)。

现有改进仍适用：PERF-MVP-161让frame统一借用theme/metrics snapshot，PERF-MVP-178让stable presentation复用compiled segment，从而将content panel的两次metrics和stable surface/separator build归零。无需为本组新增问题编号。

## 动态验收

在完整MVP shell的stable/hover/focus/loading/theme-switch上记录selector、theme/metrics acquisition、surface/separator command build和damage命中。Stable generation以上全部为0；changed panel selector/theme/metrics各<=1。保持kind mapping、content frame、separator direction、state priority、clip/order和GPU/Softbuffer pixels parity后才能移入`review.md`。

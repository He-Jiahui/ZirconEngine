---
related_code:
  - zircon_runtime/src/ui/tests/accessibility_widget_actions.rs
  - zircon_runtime/src/ui/tests/accessibility_widget_actions
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - 4 tracked Rust files and 11 test definitions statically reviewed
  - 12 accessibility snapshot calls identified
  - changed-node update counters and real AT validation pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI accessibility widget action tests逐文件性能静态审查（2026-07-18）

本批逐文件完整阅读`tests/accessibility_widget_actions.rs`与其目录4/4个tracked Rust文件、818行、11个测试。累计UI tracked source从512/783增至516/783。覆盖disclosure、popup/dialog/menu、tooltip与menu-item action的runtime alias、binding和host effect语义。

测试中有12次`accessibility_snapshot()`调用，常在单action前后重建并clone目标node；没有stable generation snapshot、changed-node TreeUpdate、action lookup或popup/tooltip state probe计数。继续回链PERF-MVP-256/257与EditorUI01 generation-owned accessibility projection；popup/tooltip bounded state联动PERF-MVP-297。

验收用1/100/10,000 accessible widgets与连续1k expand/collapse/dismiss/activate记录snapshot builds、action lookup、changed nodes、binding/effect owners、state probes和CPU p95：stable snapshot=0、single action不扫无关node、AccessKit只发changed nodes，popup/menu/tooltip语义保持。current-source Cargo与真实AT完成前保持pending。

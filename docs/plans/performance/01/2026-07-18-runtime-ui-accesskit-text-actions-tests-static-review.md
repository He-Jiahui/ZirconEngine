---
related_code:
  - zircon_runtime/src/ui/tests/accessibility_disabled_gate.rs
  - zircon_runtime/src/ui/tests/accessibility_state_values.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/accesskit.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
tests:
  - 4 tracked Rust files and 25 test definitions statically reviewed
  - full AccessKit update and 8/16 or 7/14 text mutation fanout baselines identified
  - delta update, atomic patch, Cargo, and real AT validation pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI AccessKit/text action tests逐文件性能静态审查（2026-07-18）

本批逐文件完整阅读`tests/{accessibility_disabled_gate,accessibility_state_values,accessibility_text_input_actions,accesskit}.rs` 4/4个tracked Rust文件、1,824行、25个测试。累计UI tracked source从516/783增至520/783。

AccessKit测试只覆盖完整`UiAccessibilityTreeSnapshot→TreeUpdate`，并显式断言全量node count/role/action/value/relation；没有changed-node update或stable generation零输出测试。disabled/state测试继续通过即时snapshot检查继承状态。text action测试多次明确断言SetValue/Replace产生8 reports/16 updates，SetTextSelection产生7 reports/14 updates，证明当前每字段通用mutation fanout不是偶发路径。

PERF-MVP-256/257与EditorUI01需添加generation/delta AccessKit gate；PERF-MVP-258与EditorUI03需把上述8/16、7/14基线收敛为每action单一typed transaction/report/dirty commit。100/1k/10k nodes与1/10k/100k chars记录TreeUpdate nodes/bytes、snapshot builds、mutation/report/update count、String bytes与CPU p95；stable update=0、single-node action只发changed node、text action transaction/report≤1。current-source Cargo与真实AT完成前保持pending。

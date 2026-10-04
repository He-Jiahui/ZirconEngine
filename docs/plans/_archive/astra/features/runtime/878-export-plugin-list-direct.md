---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/07/2026-09-21-export-plugin-list-direct.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
tests:
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template/direct_string_vec_tests.rs
  - tools/tests/test_runtime878_export_plugin_list_direct_performance_contract.py
---

# Runtime878 Export Plugin List Direct

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime07 generated export plugin list | Append each debug-quoted plugin name and positional separator to one final generated expression instead of allocating a child per plugin and a vector; preserve escaping, order, Unicode, and empty input. | Combined RED `2/7` plus three missing-module errors -> GREEN `7/7`; adjacent `31/31`. The 4,096-template/32-plugin model removes `131072` child strings and vector slots. Lower parity and ignored 101-pair `RUNTIME878_EXPORT_PLUGIN_LIST_DIRECT_BENCH_V1` wired. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs` | `05FC9E1A5DA9E2A56C467B91BEF822F2825E35C6517D22C6F07FBE3DE8360FDB` |
| `zircon_runtime/src/plugin/export_build_plan/plugin_selection_template/direct_string_vec_tests.rs` | `6C603D2EAD9B88A7A6BB8303A2DD9D8772C3A54E8C516F0A00E1DD31FF3A7789` |
| `tools/tests/test_runtime878_export_plugin_list_direct_performance_contract.py` | `09BBE3270D6ADBBDCD80708647CC179C33F9065D23E8CD7F0854C706B4330629` |

## Managed gate

Runtime878 was submitted with Editor897 in combined current-source v25 (PID
`28484`) at `2026-09-21T23:28:01.8534392+08:00`. A one-time bounded failure
diagnosis after independent documentation found Runtime source synchronization
failed `compile_input_changed` on a foreign-modified export-archive file; no
Rust source error is attributed. Managed current-source Runtime compilation,
lower/ignored Release tests, allocator measurement, and generated export
product p50/p95/p99 remain pending.

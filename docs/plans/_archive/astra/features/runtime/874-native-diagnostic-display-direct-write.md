---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/07/2026-09-21-native-diagnostic-display-direct-write.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics.rs
tests:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics/display_direct_write_tests.rs
  - tools/tests/test_runtime874_native_diagnostic_display_direct_write_performance_contract.py
---

# Runtime874 Native Diagnostic Display Direct Write

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime07 typed native-plugin behavior errors | Write owned diagnostics directly into the caller's formatter with positional `; ` delimiters, eliminating the full joined child while preserving empty/singleton/multi-item order and exact text. | Combined intentional RED `2/10` → GREEN `10/10`; lower legacy parity and ignored `RUNTIME874_NATIVE_DIAGNOSTIC_DISPLAY_DIRECT_WRITE_BENCH_V1` are wired with equal preallocated destinations. The 4,096-display model changes join outputs `4096→0`; adjacent combined static coverage passes `101/101`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics.rs` | `0DC80F7431D6F9F6F5055365D16A7165B9C1208FAA34CCAD66CF398DCE5E13E0` |
| `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics/display_direct_write_tests.rs` | `88D83CBF86810EC428F3658D4825949FA508D7CFA8E414E163A8DA3FDD3B3E57` |
| `tools/tests/test_runtime874_native_diagnostic_display_direct_write_performance_contract.py` | `61A6DA78A1983FF78B0513979987B4427EB6EBAB0AD96D27447F2AE488D947D8` |

## Managed gate

Runtime874 was submitted with Editor893 in asynchronous v20 (PID `15628`) at
`2026-09-21T22:24:02.7781628+08:00` rather than receiving a per-task Cargo run.
Keep it pending until that combined Windows lane supplies current-source
Runtime compilation, lower/ignored Release execution, allocator evidence, and
native-plugin behavior-error product p50/p95/p99 evidence.

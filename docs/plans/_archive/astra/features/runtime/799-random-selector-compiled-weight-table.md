---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/22/2026-09-18-random-selector-compiled-weight-table.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/random_selector_weights.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs
tests:
  - zircon_plugins/ai/runtime/src/behavior_tree/compile/random_selector_weights_tests.rs
  - tools/tests/test_runtime_random_selector_weight_table_performance_contract.py
---

# Runtime799 · random-selector compiled weight table

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime22/08F selector hot path | Compile ID-first and positional-fallback weights once per `RandomSelector`; borrow the table during ticks and retain a defensive fallback. | TDD source contract `4/4`; lower order/precedence/clamping regression; ignored `RUNTIME799_RANDOM_SELECTOR_WEIGHT_TABLE_BENCH_V1`; deterministic model `1 allocation + 1,048,576 probes -> 0 + 0` for 1,024 children. The v8 `f32 < &f32` compile failure is repaired by explicit borrowed-weight dereference and locked by the same contract; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_plugins/ai/runtime/src/behavior_tree.rs` | `01E74779277259B874857BD8EA770F140CDB84FA19637CD708B1434E1A58E61F` |
| `zircon_plugins/ai/runtime/src/behavior_tree/compile.rs` | `8DF3A362C8AFA227996BF16E86F1CA6FB88E03A2B4895FDC70A5AEAE2E961307` |
| `zircon_plugins/ai/runtime/src/behavior_tree/random_selector_weights.rs` | `E14701CFDB0006CAC2282ABB5EFB566E9AED2AC36C5A6D7C2564B4004FCD40FB` |
| `zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs` | `48551A5EF4330688D18B577585D2886990B3C04F941D94588BF0214E76694A20` |
| `zircon_plugins/ai/runtime/src/behavior_tree/compile/random_selector_weights_tests.rs` | `755243D86B111219E236FDDB53E9C4AA2A9F90D97F5D87F110CF4FA09BE77B69` |
| `tools/tests/test_runtime_random_selector_weight_table_performance_contract.py` | `3BDC41A47C0248AE69BC670E8AEF9BC0A519EF580030ADFA03C3732D56B721CB` |

Fingerprints were checked after the final local batch and are not a substitute
for managed Cargo or Release evidence.

## 性能与受管验证边界

The compiled table removes repeated parameter scans and tick-local weight-vector
allocation while preserving current selection semantics. Keep this record
`implemented_pending_validation` until the asynchronous managed batch supplies
the lower Rust result, ignored Release marker, and product selector p50/p95/p99
measurements. Runtime22's broader random authority migration remains a separate
open plan; tooling stays deferred.

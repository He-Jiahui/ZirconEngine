---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74/2026-09-21-duplicate-control-detail-single-buffer.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/control_scope.rs
tests:
  - zircon_runtime/src/ui/template/asset/compiler/control_scope/duplicate_detail_single_buffer_tests.rs
  - tools/tests/test_runtime873_duplicate_control_detail_single_buffer_performance_contract.py
---

# Runtime873 Duplicate Control Detail Single Buffer

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime74 UI template duplicate-control validation | Borrow sorted `BTreeMap` keys and append them into one exactly sized invalid-document detail, eliminating key clones, vector slots, joined child output, and outer format while preserving detection, order, delimiters, and attribution. | Combined intentional RED `2/10` → GREEN `10/10`; lower legacy parity/exact-capacity coverage and ignored `RUNTIME873_DUPLICATE_CONTROL_DETAIL_SINGLE_BUFFER_BENCH_V1` are wired. The 4,096-detail model changes child strings/vector slots/join outputs `262144/262144/4096→0/0/0`; adjacent combined static coverage passes `140/140`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/compiler/control_scope.rs` | `5A69499BC044CA772E0A9F6379D84CDF55987BAC5F223AB2F0E58FCF7D968777` |
| `zircon_runtime/src/ui/template/asset/compiler/control_scope/duplicate_detail_single_buffer_tests.rs` | `DC2E7DAC05D1E043A299D1E3B0797A5E3B5822F1B79986E1298AD14EB9539F7E` |
| `tools/tests/test_runtime873_duplicate_control_detail_single_buffer_performance_contract.py` | `ECDFD0A9BF1327819700C5E0A1FF45D5AFA89427C6BF7797EFF1AAA2EAD4B898` |

## Managed gate

Runtime873 was submitted with Editor892 in asynchronous v19 (PID `32536`) at
`2026-09-21T22:14:16.6877626+08:00` rather than receiving a per-task Cargo run.
Keep it pending until that combined Windows lane supplies current-source
Runtime compilation, lower/ignored Release execution, allocator evidence, and
invalid-template compilation product p50/p95/p99 evidence.

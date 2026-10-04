---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02/2026-09-21-close-prompt-details-direct-append.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text.rs
tests:
  - zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text/direct_append_tests.rs
  - tools/tests/test_editor888_close_prompt_details_direct_append_performance_contract.py
---

# Editor888 Close Prompt Details Direct Append

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor02 retained close prompt | Append the optional Active Scene label and at most three dirty titles directly into the final details string. Position-based separators preserve empty authored titles, order, cap, and overflow suffix while removing the temporary borrowed-name vector. | Intentional RED `1/5` → GREEN `5/5`; lower empty/prefix/empty-title/cap/overflow parity and ignored `EDITOR888_CLOSE_PROMPT_DETAILS_DIRECT_APPEND_BENCH_V1` are wired. The 4,096-render model changes temporary reference slots `12288→0`; the Editor888/Runtime869 pair passes `10/10` and adjacent Editor behavior adds `18/18`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text.rs` | `938EB0704E3C0FF8404E7100495E0CF0741AF1559329F09FB4FF5E71EC25AD57` |
| `zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text/direct_append_tests.rs` | `66BA322FAADA11F0AB7C5E6423A106A294BF72CA3D33C5A09DA386C0878749E5` |
| `tools/tests/test_editor888_close_prompt_details_direct_append_performance_contract.py` | `DC21EE627E7C090580205F55E45884E0EA6599417DEDA286A23D7ABA6881D584` |

## Managed gate

Editor888 was submitted with Runtime869 in asynchronous v15 (PID `15424`) at
`2026-09-21T21:14:35.8222835+08:00` rather than receiving a per-task Cargo run.
Keep it pending until that combined Windows lane supplies current-source Editor
compilation, lower/ignored Release execution, allocator evidence, and
close-prompt product p50/p95/p99 evidence.

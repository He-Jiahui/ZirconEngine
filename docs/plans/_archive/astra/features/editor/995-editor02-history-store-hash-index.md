---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02/2026-08-26-history-store-hash-index.md
related_records:
  - docs/plans/astra/features/editor/992-editor02-document-transaction-optimization-completion-list.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/editing/engine/transaction/engine_state.rs
  - zircon_editor/src/core/editing/engine/transaction/engine_state/hash_index_tests.rs
tests:
  - zircon_editor/src/core/editing/engine/transaction/engine_state/hash_index_tests.rs
---

# Editor995 · history-store hash index

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor02 transaction history | Resolve global/document/play-session `HistoryStore` owners through a `HashMap<HistoryContextId, HistoryStore>` while retaining the generation owner as an ordered map. Commit, undo/redo, save-token checks, teardown, and journal ordering remain unchanged. | Focused tests cover independent context ownership, removal, and generation-order preservation. The ignored marker `EDITOR02_HISTORY_STORE_HASH_INDEX_BENCH_V1` requires hash lookup P95 at least 30% below the legacy ordered path with zero hit-path allocations. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/editing/engine/transaction/engine_state.rs` | `08BE96091BDDB02081D3718148485DF62E67B16BEA6E9B4EEBDE79A211FF1FBE` |
| `zircon_editor/src/core/editing/engine/transaction/engine_state/hash_index_tests.rs` | `A8EA6D22861F48CD6A6F6BCA10086D181F916E89DD681BD66566CEFA8ACFDC9D` |

## Validation handoff

The three focused history contracts and Release marker are included in the
grouped Editor package validation; exact managed P50/P95 receipts remain pending.

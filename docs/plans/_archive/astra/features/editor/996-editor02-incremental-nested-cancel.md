---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02/2026-08-26-incremental-nested-cancel.md
related_records:
  - docs/plans/astra/features/editor/992-editor02-document-transaction-optimization-completion-list.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/editing/engine/transaction/lifecycle.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
tests:
  - zircon_editor/src/core/editing/engine/transaction.rs
---

# Editor996 · incremental nested transaction cancellation

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor02 transaction lifecycle | Validate the target before taking the edit context, then pop authoritative frames one at a time after each mutex guard is dropped. On failure only the current frame is restored; already canceled descendants remain removed, preserving reverse order and fault typing. | The source contract and existing nested-success/revert-failure oracles pass. The model removes one tail allocation per cancellation (`1→0`), has no P50 regression, and keeps optimized P95 within the 15% allowed variance. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/editing/engine/transaction/lifecycle.rs` | `D1B72AD59F1648C620D7556719C37B823184D580AF155FB7D78B84662EDA12C3` |
| `zircon_editor/src/core/editing/engine/transaction.rs` | `4AD3A4B2D7593858AE4AB857346DFB0D28F9F88BA91882AB871B62D0FE8A1F3B` |

## Validation handoff

The complete transaction-engine test group, source contracts, and model marker
join the grouped Editor package validation. Managed Cargo and Release evidence
remain pending.

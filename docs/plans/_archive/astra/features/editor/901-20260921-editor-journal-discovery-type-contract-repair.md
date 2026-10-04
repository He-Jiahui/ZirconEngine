---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/editing/engine/journal/mod.rs
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/tests/editing/transaction_engine/durable_journal.rs
---

# Editor901 Durable Journal Discovery Type Contract Repair

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Durable discovery result contract | Re-export the already-public `JournalDiscoveryReport`, `JournalDiscoveryEntry`, and `JournalDiscoveryIssue` through the existing journal and editing-engine boundaries, so consumers can name the typed result of `DurableJournal::discover()` and inspect a journal-only discovery issue. | v27 managed Editor library check reported the `JournalDiscoveryIssue` test import as inaccessible despite the type already being published by the inner durable module. No discovery behavior was changed. | implemented_pending_validation |
| Healthy journal tests | Test for no tail fault with `is_none()` in three assertions; do not impose `PartialEq` on the structured `JournalTailFault` and its nested errors solely for the test macro. | v27 reported three `Option<&JournalTailFault>` versus `None` equality errors; the fault payload assertions stay unchanged. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/editing/engine/journal/mod.rs` | `5BD0CE72EEDFA204CABE394BDC53813E27DC3DDC21F60A7753C39183BEBA6CF3` |
| `zircon_editor/src/core/editing/engine/mod.rs` | `9C5C8A0FFFB25FDD8AB24A96AAE0E8EA01EEACB5041399E456533BD6EBD47818` |
| `zircon_editor/src/tests/editing/transaction_engine/durable_journal.rs` | `5B2B11B5FF65923E073A503A0B3964797A149531AE6AE6572EF31FA360FBA84F` |

## Managed gate

The four cited diagnostics come from the terminal v27 Editor check before
these repairs. Exact Rustfmt and scoped diff checks pass, but no Rust test or
source-bound product run has followed; group a later managed Runtime/Editor
batch after additional owner-scoped shared contract convergence. No Release
performance, allocation, or product percentile target has been accepted.

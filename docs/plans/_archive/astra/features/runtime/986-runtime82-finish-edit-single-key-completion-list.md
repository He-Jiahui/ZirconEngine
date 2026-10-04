---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-26-finish-edit-single-key.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/dispatch/input_manager/text_document_session/session.rs
tests:
  - zircon_runtime/src/ui/dispatch/input_manager/text_document_session/session/tests.rs
---

# Runtime82 finish edit single key completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Document edit finalization key reuse | One owned tree/node key is borrowed for synchronization-error and history removal, then moved into the binding map. Changed history entry updates clone that key once. The test module retains the exact old function for comparison. | Changed/unchanged history and receipt regressions; grouped managed Runtime compile/test; `RUNTIME82_FINISH_EDIT_SINGLE_KEY_BENCH_V1` Release P95 at most 90% of the old path. | implemented_pending_validation |

The source and focused tests are ready for the grouped Runtime validation
manifest. Static checks and pending receipts do not count as Cargo or Release
timing acceptance.

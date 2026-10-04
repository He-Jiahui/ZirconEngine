---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-model-restore-committed-history.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/dispatch/input_manager/bound_text_model_updates/transaction.rs
tests:
  - zircon_runtime/src/ui/dispatch/input_manager/bound_text_model_updates/tests.rs
---

# Runtime82 model restore committed history completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Retain document identity and Undo when a model update only restores committed source after Preedit | The text-equal branch preserves the committed epoch while applying display/composition changes. Two manager regressions cover SetText and LoadText restoration, valid receipt keys, prior Undo, and unchanged real replacement barriers. | Managed Runtime lib execution is pending with Batch M. Native latency, memory, and Unreal product gates remain open. | implemented_pending_validation |

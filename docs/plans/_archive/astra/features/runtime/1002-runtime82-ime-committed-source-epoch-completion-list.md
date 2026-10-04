---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-ime-committed-source-epoch.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/text/document/store.rs
  - zircon_runtime/src/ui/dispatch/input_manager/text_document_session/session.rs
  - zircon_runtime/src/ui/surface/input/editable_text.rs
  - zircon_runtime/src/ui/surface/input/editable_text/mutation.rs
  - zircon_runtime/src/ui/surface/input/editable_text/property_transaction.rs
tests:
  - zircon_runtime/src/text/document/store/committed_source_tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/text_document_session/session/committed_source_tests.rs
  - zircon_runtime/tests/runtime82_inactive_composition_regression.rs
---

# Runtime82 IME committed source epoch completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Preserve original committed text during provisional IME edits and restore active snapshots | Added lazy source restoration when opening an active snapshot, source preservation for Preedit and equal-source composition finishing, retained-piece equality without a flattened snapshot, and real source/document/history regressions. Cancellation still restores source after document admission failure; a two-byte-limit manager regression covers Cancel, physical Escape, and a diagnosed unbound Commit reset. The root-owned property writer advances committed intent epochs even when the visible value and metadata are unchanged. | Static checks only. Grouped managed Runtime behavior tests pending. Source epoch and layout/text revision remain distinct; programmatic, accessibility, and bound-model reset semantics require their existing tests. Equal-source finishing may scan the full body; its million-character budget remains open. No timing comparison against the old incorrect source behavior, latency/RSS pass, or Unreal product pass is claimed. | implemented_pending_validation |

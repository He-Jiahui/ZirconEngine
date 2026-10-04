---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-editable-text-property-prepare-borrowed-validation.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/surface/input/editable_text/property_transaction.rs
tests:
  - zircon_runtime/src/ui/surface/input/editable_text/property_transaction/tests.rs
---

# Runtime82 editable text property prepare completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Borrowed property prepare for million-character InputField edits | Removed the discarded proposed body clone and borrowed TOML/display validation while retaining typed NumberField dual-property semantics and atomic rejection. Added real keyboard dispatch, direct transaction, variant parity, and ignored Release legacy comparison tests. | Static review only. Managed Windows behavior and 5+31 Release comparison are pending. Local prepare p95 must be at most 80% of exact old prepare for 1M ASCII bytes; full edit-to-present, allocation/RSS, and Unreal gates remain open. | implemented_pending_validation |

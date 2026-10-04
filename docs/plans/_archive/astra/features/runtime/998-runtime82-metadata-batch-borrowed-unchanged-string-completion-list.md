---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-metadata-batch-borrowed-unchanged-string.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/surface/property_mutation/metadata_batch.rs
tests:
  - zircon_runtime/src/ui/surface/property_mutation/metadata_batch/optimization_tests.rs
---

# Runtime82 metadata batch borrowed unchanged String completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Caret-only metadata batch body projection | Added a borrowed String/TOML String equality check before conversion; changed and non-String values retain old mutation semantics. Added direct batch, true Surface commit, variant parity, and test-local old/new Release comparison. | Static checks only. Managed behavior and Release results pending. The local one-million-character batch p95 must be at most 80% of the old batch; owned candidate construction, full edit-to-present latency, allocation/RSS, and Unreal product gates remain open. | implemented_pending_validation |

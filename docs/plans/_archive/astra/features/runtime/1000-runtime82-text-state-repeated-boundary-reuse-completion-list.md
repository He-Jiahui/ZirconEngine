---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-text-state-repeated-boundary-reuse.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
implementation_files:
  - zircon_runtime/src/ui/surface/input/text_state.rs
tests:
  - zircon_runtime/src/ui/surface/input/text_state/optimization_tests.rs
---

# Runtime82 text state repeated boundary reuse completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Repeated Unicode offsets during owned Surface text state materialization | Added five-entry stack reuse scoped to one materialization; distinct offsets retain the shared grapheme authority. Added metadata/Unicode/CRLF/absence regressions, actual property commit projection, and full HEAD materializer Release comparison. Later Runtime82 inactive-composition repair updates the committed-surface fixture to expect no active composition, leaving three repeated caret/selection offsets in that Release path. | Static checks only. Managed behavior and 5+31 Release samples pending. Local 1M-character repeated-offset p95 must be at most 80% of the old path. Owned source copying, distinct-offset scans, full input-to-present, allocation/RSS, and Unreal comparison remain outside this slice. | implemented_pending_validation |

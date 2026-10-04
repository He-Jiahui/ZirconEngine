---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/76/2026-09-26-linear-child-visibility-single-pass.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
implementation_files:
  - zircon_runtime/src/ui/layout/pass/axis.rs
tests:
  - zircon_runtime/src/ui/layout/pass/axis/tests.rs
---

# Runtime76 linear child visibility completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Linear child visibility count | Constraint generation and layout-child counting share one child traversal, then the gap is computed before the unchanged solver call. A test-local copy preserves the exact old two-pass resolver for comparison. | Visibility/gap/error and old/new output equivalence regressions, existing linear scratch regression, managed Runtime compile/test batch, and `RUNTIME76_LINEAR_CHILD_VISIBILITY_SINGLE_PASS_BENCH_V1` release P95 at least 10% below the exact old baseline. | implemented_pending_validation |

The source and focused tests are ready for the grouped Runtime validation
manifest. Static checks and pending receipts do not count as Cargo or release
timing acceptance.

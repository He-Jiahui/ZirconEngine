---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200/2026-09-26-valid-input-owner-route-single-pass.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
implementation_files:
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/owner_route.rs
  - zircon_runtime/src/ui/surface/input/keyboard.rs
tests:
  - zircon_runtime/src/ui/surface/input/validation/visibility_first_tests.rs
---

# Runtime200 valid input owner route completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Valid focused-owner route | The owner validity walk emits the bubble route for keyboard and focused input. A 64-node inline buffer avoids route heap allocation for rejected short paths, while keyboard rejection avoids a duplicate validation walk; bool-only consumers remain allocation-free. The benchmark retains the exact HEAD parent-loop validator followed by `bubble_route`. | Exact short/deep route tests, actual keyboard rejection dispatch regression, existing focused input integration regressions, managed Runtime compile/test batch, and `RUNTIME200_VALID_INPUT_OWNER_ROUTE_BENCH_V1` release P95 at least 20% below the original two-pass baseline. | implemented_pending_validation |

The source and focused tests are ready for grouped Runtime validation. Static
checks and pending receipts are not treated as passing Cargo or release timing
evidence.

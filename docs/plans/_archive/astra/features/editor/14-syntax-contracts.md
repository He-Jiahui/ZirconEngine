status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md

# Editor syntax contract repair

## Current source and repair

The retained hit-test scratch counter used a test-only attribute on an
expression, which Rust rejects before test selection. The counter now uses a
configuration-specific helper method, retaining its test observation while
keeping the production path empty. The drawer-capacity p95 benchmark now has
one named helper per percentile contract and module-owned tab-count constants
for the measurement helper. The settings error formats both named fields
through `thiserror`'s supported named-field syntax.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M31 | Restore compile-safe test accounting, drawer benchmark symbols, and settings error formatting | implemented_pending_validation | Scoped `rustfmt --check` and static diff check passed; combined Editor batch pending |

This repair preserves the existing p95 samples and 70% performance threshold.
It does not claim performance acceptance until the managed release batch
completes.

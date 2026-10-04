status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md

# Asset route lifetime repair

## Current source and repair

Asset-content hit routing returned a `PanePointerRoute` whose target contains
only copied enum data, but the lifetime was elided despite two input
references. Rust consequently rejected the function because it could not
choose an input lifetime. The route is explicitly `'static`: its frame is
copied by `PanePointerRoute::new` and its target carries no borrowed data.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M30 | Make the asset-content pointer route lifetime explicit | implemented_pending_validation | Static diff and route ownership review complete; combined Editor batch pending |

No pointer behavior or hit-test geometry changed in this repair.

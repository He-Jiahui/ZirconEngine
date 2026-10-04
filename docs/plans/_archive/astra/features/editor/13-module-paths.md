status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md

# Editor module path repair

## Current source and repair

Three Editor modules were split into sibling files and directories while
their parent declarations still used Rust's default child-module lookup. The
compiler therefore searched under `session/`, `preview/`, and
`editor_host_event_controller/` for files that already exist at the owning
module directory. Explicit path attributes now point each declaration to
its existing owner file, preserving the current folder ownership and avoiding
duplicate module copies.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M29 | Restore explicit paths for animation route tests, preview hash tests, and play gizmo | implemented_pending_validation | Static path existence checked; combined Editor batch pending |

This is a compile-structure repair. It does not claim UI correctness or
performance acceptance until the combined managed test completes.

status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md

# Durable journal format imports

## Current source and repair

The durable journal reader and writer are sibling modules of `format`, whose
binary helpers intentionally remain `pub(super)`. Their imports were pointing
at the durable root, where those helpers are not re-exported. The consumers
now import the helpers from their owning `format` module directly, preserving
the private API boundary and the journal wire format.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M32 | Repair durable journal reader and writer imports to the owning format module | implemented_pending_validation | Scoped rustfmt and static diff checks passed; combined Editor batch pending |

This is a compile-structure repair and does not claim runtime journal
correctness until the managed batch completes.

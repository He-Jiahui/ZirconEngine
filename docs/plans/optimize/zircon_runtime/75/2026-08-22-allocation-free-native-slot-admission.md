# Runtime75 Allocation-Free Native Slot Admission

Plan: docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/75/2026-08-22-allocation-free-native-slot-admission.md","zircon_editor/src/ui/asset_editor/palette/native_slots.rs"]

- Date: 2026-08-22
- Integration owner: `optimize-runtime75-catalog-native-slot-batch-m0-r1-01a00797-20260822`
- Source items: the native-slot query portion of `RUW-P1-004` and `RUW-P1-047`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Editor palette placement asks whether a native node can accept a child during insertion and drag
target qualification. The existing boolean query called `available_native_slot_names(...)`, which
built a `BTreeMap` of occupied mounts, cloned every available slot name into a new `Vec<String>`,
and then discarded the collection after checking `is_empty()`.

`default_native_mount(...)` repeated the same complete collection and retained only its first
element. The registry lookup was already borrowed and indexed; the avoidable work was entirely in
the consumer projection.

## Scope Delivered

- Native descriptor resolution returns the borrowed slot schema from the process-wide showcase
  catalog.
- Child-admission uses direct `any(...)` evaluation and allocates no map, vector, or slot string.
- Default mount selection uses direct `find(...)` and clones only the single returned slot name.
- Multiple slots remain available after occupancy; single slots become unavailable after the first
  matching child. Non-native, missing-type, and unknown-component nodes remain fail closed.

## Deterministic Performance Gate

The ignored release benchmark performs 10,000 `PropertyRow` child-admission checks per sample. The
legacy control constructs 10,000 `BTreeMap` instances and 10,000 result `Vec` instances per sample;
the optimized boolean path constructs zero collections. It warms both paths, then records 21
alternating legacy/optimized sample pairs with 11 legacy-first and 10 optimized-first pairs.

The marker includes both raw unsorted nanosecond series and nearest-rank P50/P95 values for
independent recomputation. Acceptance requires `optimized_p95_ns * 4 <= legacy_p95_ns`, or at least
75% lower measured P95. Actual timings remain pending; collection-instance counts are structural
evidence and are not reported as measured speedup.

## TDD And Static Evidence

- The release gate is RED on the prior implementation because the optimized entry delegates to the
  same collecting path as the legacy control.
- The equivalence test traverses all showcase descriptors and compares both admission and default
  mount after each declared slot is occupied.
- `rustfmt +1.94.1` completed for the owned Rust file.
- Focused behavior tests, the ignored release benchmark, external marker validation, and Editor
  package checks are pending the Runtime75 multi-task coordinator batch. No Cargo or performance
  pass is claimed.

## Remaining Scope

This slice removes allocation from one high-frequency Editor query. Provider-qualified component
authority, capability-filtered catalog snapshots, palette revision caching, schema admission,
component state transactions, and the full 10/1k/100k widget workload matrix remain open.

---
title: Editor23 Widget Promotion Dependency Hash Closure
category: zircon_editor
report_id: Editor23-widget-promotion-dependency-hash-closure-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Widget Promotion Dependency Hash Closure

## Scope

This slice removes ordered-tree membership from the local component dependency closure used when a
UI component is promoted to an external widget. Dependency discovery remains breadth-first, while
the promoted document still stores components in its canonical `BTreeMap`. The private closure set
is not serialized or presented.

## Change

- Replace `BTreeSet<String>` with `HashSet<String>` for cyclic component closure membership.
- Preserve `VecDeque` breadth-first discovery and first visit semantics.
- Preserve missing-component failure, source-component rename, reference conversion, and ordered
  document component storage.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique component names | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Closure cardinality | unique reachable components | unchanged |
| Promoted component storage | `BTreeMap` | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR23_WIDGET_DEPENDENCY_HASH_CLOSURE_BENCH_V1`. Acceptance requires hash closure P95 to be at
most 60% of ordered closure P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ac_editor23_hash_component_closure_terminates_cycles` exercises a
  real two-component cycle through the production closure function.
- `optimization_batch_20260826ac_editor23_widget_promotion_uses_hash_closure_and_ordered_output`
  requires hash closure while retaining the ordered document component lookup path.
- `optimization_batch_20260826ac_editor23_widget_dependency_hash_closure_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor23 still needs typed binding/property schemas, async bounded diagnostics and imports,
generation-qualified previews, lossless V2 editing, atomic save/reimport, and large-binding
document qualification. This slice only improves widget promotion dependency closure.

---
title: Editor hierarchy identity-index rebuild reuse
category: zircon_editor
report_id: Editor01-hierarchy-index-rebuild-reuse-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 hierarchy identity-index rebuild reuse

## Scope

The retained hierarchy bridge already stores a row-index lookup without
owning display names, but a full reflow still constructed an intermediate
ordered entity-to-control map and then walked it to construct the reverse
control-to-entity map. This slice targets that avoidable projection work only;
row ordering, duplicate-key behavior, sparse patch semantics, and selection
ownership stay unchanged.

## Implementation

`SceneHierarchyProjectionState` now stores both control directions as
`HashMap`s. `replace` clears and reserves the existing row and control maps,
then inserts row identity and (when a control is present) both directions in a
single `rows.iter().enumerate()` loop. A shorter controls slice still leaves
the unmatched rows without a control mapping, and repeated keys retain the
same last-write behavior.

## Deterministic work model

For `N` hierarchy rows and `N` authored controls, the retired path performed an
ordered map build plus a second reverse traversal (`O(N log N)` ordered work)
and replaced all map allocations. The current path performs one expected
`O(N)` pass and reuses hash-table capacity after the first reflow. At this
record's implementation snapshot, the two owned control strings remained
necessary because both lookup directions owned their keys. The follow-up
`2026-09-13-hierarchy-control-id-arc-sharing.md` now shares those payloads via
`Arc<str>` while retaining both indexes.

## Validation

- The source contract was RED against `BTreeMap`, the temporary
  `rows.iter().zip(controls)` collection, and the second reverse pass, then
  GREEN after the one-pass reusable maps landed.
- The lower Rust regression covers bidirectional lookup, selected-state
  retention, truncated controls, and stale-row removal.
- The existing hierarchy-native-authority guard was repaired to assert the
  all-logical-row invariant against the new insertion loop rather than the
  retired assignment expression.
- Focused Editor hierarchy-native-authority, hierarchy-generation,
  hierarchy-projection, and Runtime reverse-map contracts pass `19/19`; the
  updated single-process Runtime/Editor performance-plus-pressure batch covers
  351 modules and passes `1346/1346` tests in `47.701s`.
- A subsequent same-source single-process rerun of that exact non-tooling batch
  again covers 351 modules and passes `1346/1346` in `8.890s`.
- The adjacent Hit-Test/Hierarchy contract batch passes `129/129` across 25
  modules in `21.376s`.
- The wider Runtime UI plus Editor projection/pointer/cache batch passes
  `760/760` across 164 modules in `20.998s`.
- Scoped Rustfmt passes. Managed Cargo, allocation, RSS, and hierarchy
  p50/p95/p99 evidence remain pending under the external dirty
  `E:\Git\zr_vm` admission blocker.

## Acceptance boundary

This record is source/contract complete only. Keep
`validation_status: managed_validation_pending` until an owner-attributed
Windows Release run verifies compilation, index parity, allocation behavior,
and the declared hierarchy latency gates.

Follow-up allocation refinement: `2026-09-13-hierarchy-control-id-arc-sharing.md`.

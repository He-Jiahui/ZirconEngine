---
title: Runtime09C Single-entry Texture Slot Synchronization
category: zircon_runtime
report_id: Runtime09C-single-entry-texture-slot-sync-2026-08-27
date: 2026-08-27
session_id: root-runtime09c-single-entry-texture-slot-sync-20260827
implementation_status: implementation_complete
validation_status: local_contract_passed_managed_validation_pending
---

# Runtime09C Single-entry Texture Slot Synchronization

## Scope

This slice removes repeated ordered-map traversal while synchronizing a material texture slot. The
existing-value path previously performed three `get` traversals followed by one `insert` traversal
for the same key. It now uses one `BTreeMap::entry` traversal to read retained metadata and replace
the value in place.

## Change

- Use `Entry::Occupied` to read fallback, transform, and UV metadata once and replace the slot.
- Use `Entry::Vacant` to insert a new slot without a second lookup.
- Preserve texture-reference cloning, fallback/transform/UV retention, and the existing None/removal
  behavior.
- Add Rust coverage for both occupied and vacant texture-slot synchronization.

## Deterministic Performance Evidence

The standalone Rust model contains 16,384 material slots and performs eight passes of 8,192
existing-slot updates per sample across 21 alternating legacy/optimized samples.

| Existing texture-slot updates | Before | After | Reduction |
|---|---:|---:|---:|
| BTreeMap traversals per sample | 262,144 | 65,536 | 75.000% |
| P50 | 68,817,800 ns | 51,006,400 ns | 25.882% |
| P95 | 91,776,300 ns | 76,528,200 ns | 16.614% |

The model compares the complete resulting maps after every sample. Its checksum is `344,064`.

## Validation

- `python -m unittest tools.tests.test_runtime09c_single_entry_texture_slot_sync_performance_contract`
  passes all three source contracts.
- Exact-file `rustfmt --edition 2021` passes.
- The standalone optimized Rust model compiles with `rustc --edition 2021 -C opt-level=3` and
  enforces 75% lookup reduction plus at least 15% P50/P95 reduction.
- Cargo execution of the two in-source Rust tests remains pending through the session coordinator.

## Remaining Parent-plan Work

Runtime09C still owns shader artifact identity, shared compilation, renderer-wide PSO authority,
hot reload, readiness, prewarm/cook, driver cache, and editor diagnostic work. This slice only
removes redundant map traversal from material texture-slot synchronization.

---
title: Runtime75 Selection Flags Array Capacity
category: zircon_runtime
report_id: Runtime75-selection-flags-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Selection Flags Array Capacity

## Scope

Multi-select controls may normalize an Array property into Flags before applying a selection
event. The conversion already consumes every direct array value, but its retained nonempty textual
values previously grew a new output vector geometrically. This Runtime75 slice reserves the direct
input bound before retaining the same values.

## Implementation

- Extracted Array-to-Flags conversion into one streaming helper.
- Allocated its output with the direct Array length as the conservative upper bound.
- Preserved String/Enum acceptance, empty-value rejection, non-textual rejection, order, and the
  existing ownership transfer from the state property.
- Added lower filtering/capacity coverage and the ignored
  RUNTIME769_SELECTION_FLAGS_CAPACITY_BENCH_V1 Release marker.

## Deterministic work boundary

For an Array of N values, the retained Flags vector starts with capacity N rather than growing
geometrically while values are filtered. It does not change selection duplicate/removal behavior,
state validation, or option eligibility. This is allocation-shape evidence only, not allocator,
CPU/RSS, or product selection/input p50/p95/p99 evidence.

## Validation

- The TDD source contract was RED before implementation and is GREEN at 4/4:
  tools/tests/test_runtime_selection_flags_capacity_performance_contract.py.
- The combined Runtime/Editor command-palette, keyboard, menu, TreeView, TextInput, selection,
  and adjacent capacity-contract invocation passed 77/77 in 0.462s through python -B -m unittest.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.

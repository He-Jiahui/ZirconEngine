---
title: Runtime11A virtual-list reconciliation scratch reuse
category: zircon_runtime
report_id: Runtime11A-virtual-list-reconciliation-scratch-reuse-2026-09-13
date: 2026-09-13
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A virtual-list reconciliation scratch reuse

## Scope

`UiVirtualListMaterializationIndex::reconcile` is called for every retained virtual-list
scroll update. The old transactional path cloned the slot map, item-key vector, and assignment
generation vector on every request, even though the physical slot budget is bounded. This slice
retains a second candidate set per owner, preserving the published assignment until protected
slot validation succeeds while reusing the candidate buffers on warm requests.

## Implementation

- `UiVirtualListOwnerMaterialization` now owns `candidate_slots`, `candidate_item_keys`, and
  `candidate_assignment_generations` beside the published fields.
- Reconciliation uses `Clone::clone_from` into those buffers, applies the bounded planner, and
  fills only the candidate slot range. A successful transaction swaps candidate and published
  buffers; a protected-slot rejection leaves the published assignment untouched and the scratch
  state is overwritten on the next retry.
- Cloning a surface copies the published assignment but starts candidate and planner scratch
  empty, so the retained optimization does not inflate snapshot/clone payloads.
- `UiVirtualListSlotMap` supplies an explicit `Clone::clone_from` implementation for its internal
  slot vector; the retained surface scratch therefore reuses the slot-map allocation as well as
  the key and generation vectors.
- Slot identity, logical item keys, generation advancement, protected capture checks, and layout
  projection semantics remain unchanged. No logical-row traversal or unbounded allocation was
  added.

## Validation boundary

- TDD RED/GREEN source contract requires retained candidate buffers, in-place `clone_from`, and
  publication swaps; the legacy per-request `state.slots.clone()` path is rejected.
- The lower Rust regressions cover warm candidate-vector capacity/pointer retention,
  `UiVirtualListSlotMap::clone_from` slot-storage retention, and the existing protected-rebind
  atomic failure semantics.
- The latest batched Runtime/Editor UI source-contract invocation loaded 172 modules and passed
  `773/773` in 1.514s; Rustfmt and scoped whitespace/diff checks are also green. Managed
  Cargo/Windows Release allocation and navigation/virtual-list p50/p95/p99 evidence remains
  pending under the existing external worktree gate.

Current source hashes:

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/layout/virtualization/materialization.rs` | `2924EF6A5CB646E7CBF54A90284C5ECACB5AAABEBE4FAFC8DB5567BF04D3162C` |
| `zircon_runtime/src/ui/surface/virtual_list_materialization.rs` | `DB856DFE940312EAE18ACFDACAD9F0373250437DFB275FC1129DB1AE5F46734D` |
| `tools/tests/test_runtime_ui_virtual_list_surface_materialization_performance_contract.py` | `5EE50E3F07F8162D0FF348260C4B2166CA23D576A79EF5CEDC2B748232251B67` |
| `tools/tests/test_runtime_ui_virtual_list_slot_materialization_performance_contract.py` | `B5AE0A7071EEE61A2E9D6F56DFCA90DA5DD1E8AFACFAA9A99CA6994DC3EC65FC` |

## Acceptance boundary

This is implementation-complete static evidence only. It does not close Runtime11A's broader
data-source virtualization, persistent layout graph, product UI owner, or managed performance
gates. No coordinator request or status query is made for this follow-up.

---
title: Runtime200 Active Pointer Table Index
category: zircon_runtime
report_id: Runtime200-active-pointer-index-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime200 Active Pointer Table Index

## Scope

`UiActivePointerTable` is queried for nearly every pointer event. The table still
publishes its entries in insertion order, but its lookup path linearly scans that
vector. This slice adds a private pointer-id-to-index map while preserving the
public ordered entry view and all per-pointer state semantics.

## Change

- Add `HashMap<UiPointerId, usize>` membership/index storage beside the ordered
  entry vector.
- Use the index for immutable lookup, mutable lookup, and upsert replacement.
- Keep removal order stable with `Vec::remove`, repairing only the shifted
  suffix indices; clear both containers together.
- Leave hover-path reuse, button masks, capture targets, and pointer-entry
  fields unchanged.

## Complexity boundary

For `P` active pointers, exact entry/entry-mut/upsert probes move from `O(P)`
  linear scans to expected `O(1)`. Removal remains `O(P)` when a middle entry
  is removed because the existing ordered slice contract is retained. This is a
  local lookup optimization, not a claim that multi-seat identity or product
  input latency gates are complete.

## TDD and local evidence

- The source contract was run RED before implementation: the table had no
  index and all three index-maintenance assertions failed.
- After implementation the focused source contract passes `3/3`.
- Rust regression coverage exercises middle removal, ordered entries, state
  retention for the shifted entry, and clear/reinsert membership behavior.
- The combined Runtime UI plus Editor asset contract discovery passes `620/620`
  in `15.422s`; the focused index/input/asset subset passes `37/37` in `0.039s`.
- The broader Runtime/Editor performance-contract and pressure discovery passes
  `1358/1358` in `9.465s` in one process.
- The later comprehensive non-tooling Runtime/Editor loader covers 548 files
  and passes `2039/2039` in `22.986s`; the earlier receipt is retained for
  traceability.
- Scoped Python compilation, Rustfmt, `git diff --check`, and wiki validation pass.
- The existing Runtime UI/input contract set remains the adjacent regression
  scope. Scoped rustfmt and Python compilation are required before the next
  batched validation invocation.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs` | `2D49C15207E49EFCF4A47D35E75180201AD965F287191AE073DE7A2BC780A34E` |
| `zircon_runtime/src/ui/dispatch/input_manager/pointer_table/index_tests.rs` | `A17E88AB3B77955D405A6E61C8418D88636A246F9E4C04BFB0EEB9BD5C378A98` |
| `tools/tests/test_runtime_ui_active_pointer_index_performance_contract.py` | `C918535D9C2D545472FE363BC59F21A9F75C7C9BAF296FDDB808601EDB7D43A6` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor batch. Managed
Cargo/Release allocation and pointer-input p50/p95/p99 evidence remain pending;
no standalone Cargo process or coordinator polling is required.

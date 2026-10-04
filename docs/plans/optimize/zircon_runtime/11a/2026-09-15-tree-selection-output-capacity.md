---
title: Runtime11A Tree Selection Output Capacity
category: zircon_runtime
report_id: Runtime11A-tree-selection-output-capacity-2026-09-15
date: 2026-09-15
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A · Tree selection output capacity

## Scope

The indexed TreeView selection reducer already owns the selected-id set and
publishes selected IDs in source order. Its projection collected the ordered
iterator into a zero-capacity `Vec`, even though the index knew the exact number
of selected entries. This slice removes that avoidable geometric growth without
changing selection, ordering, or serialized state semantics.

## Implementation

- Expose the index-owned selected-set length as a crate-local upper bound.
- Reserve that exact bound before cloning ordered selected IDs into the reducer
  output vector, while retaining the existing source-order iterator.
- Keep unknown-ID filtering, toggle behavior, flags, and `UiValue` projection
  unchanged; the count is only a capacity hint and not a new authority.

## Regression and performance contract

The lower module
`zircon_runtime/src/ui/component/state_reducer/state_model/selected_output_capacity_tests.rs`
checks selected-count updates, source-order output, and the bounded reservation.
Its ignored paired benchmark emits
`RUNTIME788_TREE_SELECTION_OUTPUT_CAPACITY_BENCH_V1`; the Python source contract
is `tools/tests/test_runtime_tree_selection_output_capacity_performance_contract.py`.

Current source fingerprints:

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/component/state_reducer/state_model.rs` | `FE0F2781C9B862DD9A28EE24216CC8BBC45E95554F172CC79DF41C2DB48614DB` |
| `zircon_runtime/src/ui/component/state_reducer/state_model/tree_index.rs` | `767A45791F2EA4DAF176B640BFBB6988113C2449FC8555CECB62DC0BB7801D4F` |
| `zircon_runtime/src/ui/component/state_reducer/state_model/selected_output_capacity_tests.rs` | `CB42D19392A56354C57B8836A3D924EF46D78993E32C4D382779AB2B321F1914` |
| `tools/tests/test_runtime_tree_selection_output_capacity_performance_contract.py` | `4E3F915F61CE5AC9F5780371426F79DBC1F980B33C3358B30D06F7D1D2A164EE` |

For a 65,536-item selected output, the old filtered iterator model grows the
output vector 15 times; the exact index count reserves once and has zero growth
events. This is deterministic allocation-shape evidence, not a CPU, allocator,
RSS, or product p50/p95/p99 measurement.

## Local receipt

The focused TDD source contract passes `3/3`; the lower order/count regression
and ignored Release marker are wired and Rustfmt-clean. The refreshed
single-process Runtime/Editor source-contract batch loads `552` modules and
passes `1975/1975` tests in `4.790s`, with zero failures, errors, or skips.
The focused Runtime206/Runtime85/Editor787/Editor789/Runtime788 batch passes
`32/32` in one process. This is local source/model evidence only.
The broader non-tooling Runtime/Editor Python regression discovery also passes
`3724/3724` across `915` modules in `326.952s`, with zero failures, errors, or
skips.

## Validation boundary

Managed Windows Cargo/Release execution, allocator counts, and TreeView product
percentile evidence remain coordinator-owned and pending. No per-task Cargo run,
coordinator retry, or status query is made for this record. Tooling production
work remains deferred for the later Rust migration.

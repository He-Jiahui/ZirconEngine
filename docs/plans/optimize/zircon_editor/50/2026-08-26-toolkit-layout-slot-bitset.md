---
title: Editor50 Toolkit Layout Slot Bitset
category: zircon_editor
report_id: Editor50-toolkit-layout-slot-bitset-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor50 Toolkit Layout Slot Bitset

## Scope

This slice replaces `ToolkitLayout`'s allocation-backed `BTreeSet` duplicate-slot validator with a
four-bit `u8` mask. `ToolkitAreaSlot` has exactly four declared variants, each mapped to one named
match arm. The layout still preserves caller area order and rejects the first repeated slot
encountered during that order.

Tab validation, active-tab selection, immutable `Arc<[ToolkitArea]>` publication, and layout error
types are unchanged.

## Performance Workload

The release workload validates the complete four-slot set 100,000 times per sample.

| Work per sample | Before | After |
|---|---:|---:|
| Ordered tree instances | 100,000 | 0 |
| Slot tree insertions | 400,000 | 0 |
| Bit tests and sets | 0 | 400,000 |
| Output area reordering | 0 | 0 |

The ignored release gate runs 21 alternating sample pairs and emits
`EDITOR50_TOOLKIT_LAYOUT_SLOT_BITSET_BENCH_V1`. Acceptance requires allocation-free bitset
validation P95 to be at least 30% below the legacy `BTreeSet` path. Exact Windows P50/P95 timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ck_editor_toolkit_slot_bitset_preserves_order_and_duplicate_error`
  covers area order and first repeated-slot error behavior.
- `optimization_batch_20260826ck_editor_toolkit_layout_uses_allocation_free_slot_bitset` locks the
  fixed bit mapping and prevents the ordered allocation from returning.
- `optimization_batch_20260826ck_editor_toolkit_layout_slot_bitset_release_benchmark` reports
  paired release P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor50 still owns extension activation authority, owner generations, contribution reconciliation,
callback isolation, toolkit lifecycle, and product-scale qualification. This slice only removes the
allocation and ordered-tree work from fixed toolkit area-slot validation.

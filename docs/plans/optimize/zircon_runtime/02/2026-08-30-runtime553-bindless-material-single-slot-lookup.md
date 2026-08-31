---
title: Runtime Bindless Material Single Slot Lookup 553
category: zircon_runtime
report_id: Runtime553-bindless-material-single-slot-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Bindless Material Single Slot Lookup 553

The existing-material `upsert` path previously indexed the slot vector to compare payload state,
released that borrow, then indexed the same row again when applying a change. The comparison and
write now share one mutable slot-state borrow; dirty-row coalescing still occurs only after that
borrow ends.

For every changed existing material, slot-vector indexing falls from two operations to one, a 50%
reduction. A standalone Rust 1.94.1 `opt-level=3` benchmark isolated repeated bounds/index access
over four million updates and 4,096 rows, using an opaque index barrier and 11-sample medians. The
modeled path changed from 18.8780 ms to 12.3238 ms, a 34.72% improvement on this machine. Hash-map
lookup, payload encoding, dirty upload, and GPU work are excluded.

## Static evidence

- TDD RED: the existing-entry branch contained two identical slot-vector index expressions.
- TDD GREEN: that branch contains one slot lookup and writes through the retained state borrow.
- The focused behavior test covers a payload change with an unchanged logical revision.
- The new focused test is `optimization_batch_20260830es_runtime553_updates_payload_with_one_slot_lookup`.
- Ignored evidence marker: `RUNTIME553_SINGLE_SLOT_LOOKUP_BENCH_V1`.
- Existing tests preserve stable rows, dirty coalescing, delayed reuse, and unknown-release behavior.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `afce7a1114d4899b185d60ede7bbd3003ce6174cc2c3bd42b83535d65a05afbc`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Revision-only and payload-only changes update the stable row and mark it dirty exactly once.
3. Unchanged payloads remain clean, and release/reuse lifetime rules remain unchanged.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.

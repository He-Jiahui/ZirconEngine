---
title: Runtime Material Override Bulk Sort 570
category: zircon_runtime
report_id: Runtime570-material-override-bulk-sort-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Material Override Bulk Sort 570

`MaterialOverrideSet::from_slots` previously normalized a bulk input by binary-searching and
inserting every element into an ordered `Vec`. Reverse-order inputs therefore shifted an
increasing suffix for every slot and made construction O(n squared). Bulk construction now
collects once, performs a stable key sort, and deduplicates in place while copying each later value
into the retained slot. The public incremental `insert` API and last-write-wins semantics are
unchanged.

A Rust 1.94.1 `opt-level=3` standalone benchmark used 13 interleaved pairs, 4,096 reverse-order
slots, and four constructions per sample. P95 changed from `57,630,900 ns` to `143,400 ns`, a
`99.75%` reduction. A duplicate-slot probe also produced identical ordered outputs for the bulk
and legacy incremental algorithms.

## Static evidence

- TDD RED: production bulk construction used repeated binary-search insertion, and the ignored
  benchmark requires at least a 65% P95 reduction.
- TDD GREEN: the focused regression compares unsorted duplicate input with legacy incremental
  insertion and therefore covers sorted order plus last-write-wins replacement.
- Ignored benchmark marker: `RUNTIME570_MATERIAL_OVERRIDE_BULK_SORT_BENCH_V1`.
- Focused test prefix: `optimization_batch_20260831fe_runtime570_`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `a9b77e01a0156cf9a0438440e5e9d951555d1b79aef5d71cd9b9ac0a80cb64d0`.

## Acceptance gates

1. Managed Windows native Release compilation and focused renderer-common tests pass.
2. Bulk construction remains identical to incremental insertion for unsorted duplicate slots and
   serde normalization.
3. Managed ignored reverse-order benchmark retains at least a 65% P95 improvement.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted
   validation.

No direct Cargo validation, commit, push, or WeCom success is claimed by this record.

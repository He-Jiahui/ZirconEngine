---
title: Editor Sprite Atlas Dense Placement Index 571
category: zircon_editor
report_id: Editor571-sprite-atlas-dense-placement-index-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Sprite Atlas Dense Placement Index 571

`pack_source_rects` assigns every source the dense identifier produced by `enumerate`, but the
successful rectangle-pack result was copied into a `BTreeMap<usize, PackedSourceLocation>` and
looked up again for every source. The placement consumer therefore paid O(log n) lookup cost for
an index domain that is exactly `0..sources.len()`. Successful packing now validates and projects
the returned locations once into source-index order, then stores them in a
`Vec<PackedSourceLocation>` for O(1) lookup. Rectangle packing, entry order, pixel coordinates,
RGBA row copies, UV projection, padding, and failure behavior are unchanged.

A Rust 1.94.1 `opt-level=3` standalone benchmark used 21 alternating sample pairs, 4,096 dense
locations, and 128 complete lookup rounds per sample. P95 changed from `47,759,200 ns` for the
tree lookup to `1,130,200 ns` for the dense lookup, a `97.63%` reduction. Both paths produced the
same checksum.

## Static evidence

- TDD RED: the structural regression reported `dense=False, tree=True` against the original
  `PackedSourceRects` storage.
- TDD GREEN: the same regression reports `dense=True, tree=False` after the hard replacement.
- Existing focused behavior covers deterministic entry order, RGBA row copies, UV projection,
  padding, and pack failure.
- Ignored benchmark marker: `EDITOR571_DENSE_PLACEMENT_LOOKUP_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256:
  `add1d589f7ac04473546f4a4881844eda02cf83529b9df8afb496bc1648a6480`.
- Ownership preview `3998e541a34449c4920bc3c67f485156`, apply
  `b4be0f7f9c4b45f68261773ef1b87108`, fresh three-path lease
  `cfcd3ce711b9418889a2135a393a4335`, and attribution
  `3d4c32f946c04c26ae422e99a65ed857` bind the exact lifecycle.

## Acceptance gates

1. Managed Windows native Release compilation and focused sprite-atlas packer tests pass.
2. Deterministic entry order, copied pixels, UVs, padding, and failure diagnostics remain green.
3. Managed ignored dense-placement benchmark retains at least a 75% P95 improvement.
4. Commit/push and WeCom performance publication remain coordinator-owned after accepted batched
   validation.

No direct Cargo validation, commit, push, or WeCom success is claimed by this record.

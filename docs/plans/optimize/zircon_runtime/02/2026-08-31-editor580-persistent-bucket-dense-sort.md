---
title: Editor Persistent Bucket Dense Sort 580
category: zircon_editor
report_id: Editor580-persistent-bucket-dense-sort-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Persistent Bucket Dense Sort 580

Persistent hit-test bucket construction previously materialized cell entries into a `BTreeMap` and
then copied them into a sorted `Vec` before building the balanced tree. It now collects directly
into one vector and sorts by the packed cell key, preserving the exact tree order while removing an
intermediate ordered-map allocation and traversal.

## Static evidence

- Regression prefix: `optimization_batch_gy_editor580_`.
- Ignored benchmark marker: `EDITOR580_PERSISTENT_BUCKET_DENSE_SORT_BENCH_V1`.
- Performance gate: optimized P95 must be at most 70% of the legacy map-then-vector path across 17
  interleaved Release samples over 8,192 cells.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256:
  `c003f99fc4a6111c0f03263b6fe3c5efc076b66e0e2f50af651384a4b3b469be`.
- Coordinator ticket: `6280a1a4298a48609da7587d3a8302a1` (queued); source manifest hash:
  `1ca2b1be6fa6a85cd9d181a83f534f8e39a5ec758e1d450525b61eee3b9f9ed2`.
- Non-ignored behavior coverage is batched in aggregate ticket
  `6bebe849e7c24feaa38c3eecab138148` (queued).

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor580 tests pass.
2. Empty buckets, lookup behavior, balanced tree shape, and persistent update semantics remain
   unchanged.
3. Managed ignored benchmark satisfies the 70% P95 gate.
4. Commit/push and WeCom publication remain coordinator-owned after accepted validation.

No managed Cargo pass, performance result, commit, push, or WeCom success is claimed by this
record.

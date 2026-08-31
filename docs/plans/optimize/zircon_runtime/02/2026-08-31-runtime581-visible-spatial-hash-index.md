---
title: Runtime Visible Spatial Hash Index 581
category: zircon_runtime
report_id: Runtime581-visible-spatial-hash-index-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Visible Spatial Hash Index 581

The immutable visible-spatial query stored entries in a `BTreeMap` even though lookup order is
private and query results are sorted and deduplicated before publication. It now builds a
preallocated `HashMap`, reducing candidate lookup from `O(k log n)` to expected `O(k)` while
preserving deterministic entity output and query statistics.

## Static evidence

- Regression prefix: `optimization_batch_gz_runtime581_`.
- Ignored benchmark marker: `RUNTIME581_SPATIAL_QUERY_HASH_LOOKUP_BENCH_V1`.
- Performance gate: optimized P95 must be at most 70% of the legacy ordered-map lookup across 17
  interleaved Release samples over 32,768 candidates.
- Rust 1.94.1 `rustfmt --edition 2024 --check` and scoped `git diff --check` pass.
- Production/test source SHA-256:
  `a59ce4b18e50474851ed8e82dc7413b58fe65ddfd46cb983c3f47c9c92a780b1`.
- Performance ticket `323c4011b701478384384a5cf3cf00fb` and aggregate behavior ticket
  `6bebe849e7c24feaa38c3eecab138148` are queued; no terminal result is claimed.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime581 tests pass.
2. Bounds and ray queries retain sorted, deduplicated entities and unchanged statistics.
3. Managed ignored benchmark satisfies the 70% P95 gate.
4. Commit/push and WeCom publication remain coordinator-owned after accepted validation.

No managed Cargo pass, performance result, commit, push, or WeCom success is claimed by this
record.

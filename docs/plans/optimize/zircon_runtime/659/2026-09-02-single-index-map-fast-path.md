---
title: Runtime659 Single Index Map Fast Path
category: zircon_runtime
report_id: Runtime659-single-index-map-fast-path-2026-09-02
date: 2026-09-02
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_release_pending
---

# Runtime659 Single Index Map Fast Path

`parallel_map_indices` now executes a one-element map directly on the caller instead of entering
the Rayon pool and constructing a parallel range. Empty input remains allocation-free, inputs
with two or more elements retain the existing parallel path, and the single callback is invoked
exactly once with source index zero.

The deterministic regression covers empty and single-index behavior and binds the single-index
branch ahead of `pool.install`. The ignored Windows Release benchmark emits
`RUNTIME659_SINGLE_INDEX_MAP_FAST_PATH_BENCH_V1` over 17 alternating sample pairs and 50,000 calls
per sample. Its gate requires direct-map P95 to be at most 40% of the former Rayon path P95 and
records the eliminated pool installs.

This candidate is grouped with Editor659 for one coordinator validation request. Measured P95,
commit SHA, push state, and WeCom publication must be written only from an accepted coordinator
result.

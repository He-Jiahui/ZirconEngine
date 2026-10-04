---
title: Runtime612 Ordered Level Registry
category: zircon_runtime
report_id: Runtime612-ordered-level-registry-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime612 Ordered Level Registry

`DefaultLevelManager` now stores levels in a `BTreeMap<WorldHandle, LevelSystem>`. Level creation,
publication rollback, and direct lookup are infrequent registry operations, while ordered world
snapshot traversal is consumed repeatedly by runtime iteration and atomic VM type synchronization.
The previous `HashMap` forced both consumers to clone every level and then sort the snapshot by
handle on every call.

`WorldHandle` now exposes its natural `u64` total order. Ordered-map iteration therefore supplies
the same ascending-handle contract without an additional `O(n log n)` sort. Lookup and mutation
become `O(log n)`, while each ordered snapshot becomes a single `O(n)` clone pass. Existing tests
retain ascending iteration, publication rollback, handle exhaustion, and poison-safe lock recovery;
the two structure guards were updated from their stale pre-refactor insertion spelling.

The ignored Windows Release benchmark emits `RUNTIME612_ORDERED_LEVEL_SNAPSHOT_BENCH_V1` over 17
alternating sample pairs, 4,096 levels, and 64 snapshots per sample. It compares unordered value
collection plus sorting with direct ordered value collection. The gate requires ordered-snapshot
P95 to be at most 50% of clone-and-sort P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regressions, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime612 is prepared with Editor612 under request
`runtime612-editor612-ordered-level-reset-delta-performance-20260901ic-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.

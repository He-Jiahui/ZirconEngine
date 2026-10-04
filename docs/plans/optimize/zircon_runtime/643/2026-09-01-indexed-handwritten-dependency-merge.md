---
title: Runtime643 Indexed Handwritten Dependency Merge
category: zircon_runtime
report_id: Runtime643-indexed-handwritten-dependency-merge-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime643 Indexed Handwritten Dependency Merge

Handwritten asset dependencies are now merged through capacity-sized hash indexes for both the
metadata-level dependency list and its root entry. The root entry is located once before the merge
instead of once per dependency, and empty extracted dependency lists return before any index is
built. Both destination vectors still retain existing order and append each new dependency in
first-seen extractor order.

The ignored Windows Release benchmark emits
`RUNTIME643_INDEXED_HANDWRITTEN_DEPENDENCY_MERGE_BENCH_V1` over 17 alternating sample pairs with
4,096 existing and 4,096 incoming dependencies. It verifies identical ordered results, then
compares the former two linear membership scans with two preallocated hash indexes. The gate
requires indexed P95 to be at most 50% of linear-scan P95.

No direct Cargo validation was run. The coordinator owns combined Runtime643/Editor643 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime643 is prepared with Editor643 under the shared `optimization_batch_jd_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.

## 2026-09-21 borrowed-index follow-up

Runtime865 preserves Runtime643's two indexed membership checks but removes
their owned URI copies. Existing meta/root URIs are borrowed during immutable
classification, incoming dual-target admission is stored in a compact flag
buffer, and mutation begins only after both indexes are dropped. In the dense
4,096-existing-per-target plus 4,096-incoming model, URI clones change from
`20,480` to the dual-ownership minimum of `4,096`. The focused contract is
GREEN at `4/4`, the combined related batch passes `40/40`, and the 101-pair
p50/p95/p99 Release marker is wired. Managed current-source compilation,
allocator data, and asset-restoration product percentiles remain pending.

---
title: Editor624 Preallocated Dirty Removal Partition
category: zircon_editor
report_id: Editor624-preallocated-dirty-removal-partition-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor624 Preallocated Dirty Removal Partition

Incremental dirty-registry partitioning now reserves the removed-document output from the exact
changed-document upper bound. The previous vector grew from zero while scanning the already
materialized ordered change set.

The single-pass present/removed split, deterministic document ordering, reset fast path, cursor
lineage, and transaction merge semantics remain unchanged. Focused coverage compares the reserved
implementation with the unreserved algorithm and locks the capacity expression.

The ignored Windows Release benchmark emits
`EDITOR624_PREALLOCATED_DIRTY_REMOVAL_PARTITION_BENCH_V1` over 17 alternating sample pairs,
32,768 removed documents, and 64 partition passes. The gate requires preallocated output P95 to be
at most 85% of the unreserved baseline.

No direct Cargo validation was run. The coordinator owns the combined Runtime624/Editor624 Windows
Release regression and performance batch. Receipt, measured P95, commit, push, and WeCom outcome
are recorded only after coordinator completion.

---
title: Editor617 Hash Plugin Admission DFS
category: zircon_editor
report_id: Editor617-hash-plugin-admission-dfs-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor617 Hash Plugin Admission DFS

Editor plugin catalog admission now uses preallocated borrowed `HashSet<&str>` indexes for the DFS
completed and visiting states. The previous borrowed `BTreeSet` states paid ordered-tree costs even
though their iteration order was never observed.

The package graph remains a `BTreeMap<String, BTreeSet<String>>`, so package and dependency
visitation order stays deterministic. The path vector still constructs cycle diagnostics, and the
new behavior test compares the hash implementation with a borrowed ordered reference on a cycle.
No package IDs are cloned into either production membership index.

The ignored Windows Release benchmark emits `EDITOR617_HASH_PLUGIN_ADMISSION_DFS_BENCH_V1` over 17
alternating sample pairs with 16,384 long package IDs. It compares the same borrowed DFS using
ordered and preallocated hash state. The gate requires hash DFS P95 to be at most 40% of ordered
DFS P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor617 is prepared with Runtime617 under request
`runtime617-editor617-scene-plugin-admission-performance-20260901ig-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

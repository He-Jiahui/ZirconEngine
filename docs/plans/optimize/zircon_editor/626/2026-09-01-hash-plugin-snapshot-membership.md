---
title: Editor626 Hash Plugin Snapshot Membership
category: zircon_editor
report_id: Editor626-hash-plugin-snapshot-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor626 Hash Plugin Snapshot Membership

Editor plugin manager snapshot construction now reserves borrowed hash sets for faulted and active
package membership. Both paths previously built ordered trees even though consumers only perform
membership probes while preserving catalog or entry traversal order elsewhere.

Manager entry sorting, faulted-state projection, extension registration order, and immutable
snapshot publication remain unchanged. Package IDs stay borrowed from the catalog or entry slice,
so the optimization introduces no owned identifier copies.

The ignored Windows Release benchmark emits `EDITOR626_HASH_PLUGIN_SNAPSHOT_MEMBERSHIP_BENCH_V1`
over 17 alternating sample pairs with 32,768 long package IDs. The gate requires preallocated hash
membership P95 to be at most 40% of ordered-tree membership P95.

No direct Cargo validation was run. The coordinator owns combined Runtime626/Editor626 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor626 is prepared with Runtime626 under request
`runtime626-editor626-gpu-plugin-snapshot-performance-20260901ip-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

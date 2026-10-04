---
title: Editor628 Hash Watch Batch Membership
category: zircon_editor
report_id: Editor628-hash-watch-batch-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor628 Hash Watch Batch Membership

The fallback world-watch invalidation projection now uses a preallocated `HashSet<WatchToken>` for
its seen-token membership pass. The previous `BTreeSet` paid ordered insertion cost for a set whose
iteration order was never observed.

Duplicate and unknown token diagnostics remain separate `BTreeSet` values, so their stable sorted
output is unchanged. Canonical runtime batches keep their existing allocation-free fast path, and
view dirty-mask coalescing still borrows registered view IDs.

The ignored Windows Release benchmark emits `EDITOR628_HASH_WATCH_SEEN_MEMBERSHIP_BENCH_V1` over
17 alternating sample pairs with 65,536 tokens and 32,768 unique identities. The gate requires
preallocated hash-membership P95 to be at most 40% of ordered-tree P95.

No direct Cargo validation was run. The coordinator owns combined Runtime628/Editor628 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor628 is prepared with Runtime628 under request
`runtime628-editor628-gltf-watch-membership-performance-20260901ir-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

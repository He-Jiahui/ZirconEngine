---
title: Runtime611 Hash Reachability Membership
category: zircon_runtime
report_id: Runtime611-hash-reachability-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime611 Hash Reachability Membership

Zrpack trimming now uses a preallocated `HashSet<String>` for the reachable-asset membership
index. The previous `BTreeSet` paid ordered-tree insertion and logarithmic duplicate-admission
costs during breadth-first traversal even though no consumer used the set's iteration order.

Root order, dependency queue order, missing-dependency diagnostics, and canonical report order are
unchanged. The final included/trimmed rows still iterate the existing asset `BTreeMap`; only the
membership-only reachable closure changed. Focused source coverage requires capacity to match the
asset map and rejects an ordered set inside the closure.

The ignored Windows Release benchmark emits `RUNTIME611_HASH_REACHABILITY_BENCH_V1` over 17
alternating sample pairs with 32,768 long asset paths and 65,533 dependency edges. It compares the
same duplicate-admission traversal using an ordered tree and a preallocated hash index. The gate
requires hash-reachability P95 to be at most 50% of ordered-reachability P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime611 is prepared with Editor611 under request
`runtime611-editor611-reachability-descriptor-performance-20260901ib-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.

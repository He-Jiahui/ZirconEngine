---
title: Runtime610 Hash Delta Membership
category: zircon_runtime
report_id: Runtime610-hash-delta-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime610 Hash Delta Membership

Zrpack delta writing and semantic validation now build preallocated `HashSet` indexes for target
asset paths and base chunk hashes. These four indexes are used only for membership tests; the
previous `BTreeSet` construction paid ordered-tree insertion and logarithmic lookup costs without
contributing to report or manifest order.

Removed assets still follow base-manifest order, changed and reused assets still follow target
order, and `chunk_source_paths` remains a `BTreeMap`. The one set that defines the canonical sorted
chunk-hash table remains a `BTreeSet`. Existing delta change/removal equivalence coverage was
adapted to the membership-only argument types, and a new source guard requires explicit capacity
for both writer and validator indexes.

The ignored Windows Release benchmark emits `RUNTIME610_HASH_DELTA_MEMBERSHIP_BENCH_V1` over 17
alternating sample pairs with 32,768 long asset paths and 32,768 unique hashes. It compares ordered
index construction plus reverse membership scans with preallocated hash indexes. The gate requires
hash-membership P95 to be at most 50% of ordered-membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime610 is prepared with Editor610 under request
`runtime610-editor610-delta-discovery-performance-20260901ia-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

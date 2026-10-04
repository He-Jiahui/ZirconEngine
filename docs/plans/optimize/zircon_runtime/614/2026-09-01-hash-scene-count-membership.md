---
title: Runtime614 Hash Scene Count Membership
category: zircon_runtime
report_id: Runtime614-hash-scene-count-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime614 Hash Scene Count Membership

Scene entity management summaries now use a preallocated `HashSet<ResourceId>` to count unique
scenes. The previous `BTreeSet` supplied ordering that no caller observed because the set was read
only through `len()` after the aggregate pass.

All summary counters still run in the same record order, record materialization still sorts by
scene and entity ID, and the unique scene count still collapses repeated `ResourceId` values. The
focused behavior test exercises 32,768 records spanning 8,192 stable scene IDs and compares the
production summary against ordered and hash reference implementations.

The ignored Windows Release benchmark emits `RUNTIME614_HASH_SCENE_COUNT_BENCH_V1` over 17
alternating sample pairs. It compares unique scene counting with an ordered tree and a
capacity-sized hash set. The gate requires hash membership P95 to be at most 50% of ordered
membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime614 is prepared with Editor614 under request
`runtime614-editor614-scene-retirement-performance-20260901id-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

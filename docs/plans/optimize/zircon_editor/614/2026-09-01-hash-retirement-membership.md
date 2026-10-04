---
title: Editor614 Hash Retirement Membership
category: zircon_editor
report_id: Editor614-hash-retirement-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor614 Hash Retirement Membership

Runtime-event consumer contribution retirement now builds a preallocated borrowed
`HashSet<&str>` over removed consumer IDs. The previous borrowed `BTreeSet` was used only for
membership while pruning quarantined consumers, user-disabled consumers, and the round-robin
cursor.

The removed-ID vector remains the canonical returned order, registry publication and active
consumer retirement are unchanged, and the temporary membership index allocates no consumer-ID
copies. Existing lifecycle coverage continues to lock contribution ownership, cleanup-error
publication, and busy lifecycle rejection; a focused source guard requires explicit capacity and
borrowed insertion.

The ignored Windows Release benchmark emits `EDITOR614_HASH_RETIREMENT_MEMBERSHIP_BENCH_V1` over
17 alternating sample pairs with 16,384 long removed IDs and 32,768 membership probes. The gate
requires hash membership P95 to be at most 50% of ordered membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor614 is prepared with Runtime614 under request
`runtime614-editor614-scene-retirement-performance-20260901id-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

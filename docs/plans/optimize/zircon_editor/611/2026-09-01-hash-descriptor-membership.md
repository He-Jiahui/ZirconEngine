---
title: Editor611 Hash Descriptor Membership
category: zircon_editor
report_id: Editor611-hash-descriptor-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor611 Hash Descriptor Membership

Editor plugin catalog construction now builds a preallocated borrowed `HashSet<&str>` for
descriptor package-ID membership. The previous borrowed `BTreeSet` supplied ordering that no
consumer observed while every runtime-manifest admission check paid logarithmic lookup cost.

The descriptor vector remains alive for the full borrowed-index lifetime, so the membership index
does not clone package IDs. Runtime manifests still keep first-occurrence precedence, duplicate
diagnostics remain stored in a `BTreeSet<String>` for deterministic output, and registration order
still follows descriptor order. Focused behavior coverage locks the distinction between duplicate
editor manifests and duplicate runtime-only manifests.

The ignored Windows Release benchmark emits `EDITOR611_HASH_DESCRIPTOR_MEMBERSHIP_BENCH_V1` over
17 alternating sample pairs with 32,768 package IDs of 184 bytes. It compares reverse membership
scans over a borrowed ordered tree and a preallocated borrowed hash index. The gate requires hash
membership P95 to be at most 40% of ordered-membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor611 is prepared with Runtime611 under request
`runtime611-editor611-reachability-descriptor-performance-20260901ib-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.

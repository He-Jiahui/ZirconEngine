---
title: Runtime629 Preallocated Project Manifest Membership
category: zircon_runtime
report_id: Runtime629-preallocated-manifest-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime629 Preallocated Project Manifest Membership

Project manifest validation now reserves both duplicate-detection sets from their exact input
lengths. The asset-root and UI-root membership passes previously grew from zero for every
validation, even though both source slices are already materialized.

Duplicate ordering, overlap checks, UI scheme validation, and all error payloads remain unchanged.
The capacity hint is bounded by the corresponding manifest field and does not retain source data.

The ignored Windows Release harness emits
`RUNTIME629_PREALLOCATED_MANIFEST_ROOT_MEMBERSHIP_BENCH_V2`. It is a helper microbenchmark: it
constructs root strings and calls standalone membership helpers rather than
`ProjectManifest::validate`, so it is not product acceptance. Its input is exactly
`MAX_PROJECT_ASSET_ROOTS` (4,096), the admitted manifest-root limit; it no longer uses the
invalid 32,768-root synthetic workload. It collects 31 alternating sample pairs and reports
nearest-rank p50/p95/p99 for both paths.

A managed Windows Release measurement through the real caller is still pending. That real caller
must include manifest validation and its surrounding product workload; the helper microbenchmark
cannot close the performance or product-validation gate.

No direct Cargo validation was run. The coordinator owns the combined Runtime629/Editor629 Windows
Release regression and performance batch. Receipt, measured P95, commit, push, and WeCom outcome
are recorded only after coordinator completion.

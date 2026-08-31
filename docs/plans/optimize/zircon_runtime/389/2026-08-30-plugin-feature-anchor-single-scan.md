---
title: Runtime389 Plugin Feature Anchor Single Scan
category: zircon_runtime
report_id: Runtime389-plugin-feature-anchor-single-scan-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime389 Plugin Feature Anchor Single Scan

Plugin render-feature insertion now scans the existing feature list once for the requested anchor
names, caches each feature name for that pass, and stops after all unique anchors have been found.
The previous behavior of selecting the greatest `index + 1` across anchors is preserved, duplicate
anchor occurrences do not change the selected insertion point, and empty/unknown anchor sets still
return `None`.

The ignored Windows Release benchmark emits `RUNTIME389_PLUGIN_FEATURE_ANCHOR_SINGLE_SCAN_BENCH_V1`
over 17 paired samples with 512 features and four anchors, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime389 is submitted with Runtime390 under request
`runtime389-runtime390-performance-batch-20260830cm-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.

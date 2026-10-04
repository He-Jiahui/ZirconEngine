---
title: Runtime618 Preallocated Native Resource Validation
category: zircon_runtime
report_id: Runtime618-preallocated-native-resource-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime618 Preallocated Native Resource Validation

Native plugin registration validation now reserves its borrowed resource-ID membership index from
the manifest resource count. The index previously started empty and repeatedly grew while every
resource count was already available on the manifest.

Resource field validation order, borrowed `&str` membership, and first-duplicate diagnostics are
unchanged. Existing behavior coverage continues to lock the first duplicate error, while the new
focused source guard rejects an unreserved production index.

The ignored Windows Release benchmark emits
`RUNTIME618_PREALLOCATED_NATIVE_RESOURCE_BENCH_V1` over 17 alternating sample pairs with 32,768
unique long resource IDs. The gate requires preallocated membership P95 to be at most 85% of
unreserved membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime618 is prepared with Editor618 under request
`runtime618-editor618-native-resource-menu-performance-20260901ih-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

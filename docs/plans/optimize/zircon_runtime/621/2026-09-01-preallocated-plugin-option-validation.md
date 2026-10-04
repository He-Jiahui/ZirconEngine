---
title: Runtime621 Preallocated Plugin Option Validation
category: zircon_runtime
report_id: Runtime621-preallocated-plugin-option-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime621 Preallocated Plugin Option Validation

Plugin enum-option validation now reserves its borrowed enum-value membership index from the
declared value count. The previous hash set grew incrementally even though the option manifest
exposed the full slice before validation.

Default token validation, enum value traversal, borrowed identities, first-duplicate diagnostics,
and final default-membership validation remain unchanged. Existing behavior coverage locks the
first duplicate error, while focused source coverage rejects an unreserved production index.

The ignored Windows Release benchmark emits `RUNTIME621_PREALLOCATED_PLUGIN_OPTION_BENCH_V1` over
17 alternating sample pairs with 32,768 unique long enum values. The gate requires preallocated
membership P95 to be at most 85% of unreserved membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime621 is prepared with Editor621 under request
`runtime621-editor621-option-graph-node-performance-20260901ik-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

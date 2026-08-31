---
title: Editor578 Overlay Invalidation Single Pass
category: zircon_editor
report_id: Editor578-overlay-invalidation-single-pass-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor578 Overlay Invalidation Single Pass

Softbuffer diagnostic planning now formats overlay text from the current refresh counters and
invalidation counters directly. The previous loop cloned `HostRefreshDiagnostics` solely to
replace three invalidation fields before formatting; the new path preserves startup text and all
formatted bytes without that intermediate clone.

Regression coverage compares the direct path with the legacy clone-and-overlay sequence. The
ignored Windows Release benchmark emits `EDITOR578_OVERLAY_INVALIDATION_SINGLE_PASS_BENCH_V1`
over 31 alternating sample pairs of 131,072 overlay strings. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Editor578 is prepared with Runtime578 under request
`runtime578-editor578-pmrem-overlay-performance-20260831gw-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

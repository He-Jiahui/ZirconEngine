---
title: Runtime584 Animation Reference Borrow
category: zircon_runtime
report_id: Runtime584-animation-reference-borrow-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime584 Animation Reference Borrow

Animation graph reference resolution now borrows the compile element while valid node references
are looked up. The former iterator cloned the element for every reference before the lookup. Empty
and unknown references still clone the element into the emitted diagnostic, preserving diagnostic
ownership, identifiers, messages, and source attribution.

Regression coverage verifies that repeated valid references preserve their resolved indexes and do
not emit diagnostics. The ignored Windows Release benchmark emits
`RUNTIME584_ANIMATION_REFERENCE_BORROW_BENCH_V1` over 21 alternating sample pairs and 65,536
valid references per sample with a long graph-node identifier. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.50`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime584 is prepared with Editor584 under request
`runtime584-editor584-animation-log-fallback-performance-20260831hc-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

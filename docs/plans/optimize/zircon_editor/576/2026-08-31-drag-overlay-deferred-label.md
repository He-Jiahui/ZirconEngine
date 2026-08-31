---
title: Editor576 Drag Overlay Deferred Label
category: zircon_editor
report_id: Editor576-drag-overlay-deferred-label-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor576 Drag Overlay Deferred Label

Drag-overlay label selection now returns a trimmed borrowed string. The renderer converts it to an
owned command payload only after the text frame passes the zero-area geometry gate. Collapsed or
fully unavailable preview text therefore skips the prior heap allocation while preserving label
priority, whitespace trimming, and emitted paint command content.

Regression coverage verifies label priority, trimming, and allocation placement after the geometry
gate. The ignored Windows Release benchmark emits `EDITOR576_DEFERRED_DRAG_LABEL_BENCH_V1`
across 31 alternating sample pairs of 100,000 collapsed-preview attempts. Allocations in that path
fall from 100,000 to zero per sample. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Editor576 is prepared with Runtime576 under request
`runtime576-editor576-sdf-drag-performance-20260831gu-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.

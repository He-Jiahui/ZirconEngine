---
title: Editor579 Origin Axis Borrowed Defaults
category: zircon_editor
report_id: Editor579-origin-axis-borrowed-defaults-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor579 Origin Axis Borrowed Defaults

Popup origin-axis projection now borrows string attributes and static defaults through `Cow`
instead of allocating a `String` for every absent axis attribute. Existing non-empty author values,
empty-value fallback, origin offsets, and popup placement behavior remain unchanged.

Regression coverage verifies that missing attributes return a borrowed default. The ignored
Windows Release benchmark emits `EDITOR579_ORIGIN_AXIS_BORROWED_DEFAULTS_BENCH_V1` over 21
alternating sample pairs and 1,000,000 four-axis projections per pair. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Editor579 is prepared with Runtime579 under request
`runtime579-editor579-shader-origin-performance-20260831gx-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

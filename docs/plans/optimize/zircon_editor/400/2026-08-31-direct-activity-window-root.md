---
title: Editor400 Direct Activity Window Root
category: zircon_editor
report_id: Editor400-direct-activity-window-root-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor400 Direct Activity Window Root

Activity window descriptors now build their reflection root with one exact-capacity String and
directly append the stable `editor/windows/` prefix plus the window ID. This removes formatting
machinery from descriptor registration while preserving the public descriptor fields and exact
reflection path bytes.

Regression coverage verifies the complete path and retained window identity. The ignored Windows
Release benchmark emits `EDITOR400_DIRECT_ACTIVITY_WINDOW_ROOT_BENCH_V1` over 17 alternating paired
samples, each constructing 262,144 representative reflection roots. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.80` (at least 20% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor400 is prepared with Runtime470 under request
`runtime470-editor400-performance-batch-20260831fn-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.

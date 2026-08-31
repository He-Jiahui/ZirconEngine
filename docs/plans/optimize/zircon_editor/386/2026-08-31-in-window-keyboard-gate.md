---
title: Editor386 In-Window Keyboard Gate
category: zircon_editor
report_id: Editor386-in-window-keyboard-gate-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor386 In-Window Keyboard Gate

Retained popup keyboard navigation now returns before pagination arithmetic when an ordinary
Next/Previous command remains inside the current row window. Accept/Cancel and no-op First/PageUp
commands receive the same early treatment. The former path calculated terminal page offsets,
including integer division, before discovering that no window request was required.

Terminal-row wrapping, deep First/Last/PageUp/PageDown requests, query propagation, and row-level
navigation remain unchanged. Regression coverage exercises every early command alongside the
existing terminal-window tests.

The ignored Windows Release benchmark emits `EDITOR386_IN_WINDOW_KEYBOARD_GATE_BENCH_V1` over 17
alternating paired samples, each probing 2,097,152 ordinary in-window Next commands. The optimized
path performs the row-boundary gate without terminal page arithmetic. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.60` (at least 40% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor386 is prepared with Runtime456 under request
`runtime456-editor386-performance-batch-20260831ex-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.

---
title: Editor597 Pane Info Ownership Move
category: zircon_editor
report_id: Editor597-pane-info-ownership-move-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor597 Pane Info Ownership Move

Workbench pane projection now builds the native body before constructing `PaneData`. This ends the
last borrow of the owned pane `info` string before the host model is assembled, allowing the string
to move directly into `SharedString` instead of cloning it and immediately dropping the original.

Pane metadata selection, presentation construction, native body content, empty-state actions,
viewport state, and field order in the resulting model retain their previous contracts. Focused
tests cover shared-string content equivalence and require native-body precomputation to precede the
ownership move.

The ignored Windows Release benchmark emits `EDITOR597_PANE_INFO_MOVE_BENCH_V1` over 21 alternating
sample pairs, a 16,384-byte pane information string, and 2,048 conversions per sample. The legacy
model allocates a cloned string before conversion; the optimized model transfers its owned buffer.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.35`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor597 is prepared with Runtime597 under request
`runtime597-editor597-prototype-pane-performance-20260901hp-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.

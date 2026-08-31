---
title: Editor380 Duplicate Command Label Scan
category: zircon_editor
report_id: Editor380-duplicate-command-label-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor380 Duplicate Command Label Scan

Command palette filtering now avoids a second case-insensitive substring scan when a command's
label is the same text as its ID. This is the default projection shape, where the constructor
copies the ID into the label until metadata supplies a distinct display label.

Whitespace normalization, empty-query behavior, ID-first short circuiting, distinct label
matching, ASCII case folding, and long-query rejection remain unchanged. Equal-length values are
compared only after the ID scan misses; queries longer than both values return without that
comparison. Regression coverage compares default and customized labels with the former two-scan
implementation.

The ignored Windows Release benchmark emits `EDITOR380_DUPLICATE_COMMAND_LABEL_SCAN_BENCH_V1`
over 17 alternating paired samples. Each sample performs 32,768 shared-prefix misses against a
512-byte default ID/label. The legacy path executes two substring scans per check and the optimized
path executes one scan plus one equality check. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.65`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor380 is prepared with Runtime450 under request
`runtime450-editor380-performance-batch-20260831er-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.

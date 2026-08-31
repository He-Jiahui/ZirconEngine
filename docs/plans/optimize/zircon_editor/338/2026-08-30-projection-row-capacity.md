---
title: Editor338 Projection Row Capacity
category: zircon_editor
report_id: Editor338-projection-row-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor338 Projection Row Capacity

Retained workbench projection now reserves the node count before indexing non-empty control ids.
Control-id filtering, row numbering, duplicate last-write behavior, model metadata, and document
identity checks remain unchanged while large retained documents avoid repeated hash-table growth
and rehashing.

The ignored Windows Release benchmark emits `EDITOR338_PROJECTION_ROW_CAPACITY_BENCH_V1` over 17
alternating paired samples with 32,768 template nodes, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor338 is submitted with Editor339 under request
`editor338-editor339-performance-batch-20260830cv-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.

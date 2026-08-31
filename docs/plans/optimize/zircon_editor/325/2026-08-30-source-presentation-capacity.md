---
title: Editor325 Source Presentation Capacity
category: zircon_editor
report_id: Editor325-source-presentation-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor325 Source Presentation Capacity

Asset source presentation now reserves exact output lengths for outline labels and structured
diagnostic labels before formatting entries. Selection lookup, label formatting, source ordering,
and roundtrip status are unchanged.

The ignored Windows Release benchmark emits `EDITOR325_SOURCE_PRESENTATION_CAPACITY_BENCH_V1`
over 17 paired samples with 256 items per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor325 is submitted with Runtime379 under request
`runtime379-editor325-performance-batch-20260830ca-v2`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.

The v1 ticket `810be5cc03614519a49f6a6a259db9fd` ended `failed` after its receipt retained the
pre-rustfmt Runtime379 hash. It is not valid evidence for the current source.

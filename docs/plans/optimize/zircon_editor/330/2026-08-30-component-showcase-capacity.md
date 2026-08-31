---
title: Editor330 Component Showcase Capacity
category: zircon_editor
report_id: Editor330-component-showcase-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor330 Component Showcase Capacity

Component-showcase host projection now reserves its template-node output from the owned host-node
count before conversion. Node order, per-node conversion, failed-node filtering, layout, and model
publication remain unchanged.

The ignored Windows Release benchmark emits `EDITOR330_COMPONENT_SHOWCASE_CAPACITY_BENCH_V1` over
17 paired samples with 512 nodes per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor330 is submitted with Runtime384 under request
`runtime384-editor330-performance-batch-20260830cf-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.

Validation attempt: ticket `84553760a0ff42c6a0968c82b0723873` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`. Cargo did not start; regression, P95, commit/push, and
WeCom success evidence remain pending.

---
title: Runtime379 Shader IDE Preview Capacity
category: zircon_runtime
report_id: Runtime379-shader-ide-preview-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime379 Shader IDE Preview Capacity

Shader IDE preview assembly now reserves capacities for converted assembly segments, include-index
entries using iterator size hints, recursive module includes, and visited shader URIs. Include
traversal, deduplication, assembly order, and preview segment values are unchanged.

The ignored Windows Release benchmark emits `RUNTIME379_SHADER_IDE_PREVIEW_CAPACITY_BENCH_V1`
over 17 paired samples with 256 entries per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime379 is submitted with Editor325 under request
`runtime379-editor325-performance-batch-20260830ca-v2`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.

The v1 ticket `810be5cc03614519a49f6a6a259db9fd` ended `failed` after its receipt retained the
pre-rustfmt Runtime379 hash. It is not valid evidence for the current source.

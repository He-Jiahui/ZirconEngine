---
title: Runtime390 Shader Layout Hash Lookup
category: zircon_runtime
report_id: Runtime390-shader-layout-hash-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime390 Shader Layout Hash Lookup

Global shader bind-group layout projection now uses capacity-sized hash maps and sets for resource
name indexing and declared-name membership. Duplicate resource rejection, unknown-resource
rejection order, missing-resource diagnostics, ABI checks, resource-kind checks, and the outer
binding sort remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME390_SHADER_LAYOUT_HASH_LOOKUP_BENCH_V1` over 17
paired samples with 512 resource names, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime390 is submitted with Runtime389 under request
`runtime389-runtime390-performance-batch-20260830cm-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.

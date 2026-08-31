# Runtime232 Builtin Candidate Registry Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime232-editor178-performance-batch-20260826gl-v1`

## Problem

Builtin runtime profile selection materialized a candidate Vec and ID HashMap from an already owned
module Vec without reserving its known length, growing and rehashing both containers during startup.

## Optimization

- Initialize candidate storage and the ID index from the exact incoming module count.
- Keep empty input allocation-free and preserve duplicate detection, candidate order, and lookup indices.
- Leave dependency-closure and final selected-module behavior unchanged.

## Regression Contract

The `optimization_batch_20260826gl_` Runtime tests verify both capacities and the source contract,
and provide an ignored paired release benchmark emitting
`RUNTIME232_BUILTIN_CANDIDATE_REGISTRY_CAPACITY_BENCH_V1`. It builds 64 registries with 4,096
candidates per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.

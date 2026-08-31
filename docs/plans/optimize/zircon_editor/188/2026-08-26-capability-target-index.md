# Editor188 Capability Target Index

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime242-editor188-performance-batch-20260826gv-v1`

## Problem

Every editor capability update cloned the full previous list and compared each capability against
every target with a nested linear scan. Large plugin capability batches made removal projection
quadratic before applying the runtime configuration.

## Optimization

- Build one borrowed target-capability HashSet for the update.
- Clone only previous capabilities that survive the indexed membership test.
- Preserve disable ordering and duplicates, plus enable-mode sorting and deduplication.

## Regression Contract

The `optimization_batch_20260826gv_` Editor tests cover enable and disable projection semantics,
enforce the single target-index source contract, and provide an ignored paired release benchmark
emitting `EDITOR188_CAPABILITY_TARGET_INDEX_BENCH_V1`. It repeatedly removes 768 targets from 1,024
capabilities and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.

# Runtime243 Archetype Signature Hash Dedup

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime243-editor189-performance-batch-20260826gw-v1`

## Problem

Archetype signature construction sorted every supplied component id before removing duplicates. Large
duplicate-heavy import or reconstruction inputs therefore paid sorting cost for every repeated id,
even though the canonical signature retained only a small unique component set.

## Optimization

- Keep the existing in-place sort and dedup path for inputs below 128 components.
- For larger inputs, insert component ids into a pre-sized HashSet before materialization.
- Sort only the unique component ids required by the deterministic signature contract.

## Regression Contract

The `optimization_batch_20260826gw_` Runtime tests preserve sorted unique component semantics,
enforce the adaptive hash-dedup source shape, and provide an ignored paired release benchmark
emitting `RUNTIME243_ARCHETYPE_SIGNATURE_HASH_DEDUP_BENCH_V1`. It repeatedly normalizes 4,096
component ids drawn from 16 unique ids and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.

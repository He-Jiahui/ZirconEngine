# Runtime208 Virtual Geometry Binary Dump Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime208-editor154-performance-batch-20260826fm-v1`

## Problem

Virtual geometry cook binary dump encoding grew its byte vector from empty although every fixed
field, length-prefixed string, child list, dependency list, and matched page payload size was known
before encoding.

## Optimization

- Compute the exact encoded byte count with named field counts, `size_of`, and saturating arithmetic,
  then reserve the complete dump once.
- Preserve binary format version, deterministic sort order, missing payload sentinel, unmatched
  payload handling, and all encoded field widths.

## Regression Contract

The `optimization_batch_20260826fm_` Runtime tests encode 256 real hierarchy, cluster, page,
dependency, root-range and payload records, require the computed capacity to equal the final byte
length, enforce the production reservation, and provide an ignored paired release benchmark
emitting `RUNTIME208_VIRTUAL_GEOMETRY_BINARY_DUMP_CAPACITY_BENCH_V1`. It writes a 16 KiB byte output
512 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.

# Runtime310 Single-Allocation Virtual Geometry Page Payload

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime310-editor255-performance-batch-20260829aj-v1`

## Problem

Virtual geometry page payloads started with an empty byte vector even though their encoded size is
fully determined by seven header words and four words per payload item. Large pages repeatedly
grew and copied the byte buffer while encoding.

## Optimization

- Compute the final payload byte capacity from the exact word layout.
- Allocate that byte capacity before writing headers and item summaries.
- Preserve the binary payload order and little-endian encoding.

## Regression Contract

The `optimization_batch_20260829aj_` Runtime tests verify exact capacities and guard the production
preallocation contract. The ignored paired release benchmark emits
`RUNTIME310_SINGLE_ALLOCATION_VIRTUAL_GEOMETRY_PAGE_PAYLOAD_BENCH_V1`. It builds 40,000 payloads
with 64 items per sample, changes a growing buffer to exact single allocation, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.

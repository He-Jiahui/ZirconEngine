# Runtime298 Allocation-Free Discrete Channel-Name Check

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime298-editor244-performance-batch-20260829y-v1`

## Problem

Every canonical discrete-channel validation formatted `discrete_{channel_count}` into a temporary
`String` before comparing it with the stored layout name. Validation is used by the broader audio
layout contract, so repeated checks paid one allocation even though both operands already contain
all information needed for a byte-level comparison.

## Optimization

- Strip the fixed `discrete_` prefix and parse the decimal suffix directly from borrowed bytes.
- Reject empty, zero-padded, non-decimal, mismatched, and overflowing suffixes so the accepted bytes
  remain identical to Rust's canonical `u16` display representation.
- Preserve the existing nonzero-channel and empty-speaker requirements without allocating.

## Regression Contract

The `optimization_batch_20260829y_` Runtime tests cover canonical boundary values and malformed or
overflowing suffixes, then guard the hot validator against reintroducing `format!`. The ignored
paired release benchmark emits `RUNTIME298_ALLOCATION_FREE_DISCRETE_CHANNEL_NAME_CHECK_BENCH_V1`.
It performs 200,000 mixed valid and invalid checks per sample, reduces result allocations per check
from one to zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.

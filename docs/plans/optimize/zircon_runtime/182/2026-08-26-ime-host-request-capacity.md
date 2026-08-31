# Runtime182 IME Host Request Capacity

- Date: 2026-08-26
- Session: `root-runtime-interface03-activate-link-failure-20260831` (current-source convergence)
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_queued`
- Validation request: `runtime-six-hotpath-performance-batch-20260831-final`

## Problem

Batch input-method dispatch appended into the Runtime host-request vector without using the
iterator's known request-count lower bound. An input-method request can emit enable state, cursor
geometry, and surrounding text, so a fresh or tight vector repeatedly grew while expanding a
multi-request batch.

## Optimization

- Convert the batch to its iterator once and read its size-hint lower bound.
- Reserve up to three host-request slots per known input-method request before expansion.
- Preserve disable short-circuiting and optional cursor/surrounding-text behavior.

## Regression Contract

The shared `runtime_hotpath_batch_` filter owns three Runtime182 tests: expansion behavior,
size-hint capacity source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `RUNTIME182_IME_HOST_REQUEST_CAPACITY_BENCH_V1`, expands 256 requests into 768 real
`ImeHostRequest` values 1,024 times per sample, replaces growth-driven allocation with one reserve,
and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Deterministic Evidence

For the benchmark workload, 1,024 append batches admit 256 input-method requests each. The lower
bound therefore plans 768 host-request slots per batch, 262,144 input requests and 786,432 maximum
host-request values per sample. Planned reserve calls change from zero to one per batch. Allocator
growth counts are intentionally not inferred because they depend on Rust's capacity policy; exact
P50/P95 remains managed release evidence.

## Validation Ownership

One combined managed Windows release command must run all six Runtime177/Runtime182 tests under the
shared filter. The coordinator owns exact P50/P95 backfill, commit, push to `origin/main`, and the
one-shot WeCom report after a pushed SHA exists.

After shared HEAD advanced, current hashes were re-attested under lease request
`afb668dd6f084a3b9389229ea6085fc6`. Current-source batch ticket
`dd6072a2666c4943bbf2a496d941079c` was queued from snapshot `2510` by request
`runtime-six-hotpath-performance-batch-20260831-current-head-r3` (receipt
`73ee20784cb846e2bcfef6cbddb0f952`). It covers 12 Rust behavior tests, 6 ignored release
benchmarks, and 27 Python contracts; the coordinator owns terminal timing, commit, push, and the
one-shot WeCom report.

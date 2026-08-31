# Editor79 Export Log UTF-8 Buffer Adoption

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826cp_`

## Problem

The Editor export stage runner already returned owned stdout/stderr `Vec<u8>` buffers, but
`command_diagnostics` decoded every non-empty buffer with `from_utf8_lossy(...).into_owned()`.
Valid UTF-8 command output therefore incurred a second allocation and full byte copy.

## Optimization

- Decode with `String::from_utf8(bytes)` so valid output adopts the existing allocation.
- Preserve stdout-before-stderr order and empty-buffer filtering.
- Preserve replacement-character decoding for invalid UTF-8 through the original lossy fallback.

## Test And Performance Contract

- The behavior regression covers valid stdout, invalid stderr, exact replacement bytes, output
  order, and two empty buffers.
- The source regression requires owned UTF-8 decoding plus the invalid-byte fallback and rejects
  the old unconditional lossy-copy expression.
- Ignored release evidence prints `EDITOR79_EXPORT_LOG_UTF8_BUFFER_ADOPTION_BENCH_V1` for 21
  alternating sample pairs over two owned 1 MiB valid UTF-8 streams. Input cloning is outside the
  timed region.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo results,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.


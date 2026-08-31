---
title: Runtime12 Constant-Width F64 Random Access
category: zircon_runtime
report_id: Runtime12-constant-width-f64-random-access-2026-08-26
date: 2026-08-26
session_id: root-runtime12-f64-random-access-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime12 constant-width f64 random access

## Scope

- Parent gaps: the `readF64LeAt` portion of `WOC-ZRRT-P1-043` and `WOC-ZRRT-P1-044`.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: the reachable WOC binary codec, a dedicated Zr package, the source/performance contract, and this record.
- This slice removes only the full-payload copy and prefix scan from fixed-width random f64 access. `ByteReader` construction, variable-length `readBytes`, snapshot materialization, state digesting, and complete Runtime12 qualification remain open.

## Change

- `readF64LeAt` retains its exact eight-byte range contract, but now reads only the selected eight bytes and validates each wire byte is at most 255.
- The fixed-width loader is unrolled, so lookup cost is independent of the payload length and byte offset.
- Sequential and random-access readers share one finite IEEE-754 decoder, preserving finite-only values, subnormal handling, signed zero, and exponent bounds without maintaining two floating-point implementations.
- `binary.selfTest` now covers a valid `0.5` value behind a nonzero prefix, and a dedicated package makes that contract independently compilable and executable by the pinned ZrVM validator.

For a 65,536-byte payload with the f64 at offset 65,528, the previous path copied 65,536 bytes, validated/skipped 65,528 prefix bytes, and read eight target bytes. The new path validates and reads only eight target bytes. The modeled byte touches therefore fall from 131,072 to 8, a 99.994% structural reduction.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime12_f64_random_access_performance_contract -v` initially passed 1/7, failed five guards, and errored on the absent dedicated package.
- GREEN: the same command passes 7/7 after the codec and package changes.
- The private benchmark syntax and parity checks pass; the modeled decoded value is exactly `0.5` for both paths.
- Scoped `git diff --check` is part of managed validation because candidate materialization owns the exact five paths, including the two package files ignored by the broad repository examples rule.

The deterministic Node model measures 31 alternating legacy/optimized sample pairs with 128 reads per sample, nearest-rank percentiles, a 65,536-byte payload, and an observable prefix-read sink.

| Metric | Copy + prefix scan | Fixed eight-byte access | Change |
|---|---:|---:|---:|
| P50 | 135.7090 ms | 0.0829 ms | -99.939% |
| P95 | 231.3680 ms | 0.2277 ms | -99.902% |
| byte touches / read | 131,072 | 8 | -99.994% |

These timings isolate fixed-width f64 random access at a late payload offset. They do not claim end-to-end world tick, complete codec, snapshot, ZrVM startup, or state projection latency.

## Async validation

One coordinator batch must run the seven Python contracts, the parity/performance model, scoped diff checks, and `woc_protocol_binary_random_access_tests.zrp` against pinned external ZrVM commit `60f6bcf4dd22bb6f5247e353bd0d97964758f157` in one managed Cargo group.

Acceptance requires the Zr package to compile and return zero, the seven contracts to pass, model parity to remain exact, structural byte-touch reduction to remain at least 99%, and P50/P95 reductions to remain at least 35%. Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include managed P50/P95 and byte-touch reductions and label them as fixed-width f64 random-access-only evidence.

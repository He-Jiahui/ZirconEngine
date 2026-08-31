---
title: Runtime07 Streaming Native Benchmark Record
category: zircon_runtime
report_id: Runtime07-streaming-native-benchmark-record-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Streaming Native Benchmark Record

## Scope

This slice removes intermediate strings from native benchmark record emission after the measured
core interval. It preserves JSON field order, escaping, numeric formatting, latency finalization,
empty-latency fields, counter order, the single stderr record, and the benchmark timing boundary.

## Change

- Stream JSON string escaping through a borrowed `Display` wrapper instead of allocating one
  encoded `String` for each metadata and counter name.
- Stream counter fields directly into the final formatter instead of creating per-counter strings,
  collecting a `Vec<String>`, and joining it.
- Retain the optional finalized latency summary by value, then stream both present and absent field
  sets without an owned latency string.
- Add exact Rust regressions for escaped counter names and both latency variants plus a Python source
  contract covering the allocation-free field wrappers.

## Deterministic Performance Evidence

The standalone optimized Rust model formats a complete benchmark record with 4,096 escaped counter
names across 31 alternating samples. Both paths produced byte-identical 151,058-byte JSON records.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 16,398 | 10 | 99.939% |
| Requested allocation bytes | 1,416,632 | 452,166 | 68.082% |
| Record-format P50 | 3.6421 ms | 1.6319 ms | 55.193% |
| Record-format P95 | 7.6968 ms | 4.4236 ms | 42.527% |

Evidence marker: `RUNTIME07_STREAMING_NATIVE_BENCHMARK_RECORD_MODEL_V1`.

The model intentionally returns a final `String` so both complete records can be compared. The
production path writes through one `eprintln!` formatter and therefore does not require that modeled
final-string allocation.

## Validation

- `python tools/tests/test_runtime07_streaming_native_benchmark_record_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts full record byte equality before recording metrics.
- Rust regressions retain JSON escaping, escaped counter bytes, and Some/None latency field bytes.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch with
  the catalog selection completion candidate.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.

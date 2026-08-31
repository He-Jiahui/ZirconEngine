---
title: Runtime07 Borrowed ZrVM Export Arguments
category: zircon_runtime
report_id: Runtime07-borrowed-zrvm-export-arguments-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed ZrVM Export Arguments

## Scope

This slice removes the deep clone of every owned `ScriptHostValue` passed to a real ZrVM export.
The export call receives an immutable argument slice, but the lowering pipeline previously used
`iter().cloned()` before converting each value. String and byte-array arguments therefore copied
their complete backing storage even though ZrVM constructors only read that storage.

## Change

- Change the shared ZrVM value-lowering helper to accept `&ScriptHostValue`.
- Lower the export argument slice directly through `iter().map(to_zr_value)`.
- Keep host callback return values owned while borrowing them only for the lowering operation.
- Preserve scalar, string, byte-array, and packed host-handle transport behavior.
- Extend real-backend tests so borrowed string and byte inputs remain usable after lowering.
- Add a Python source performance contract for the borrowed conversion shape.

## Deterministic Performance Evidence

The standalone optimized Rust model executes 131,072 export-equivalent calls over 31 alternating
samples. Each call reads four representative arguments: two strings, one 128-byte buffer, and one
integer. It isolates the removed host-input clone and deliberately excludes the ZrVM FFI and the
shared output-vector allocation. Both implementations produced checksum
`14389345025810844627` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 131,072 calls | 393,216 | 0 | 100.000% |
| Requested allocation bytes | 22,020,096 | 0 | 100.000% |
| Run 1 wrapper P50 | 53.6524 ms | 14.6422 ms | 72.709% |
| Run 1 wrapper P95 | 77.5711 ms | 40.0699 ms | 48.344% |
| Run 2 wrapper P50 | 58.6897 ms | 16.1304 ms | 72.516% |
| Run 2 wrapper P95 | 101.5908 ms | 28.1153 ms | 72.325% |

Evidence marker: `RUNTIME07_BORROWED_ZRVM_EXPORT_ARGUMENTS_MODEL_V1`.

The latency percentages apply only to the argument-clone wrapper represented by the model. They
are not an end-to-end ZrVM export-throughput claim.

## Validation

- `python tools/tests/test_runtime07_borrowed_zrvm_export_arguments_performance_contract.py`:
  4 passed after 3 of 4 pre-change checks failed.
- The standalone Rust model preserves all argument reads; two runs kept identical allocation
  profiles and checksums with positive P50/P95 results.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain the existing value
  conversion and round-trip tests.

## Remaining Parent-plan Work

This local copy removal does not remove the output `Vec<zrvm::Value>`, ZrVM's own value
materialization, the process-global VM lock, execution-budget gaps, typed ABI work, debugger and
profiler gaps, or product-scale editor/app/export/cook acceptance owned by the Runtime07 parent
plan.

---
title: Runtime07 Reused ZrVM Export Argument Buffer
category: zircon_runtime
report_id: Runtime07-reused-zrvm-export-argument-buffer-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Reused ZrVM Export Argument Buffer

## Scope

This slice removes the repeated heap allocation for the temporary `Vec<zrvm::Value>` used by real
ZrVM export calls. The preceding borrowed-argument slice removed deep copies, but still collected
a fresh output vector on every invocation. The ZrVM binding values are process-global raw-pointer
objects, so the reusable capacity remains inside the lock-guarded `ZrVmRuntimeOwner`.

## Change

- Keep one private lowered-argument buffer in each runtime owner.
- Move the buffer out under the process-wide ZrVM lock, clear and reserve it, then lower borrowed
  host arguments into it.
- Recycle the cleared buffer on both conversion failure and export-call completion.
- Recycle before unwrapping the returned value so no `zrvm::Value` is retained in the reusable
  buffer while return conversion runs.
- Clear the buffer during owner destruction before dropping the session, registrations, and runtime.
- Add behavior coverage for capacity reuse, pointer stability, and failure cleanup.
- Add a Python source performance contract for the owner-scoped buffer lifecycle.

## Deterministic Performance Evidence

The standalone optimized Rust model executes 262,144 export-equivalent calls with four scalar
arguments over 31 alternating samples. It isolates the temporary argument-vector container and
excludes host-value cloning, ZrVM FFI, and VM execution. Both implementations produced checksum
`11630431302037718930` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Container allocation calls per 262,144 calls | 262,144 | 1 | 99.9996% |
| Requested allocation bytes | 8,388,608 | 32 | 99.9996% |
| Run 1 wrapper P50 | 20.3566 ms | 1.0592 ms | 94.797% |
| Run 1 wrapper P95 | 45.4316 ms | 3.0968 ms | 93.184% |
| Run 2 wrapper P50 | 20.2012 ms | 1.0277 ms | 94.913% |
| Run 2 wrapper P95 | 43.3452 ms | 1.9421 ms | 95.519% |

Evidence marker: `RUNTIME07_REUSED_ZRVM_EXPORT_ARGUMENT_BUFFER_MODEL_V1`.

The latency percentages apply only to the temporary argument-vector container represented by the
model. They are not an end-to-end ZrVM export-throughput claim.

## Validation

- `python tools/tests/test_runtime07_reused_zrvm_export_argument_buffer_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- Runtime07 static contract batch: `233/233` passed before this slice; rerun with this contract is
  required before integration.
- The standalone Rust model preserves argument order and checksum; two runs kept matching
  allocation profiles and checksums with positive P50/P95 results.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain conversion, round-trip,
  and owner cleanup tests.

## Remaining Parent-plan Work

This local container reuse does not remove ZrVM value materialization, per-value FFI allocations,
the process-global VM lock, execution-budget gaps, typed ABI work, debugger and profiler gaps, or
product-scale editor/app/export/cook acceptance owned by the Runtime07 parent plan.

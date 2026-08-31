---
title: Runtime07 Borrowed ZrVM Export Identity Components
category: zircon_runtime
report_id: Runtime07-borrowed-zrvm-export-identity-components-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed ZrVM Export Identity Components

## Scope

This slice removes the remaining unconditional `module.export` formatting from the real ZrVM
export call. The earlier lazy-label slice removed the conversion wrapper's `"export "` prefix
allocation, but `call_export` still built the complete identity before argument lowering and export
dispatch. Successful calls and missing optional exports therefore materialized text used only by
return-conversion failures.

## Change

- Forward borrowed `module_name` and `export_name` components into return-value conversion.
- Remove the unconditional `format!("{module_name}.{export_name}")` from `call_export`.
- Interpolate the complete identity only inside existing typed error messages.
- Preserve exact `export module.export` diagnostics for unsupported values and byte-array errors.
- Update the earlier lazy-label source contract to the current two-component helper shape.
- Add a dedicated Python performance contract for the caller-side allocation removal.

## Deterministic Performance Evidence

The standalone optimized Rust model executes 262,144 export-identity projections over 31
alternating samples using `gameplay.player` and `update_controller`. It isolates the removed
caller-side formatting and excludes argument lowering, ZrVM FFI, and return conversion. Both
implementations produced checksum `7933287483413652612` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 262,144 calls | 786,432 | 0 | 100.000% |
| Requested allocation bytes | 27,525,120 | 0 | 100.000% |
| Run 1 wrapper P50 | 113.5761 ms | 4.7624 ms | 95.807% |
| Run 1 wrapper P95 | 146.3962 ms | 8.3957 ms | 94.265% |
| Run 2 wrapper P50 | 113.6573 ms | 4.6715 ms | 95.890% |
| Run 2 wrapper P95 | 177.4612 ms | 8.7574 ms | 95.065% |

Evidence marker: `RUNTIME07_BORROWED_ZRVM_EXPORT_IDENTITY_MODEL_V1`.

The latency percentages apply only to the identity-formatting wrapper represented by the model.
They are not an end-to-end ZrVM export-throughput claim.

## Validation

- `python tools/tests/test_runtime07_borrowed_zrvm_export_identity_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- The earlier lazy export-value-label contract remains active against the current helper shape.
- The standalone Rust model preserves the complete identity checksum; two runs kept identical
  allocation profiles and checksums with positive P50/P95 results.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain the existing contextual
  return-conversion error tests.

## Remaining Parent-plan Work

This local allocation removal does not remove ZrVM value materialization, the output argument
vector, the process-global VM lock, execution-budget gaps, typed ABI work, debugger and profiler
gaps, or product-scale editor/app/export/cook acceptance owned by the Runtime07 parent plan.

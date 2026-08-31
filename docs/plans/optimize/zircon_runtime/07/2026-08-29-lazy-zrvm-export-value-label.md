---
title: Runtime07 Lazy ZrVM Export Value Label
category: zircon_runtime
report_id: Runtime07-lazy-zrvm-export-value-label-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Lazy ZrVM Export Value Label

## Scope

This slice removes unconditional export-label formatting from real ZrVM return-value conversion.
The caller already supplies a complete `module.export` label, but the conversion wrapper previously
allocated `"export {label}"` before inspecting the returned value. Successful scalar and string
returns therefore paid for text used only by failure diagnostics.

## Change

- Pass the borrowed export label directly through scalar and byte-array conversion.
- Materialize the `export` prefix only inside the existing error-format branches.
- Preserve the exact error text for unsupported values and byte-array length, element, type, and
  range failures.
- Preserve successful return ownership and ZrVM value reads.
- Add a Python source performance contract for the lazy diagnostic-label shape.

## Deterministic Performance Evidence

The standalone optimized Rust model converts 262,144 scalar returns over 31 alternating samples
with the same `gameplay.player.update` export label. It isolates only the wrapper-label work, not
the complete ZrVM call or FFI cost. The legacy `format!` grows its output in two allocation steps;
the optimized success path retains the same label-derived checksum without materializing the
string. Both implementations produced checksum `17411752368240706263` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 262,144 scalar returns | 524,288 | 0 | 100.000% |
| Requested allocation bytes | 11,272,192 | 0 | 100.000% |
| Run 1 wrapper P50 | 69.4800 ms | 0.3561 ms | 99.487% |
| Run 1 wrapper P95 | 104.9346 ms | 0.5442 ms | 99.481% |
| Run 2 wrapper P50 | 69.1225 ms | 0.3516 ms | 99.491% |
| Run 2 wrapper P95 | 147.3999 ms | 0.5853 ms | 99.603% |

Evidence marker: `RUNTIME07_LAZY_ZRVM_EXPORT_VALUE_LABEL_MODEL_V1`.

The latency percentages above apply only to the removed label wrapper. They are not an end-to-end
ZrVM export throughput claim.

## Validation

- `python tools/tests/test_runtime07_lazy_zrvm_export_value_label_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- The standalone Rust model retains label participation in the result checksum; two runs kept
  identical allocation profiles and checksums with positive wrapper P50/P95 results.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain existing return/error
  conversion tests.

## Remaining Parent-plan Work

This local allocation removal does not replace product traces or resolve the process-global ZrVM
lock, argument cloning, typed ABI, execution budgets, debugger/profiler surface, or full product
acceptance owned by Runtime07.

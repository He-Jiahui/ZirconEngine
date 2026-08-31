---
title: Runtime07 Single ZrVM Host Callback Label
category: zircon_runtime
report_id: Runtime07-single-zrvm-host-callback-label-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Single ZrVM Host Callback Label

## Scope

This slice removes redundant function-name and label ownership while registering real ZrVM host
callbacks. Arity validation previously built a temporary `module.function` label even on success;
callback construction then cloned the function name and cloned that label again before retaining one
owned callback label.

## Change

- Format the qualified label only inside the four arity-error branches.
- Validate successful arities without allocating diagnostic text.
- Build exactly one owned callback label after validation, directly from borrowed descriptor names.
- Preserve callback closure ownership, function metadata, arity checks, diagnostics, capability
  capture, and ZrVM registration behavior.
- Add a Python source performance contract for the single-label registration shape.

## Deterministic Performance Evidence

The standalone optimized Rust model registers 4,096 valid host functions over 31 alternating
samples. Each implementation retains one final callback label per function; only temporary
validation/name/label ownership is compared. Both implementations produced checksum
`17411752368240706263` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 4,096 functions | 32,769 | 12,289 | 62.498% |
| Requested allocation bytes | 1,015,808 | 471,040 | 53.629% |
| Run 1 P50 | 4.1253 ms | 2.1356 ms | 48.232% |
| Run 1 P95 | 8.3857 ms | 3.6486 ms | 56.490% |
| Run 2 P50 | 4.0262 ms | 2.0666 ms | 48.671% |
| Run 2 P95 | 6.9448 ms | 2.8895 ms | 58.393% |

Evidence marker: `RUNTIME07_SINGLE_ZRVM_HOST_CALLBACK_LABEL_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_single_zrvm_host_callback_label_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- The standalone Rust model retains one owned callback label per function and equivalent checksum;
  two runs kept identical allocation profiles and positive P50/P95 results.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain all arity error-message
  coverage.

## Remaining Parent-plan Work

This local registration optimization does not resolve the process-global ZrVM lock, execution
budgets, typed ABI, debugger/profiler surface, or product-scale editor/app/export/cook acceptance
owned by Runtime07.

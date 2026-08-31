---
title: Runtime07 Shared ZrVM Extension Host Caller
category: zircon_runtime
report_id: Runtime07-shared-zrvm-extension-host-caller-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Shared ZrVM Extension Host Caller

## Scope

This slice removes repeated deep copies of the authenticated `VmInterfaceCaller` when registering
the ZrVM extension host module. The caller contains a variable-length `CapabilitySet`; four native
callback closures previously cloned that capability vector independently.

## Change

- Store the authenticated caller in one `Arc<VmInterfaceCaller>` during host-module setup.
- Give each callback an `Arc` reference and borrow the caller for registry operations.
- Preserve caller slot/generation/capabilities, registry clones, callback ownership, capability
  checks, error mapping, and registration order.
- Add a Python source performance contract for shared caller ownership.

## Deterministic Performance Evidence

The standalone optimized Rust model registers four callbacks with a 256-entry capability set over
31 alternating samples. It retains one owned caller and four callback captures in both variants;
only the deep capability-vector copies are removed. Both implementations produced checksum
`5065451274042193045` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per four callback captures | 1,029 | 1 | 99.903% |
| Requested allocation bytes | 55,392 | 32 | 99.942% |
| Run 1 P50 | 0.1680 ms | 0.0003 ms | 99.821% |
| Run 1 P95 | 0.3936 ms | 0.0006 ms | 99.848% |
| Run 2 P50 | 0.1911 ms | 0.0003 ms | 99.843% |
| Run 2 P95 | 0.8899 ms | 0.0009 ms | 99.899% |

Evidence marker: `RUNTIME07_SHARED_ZRVM_EXTENSION_HOST_CALLER_MODEL_V1`.

The latency percentages apply to callback-capture setup only, not to later host calls or ZrVM
execution.

## Validation

- `python tools/tests/test_runtime07_shared_zrvm_extension_host_caller_performance_contract.py`:
  4 passed after 3 failures and 1 missing-shape failure on the pre-change source.
- The standalone Rust model retains four callback references and equivalent checksum; two runs kept
  stable allocation profiles and positive P50/P95 results.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain all extension caller
  authorization tests.

## Remaining Parent-plan Work

This local setup optimization does not resolve the process-global ZrVM lock, execution budgets,
typed ABI, debugger/profiler surface, or product-scale editor/app/export/cook acceptance owned by
Runtime07.

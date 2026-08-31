---
title: Runtime07 Shared ZrVM Host Capability Set
category: zircon_runtime
report_id: Runtime07-shared-zrvm-host-capability-set-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Shared ZrVM Host Capability Set

## Scope

This slice removes repeated deep copies of the host `CapabilitySet` while registering native ZrVM
functions. The capability vector is immutable during registration and callback execution, so one
owned `Arc<CapabilitySet>` can be shared by all function builders.

## Change

- Clone the host capability set once at the host-module registration boundary.
- Capture an `Arc<CapabilitySet>` in each native callback instead of copying its `Vec<String>`.
- Borrow the shared capability set during `ScriptCallSite::call`.
- Preserve capability membership, callback ownership, function metadata, registration order, and
  all existing authorization/error behavior.
- Add a Python source performance contract for the shared capability ownership shape.

## Deterministic Performance Evidence

The standalone Rust model registers 4,096 native functions against a 128-entry capability set over
31 alternating samples. It isolates capability capture and excludes ZrVM module construction,
FFI registration, and callback execution. Both implementations produced checksum
`17411752368240706263` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Capability capture allocation calls | 528,385 | 1 | 99.9998% |
| Requested allocation bytes | 28,409,856 | 32,768 | 99.8846% |
| Run 1 wrapper P50 | 119.1663 ms | 0.0452 ms | 99.962% |
| Run 1 wrapper P95 | 173.7268 ms | 0.1347 ms | 99.923% |
| Run 2 wrapper P50 | 133.1263 ms | 0.0493 ms | 99.963% |
| Run 2 wrapper P95 | 170.1353 ms | 0.1381 ms | 99.919% |

Evidence marker: `RUNTIME07_SHARED_ZRVM_HOST_CAPABILITY_SET_MODEL_V1`.

The latency percentages apply only to capability capture represented by the model; they are not an
end-to-end ZrVM registration or host-call throughput claim.

## Validation

- `python tools/tests/test_runtime07_shared_zrvm_host_capability_set_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- The standalone Rust model compiled with `rustc -C opt-level=3` and passed twice with identical
  allocation profiles and checksums.
- Exact-file Rust/model formatting, Python compilation, the Runtime07 source-contract batch, and
  scoped diff checks are required before snapshot publication.
- Managed tests must compile the real `backend-zr-vm` feature and retain capability authorization
  coverage for native host callbacks.

## Remaining Parent-plan Work

This local ownership optimization does not remove ZrVM value materialization, the process-global VM
lock, execution-budget gaps, typed ABI work, debugger/profiler gaps, or product-scale
editor/app/export/cook acceptance owned by the Runtime07 parent plan.

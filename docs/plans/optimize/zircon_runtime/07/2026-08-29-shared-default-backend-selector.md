---
title: Runtime07 Shared Default Backend Selector
category: zircon_runtime
report_id: Runtime07-shared-default-backend-selector-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Shared Default Backend Selector

## Scope

This slice removes the `String` allocation performed whenever `VmPluginManager::load_package`
reads its stable default backend selector. The manager now shares selector storage internally while
retaining the public owned-name accessor for compatibility.

## Change

- Store the selected backend as `RwLock<Arc<str>>`.
- Add a private shared-selector accessor that only increments the `Arc` reference count.
- Route default package loading through the shared accessor.
- Preserve `selected_backend_name() -> String`, backend availability validation, selector updates,
  poisoned-lock recovery, and all explicit-backend load paths.
- Keep the existing runtime-owned discovery worker changes in the same file unchanged.
- Add a Python source contract for the shared internal read path and owned public API boundary.

## Performance Target

For 262,144 repeated selector reads, the isolated model must eliminate all read-side allocation and
improve P95 read time by at least 75% without changing selector content or the output checksum.

## Deterministic Performance Evidence

The standalone optimized Rust model reads a 36-byte selector through `RwLock` over 31 alternating
samples. It includes lock acquisition and release, `String` clone or `Arc` reference-count clone,
and selector consumption while excluding backend resolution and package loading. Both
implementations produced checksum `17411752368240706263` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 262,144 reads | 262,144 | 0 | 100.000% |
| Requested allocation bytes | 9,437,184 | 0 | 100.000% |
| Run 1 read P50 | 20.3926 ms | 2.7747 ms | 86.394% |
| Run 1 read P95 | 67.9666 ms | 11.5330 ms | 83.031% |
| Run 2 read P50 | 20.5649 ms | 2.7370 ms | 86.691% |
| Run 2 read P95 | 75.5047 ms | 6.8664 ms | 90.906% |

Evidence marker: `RUNTIME07_SHARED_DEFAULT_BACKEND_SELECTOR_MODEL_V1`.

The performance target is met in both runs. These percentages apply only to selector reads; they
are not an end-to-end VM package load latency claim.

## Validation

- The Python source contract failed all 4 checks against the old `RwLock<String>` path and passed
  all 4 checks after shared storage was introduced.
- The standalone model compiled with `rustc +1.94.1 -C opt-level=3` and passed twice with identical
  allocation profiles and checksums.
- Exact-file formatting, Python compilation, the Runtime07 source-contract batch, and scoped diff
  checks are required before snapshot publication.
- Managed Runtime tests must compile default and explicit backend load paths plus poisoned-lock
  recovery before integration.

Managed batch request: `runtime07-vm-gc-six-task-batch-20260830-v1`.

Validation attempt: ticket `a45b8eb5c82d46bab783834a6da58f6a` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

This selector ownership optimization does not change backend resolution, package discovery,
process-global ZrVM locking, execution budgets, typed ABI work, debugger/profiler gaps, or
product-scale editor/app/export/cook acceptance owned by the Runtime07 parent plan.

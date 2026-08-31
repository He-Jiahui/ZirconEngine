---
title: Runtime07 Borrowed Active Registration Identity
category: zircon_runtime
report_id: Runtime07-borrowed-active-registration-identity-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Active Registration Identity

## Scope

This slice removes the `String` clone performed for every candidate row while `latest_active`
selects the newest eligible VM host-interface registration. Registration keys outlive the local
selection map, so the identity can borrow `key.id.as_str()` until selection and sorting finish.

## Change

- Store `(PluginSlotId, &str)` in the temporary latest-generation selection map.
- Preserve active-generation filtering, include predicates, newest-generation replacement,
  registration cloning, and deterministic identity sorting.
- Keep the existing editor-operation segment validation optimization in the same source file
  unchanged.
- Add a Python source contract that rejects owned registration identity projection.

## Performance Target

For 8,192 logical identities with four generations each, the isolated model must reduce allocation
calls by at least 95%, requested allocation bytes by at least 25%, and P95 selection time by at
least 40% without changing selected generations or the output checksum.

## Deterministic Performance Evidence

The standalone optimized Rust model processes 32,768 registration rows over 31 alternating
samples. It retains HashMap selection, latest-generation replacement, result collection, sorting,
and value copying while excluding registry locking and active-slot construction. Both
implementations produced checksum `17411752368240706263` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per selection | 32,783 | 15 | 99.954% |
| Requested allocation bytes | 2,981,900 | 1,998,892 | 32.966% |
| Run 1 selection P50 | 14.3668 ms | 11.1116 ms | 22.658% |
| Run 1 selection P95 | 79.4970 ms | 24.7754 ms | 68.835% |
| Run 2 selection P50 | 15.8915 ms | 12.3595 ms | 22.226% |
| Run 2 selection P95 | 35.6692 ms | 19.5456 ms | 45.203% |

Evidence marker: `RUNTIME07_BORROWED_ACTIVE_REGISTRATION_IDENTITY_MODEL_V1`.

The performance target is met in both runs. These percentages apply only to latest-registration
selection; they are not an end-to-end active snapshot publication latency claim.

## Validation

- The Python source contract failed 2 of 3 checks against the old owned identity and passed all 3
  checks after the borrowed-key change.
- The standalone model compiled with `rustc +1.94.1 -C opt-level=3` and passed twice with identical
  allocation profiles and checksums.
- Exact-file formatting, Python compilation, the Runtime07 source-contract batch, and scoped diff
  checks are required before snapshot publication.
- Managed Runtime tests must compile and exercise active system, behavior, RPC, and editor
  registration selection before integration.

Managed batch request: `runtime07-vm-gc-six-task-batch-20260830-v1`.

Validation attempt: ticket `a45b8eb5c82d46bab783834a6da58f6a` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

This temporary-key optimization does not change active snapshot value ownership, package
resolution, backend execution, the process-global ZrVM lock, execution budgets, typed ABI work,
debugger/profiler gaps, or product-scale editor/app/export/cook acceptance owned by the Runtime07
parent plan.

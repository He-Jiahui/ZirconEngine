---
title: Runtime07 Borrowed VM Backend Selectors
category: zircon_runtime
report_id: Runtime07-borrowed-vm-backend-selectors-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed VM Backend Selectors

## Scope

This slice removes the temporary owned selector vector returned by every `VmBackendFamily` while
`VmBackendRegistry::names` builds its final owned result. Family selectors remain borrowed only
during a synchronous object-safe visitor call; the registry still owns every returned name.

## Change

- Replace `selectors() -> Vec<String>` with `visit_selectors(&mut dyn FnMut(&str))`.
- Append borrowed selectors directly into the registry result vector.
- Hard-cut all Runtime and ZrVM family implementations to the visitor contract.
- Preserve selector order, final sort/dedup behavior, dynamic family-owned selectors, aliases,
  unknown-selector errors, and poisoned-lock recovery.
- Add a Python source contract that rejects the old owned family projection.

The existing ZrVM runtime module document remains truthful and does not describe selector return
ownership, so this public trait change requires no module-document edit.

## Performance Target

For a 4,096-family, four-selector registry projection, the isolated model must reduce allocation
calls by at least 15%, requested allocation bytes by at least 25%, and P95 projection time by at
least 40% without changing the output checksum.

## Deterministic Performance Evidence

The standalone optimized Rust model projects 16,384 selector names over 31 alternating samples.
It includes trait-object dispatch and final owned string construction while excluding registry
locking, sorting, deduplication, and backend resolution. Both implementations produced checksum
`17411752368240706263` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per projection | 20,481 | 16,385 | 19.999% |
| Requested allocation bytes | 1,130,496 | 737,280 | 34.783% |
| Run 1 projection P50 | 3.4708 ms | 3.2180 ms | 7.284% |
| Run 1 projection P95 | 22.6377 ms | 10.2103 ms | 54.897% |
| Run 2 projection P50 | 3.7174 ms | 3.2918 ms | 11.449% |
| Run 2 projection P95 | 16.6799 ms | 7.4506 ms | 55.332% |

Evidence marker: `RUNTIME07_BORROWED_VM_BACKEND_SELECTORS_MODEL_V1`.

The performance target is met in both runs. These percentages apply only to selector projection;
they are not an end-to-end plugin discovery or backend resolution throughput claim.

## Validation

- The Python source contract failed against the old trait and passed all 4 checks after the hard
  cut.
- The standalone model compiled with `rustc +1.94.1 -C opt-level=3` and passed twice with identical
  allocation profiles and checksums.
- Exact-file formatting, Python compilation, source-contract batching, and scoped diff checks are
  required before snapshot publication.
- Managed tests must compile Runtime and ZrVM trait implementations together before integration.

Managed batch request: `runtime07-vm-gc-six-task-batch-20260830-v1`.

Validation attempt: ticket `a45b8eb5c82d46bab783834a6da58f6a` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

This projection optimization does not change package resolution, backend execution, the
process-global ZrVM lock, execution budgets, typed ABI work, debugger/profiler gaps, or product-scale
editor/app/export/cook acceptance owned by the Runtime07 parent plan.

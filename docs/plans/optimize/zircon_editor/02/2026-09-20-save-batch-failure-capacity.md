---
title: Editor858 save-batch failure capacity
category: zircon_editor
report_id: Editor858-save-batch-failure-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor858 - save-batch failure capacity

## Scope

`SaveDirtyViewsRequest::prepare` first materializes and sorts the candidate
batch, then records every duplicate, stale, toolkit, resource, writability,
reference, and byte-overflow failure before publishing the request. The
candidate batch is therefore a conservative upper bound for the failure output,
but that vector previously started at zero capacity.

## Optimization

- Reserve `candidates.len()` immediately after candidate collection and before
  preflight failure checks.
- Preserve sorted document order, failure multiplicity/order, partial failure
  reporting, and successful intent publication.
- Leave completion application, toolkit indexing, and tooling production
  untouched.

## TDD and deterministic evidence

The Python source/model contract was run RED before the reservation and lower
owner existed, then GREEN at `4/4`. The lower Rust owner checks failure order and
empty batches and carries the ignored
`EDITOR858_SAVE_BATCH_FAILURE_CAPACITY_BENCH_V1` marker. A 4,096-candidate
failure model changes modeled geometric growth from `12` to `0`.

## Local validation

- Standalone `rustc --edition 2021 --test` execution of the lower owner passes
  `2/2` non-ignored tests; its single benchmark marker remains intentionally
  ignored for the managed Windows Release lane.
- `tools/tests/test_editor_save_batch_failure_capacity_performance_contract.py`:
  `4/4`.
- The combined focused Runtime/Editor source-contract batch, including the
  preceding V2/render/material slices plus Runtime857/858, Editor857, Runtime859,
  and Editor858, passes `46/46` tests with zero failures, errors, or skips.
- Exact-file `rustfmt --edition 2021 --check` passes for the production and
  lower Rust owners; `python -m py_compile` passes for both new contracts.
- The refreshed expanded non-tooling source-contract loader covers `967` files
  and passes `4092/4092` tests in `375.582s` under the explicit
  performance-or-contract filename filter (tooling, export, and coordinator
  files excluded), with zero load errors, failures, errors, or skips. The run
  also printed two existing shader-prewarm fixture Cargo command lines inside
  tests; it was not a managed Windows Release/Cargo acceptance run.
- No managed Windows Cargo/Release validation command was started locally.
  Allocator and save-preflight product p50/p95/p99 evidence remain pending
  behind the shared external worktree gate; isolated contract fixtures are not
  managed acceptance evidence.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/asset/dirty/save_batch.rs` | `9FA5211B5C33C1B95EECE26C29ADCC54BB761022284D4DCC6ED0E50A5406A96A` |
| `zircon_editor/src/core/asset/dirty/save_batch/capacity_tests.rs` | `1EDD04F6BD58BE6B329E124C061079FD09DB1B292C6B2BF4D530ADB20AE6912E` |
| `tools/tests/test_editor_save_batch_failure_capacity_performance_contract.py` | `D48C3C96BD62529DD79E501C351731EB1B566D42DE77D1D4C8227F7022E7BD19` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Editor tree,
executes the lower regression and ignored marker, and supplies allocator plus
save-preflight product p50/p95/p99 measurements. Tooling production remains
deferred for the later Rust migration; coordinator status is not polled here.

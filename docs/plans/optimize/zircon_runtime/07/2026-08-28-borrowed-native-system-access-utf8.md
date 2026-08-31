---
title: Runtime07 Borrowed Native System Access UTF-8
category: zircon_runtime
report_id: Runtime07-borrowed-native-system-access-utf8-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Native System Access UTF-8

## Scope

This slice removes transient UTF-8 ownership and geometric final-string growth while native ABI V4
system access declarations are decoded. It preserves pointer and length validation, UTF-8 error
sources, access-mode and domain validation precedence, output ordering, and the exact
`mode:domain:stable_id` contract.

## Change

- Add a closure-scoped `read_utf8_with` decoder so validated UTF-8 may be consumed without allowing
  a borrow from foreign ABI storage to escape the unsafe decode boundary.
- Keep `read_utf8` behavior unchanged by delegating its owned-string projection through the same
  validated helper.
- Format each system access ID directly from the borrowed stable ID, eliminating the temporary
  owned stable-ID string.
- Build the final access ID with the exact mode, separators, domain, and stable-ID byte capacity so
  each result needs one allocation rather than repeated `format!` growth.
- Add Rust exact-output regressions for borrowed UTF-8 mapping and access-ID construction, plus a
  Python performance structure contract.

## Deterministic Performance Evidence

The standalone optimized Rust model decodes the ABI maximum of 4,096 access declarations across 31
alternating samples, covering read/write and component/resource variants with owned fixture storage
exposed as borrowed UTF-8. It asserts complete `Vec<String>` equality for every measured pair. Both
paths produced checksum `15965353494038305436`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 16,385 | 4,097 | 74.995% |
| Requested allocation bytes | 544,086 | 302,422 | 44.417% |
| Decode P50 | 2.3793 ms | 0.7482 ms | 68.554% |
| Decode P95 | 4.6717 ms | 1.3335 ms | 71.456% |

Evidence marker: `RUNTIME07_BORROWED_NATIVE_SYSTEM_ACCESS_UTF8_MODEL_V1`.

The borrow-only intermediate version was not accepted because a repeat P95 regressed. Adding exact
final-string capacity removed the remaining per-row growth and produced stable results; a second
full run improved P50 by 68.626% and P95 by 61.210%.

## Validation

- `python tools/tests/test_runtime07_borrowed_native_system_access_utf8_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks; the exact-capacity extension then
  failed the expected 1 of 3 checks before the final implementation.
- The standalone Rust model compiled with Rust 1.94.1, asserts complete ordered output equality,
  and passed two final 31-sample runs.
- Existing ABI tests retain invalid UTF-8 source preservation, unknown-stage typing, pointer
  validation, and V4 registration coverage.
- The existing `test_plugins_01_host_api_adapter_boundary.py` suite ran 13 tests with 11 passing;
  its two failures predate and do not intersect this candidate: the current untracked adapter split
  lacks `ecs_registration/tests.rs`, and the contract still expects the V4 policy struct in
  `registration_policy/mod.rs` although the current untracked implementation moved it to
  `policy.rs`.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch; this candidate will be paired with another completed optimization.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.

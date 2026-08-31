---
title: Runtime07 Preallocated VM Reflection Registrations
category: zircon_runtime
report_id: Runtime07-preallocated-vm-reflection-registrations-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated VM Reflection Registrations

## Scope

This slice removes growth reallocations from the public-component registration vector built by
`VmReflectionSchema::from_state_schema`. The state schema already provides a safe row-count upper
bound, so the projection can allocate once before filtering without changing declaration order or
reflection validation.

## Change

- Preallocate the registration vector with `schema.types.len()` before the existing single pass.
- Reuse one predicate for public, component, and non-resource visibility checks.
- Preserve `TypeRegistry::register_vm_type` validation, stable declaration order, cloned
  registration ownership, and error propagation.
- Add a Python source performance contract that rejects an empty growth vector and a separate
  counting scan.

An exact eligible-row counting prototype was measured and rejected because its second scan caused
unstable P95 regressions. The accepted upper-bound strategy trades at most unused vector capacity
for one-pass construction and stable tail latency.

## Performance Target

For 8,192 schema rows with a 3/4 public-component ratio, the isolated projection model must reduce
allocation calls by at least 90%, requested allocation bytes by at least 45%, and P95 projection
time by at least 20% in both runs. P50 regression must remain below 5%. The output checksum must be
identical.

## Deterministic Performance Evidence

The standalone optimized Rust model projects 6,144 registrations from 8,192 schema rows. Each
latency sample performs 256 full projections, with 31 alternating samples per run; allocation
profiles cover one projection. Both implementations produced checksum
`15732308755787758795` in both runs.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per projection | 12 | 1 | 91.667% |
| Requested allocation bytes | 262,080 | 131,072 | 49.988% |
| Run 1 projection P50 | 6.6273 ms | 6.7733 ms | -2.203% |
| Run 1 projection P95 | 18.9844 ms | 13.1181 ms | 30.901% |
| Run 2 projection P50 | 5.4456 ms | 5.7109 ms | -4.872% |
| Run 2 projection P95 | 9.7595 ms | 7.3883 ms | 24.296% |

Evidence marker: `RUNTIME07_PREALLOCATED_VM_REFLECTION_REGISTRATIONS_MODEL_V1`.

The performance target is met in both runs. These measurements isolate registration-vector
construction; they are not an end-to-end reflection schema installation latency claim.

## Validation

- The final Python source contract failed 2 of 3 checks against the counting-scan implementation
  and passed all 3 checks after the one-pass upper-bound change.
- The standalone model compiled with `rustc +1.94.1 -C opt-level=3` and passed twice with identical
  allocation profiles and checksums.
- Exact-file Rust formatting, Python compilation, the Runtime07 source-contract batch, and scoped
  diff checks are required before snapshot publication.
- Managed Rust compilation and reflection tests remain pending in the next asynchronous Runtime07
  validation batch.

Managed batch request: `runtime07-vm-gc-six-task-batch-20260830-v1`.

Validation attempt: ticket `a45b8eb5c82d46bab783834a6da58f6a` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

This vector allocation optimization does not change reflection registration ownership, schema
generation, hot-reload staging, typed marshalling, execution budgets, debugger/profiler gaps, or
product-scale editor/app/export/cook acceptance owned by the Runtime07 parent plan.

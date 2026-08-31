---
title: Runtime09H2 Zero-Clone Effect Disable
category: zircon_runtime
report_id: Runtime09H2-zero-clone-effect-disable-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-zero-clone-disable-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 zero-clone effect disable

## Scope

- Parent scope: the Runtime09H2 post-process stack mutation path and its CPU performance qualification.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: `PostProcessStackDescriptor::with_effect_disabled`, its source/performance contract, direct Rust behavior tests, and this record.
- This slice removes owned resource-name copies and linear membership scans when a compile option disables an effect. It does not close HDR correctness, effect algorithms, GPU timing, quality tiers, visual oracles, or the remaining Runtime09H2 acceptance gates.

## Change

- Matching provider output vectors are temporarily moved out of their effects instead of cloning every resource name.
- A borrowed `HashSet<&str>` is reserved to the exact disabled-output reference count and used for consumer input removal.
- After the mutation pass, every provider output vector is restored at its original effect index, preserving disabled-effect metadata and effect order.
- The existing rule that removes `after(kind)` dependencies even when no matching provider exists remains intact.
- Three direct Rust tests cover output restoration, multiple matching providers, consumer pruning, and dangling dependency cleanup.

For a 4,096-effect plugin-scale stack with 2,048 disabled outputs, the model reduces membership pruning from repeated linear string scans to one preallocated borrowed index. No resource-name allocation remains in the optimized mutation.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_zero_clone_effect_disable_performance_contract -v` initially failed 4/4 against the cloned `Vec<String>` implementation.
- During RED review, an initially proposed absent-provider early return was rejected because it would have changed the existing dangling-dependency cleanup contract.
- GREEN: the corrected source contract passes 4/4 after the borrowed index and metadata restoration are implemented.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and `git diff --check` pass for the owned files.
- The standalone model is compiled with `rustc +1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/optimized sample pairs over 4,096 effects, eight required inputs per consumer, and 2,048 outputs from the disabled provider. Input construction and cloning are excluded from the timed mutation. It checks the complete metadata checksum and provider output equality, and a process-local allocator counts mutation allocations. Three local runs passed the acceptance thresholds; the table records the latest run.

| Metric | Cloned output `Vec` + linear scans | Moved outputs + borrowed hash index | Change |
|---|---:|---:|---:|
| P50 | 231.2452 ms | 4.8506 ms | -97.902% |
| P95 | 339.9285 ms | 8.2176 ms | -97.583% |
| allocations / mutation | 2,049 | 2 | -99.902% |

The other two runs produced P50 reductions of 97.776% and 97.675%, P95 reductions of 90.538% and 95.201%, and the same 99.902% allocation reduction. These timings isolate CPU stack mutation for plugin-scale descriptors; they do not claim GPU frame time or complete pipeline compile latency.

## Async validation

One coordinator batch must run the four Python source contracts, all 23 post-process stack Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 23/23 Rust tests to pass, exact metadata checksum and provider-output parity, allocation reduction of at least 99%, and P50/P95 reductions of at least 80%. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as 4,096-effect CPU stack-mutation evidence.

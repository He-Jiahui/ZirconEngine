---
title: Runtime98 Hashed GPU Cache Membership
category: zircon_runtime
report_id: Runtime98-hashed-gpu-cache-membership-2026-08-28
date: 2026-08-28
session_id: root-runtime98-hashed-gpu-cache-membership-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime98 Hashed GPU Cache Membership

## Scope

`HybridGiRuntimeState::apply_gpu_cache_entries` consumes a GPU readback snapshot, rejects probes
without live feedback, retains the first slot for each live probe, evicts CPU residents absent from
the snapshot, and then promotes retained entries in input order. The previous implementation used
a dynamically grown vector and two `BTreeSet` membership indexes: one for duplicate rejection and
a second rebuilt from the unique vector for eviction checks.

The completion path now preallocates its unique entry vector and one `HashSet` from the input
bound. The same live-ID set performs duplicate rejection and resident eviction membership. The set
is never iterated, so hash iteration order cannot affect observable behavior; the unique vector
continues to own first-slot-wins and promotion order, while resident eviction keeps the existing
resident map order.

## Performance Evidence

The isolated Rust model mirrors 65,536 GPU cache entries, duplicates, live-probe filtering, 32,768
prior residents, first-slot-wins ordering, and resident eviction order. It compares the previous
dynamically grown vector and two `BTreeSet` indexes with the capacity-bounded vector and one reused
`HashSet`. Each run uses 31 alternating sample pairs with two repetitions and was compiled with
`rustc +1.94.1 -O -C target-cpu=native` on Windows.

| Metric | Two-tree path | Reused hash-membership path | Change |
|---|---:|---:|---:|
| Allocator calls | 15,912 | 30 | -99.811% |
| Cumulative requested bytes | 3,979,312 | 2,621,440 | -34.123% |
| P50 | 23,070,400 ns | 5,112,400 ns | -77.840% |
| P95 | 37,941,900 ns | 6,554,500 ns | -82.725% |

The baseline and optimized checksums remained identical at `10,253,912,412,054,652,440`.

Model sources:

- `.codex/state/session-coordinator/plugins19-hashed-gpu-cache-entry-membership-model.rs`
- `.codex/state/session-coordinator/plugins19-hashed-gpu-cache-entry-membership-model-result.md`

The model retains its original Plugins19 filename because the candidate was first investigated
against that older review. Runtime98 lists Plugins19 as a plan source and is the current HGI owner.
The isolated measurements do not replace managed Cargo behavior tests or GPU frame profiling.

## Contracts And Validation

- `tools/tests/test_runtime98_hashed_gpu_cache_membership_performance_contract.py` locks one
  capacity-bounded hash index, reuse across deduplication and eviction, unique-vector capacity,
  first-slot-wins insertion, input-order promotion, the focused Rust behavior regression, and the
  absence of a second resident index.
- TDD RED failed all three contract tests against the two-tree path; the implemented contract
  plus its focused behavior-test guard now passes 4/4.
- Python bytecode compilation, scoped `rustfmt +1.94.1 --edition 2021 --check`, and scoped
  `git diff --check` pass.
- The post-implementation model passes its allocation, byte, P50, P95, and checksum gates.
- A focused Rust regression now covers dead-probe filtering, absent-resident eviction,
  first-slot-wins duplicate handling, and input-order promotion under a slot conflict.
- Cargo compilation and that focused behavior regression are submitted together in a managed
  asynchronous coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime98 still owns the GPU-resident authority cutover, sparse feedback, virtual residency,
material-correct surface capture, hierarchical trace, directional radiance, reconstruction,
single composition ownership, and full GPU qualification. This slice only bounds and accelerates
membership work in the existing CPU completion bridge; it does not close the parent readback P0.

# Runtime64 Allocation-Free No-Op Readiness Updates

Plan: docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
Milestone: M64.10 focused scale slice
Status: implementation_complete; managed_validation_pending
Files: ["docs/plans/optimize/zircon_runtime/64/2026-08-26-allocation-free-noop-readiness-updates.md","tools/tests/test_runtime64_readiness_noop_performance_contract.py","zircon_runtime/src/core/resource/manager/readiness_projection.rs"]

- Date: 2026-08-26
- Integration owner: `root-runtime64-allocation-free-noop-readiness-20260826`
- Plan items: readiness immutable-generation foundation, M64.10 100K scale/soak, RAR-G46
  commit/read P95 evidence

## Problem

`ResourceReadinessProjection::apply_updates(...)` constructed a new
`Arc<ResourceRecord>` for every incoming `Some(record)` before checking whether the source already
contained the same record, runtime state, and payload type. Stable-frame or repeated publication
batches therefore allocated and destroyed one Arc box per resource even though `changed_ids`
remained empty and the immutable readiness generation was correctly reused.

The changed path also borrowed the newly constructed source and cloned it into `self.sources`, then
dropped the local source. That added one avoidable Arc increment/decrement pair to every actual
change.

## Scope Delivered

- `source_matches_update(...)` compares the incoming owned record by reference against the current
  Arc payload before any new Arc is constructed.
- The preflight preserves the previous derived-`PartialEq` semantics exactly: `Some/Some` compares
  record, runtime state, and payload type; `None/None` is unchanged; mixed presence is a change.
- No-op `Some` updates now return without source allocation, reverse-dependency mutation, closure
  traversal, or generation publication.
- Changed `Some` updates move the newly constructed source into the map instead of cloning it.
- A Rust behavior test proves identical `Some` updates preserve both source-record and generation
  Arc identity, while an absent `None` update preserves the empty generation.

## TDD And Verification Evidence

- RED: the initial focused contract reported `2/2` errors because the borrowed preflight helper did
  not exist and `apply_updates(...)` still allocated before comparison.
- GREEN after implementation: the two source contracts passed.
- A Rust behavior-test presence contract was then added RED and became GREEN after the identity
  regression was added. The final focused contract passes `3/3`.
- `rustfmt +1.94.1 --check` passed for the owned Rust file.
- `git diff --check` passed for the owned source and contract. The new Rust behavior test remains
  pending the asynchronous managed Cargo batch because direct Cargo execution is prohibited.

## Performance Evidence

The independent `rustc +1.94.1 -O` model uses the same old allocate-then-compare kernel and the new
borrowed preflight kernel. It preloads 131,072 current sources, submits 131,072 identical owned
record updates, counts allocations through a process global allocator, warms both paths three times,
and records 21 alternating legacy/preflight sample pairs with nearest-rank P50/P95.

Deterministic allocation result in every sample:

- legacy source Arc allocations: `131,072`;
- preflight source Arc allocations: `0`;
- reduction: `100%`.

Two passing release-mode executions produced:

| Run | Legacy P50 | Preflight P50 | P50 reduction | Legacy P95 | Preflight P95 | P95 reduction |
|---|---:|---:|---:|---:|---:|---:|
| A | 59.891 ms | 33.167 ms | 44.6207% | 81.077 ms | 41.224 ms | 49.1549% |
| B | 55.024 ms | 33.681 ms | 38.7890% | 72.475 ms | 43.110 ms | 40.5172% |

The managed validation gate requires zero preflight allocations, exactly 131,072 legacy
allocations, at least 25% lower P50, and at least 15% lower P95. Both passing runs clear every gate;
the lower timing thresholds retain margin for shared-machine tail noise while the zero-allocation
requirement remains exact.

## Remaining Scope

This slice removes stable no-op readiness allocation and one changed-path Arc clone. It does not
close Runtime64's exact type/schema admission, asynchronous load tickets, frame-thread I/O removal,
version leases, cache budgets, dependency SCC validation, project lifecycle, fault injection,
100K/1M concurrent authority benchmark, or Unreal comparison requirements. Those remain open and
must not be inferred from this focused projection result.

## 2026-08-31 Current-Source Reconciliation

The readiness projection now canonicalizes dependency IDs so reordered or duplicate dependency
inputs preserve generation identity. The first implementation cloned every incoming
`ResourceRecord` during preflight, including the stable identical-update path, regressing this
slice's zero-allocation contract.

- Runtime state and payload type mismatches now reject before record comparison.
- Byte-for-byte identical records return through the borrowed fast path before any clone.
- Only a record that differs from the stored canonical form is cloned, sorted, and deduplicated.
- The existing reordered/duplicate dependency identity test is active rather than ignored, so the
  allocation-free fast path and canonicalizing slow path are both covered.
- Static Runtime performance contracts pass `970/970`; managed Cargo validation remains pending in
  the coordinated batch.

---
title: Runtime Borrowed Handwritten Metadata Dependency Index
category: zircon_runtime
report_id: Runtime865-borrowed-handwritten-meta-dependency-index-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime865 Borrowed Handwritten Metadata Dependency Index

## Finding

Runtime643 removed two linear membership scans, but its metadata/root indexes
owned cloned `AssetUri` values. Each accepted candidate was then cloned again
for the meta index, meta destination, and root index before the original value
was finally moved into the root destination. Existing dependency URIs were also
cloned into both indexes even though classification is completed before either
destination mutates.

## Optimization

- Build capacity-sized `HashSet<&AssetUri>` indexes borrowing the existing meta
  and root dependency vectors.
- Classify the immutable incoming vector once and store meta/root admission in
  one byte per candidate, with exact addition counts for both destinations.
- Drop both borrowed indexes before mutation, reserve the exact accepted count
  in each destination, and consume the incoming vector.
- Move single-destination candidates and clone only a candidate that must be
  owned by both meta and root lists.
- Preserve the early-empty path, one root lookup, existing-first order,
  first-seen candidate order, and independent meta/root deduplication.

## TDD and deterministic evidence

The Runtime865 contract was observed RED with `4/4` failures, then GREEN at
`4/4`. The lower parity model covers different existing meta/root sets,
duplicates, single-destination admission, dual-destination admission, and exact
stable order.

For 4,096 existing URIs per destination and 4,096 distinct incoming URIs
accepted by both destinations, the retired indexed path performs 20,480 URI
clones: 8,192 existing-index clones plus three clones per incoming URI. The new
path performs 4,096 URI clones, the ownership minimum for publishing the same
incoming value into two destination vectors; the other owner receives the
original moved value. The deterministic clone reduction is `80%`.

The ignored 101-pair Release marker
`RUNTIME865_HANDWRITTEN_META_DEPENDENCY_BORROWED_INDEX_BENCH_V1` reports
alternating owned-index and borrowed-index p50/p95/p99 samples and requires the
borrowed-index p95 to improve.

## Local validation boundary

- Exact-file Rustfmt passes for the production owner and lower model.
- The combined Runtime865/864/863, UI visitor, Runtime200/205, and Runtime87
  batch passes `40/40` in `0.032s`, with zero failures or errors.
- The one-process post-change non-Tooling loader passes `4216/4216` tests across
  `993` files in `458.125s`; the dated-record audit matches `100/100` source
  hashes across `19` Runtime/Editor records.
- Runtime865 landed after v6 and was submitted with Runtime864 in the
  asynchronous v7 multi-task current-source batch; it was not submitted alone
  and v7 has not been polled.
- Local source/model evidence does not establish Windows compilation, actual
  allocator counts, or asset-restoration product p50/p95/p99 behavior.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs` | `A1207221C7FB47BA6F8595843484D73CD723C3CC7876C3449537213FF3D03A73` (shared current-worktree hash after Runtime866; the Runtime865 merge remains covered by its unchanged lower/contract files and the `13/13` adjacent batch) |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/optimization_batch_jd_runtime643_tests.rs` | `91995ECB1AFD392925A6A73E1D0390EDA0B409901AAECB36730797E659227D93` |
| `tools/tests/test_runtime865_handwritten_meta_dependency_borrowed_index_performance_contract.py` | `B1790BC412C28602CD7BB6C060056DF5C30D7DA09468C75FF5C85474D97DEC07` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the next combined Windows lane compiles current Runtime, executes the lower
regression and ignored Release marker, and supplies allocator plus asset-
restoration product p50/p95/p99 evidence. The clone model is not product
acceptance.

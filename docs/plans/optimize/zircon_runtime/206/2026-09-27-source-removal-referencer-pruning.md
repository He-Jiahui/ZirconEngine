---
title: Runtime206 Source Removal Referencer Pruning
category: zircon_runtime
date: 2026-09-27
session_id: astra-optimize-20260926-batch-a
implementation_status: implemented_pending_validation
validation_status: static_review_complete_managed_tests_pending
performance_status: release_measurement_pending
related_code:
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
  - zircon_runtime/src/asset/registry/targeted.rs
tests:
  - zircon_runtime/src/asset/registry/asset_registry_index/source_removal_pruning_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/206/2026-09-27-incremental-referencer-bucket-pruning.md
---

# Runtime206 source removal referencer pruning

## Current source finding

Runtime1006 repairs dependency replacement, but `remove_source_path` still
finishes every source removal with `retain` over the entire UUID reverse
index. This also scans the whole map for an absent source. Real callers in
`incremental.rs`, `rebuild.rs`, `targeted.rs`, `deletion.rs` and `relocation.rs`
invoke this primitive during additions, removals and source replacement.
Repeated removals therefore add whole-registry work for each changed source.

The candidate removes an emptied reverse bucket immediately after removing
an outgoing edge from a removed row. Remaining owners that refer to a deleted
target keep their incoming edge until the existing affected-owner refresh.
Source grouping, labels, canonical paths, type/tag/package/path postings,
AssetId lookup, path intent and diagnostics keep their existing owners.
There is no new cache, public API or persistent metadata representation.

The nonempty UUID-bucket invariant makes this local cleanup sufficient:
insertion creates populated buckets, dependency replacement prunes its old
affected buckets, and source removal now prunes each removed owner's outgoing
buckets. The separate deferred path cleanup during initial construction stays
at its existing build boundary. An absent source creates no empty bucket and
is a true no-op.

## Successor scope and preservation

This P slice follows O's terminal admission failure. O's first request was
rejected while capturing a changing foreign `zr_vm` file; its six remaining
lanes were not submitted. The O manifest remains immutable. P records the
previous production SHA
`1d27980d98d198fe41022d34c1ca1e08e873e4fe21a4ac589b568e49feb290d5`
and introduces this scoped successor version.

Only the source-removal function and a test-module binding change in the
production file. Reversing those edits must restore the complete preimage,
including all earlier and foreign index changes. `targeted.rs` stays read
only. Its real `prepare_source_removal` clone and owner-refresh path is tested.
The complete old removal function and candidate-preparation wrapper were
frozen before the production edit; baseline names and their mutual call are
the only semantic-neutral substitutions.

## Executable behavior coverage

Four regressions exercise the real registry operations:

- A root and labeled subasset share a source and have self, shared, duplicate
  and missing UUID dependencies. Removing that source preserves other owners'
  incoming references and every secondary index. Surviving rows and postings
  are checked against an independent rebuild; forward and reverse relations
  are reconstructed and compared in full.
- Absent and repeated deletions are exact no-ops. Removing all remaining
  sources leaves the complete default index, including empty postings.
- The actual `prepare_source_removal` produces the same full candidate and
  affected-owner set as the frozen wrapper, refreshes the remaining owner's
  dependencies, emits the precise unresolved-path diagnostics and preserves
  the input snapshot.
- Reinserting the removed rows through `insert_checked` and restoring their
  path intent recovers the complete original index and permits another removal.

These tests were authored before the production edit. Dynamic red/green
execution has not run; source comparison and rustfmt are not test execution.

## Runnable Release evidence

Ignored test: `runtime206_source_removal_pruning_release_profile`.
Marker: `RUNTIME206_SOURCE_REMOVAL_PRUNING_BENCH_V1`.

| Corpus | Timed production work | Pending evidence |
|---|---|---|
| 10K, 100K and 1M rows | Absent-source removal, one-source removal, and 128-source removal | Each case has five warmup pairs and 31 alternating old/new pairs; local new p95 <= 80% of old p95 guard |
| 10K and 100K rows | Actual `prepare_source_removal`, including its required full index clone and affected-owner refresh | Five warmup pairs and 31 alternating pairs, full candidate equality and raw distributions; original product budget remains open |
| 100K rows, all 100K sources removed | Actual source-removal primitive for every source | Five warmups and 31 current-path samples; old quadratic whole-map-per-source baseline is not repeated at this scale |

The large corpus reuses one actual registry. Saved rows and path intent are
prepared before timing and restored afterward. Every source, remaining row,
forward dependency and reverse bucket is checked against the known ring,
with cardinality checks rejecting extra keys. Registry creation, fixture
preparation, assertions, restoration and final teardown are outside timing;
the mutation's own removals and destruction remain inside it. Real candidate
preparation separately includes cloning in its measured operation, while
candidate assertions and destruction stay outside.

Reports contain the raw nanosecond samples, nearest-rank p50/p95/p99 and
OS/architecture/processor/crate identity. The 80% guard is a local comparison,
not the original Runtime206 product qualification. The batch's source manifest
and managed receipt must identify the exact executable before accepting a run.

## Acceptance and records

Independent source review is complete. Managed behavior execution and all
Release results remain pending. Runtime206 M206-5, P1-039 and G16/G17/G24 are not closed by this
slice. The complete 10K/100K/1M qualification still requires allocation/RSS,
I/O, watcher-to-visible correctness and latency, plus applicable native and
Unreal comparison budgets. Removing this scan does not remove the candidate
clone or claim a measured end-to-end latency or memory improvement.

Four-path claim: `92da8eee5e7d44e39a7cce0cf5eba572`.
Optimize authorization: `42d2260df03b47b189e9c48d1748c0fd`.
Astra authorization: `053f46538d7f49dd9becc1ec5ecf83ee`.
Preimages, frozen functions, successor scope, test-first record and checks use
prefix `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-p-runtime206-source-removal`.
The corresponding completion list is
`docs/plans/astra/features/runtime/1008-runtime206-source-removal-referencer-pruning-completion-list.md`.

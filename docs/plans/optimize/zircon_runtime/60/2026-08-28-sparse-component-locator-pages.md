---
related_code:
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse/locator.rs
  - zircon_runtime/src/scene/ecs/entity/internal.rs
implementation_files:
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse/locator.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md
tests:
  - zircon_runtime/src/scene/ecs/storage/component_storage/sparse/tests.rs
  - tools/tests/test_runtime_sparse_component_locator_pages_contract.py
  - tools/benchmarks/runtime_sparse_locator_pages.rs
doc_type: milestone-detail
report_id: Runtime60-sparse-component-locator-pages-2026-08-28
date: 2026-08-28
session_id: root-runtime60-sparse-component-locator-pages-20260828
implementation_status: partial
validation_status: source_model_harness_passed_diagnostics_managed_cargo_product_profile_pending
---

# Runtime60 Sparse Component Locator Pages

## Scope

This slice addresses the production source defect in `RECS-P1-11`: a sparse component locator is
currently a `Vec<Option<SparseRowLocation>>` resized to the highest entity index and never shrunk.
The dense entity/value arrays are already the row authority, so the locator only needs to map a
generation-aware entity index to that dense row. This work does not change component values,
change ticks, table ownership, query qualification, or public entity identity.

## Baseline Bottleneck

The current insert path is `O(max entity index)` in allocated slots rather than `O(live locator
pages)`. Removing the high row clears one slot but retains the entire vector capacity. One sparse
component on entity index 4,000,000 therefore retains about 96 MB on the measured x86_64 layout;
the highest valid `u32` index projects to about 103 GB and is not operationally admissible.

The defect is structural, not a polling or allocator micro-detail. Bevy's current `SparseArray`
uses the same continuous `Vec<Option<V>>` shape and explicitly documents excess memory for sparse
high keys, so it is a useful negative comparison rather than the target. Unreal's `TChunkedArray`
keeps fixed chunks behind a directory, while `TSparseArray` exposes explicit `Shrink`/`Compact`
lifecycle. Zircon needs both properties while preserving entity-index lookup.

## Profile Model And Decision

`tools/benchmarks/runtime_sparse_locator_pages.rs` is a standalone Rust 1.94.1 release model built
and executed under `E:/Git/ZirconEngine/target/codex`; it does not use Cargo or create C-drive
artifacts. It compares the current continuous vector, a four-level radix, a page HashMap, a private
open-addressed page table, and an adaptive packed-prefix/page directory with identical decoded
observations and checksums.

| Scenario | Continuous locator | Hybrid locator | Result |
| --- | ---: | ---: | ---: |
| Slot layout | 24 B | 8 B | -66.667% per allocated slot |
| One row at index 4,000,000 | 96,000,024 B | 2,048 B | -99.9979% |
| 262,144 contiguous rows | 6,291,456 B | 2,097,152 B | -66.667% |
| 263 rows across a high 262,144-index cluster | 192,000,048 B capacity | 2,097,152 B | -98.9077% |
| 1,024 low + 1,024 high rows | 192,000,048 B capacity | 16,384 B | -99.9915% |
| Highest valid projected index | 103,079,215,080 B | 2,048 B | bounded |

After correcting decoded observations and routing each lookup to the ordered span first, three fresh
31-pair alternating runs placed dense P50 between 10.3439% faster and 4.2631% slower. Across 1%, 5%,
10%, 25%, 75%, and 0.1%-gapped low-index scenarios the worst P50 regression was 28.0677%. The high
offset window improved mixed P50 by 6.4199%-15.0733% and hit-only P50 by 7.2077%-13.7094%. Alternating
hits across simultaneously retained low/high spans regressed P50 by 4.3081%-16.1994%. These runs
meet the candidate threshold of at most 30% P50 regression, at least 60% dense locator modeled-
allocation reduction, and at least 99% single-high-water reduction. P95 was noisy on the interactive
desktop and is not accepted as product tail-latency evidence.

Hash-only, radix-only, page-open-addressed, and row-open-addressed candidates were rejected because
indirection taxed hot queries. Before adding the offset window, the real sparse HashMap branch
regressed high-cluster mixed P50 by 446.0091% and hit P50 by 203.3120%; monomorphized radix and
row-open candidates still regressed representative high-cluster P50 by at least 63.6674%. The
offset window brought those paths under the threshold while retaining a density bound. A truly
disjoint third cluster still uses the sparse overflow as a memory-first cold path; it is not yet
claimed as hot-query qualified and remains part of the product-profile boundary below.

## Implemented Owner

- Use 256-slot pages. The power-of-two shape keeps split operations to shift/mask and makes one
  allocated page 2 KiB of packed locations on x86_64.
- Maintain an 8-byte packed zero-based prefix plus one page-aligned offset window. Either span may
  cover at most 1,024 slots per live locator at promotion, so dense low indices and one high local
  cluster both stay on direct indexed lookup without making the highest index the allocation size.
- Keep further disjoint pages in an internal identity-hashed directory. A `BTreeSet` is the ordered
  ownership index for range absorption, eliminating full-directory scans and stale heap keys when
  either flat span grows. Entity indices are trusted engine-generated integers; the hasher remains
  private and must not be reused for caller-controlled keys.
- Pack `(generation, dense_row + 1)` into `NonZeroU64`; `Option<SparseRowLocation>` remains 8 B.
  The invalid entity index reserves enough row-domain space for the non-zero encoding.
- Track occupied rows per page. Removing the last row retires the page; an empty locator releases
  all span and directory capacity. Empty edges are trimmed; a sparse zero prefix can rebase its
  retained tail into the offset window. A span below 1/2,048 density demotes to pages only after
  trimming/rebasing fails. The 1/1,024 promotion and 1/2,048 demotion thresholds provide hysteresis.

Lookup is average `O(1)`. Existing-row mutation is `O(1)`; page ownership changes add
`O(log sparse pages)` ordered-index work and flat spans grow amortized. Retained location memory is
`O(flat prefix slots + flat window slots + allocated sparse pages * 256)` under the density and
compaction bounds. Swap-remove still repairs exactly one locator, and generation checks remain
mandatory before returning a dense row.

## Qualification Boundary

The source contract was observed red before production implementation: one of two focused tests
failed only because the old vector owner and resize path remained. It is now green 3/3. A direct
Rust 1.94.1 test harness compiled the real production owner and passed 16/16 behavior tests covering
highest valid index, page retirement, shared-page retention, zero-prefix/window coexistence and
absorption, high-window growth, edge trimming, prefix rebasing, truly disjoint demotion, third-cluster
overflow, sparse-key reinsertion, cross-representation deletion compaction, generation rejection,
cross-page swap repair, packed slot size, and a 20,000-operation reference model. The non-test compile harness, exact rustfmt, focused source
contract, and benchmark checksums also pass.

`RECS-P1-11` remains partial: production locator-byte diagnostics are not yet aggregated through
the shared `ComponentStorage` owner. Runtime08 Cargo, million-entity counters/RSS slope, real scene
query P95, WPR wakeups/CPU, and power remain managed qualification work; modeled allocation totals
are not allocator/RSS measurements, and this microbenchmark makes no product power claim. Product
profiling must also establish whether the truly disjoint overflow is cold; otherwise a later bounded
multi-window policy is required before that branch can be treated as hot-query qualified.

Status: `runtime_08_60_sparse_component_locator_algorithm_source_passed_diagnostics_cargo_product_profile_deferred`.

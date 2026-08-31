# Frameworks01 M1 Resource Management Projection COW Page Plan

## Status

- Plan owner: Frameworks01
- Session: `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`
- State: `three_authority_algorithm_preserved / post_hard_cut_current_hash_release_profile_green /
  product_trace_and_power_pending / milestone_not_accepted`
- Reviewed current HEAD: `0aeb32c037cf30028d7a8950ce373ae052c97c38`
- Scope: immutable `ResourceManagementGeneration` mutation cost inside
  `ResourceManagementProjection::apply_delta`
- Excluded: public management query API, Resource registry authority, readiness graph, event log,
  durable IO, physical `zr_resource` application, Editor projections, IBL, product power, and any
  compatibility path

This is an implementation plan, not an accepted output record. It follows the required order:
whole-module and consumer review, Unreal-led reference review, direct current-source measurement,
rejected alternatives, locked architecture, then production TDD and post-profile.

## Current-source architecture review

`ResourceAuthority` publishes one immutable management generation under the same write lock as the
registry mutation. The pre-cut generation owned 64 ID-hashed row shards. Each shard duplicated the
same rows in two forms: an ordered `Arc<[Arc<ResourceManagementRow>]>` for scans and a
`HashMap<ResourceId, Arc<ResourceManagementRow>>` for ID lookup. Locator lookup searched those
row shards; there was no independent pre-cut locator index.

`apply_delta` currently rebuilds every ID shard touched by a transaction:

1. clone all rows of the old shard into a temporary `HashMap`;
2. apply removals and replacements to that temporary map;
3. collect every value into a new `Vec`;
4. sort the complete shard by canonical locator and ID;
5. rebuild the final per-shard ID map from the sorted rows;
6. clone and update affected locator shards;
7. publish one new immutable generation.

Because `ResourceId` hashing is intentionally unrelated to canonical locator order, even a small
batch spread over the ID space touches many shards. For `N` published rows and `S` affected shards,
the update cost is proportional to all rows in those shards, not to changed rows `D`: roughly
`O(N * S / 64 + sum(shard_len * log shard_len))`. All of this work occurs while the Resource
authority write lock is held. Full project open, targeted/full scan publication, builtin
registration, close, relocation, reload, and ready publication all reach this transaction path.

Public consumers require these invariants:

- ID and locator lookup return the same exact row as ordered scans;
- scan and page order is canonical locator followed by ID across resource kinds and schemes;
- generation sequence advances once per projected batch and stable/nonprojected updates reuse the
  exact previous `Arc` generation;
- summary counts, rename/swap/remove behavior, and structural sharing remain correct;
- readers retain immutable snapshots after later commits;
- callbacks/events remain outside the authority write lock.

## Reference-engine review

Unreal Asset Registry is the primary reference. `AssetDataMap` uses stable slot-like asset storage
plus `AssetByObjectName` hashed lookup and performs `FindByHash`/`RemoveByHash` rather than rebuilding
an ordered catalog for every mutation. Package-name indexes use compact direct-or-indirect arrays,
and the initial gather reserves the state once when scale warrants it. `FEventContext` collects
ordered transaction-local events while mutation is locked and broadcasts after the lock is
released. The applicable rule is to separate identity lookup, ordered enumeration, and deferred
publication while keeping one transaction authority.

Bevy is a secondary Rust check: dense asset IDs and UUID IDs use distinct storage strategies,
mutations append to an ordered event vector, and event delivery drains one batch. It likewise does
not require an ID hash partition to double as the enumeration-order partition. Zircon retains its
immutable generation, canonical locator order, retained event stream, and cross-thread snapshot
contract; neither reference API is copied.

## Direct current-source pre-profile

The D-drive release harness directly `include!`s current
`management_generation.rs` and `manager/management_projection.rs`, builds the real row/shard types,
and measures only generation delta application. It uses Rust 1.94.1,
`-C opt-level=3 -C target-cpu=native -C debuginfo=1`, 11 samples per ordinary case and seven for
100,000-row cases. A counting global allocator reports allocation requests made inside the timed
region.

Artifacts:

- root: `D:\zircon-frameworks01-r12-resource-management-projection-20260827`;
- harness SHA-256: `fd31ad1c52ce701397fb1263f76607800b2e573eebfaa2173356f7060f1ff4fa`;
- executable SHA-256: `8d7209d44f11a2ec41f287cb648a4cf64291a2ce73ebfce18938fd84317a712f`;
- CSV SHA-256: `63df24e7cf561090c1bc18dafd7e9a765ca5f5b0c56cd3806429879f11bcb1a0`;
- current production inputs:
  `management_projection.rs` = `b3c1c90d1472f34f78390ae1fe333d4ac8770ea7817d1a9f2ad616e4cbe8887c`,
  `management_generation.rs` = `4943856a4f71de4202fcd6c03092d1b6e56917e2b2eb570cc10d6160259034db`.

| Published rows | Changed rows | Distribution | Current median | Allocations | Requested bytes |
| ---: | ---: | --- | ---: | ---: | ---: |
| 4,096 | 4,096 | contiguous | 5.500 ms | 12,620 | 0.89 MiB |
| 100,000 | 1 | single | 0.529 ms | 14 | 108.1 KiB |
| 100,000 | 64 | contiguous | 52.948 ms | 441 | 4.11 MiB |
| 100,000 | 64 | spread | 52.845 ms | 441 | 4.11 MiB |
| 100,000 | 4,096 | contiguous | 93.274 ms | 12,682 | 7.11 MiB |
| 100,000 | 4,096 | spread | 81.161 ms | 12,682 | 7.11 MiB |
| 100,000 | 100,000 | all | 178.939 ms | 300,394 | 19.91 MiB |
| 100,000 | 100,000 | no projected change | 49.383 ms | 2 | 724 B |

One changed row cloning about 108 KiB and 64 spread changes taking about 52.8 ms prove that work is
coupled to hash-shard population. The no-op case also scans every input row but correctly performs
almost no allocation and retains the old generation.

## Rejected alternatives

Changing only the temporary container or sort is rejected. A delta-only map plus sorted linear
merge improved sparse cases but rebuilt complete changed shards and regressed the 4,096-row dense
case and some full generations. It did not remove the underlying ID/order partition coupling.

A first 1,024-ID-shard plus 256-row ordered-page model demonstrates the required separation. On the
same current-source row contract it measured 19.1 us for one change, 0.184/0.940 ms for 64
contiguous/spread changes, and 13.513/22.134 ms for 4,096 contiguous/spread changes. Relative to the
current baseline these are approximately 96.4%, 99.7%/98.2%, and 85.5%/72.7% lower. The model's
all-row result was 232.139 ms, 29.7% slower, so a sparse-only page algorithm is also rejected.

`BTreeMap`/tree nodes, tombstones, an always-dense rebuild, and a new external persistent-collection
dependency remain rejected: they add per-row node traffic or fail either sparse or dense scale, and
none is required to preserve immutable publication.

## Locked architecture

Hard-cut the private generation storage into three independent COW authorities:

```text
ordered pages: Arc<[Arc<[Arc<ResourceManagementRow>]>]>
ID index:      Arc<[Arc<ResourceManagementIdMap<Arc<ResourceManagementRow>>>]>
locator index: Arc<[Arc<HashMap<Arc<str>, ResourceId>>]>
```

- ordered pages are globally canonical by locator/ID and target a bounded row count per page;
- ID and locator indexes use finer-grained shards independent of ordered-page placement;
- sparse same-key updates copy only affected ordered pages and affected index shards;
- rename/add/remove locate old/new pages by immutable page boundary keys, perform ordered page-local
  edits, and split or merge pages without retaining tombstones;
- derived ID entries clone the exact authoritative ordered-page row `Arc`; differential acceptance
  verifies scan/ID/locator identity without adding an O(N) production validation scan;
- ordered storage, ID index, and locator index choose independently between reuse, sparse COW, and
  rebuild; one global `Rebuild` decision may not discard all three authorities together;
- summary is always updated from the authoritative delta and is never recomputed by scanning every
  published row;
- ordered storage uses page replacement for same-key changes, range rebalance for structural
  changes, and a global ordered merge only when affected ranges cover the generation;
- ID index rebuild is reserved for ID-dense replacement; locator index rebuild is reserved for
  locator-dense replacement. Otherwise each accelerator clones only affected shards even when the
  ordered view requires a global merge;
- dense ordered merge sorts only structural/new insertions, then merges once with surviving
  canonical rows before publishing one generation;
- scans traverse globally ordered pages directly, eliminating the 64-way heap merge; page/scan
  filtering semantics and the public API remain unchanged;
- old private 64-shard row representation, merge heap, forwarding aliases, compatibility branch,
  and duplicate implementation are deleted in the same cut.

Expected same-key sparse work is `O(D log P + copied_page_rows + copied_index_shard_entries)`, where
`P` is the page count. A structural ordered merge is
`O(D log P + K log K + copied_page_rows)`, where `K` is the structural/new insertion count; when
ranges cover the full generation, `copied_page_rows = N`. Accelerator work remains independently
`O(D + copied_index_shard_entries)` unless an index-dense transaction selects an `O(N)` rebuild.
Read lookup remains expected `O(1)`; ordered scan is `O(N)`; storage remains `O(N)` with bounded
structural-sharing overhead.

## TDD and acceptance gates

Production editing starts only after coordinator lease/attribution for exact owners. Tests must make
the current source RED for the new private contract, then cover:

1. same-key sparse update reuses every unaffected ordered page and index shard;
2. spread updates copy only the pages/shards they touch;
3. add/remove/rename, locator swaps, custom IDs, mixed kinds/states, and summary counts preserve the
   exact public semantics;
4. page split/merge boundaries preserve canonical locator/ID order with zero duplicate/missing row;
5. no-op and nonprojected updates retain the exact previous generation `Arc`;
6. dense planning produces the same generation as a simple canonical oracle;
7. scan/page/ID/locator results are differential-tested against that oracle;
8. old merge-heap/private 64-row-shard implementation and compatibility path are absent;
9. production/test files remain under the repository structure budgets;
10. focused and full direct `zr_resource` tests pass before managed Cargo is requested.

The exact post-profile must repeat the table above and add rename/add/remove plus mixed random
batches. Sparse acceptance requires at least 70% median reduction for 100,000/64 spread and
100,000/4,096 spread, at least 50% requested-byte reduction for 100,000/64 spread, and no sparse
case above 15% regression. Dense 100,000-row publication and 4,096-row publication may not regress
more than 15%. Results must include source/executable/CSV hashes and may not be promoted to product
frame time, power, or cross-engine parity.

Managed Windows Cargo, Runtime/App/Editor product validation, independent review, physical
`zr_resource` application, milestone commit, and WeCom notification remain separate coordinator
gates. The two frozen mixed Editor blobs and public
`core::resource::io::{atomic_write, atomic_write_new}` facade remain untouched.

## First implementation under rework

The private 64 ID-hashed ordered-row shards and 64-way merge scan were deleted. Exact8 currently
now owns globally canonical 256-row COW pages, 1,024 private ID shards, and 1,024 locator shards.
`apply_delta` builds one randomized `HashMap<ResourceId, ProjectedResourceChange>`, selects sparse
COW or a global dense rebuild from estimated copied page/index entries, bins ID changes once, applies locator
removals before insertions for swaps, and reuses the exact locator `Arc<str>` when identity is
unchanged. ID shard selection, ID maps, and locator shard selection use three independent randomized
hash authorities; there is no fixed or predictable production hasher.

The implementation also removes the root file's embedded 377-line test module. Current physical
structure is 613 lines for `management_generation.rs`, 237 for its projection tests, 680 production
lines for `manager/management_projection.rs`, 769 for the folder-backed tests, 311 for
registry-export optimization tests, and 73 for `resource/mod.rs`; every owned Rust file remains
below the 800-line modularization gate.
No production `ResourceManagementShard`, `RESOURCE_MANAGEMENT_SHARD_COUNT`, `BinaryHeap`, or merge
candidate remains. This is a private hard cut with no forwarding alias or compatibility branch.

Current production SHA-256 fingerprints after exact `rustfmt 1.94.1` are:

- `management_generation.rs`: `4b4051160c25f4453ed3a5df403cbb102ef9b2aea0bc70d1e37d41f505a2f2c6`;
- `management_generation/tests/projection.rs`:
  `024bb45c917690bfb3ec5062a011ded18e47b2c26a4cef28c603eea6ff17f894`;
- `manager/management_projection.rs`:
  `bc2c1933baa329de83e53a753fa0dae7702042287bca3afd4bdd8d91bd5c997a`;
- `manager/management_projection/tests.rs`:
  `5e5265bc5f3b3855065ea4a7c7523b8f2f3b42b2595528a085b3a1b35592b8b1`;
- `manager/registry_export/optimization_tests.rs`:
  `db9c4163e9749da7d9a2901ed654116012e79a8b7ff4f550ba83abc8651b4f10`;
- `resource/mod.rs`: `2d711360a422001e1af9754d4b10f45fc1c2cecd87859109e5b73044d2fb2037`.

The final independent read-only review rechecked these six fingerprints before and after review and
returned `C0/I0/M0, Ready` for the code/evidence candidate. It found no correctness issue in the
independent ordered/ID/locator decisions, duplicate and swap semantics, full removal/recovery,
structural range merge, page bounds, `Arc` sharing, delta summary, randomized hash authorities, or
the selected sparse/dense thresholds. `Ready` here authorizes the next acceptance stage only; it is
not a managed Cargo result or milestone acceptance.

## Direct current-source verification

The final minimal-interface harness compiles the final production source with Rust 1.94.1 and places
its binary only under `D:\zircon-frameworks01-r12-resource-management-projection-20260827`:

- minimal interface behavior: `11` deterministic cases and `24` mixed random differential batches,
  passed; final executable SHA-256
  `18c5e294841c9e7e859d33d4e1db0a1696a563c179e528f58a4db8d511eb0936`;
- the prior exact8 real-`zircon_runtime_interface` harness passed sparse/dense revision, rename, and
  remove across 4,096 rows (`4/4`), but is not promoted as a final-source compile ticket;
- the prior exact8 real-`ResourceId(Uuid)` harness passed 100,000 ID and locator distributions with
  independent randomized authorities, but the final-source rerun could not complete against the
  shared stale rlib chain and is likewise not a final-source compile ticket.

The behavior suite covers exact scan/ID/locator row identity, structural page/index sharing, no-op
generation reuse, locator swaps, rename/add/remove, and dense/sparse equality to a simple canonical
oracle. `git diff --check` and exact rustfmt are green for the tracked Rust owners; direct-rustc
warnings are only expected dead-code warnings from partial-file harnesses. Managed current-hash
Cargo remains mandatory before milestone acceptance.

## Exact8 post-profile failure and structural redesign evidence

Three release rounds alternated the reconstructed pre-cut executable and exact8. Each entry is the
median of three per-round medians. The original pre-profile source hash was recorded but its source
file was not retained; structural comparisons therefore use the closest retained same-day D-drive
snapshot and are explicitly evidence against a reconstructed pre-cut algorithm, not exact binary
identity.

| Published | Changed | Mode | Reconstructed old | Exact8 | Time delta | Result |
| ---: | ---: | --- | ---: | ---: | ---: | --- |
| 100,000 | 64 | spread revision | 45.751 ms | 0.996 ms | -97.82% | pass |
| 100,000 | 4,096 | spread revision | 74.592 ms | 29.422 ms | -60.56% | fail: requires -70% |
| 4,096 | 4,096 | revision | 4.863 ms | 6.275 ms | +29.03% | fail: exceeds +15% |
| 100,000 | 100,000 | revision | 176.103 ms | 271.768 ms | +54.32% | fail |
| 100,000 | 4,096 | rename spread | 82.413 ms | 163.318 ms | +98.17% | fail |
| 100,000 | 4,096 | remove spread | 61.839 ms | 128.359 ms | +107.57% | fail |
| 100,000 | 4,096 | mixed spread | 67.573 ms | 192.327 ms | +184.62% | fail |

The exact8 acceptance matrix is therefore red. Canonical evidence is
`D:\zircon-frameworks01-r12-resource-management-projection-20260827\exact8-three-round-summary.csv`.
Round spread is retained in that CSV; the shared machine reached about 87% CPU during collection, so
no single timing sample is treated as a product or power claim.

The D-drive stage profiler replicated exact8's real `Rebuild` path for 100,000 published rows and a
4,096-operation mixed structural batch. Median-of-three-round stage results were:

| Stage | Median | Allocations | Requested bytes |
| --- | ---: | ---: | ---: |
| summary + locator index rebuild | 90.93 ms | 6,218 | 6,960,820 |
| ID index rebuild | 41.88 ms | 2,051 | 2,517,136 |
| existing-row scan | 13.51 ms | 12 | 865,496 |
| ordered merge | 10.08 ms | 1 | 799,992 |
| planner | 5.48 ms | 33 | 112,044 |
| change collection | 4.36 ms | 8,202 | 832,842 |
| structural insertion sort | 0.51 ms | 1 | 21,840 |
| page construction | 0.41 ms | 784 | 1,618,768 |

Full summary/locator plus ID accelerator rebuilds account for roughly 76% of the decomposed work;
sorting and page construction are not the bottleneck. WPR CPU sampling was attempted first, but the
non-elevated Session could not enable the system profiling policy (`0xc5585011`), produced no ETL,
and left no active recorder. The deterministic stage harness is retained as
`exact8-current-rebuild-stages-summary.csv`.

As a counterfactual, the same exact8 code was forced to rebuild ordered pages while applying summary,
ID, and locator changes through their existing delta paths. For 4,096 structural changes it reduced
mixed/remove/rename to 50.80/33.96/58.20 ms: 64-74% below exact8 and 25-45% below the reconstructed
pre-cut medians. This validates the three-authority redesign before production editing. Evidence is
`exact8-current-forced-sparse-summary.csv`.

The first three-authority production candidate removed the structural regressions: 4,096
mixed/remove/rename measured 46.62/27.05/57.06 ms in the first release round. It did not close the
dense gate: 100,000 same-key revisions measured 306.34 ms, while 4,096 spread revisions measured
24.30 ms and remained about 2.6 percentage points short of the 70% improvement gate. A second
median-of-three stage profile of the dense candidate reported:

| Dense same-key stage | Median |
| --- | ---: |
| replace ordered pages | 106.80 ms |
| collect changes | 90.40 ms |
| projection plan | 77.11 ms |
| sparse ID index | 43.11 ms |
| summary delta | 3.47 ms |

The dense planner was repeating page binary searches and constructing a complete touched-shard set;
page replacement repeated those searches for every row. Dense ID COW was also no faster than the
exact-reserved two-pass ID build. The second production cut is therefore locked to: choose ID
rebuild independently for ID-value-dense batches, omit unused ID/locator shard-set construction,
select all ordered pages directly for dense same-key batches, and use one linear page scan instead
of per-row page and row binary searches. Evidence is
`three-authority-dense-stages-summary.csv`.

The final production cut removes the global `Sparse | Rebuild` coupling, preserves one immutable
publication boundary, and independently selects ordered-storage, ID-index, and locator-index work.
Three final release rounds produced the following median-of-round-medians results against the
reconstructed pre-cut algorithm:

| Published | Changed | Mode | Old | Final | Time delta | Bytes delta |
| ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 4,096 | 4,096 | revision | 4.863 ms | 3.124 ms | -35.76% | +7.87% |
| 100,000 | 1 | revision | 0.452 ms | 0.016 ms | -96.44% | -45.04% |
| 100,000 | 64 | spread revision | 45.751 ms | 0.522 ms | -98.86% | -88.57% |
| 100,000 | 4,096 | spread revision | 74.592 ms | 20.259 ms | -72.84% | -31.09% |
| 100,000 | 4,096 | add | 81.486 ms | 21.531 ms | -73.57% | +12.78% |
| 100,000 | 4,096 | mixed spread | 67.573 ms | 37.904 ms | -43.91% | +68.64% |
| 100,000 | 4,096 | remove spread | 61.839 ms | 34.348 ms | -44.46% | +66.64% |
| 100,000 | 4,096 | rename spread | 82.413 ms | 43.158 ms | -47.63% | +68.22% |
| 100,000 | 100,000 | revision | 176.103 ms | 131.123 ms | -25.54% | -18.03% |
| 100,000 | 100,000 | no projected change | 38.067 ms | 38.123 ms | +0.15% | -100.00% |

The required gates pass: 64-spread time/bytes improve 98.86%/88.57%, 4,096-spread time improves
72.84%, both dense revision cases improve, and no measured sparse mutation regresses. The full
19-case summary SHA-256 is
`444f8cf347b52024c3fbff85a516bfb1f5a874a156c634d965c9293a45742425`;
round CSV hashes are
`65cf84d8a3620a8f131e792df92852b3de7177c4069bbc73a36cdac6b15125ad`,
`82c17ba799cc304b7549828d95430df189056b966fa1fb3022fc8b625d6b099c`, and
`1c16b21cc07a18ad5fa579d820531854ee2c1ff06455fcee19c8eab66b912e28`.

The small initial-build cases remain slower in relative terms because constructing 1,024 ID and
1,024 locator shards costs 0.34-0.48 ms at 256/257 rows; this is an absolute sub-millisecond fixed
cost and remains a future adaptive/empty-shard-sharing item, not hidden acceptance evidence. Final
round timing spread reached 22-174% under shared-machine load, so medians are retained with the raw
rounds and are not promoted to product frame-time, power, or cross-engine energy-parity claims.

## Managed and coordination state

The priority documentation gates were rerun on the shared current tree. Two `python
tools/check_conventions.py --only docs --json` snapshots drifted from 1,352 violations across 372
documents to 1,353 across 373 while foreign owners were active, but structured filtering reported
`0` violations for this child plan and `0` for the Frameworks01 parent plan in both runs. A targeted scan of
`docs/plans/engine-code-review-findings-2026-06.md` found no `ResourceManagement`,
`management_projection`, or `management_generation` legacy row (`0` matches). The applicable common
findings are covered by the three-authority split, COW `Arc` reuse, removal of the old shard/heap/API
track, and the sub-800-line physical owners; the Runtime15 review-findings document is not edited or
claimed by this slice.

Managed Windows Cargo job `a576e52426bf4a7e9928b7ebc8093f7e`, run
`9d3ca1128a1142c0b5a12b96654fb899`, ran from 14:37:16Z to 15:14:10Z and naturally exited 101. It
was blocked before compiling this `zircon_runtime` slice by three foreign `zr_rhi_wgpu` E0308
diagnostics at `production/device/diagnostics.rs:665`, `:722`, and `:771`: callers pass `u32`
`copy_row_bytes` to `DiagnosticTextureReadbackLayout::new(u64, u32)`. Frameworks01 did not edit the
RHI owner. The job was released through the coordinator, its D-drive target was deleted, and no
Cargo/rustc process remains. Its stderr SHA-256 is
`378d91da0e78bcb3582d799b8bb1bcd3b5a403022ebec34a4b2cc59e7d9b9c12`.

This job also started before the final exact-reserve source edit, so it is development compile
evidence only and not a current-hash acceptance ticket. Independent review is green, but the slice
therefore still does not promote Frameworks01 M1, create a milestone commit, or send a WeCom
notification. The public `core::resource::io::{atomic_write, atomic_write_new}` facade remains a
required stable Resource IO export for runtime/Editor/IBL consumers; the reported UI12 unresolved
imports are a physical/projection split blocker, not authorization to migrate those foreign
consumers. No IBL or frozen Editor source was changed.

## 2026-08-31 post-hard-cut reference and evidence re-review

The physical `zr_resource` hard cut changed paths, imports and assembly after the original release
profile. A fresh whole-owner review at HEAD `399f2318150ae4fa0df3a2543133b03b80099288` confirms that
the three-authority algorithm remains present, but the old performance fingerprints are not current
physical-source fingerprints. They must not be promoted as current-hash latency, allocation, power
or engine-parity evidence.

The primary Unreal reference remains structurally consistent with the locked design. Current local
`AssetRegistryState.h` keeps stable asset storage and independent package-name, package-path, class
and optional tag accelerators; its tag-index documentation explicitly treats query speed versus
index memory as a policy decision. `AssetDataMap.h` keeps stable slot-like asset storage plus hashed
object-name lookup instead of rebuilding one ordered/hash hybrid container. `AssetRegistry.cpp`
retains one mutation state and publishes ordered asset events after the protected mutation phase.
Zircon intentionally differs by publishing immutable cross-thread generations, but it follows the
same separation: canonical ordered storage is not an ID-hash partition, and each accelerator may
choose sparse COW or dense rebuild independently.

The current physical fingerprints and sizes are:

- `management_generation.rs`: 612 lines,
  `333e8d82759576fd8a10dfa236fe184b8b4b9caf08c50e4d917eb2c7aa62bf79`;
- `management_generation/tests/projection.rs`: 237 lines,
  `52ecf78e084d293919da61fbd95b9bd9c055637824d6f4a3a5ce450ad2031cd2`;
- `manager/management_projection.rs`: 680 lines,
  `3594095e6bb66cf9f67f728595a876f8f000fc4195b28e984e85a17d9fecca06`;
- `manager/management_projection/tests.rs`: 769 lines,
  `3331339f86f4a973c2f637250cd971ed9ab0527167b07253a1a67f711b4d989f`;
- `manager/registry_export/optimization_tests.rs`: 311 lines,
  `a8f80b761741318f0d012628c093c340d40ca861ffa62eda55dfea58807311d4`;
- crate `lib.rs`: 71 lines,
  `4347e84f07ebd59dfb6167b75a85f982ab80bf2fb3b046e0192b6ad7ede3212d`.

Coordinator job `4bcc417333ec45039d835225e5c41448`, run
`d6f1b5b6fdd645e0a8c76b4bbab4b424`, compiled this physical owner and passed the complete Resource
library suite with 191 passed, 0 failed and 4 ignored; libtest execution was 9.48 seconds and the
cold isolated dependency/download/build phase was 20 minutes 25 seconds. This closes current-hash
compile and behavior evidence for the management implementation because none of the six paths
changed after that run. It does not refresh release-mode algorithm measurements.

Before any further change to page capacity, sparse/dense thresholds, index shard counts, hash
authorities or publication strategy, the exact current physical crate must rerun the retained
100,000-row matrix with the counting allocator and the stage profiler. Acceptance still requires
the existing sparse/dense regression gates, raw round dispersion, a product workload trace, and
power sampling. Until those results exist, the correct engineering action is to preserve the
current algorithm and continue foundation/integration closure rather than make another unprofiled
optimization.

## 2026-08-31 current-source profile reproducibility repair plan

The promised retained release harness is no longer materialized: the recorded
`D:/zircon-frameworks01-r12-resource-management-projection-20260827` directory does not exist, and
the current crate contains no 100,000-row management projection workload or counting allocator.
Only correctness and planner-boundary tests remain. Reusing the historical CSV without a
reproducible current-source executable would violate the optimization gate and make the next
threshold change unauditable.

The lowest repair is test-only infrastructure, not a production algorithm change:

1. add one folder-backed ignored profile owner below
   `manager/management_projection/tests/` and leave `management_projection.rs` unchanged;
2. require release mode, one test thread and at least 31 measured samples after warmup;
3. require an explicit absolute `ZR_RESOURCE_MANAGEMENT_PROFILE_DIR` on a non-C drive; never fall
   back to `%TEMP%`, workspace `target`, or another implicit output directory;
4. emit raw sample CSV plus summary CSV containing p50/p95/MAD wall time, allocation count,
   requested bytes and peak live bytes, along with the exact projection/generation source hashes;
5. cover 4,096 dense revision; 100,000-row 1/64/4,096/dense revision; 4,096 add, mixed, remove and
   rename; no-projected-change; and initial-build page-boundary/large cases;
6. mark metadata queries, RSS and power unavailable in harness metadata rather than synthesizing
   them. WPR/ETW and power sampling remain a separate external same-command gate.

The static Resource boundary guard is written RED first to keep these requirements executable.
After the harness compiles and its helper tests pass, one managed release profile may be scheduled;
the harness source alone does not refresh performance evidence. No page/shard/threshold change is
authorized until the resulting current-source report and external system trace are reviewed.

## 2026-08-31 current-source profile execution status

The reproducibility infrastructure is source-complete but no current performance sample exists yet.
The first real managed attempt, job `2537e95c94d54f618e244ca45e3ea73a`, run
`4b7efb3f35344d9e91d5c1ffdd732266`, correctly failed in 3.5 seconds because the initial test-only
implementation added `sha2` to the crate while `--locked` prohibited a lockfile update. Its empty
stdout SHA-256 is `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` and stderr SHA-256 is
`f1ed48fbd867ed8685f6592be7976498c6a2843e31f37d29232cd7bd369b9af2`. This is a dependency-contract
RED, not a profile result.

The test-only source fingerprint was then changed to the crate's existing locked `blake3` dependency;
the added `sha2` dev dependency was removed. The second managed attempt, job
`64107e1527764083b78888c199cccf5f`, run `4630a785c5304a90957462c8f04c6581`, passed lockfile
resolution and compiled dependencies for approximately 193 seconds. It stopped before `zr_resource`
on foreign Runtime Interface E0026/E0027 at
`zircon_runtime_interface/src/runtime_api/host/ui_host_request.rs:139`: the dispatch enum owns
`link_target`, while the new host-request bridge still patterns the removed Rust field `href`. Its
empty stdout SHA-256 is `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; stderr SHA-256 is
`478c23dbf88a1223e7ad0b28d00df6e7f12a63ddd6077208de9f60294037e654`. The job auto-released and no
Cargo/rustc process remains.

Frameworks01 did not edit either UI/Runtime Interface source. The unique handoff is
`failure-2026-08-31-runtime-interface-ui-activate-link-field-mismatch.md` and was routed to the UI and
Runtime architecture tasks. Current test-only owners are:

- management profile: 477 lines, SHA-256
  `75c5ac9ed146933bf4cd988179b2c546f4d4624239d7c31b9d3c4994cb11d7c7`;
- shared counting allocator: 116 lines, SHA-256
  `7d6f9b2740ebf8d7877fc5b5b3f6e3992a1ac253eedc91f4db13aca2145c8a8d`;
- static boundary contract: 268 lines, SHA-256
  `529f915e8ca179b5e8d342eef377d62b2be671dce5a1ab6add5959a797cf1ec5`.

The Resource boundary plus conditional-write static suite is 22/22 GREEN in 45.343 seconds;
Rustfmt/diff checks are green. These results validate the harness shape only. Current-hash latency,
allocation, RSS, metadata, power, engine parity, and any further page/shard/threshold optimization
remain pending.

### 2026-08-31 current-source profile attempt R3

After RuntimeInterface03 materialized the typed `link_target` bridge, Frameworks01 ran the identical
release harness again as managed job `28eb6b1ee6a649e79a8cac8c19dc5c21`, run
`071e0c99214e4abd965e52a0ebf9bfda`, with reports and Cargo target on E. The build no longer emitted
the prior `ActivateLink` E0026/E0027 diagnostics. It compiled the same dependency chain for about
ten minutes and then stopped before `zr_resource` on two foreign E0502 diagnostics in
`zircon_runtime_interface/src/reflect/schema_catalog/admission.rs`: an alias `HashSet<&str>` retained
immutable borrows while the compile snapshot entered `fields.iter_mut()` and sorted aliases.

The coordinator restarted after the supervisor exited and reconciled the run as completed with
`cargo_run_reconciled_from_orphaned_job`; run-status request
`defff507e6d54b01b839145d3c737a9d` preserves the complete rustc diagnostics. Stderr SHA-256 is
`8ebdbf17fdc749490e2ce382d75d4d6c3dfaa5e38671a58bccf9e4fad545e0c4`; stdout is empty with
SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`. Explicit release request
`823f601d6957426e9c884c15fd27f346` left no Cargo/rustc process.

The Runtime Interface owner advanced `admission.rs` after that compile snapshot to SHA-256
`18d866b7ecbad235a8c83d34fea59d6a28ccc10f3275e0f5e462c90e0abb2ba7`: alias validation now uses a
second immutable loop, explicitly drops the borrowed name set, and only then sorts aliases. That is
source-level evidence, not a Frameworks validation or integration receipt. The exact compiler
failure was returned to the active Runtime architecture owner; Frameworks01 did not edit or claim
the file. No ResourceManagement sample was emitted, so the current profile/performance gate remains
open and no production page/shard/threshold change is authorized.

### 2026-08-31 current-source profile attempt R4

The identical release command next acquired managed job
`84f3507f1dee480184e94f5cbaf9fdb2`. The job was terminal and released, but rustc through sccache
failed before `zr_resource` because its job-scoped `temporary\sccacheFuHIFi\deps.d` parent path had
disappeared (OS error 3). Cargo returned 101 and the wrapper returned 1; no raw CSV, summary CSV,
metadata, latency, allocation, RSS, or power record exists. Job lookup request
`949f999a4cf64dce8fecfc368d95cf3d` records start 03:29:51, finish 03:32:24 and release 03:32:31
+08:00, with no Cargo/rustc process left afterward.

The canonical cross-plan record is
`failure-2026-08-31-managed-cargo-sccache-temporary-path-lifecycle.md`. It is assigned to the active
App08 runtime-artifact-reuse/compact-validation owner; Frameworks01 has not edited or claimed the
tooling scripts. Until the owner returns deterministic lifecycle tests and a successful exact-command
receipt, the current three-authority production algorithm is preserved and no page, shard, threshold,
generation, or publication change is authorized.

### 2026-08-31 current-source profile attempt R5

The first App08 return did not satisfy the exact-command gate. Managed job
`680c28eeb45f44ada781073ea28a3e50` reused R4 and failed before `zr_resource`: the persistent sccache
PID 1660 still created temporary work below deleted R4 scratch
`84f3507f...\temporary\sccacheY8m77e`, even though the fresh client exported
`SCCACHE_CLIENT_SIDE=1`. Cargo returned 101 and no raw CSV, summary, metadata, latency, allocation,
RSS, or power record exists. The Failure remains open for a stable non-C server TEMP or controlled
server rebind and a realistic dep-info/link regression. The COW algorithm remains unchanged.

### 2026-08-31 current-source profile R9 interpretation correction

The repaired exact-origin release run is now complete. Managed job
`f2f32800...` exited zero and emitted 14 scenarios with 31 measured samples after 3 warmups. The raw,
summary and metadata artifacts have SHA-256 `8C8BB282...01230`, `1244BF20...D05CD` and
`F7CC3E2B...79A07`. The retained source inputs are profile
`75C5AC9ED146933BF4CD988179B2C546F4D4624239D7C31B9D3C4994CB11D7C7`, projection
`0D2E6818A0EB770D0F0CC7CF5FCB6292EB00CDAF8F4F7989F544826FFFFEA202`, and generation
`016E75CFF5CA999E076A0323B61310DE5497FCD15B13C486BF7083432DCB24A5`.

The reported `no_projected_change_100000` p50 of 51.1567 ms is not a production full-registry scan.
The harness constructs 100,000 clones of the already published records and calls the private
`ResourceManagementProjection::apply_delta` directly. A fresh whole-call-graph audit finds exactly one
production call site, in `manager/commit.rs`; it filters staged entries with `before != record` and
passes only `changed_records` plus true removals. Consequently, a no-op transaction does not replay the
published Resource registry through the projection owner, and an ordinary frame has no projection call
at all unless a Resource transaction commits.

The 51.1567 ms sample remains useful as a misuse upper bound for feeding a full unchanged snapshot into
a delta-only private API. It is not evidence of a current engine bottleneck and does not authorize a
page-size, shard-count, threshold, hash-authority or publication change. Sparse revision, dense revision,
structural mutation and initial-build rows remain valid current-source projection measurements. Any next
optimization must first profile the actual transaction/commit entry or a product workload and attribute
cost by stage; a synthetic private-API no-change row cannot substitute for that trace.

This correction preserves the three-authority algorithm and adds no compatibility path or second cache.
It also preserves the structure gate: `management_generation.rs` is 722 production lines,
`manager/management_projection.rs` is 684 production lines, the folder-backed profile owner is 477 lines,
and the root projection test owner is 778 lines, all below the applicable 800-line review threshold.

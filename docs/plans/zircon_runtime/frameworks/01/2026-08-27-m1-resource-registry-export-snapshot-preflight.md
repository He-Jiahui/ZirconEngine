---
title: Frameworks01 M1 Resource Registry Export Snapshot Preflight
category: zircon_runtime
report_id: Frameworks01-m1-resource-registry-export-snapshot-preflight-2026-08-27
date: 2026-08-27
session_id: frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825
implementation_status: implementation_complete
validation_status: direct_projection_profile_and_independent_review_green_managed_cargo_foreign_blocked
---

# Frameworks01 M1 Resource Registry Export Snapshot Preflight

## Scope and status

This record began by reviewing `ResourceManager::ready_records_for_kind` before implementation. The
pre-existing one-line stable-to-unstable sort change and its ignored benchmark were transferred from
an archived Frameworks01 attribution, rejected as complete-path evidence, and superseded by the
snapshot/query-planner implementation recorded below.

| Work item | Status | Evidence |
| --- | --- | --- |
| Current owner and consumer review | complete | Resource authority, management projection, registry export, and all three production/test consumers inspected |
| Unreal-led reference review | complete | Unreal Asset Registry accelerated indexes/query selection plus Bevy and Fyrox Rust storage cross-checks |
| Existing benchmark validity review | complete | Exact projection run shows the timed region rebuilds and parses every fixture record |
| Candidate architecture and profiling matrix | complete | Exact D-drive paired profile v5 passed the implementation gate |
| Production implementation | complete | Private same-generation snapshot plus adaptive registry/management query planner |
| Focused validation | complete | Red contract reproduced; 10 focused tests and exact `zr_resource` projection passed |
| Postimplementation profile | complete | 20 workloads plus writer-attempt handoff passed every implementation gate |
| Independent review | complete | First review findings fixed; final exact4 review C0/I0/M0, Ready |
| Managed Cargo validation | foreign blocked | Job `03ec12d4aed34026a43ca913a6c901e8` reached `zircon_runtime_interface` and stopped on one foreign E0599 before this slice was diagnosed |

## Current architecture

The capability belongs to `zircon_runtime::core::resource`, the canonical runtime foundation. The
public method is a resource-manager access surface; it must not move into an asset, graphics, or
editor consumer.

`ResourceAuthority` publishes the following state atomically under one `RwLock`:

- the full `ResourceRegistry`, backed by copy-on-write `Arc<HashMap<...>>` maps;
- the immutable `ResourceManagementGeneration`, published by `ResourceManagementProjection`;
- payload, runtime-slot, and readiness state.

The pre-change export holds the authority read lock while it scans every registry record, filters by
kind/state/revision, clones every matching full record, and sorts the matches by locator/id. Its
complexity is `O(N + K log K)` and its writer exclusion window includes all clone and sort work.

The management generation already stores locator/id ordered immutable rows in 64 shards. Its scan
performs `O(N + K log 64)` work: rejected rows advance within a shard without heap traffic, while
matching rows take part in the 64-way merge. A row is compact metadata, not a full record, so a
consistent export also needs the matching registry snapshot.

### Consumer semantics

Only three current consumers use the API:

- shader prewarm manifest generation consumes all ready shader records;
- custom shading-model include resolution consumes all ready shader records and rejects ambiguous
  include tokens;
- an asset pipeline revision test finds one live shader revision.

The first two require a consistent snapshot. Manifest output also requires deterministic locator/id
order. No caller requires a borrowed read guard or mutation visibility during enumeration.

## Reference evidence

### Unreal Engine: dominant system-scale reference

`dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h`
stores accelerator collections including `CachedAssetsByPackageName`, `CachedAssetsByPath`, and
`CachedAssetsByClass` (lines 760-771). Class enumeration reads the matching accelerator directly
(lines 1002-1026).

`dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp` updates those
indexes when an asset enters the authority (lines 3445 onward). `EnumerateAssets` (line 1818 onward)
orders selective filters first and dynamically chooses indexed intersection versus filtering the
current candidate set. The code comments explicitly account for construction and lookup cost rather
than assuming every extra index is free. The preallocation path also avoids reserving asset-scale
capacity for low-cardinality accelerator maps.

`dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryStateTest.cpp` profiles a
complete filtered enumeration over one million assets and verifies a one-record result. It does not
profile sorting in isolation.

The reusable principle is: query an immutable/cached authority through selective accelerator data,
keep full-query costs visible, and pay write/memory amplification only for demonstrated hot query
dimensions. Zircon must not copy Unreal's pointer containers or add an index without workload proof.

### Rust landing-zone cross-checks

`dev/bevy/crates/bevy_asset/src/assets.rs` keeps dense and UUID storage separate and exposes direct
iterators. Its iterator includes an explicit note that another skip-list accelerator needs a
cost/benefit decision. `dev/Fyrox/fyrox-resource/src/manager.rs` likewise exposes direct iteration
and clones handles only when a caller requests an owned resource list. Neither reference sorts an
entire registry to answer an unordered storage query.

These references support a Zircon-specific split: immutable snapshot publication and deterministic
export are valid, but a new per-kind index is justified only if the full-path profile shows repeated
kind export is hot enough to repay mutation and memory cost.

## Existing performance evidence is insufficient

The current ignored benchmark was executed from the exact Windows projection test binary:

`D:\zircon-frameworks01-r12-resource-commit-ordering-20260827\zr_resource_commit_ordering_tests.exe`

It reported:

| Variant | p95 for 16 iterations |
| --- | ---: |
| stable sort | 1,142,521,500 ns |
| unstable sort | 1,046,433,900 ns |

The timed loop calls `fixture()` on every iteration. That function formats 4,096 strings, parses
4,096 locators, hashes IDs, constructs full records, and only then sorts. It never calls
`ready_records_for_kind`, never acquires the authority lock, never scans registry state, and never
measures writer delay. The approximately 8.4% aggregate delta therefore cannot establish a product
bottleneck or a registry-export improvement.

## Candidate algorithms

### A. Local unstable sort

Keep the existing registry scan and long-held lock, changing only the sorting primitive.

- Total work: `O(N + K log K)`.
- Writer exclusion: `O(N + K log K)`.
- Extra persistent memory: none.
- Decision: reject as the target architecture. It leaves the structural lock and scan problem
  unchanged and lacks complete-path evidence.

### B. Copy-on-write registry snapshot, then filter and sort

Clone the `ResourceRegistry` under the authority lock, release the lock, then run the current scan,
clone, and deterministic sort against the snapshot.

- Total work: `O(N + K log K)`.
- Writer exclusion: `O(1)` Arc clones.
- Extra persistent memory: none; transient snapshot retains existing maps.
- Decision: mandatory comparison baseline. It isolates concurrency benefit from query-algorithm
  changes.

### C. Atomic registry plus management snapshot, then ordered merge

Clone both the copy-on-write registry and its paired immutable management generation while holding
the same authority read lock. Release the lock, scan ready rows for the requested kind in existing
locator/id order, and clone full records by ID from the captured registry.

- Total work: `O(N + K log 64)`.
- Writer exclusion: `O(1)` Arc clones.
- Extra persistent memory: none.
- Semantics: the registry and projection are from the same authority generation; output order is
  unchanged and no post-sort is needed.
- Decision: preferred implementation candidate, subject to sparse and dense workload gates.

### D. Persistent ready-record index by kind

Publish a locator/id ordered ready-row or record index per `ResourceKind` in every management
generation, similar to Unreal's class accelerator.

- Query work: `O(K)`.
- Mutation work: copy/update the affected kind index, or rebuild indexes.
- Extra persistent memory: at least one additional row/ID reference per indexed ready resource plus
  map and allocation overhead.
- Decision: defer unless profiles show this export is frequent and candidate C remains material.
  Three current startup/integration consumers do not justify unconditional write amplification.

### E. Adaptive immutable snapshot export

Capture the paired registry and management generation once, then choose between B and C from the
already-published `N` and `K` summaries. The measured cost model is:

- use management scan for at least 32,768 total records when `K <= 64` or `K/N >= 10%`;
- use management scan for 4,096-32,767 total records when `K/N >= 25%`;
- otherwise scan the copy-on-write registry snapshot into an exact-`K` vector and sort only when
  more than one row matched.

This follows Unreal's query-planning principle without adding Unreal's persistent per-class memory
cost. The thresholds are workload classes, not consumer names, resource kinds, or syntax cases.

## Profiling plan and acceptance gates

All generated binaries, raw samples, and summaries must stay under a dedicated `D:` evidence root.
Cargo remains coordinator-only; direct `rustc` projection validation is allowed.

Profile exact full-path implementations A, B, and C with already-built managers so fixture creation
is outside the timed region:

| Dimension | Values |
| --- | --- |
| Total records `N` | 1,024; 10,000; 100,000 |
| Matching ready records `K` | 1; 1%; 10%; 100% of `N` |
| Locator order | insertion ordered; reverse; deterministic shuffled |
| Metric | total median/p95; authority-lock hold median/p95; requested/max allocation bytes |
| Contention | 1 export reader plus 1 commit writer; 4 readers plus 1 writer |
| Writer metric | commit lock-acquisition delay median/p95/max |

Correctness must prove:

- exact locator/id ordering parity with the pre-change implementation;
- ready state and non-zero revision filtering;
- a registry/projection snapshot cannot mix revisions during concurrent commits;
- empty, single-match, sparse, dense, reverse-order, and repeated-export cases;
- all three current consumers continue to observe complete deterministic results.

Candidate C is accepted only when:

- authority-lock hold p95 falls by at least 90% for 10,000 and 100,000 records;
- writer delay p95 falls by at least 75% under the one-reader contention profile;
- no tested total-time median regresses by more than 15%;
- dense 100,000-record total time improves by at least 20% versus A;
- exact output and concurrent snapshot tests pass;
- no persistent per-kind index is added.

If C fails the sparse-workload gate but B passes the concurrency gates, choose B. If neither lock
profile shows material writer delay in the actual consumer cadence, retain the current algorithm and
remove the unsupported performance claim. Candidate D requires a separate frequency and memory
profile before implementation.

## Preimplementation profile result

The exact hard-cut projection was copied to
`D:\zircon-frameworks01-r12-resource-registry-export-profile-20260827`. It was compiled directly
with `rustc -C opt-level=3`; no Cargo job or repository target directory was used. Managers were
built before timing. Eleven paired samples covered 1,024, 10,000, 32,768, and 100,000 total records
at one, 1%, 10%, 25%, and 100% matching records. This was exploratory query evidence, not final
acceptance evidence: independent review later proved that its single barrier did not establish a
writer attempt before the reader could release the lock.

Selected v5 results:

| Workload | Current median | Adaptive median | Delta | Current lock p95 | Adaptive lock p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1,024 / 1 | 3,247 ns | 3,500 ns | +7.79% | 3,922 ns | 60 ns |
| 1,024 / 256 | 106,150 ns | 117,202 ns | +10.41% | 141,950 ns | 517 ns |
| 10,000 / 1,000 | 707,312 ns | 687,025 ns | -2.87% | 1,169,887 ns | 2,525 ns |
| 10,000 / 10,000 | 14,793,437 ns | 12,496,150 ns | -15.53% | 34,494,400 ns | 3,475 ns |
| 32,768 / 3,276 | 5,589,600 ns | 6,341,400 ns | +13.45% | 8,093,550 ns | 1,200 ns |
| 32,768 / 32,768 | 66,421,000 ns | 45,635,350 ns | -31.29% | 89,684,700 ns | 4,900 ns |
| 100,000 / 1,000 | 4,241,750 ns | 3,718,600 ns | -12.33% | 5,668,350 ns | 800 ns |
| 100,000 / 100,000 | 241,473,150 ns | 165,685,900 ns | -31.39% | 239,285,350 ns | 1,350 ns |

Every measured query median regression is below the 15% gate. For the dense 100,000-record workload,
requested allocation falls from 72,804,960 to 30,001,024 bytes (-58.79%) and measured peak
additional allocation falls from 38,203,008 to 30,001,024 bytes (-21.47%). Its writer-wait rows are
withdrawn rather than used as a gate. No persistent index or additional generation memory is
introduced.

Evidence hashes:

- optimized executable: `24660c3437b61ade26b006fb4961595899a82523ec8cee33603098c7723d2e18`;
- v5 raw CSV: `73c6d3a691e38b553d78bd81b0d49853938ad4ab945a48bfd166d646a7bfacd2`;
- profiled resource rlib: `08cbc257959b77af93adaa78ef5d165010a29452ccf97b16d956d533066ed1e5`.

The exploratory source files were overwritten by the corrective reruns and are intentionally not
listed as retained evidence. The three hashes above still resolve to files in the D-drive evidence
root; the canonical postimplementation section below has a complete source/binary/CSV hash set.

The profile proves CPU time, allocation, authority-lock, and writer-wait changes. It does not include
hardware package-energy counters, so no power-consumption or cross-engine energy-equivalence claim
is made.

## Implemented boundary

Candidate E is implemented with one private export-snapshot type inside the existing
`manager/registry_export.rs` responsibility. Capture registry plus management generation under one
authority read lock, then perform all enumeration and cloning after the guard is dropped. Do not add
a public registry snapshot API, do not leak `ResourceAuthority`, and do not move behavior into a
root `mod.rs` file. Existing `ready_records_for_kind` remains the public facade. Its output order is
hard-cut to canonical locator display plus resource id so registry and management planner paths have
one deterministic cross-scheme contract; the old `ResourceScheme` enum-order behavior is not kept as
a compatibility branch.

This is a leaf implementation inside the approved resource-manager facade. It adds no compatibility
path, no new cross-crate dependency, and no editor or graphics ownership.

The query planner uses private module constants because these thresholds are an implementation cost
model, not a cross-crate contract. It chooses the existing management generation for profiled large,
very sparse, or dense cases; other cases scan the copy-on-write registry snapshot into an exact
ready-count capacity and sort only when more than one row matched. The management path consumes the
existing canonical-display locator/id ordered scan and looks up full records in the paired registry
snapshot. Every row must match kind, locator, revision, and state; the exhausted scan must also match
the published ready count. Missing rows, extra rows, missing registry records, or metadata drift
return to the complete registry truth. This adds no second persistent index or mutation-time memory
amplification.

## Implementation and focused validation result

The structural contract was first compiled against the pre-change exact projection. The focused
test failed at the expected missing `ResourceRegistryExportSnapshot` assertion, establishing the
long-held registry guard as the red condition. After implementation, the ten registry-export tests
cover:

- same-authority registry/management capture and immediate guard release;
- the profiled adaptive thresholds, including exact boundary values;
- canonical-display locator/id order across all schemes and ready/non-zero-revision filtering;
- management-scan parity with registry-scan export, including a planner-selected cross-scheme case;
- paired registry and management row metadata;
- atomic same-generation results during concurrent whole-batch commits;
- complete registry fallback instead of partial output for missing registry ids, incomplete scans,
  or row metadata drift injected across the internal pairing invariant.

Direct Windows `rustc --test` validation used the exact generated `zr_resource` projection under
`D:\zircon-frameworks01-r12-resource-registry-export-red-20260827`. Focused result: 10 passed, 0
failed. Full projection result: 120 passed, 0 failed, 2 ignored. The generated hard-cut projection
still reports seven pre-existing unused/dead-code warnings outside this slice; no touched production
path adds `panic!`, `unwrap`, `expect`, `allow(dead_code)`, a long registry guard, or a compatibility
branch.

Retained test evidence hashes:

- test binary: `dc529e24d293380ede46b2dbc29ef049f12520b529f11a859d099ce58677348f`;
- focused stdout: `08ab81099359e20d49e404db79bbc6f91e50f26bb1fd405709769c3bca57c81d`;
- full stdout: `c25baf4ecda05d9098d7b1db9aaaf95552abea9125350a6d712ff06722c217c2`;
- both retained stderr logs are empty.

## Postimplementation profile result

The same D-drive profiler was rebuilt with the production snapshot/planner implementation and
timing-only wrappers. Two initial 11-sample reruns exposed contradictory isolated regressions and are
not treated as stable conclusions. Independent review then rejected the single-barrier writer claim
and found that the first corrective run compared the new canonical-display comparator against the
old enum-order baseline; that 42-row CSV is retained but invalid for acceptance. The harness now
requires the reader to hold the authority guard, rendezvous, and observe the writer-attempt release
store before it may continue. Both variants also use the same canonical-display comparator.

Two complete equal-comparator reruns used 31 samples, three warmups per variant, and per-sample
iterations of 100/20/5/3 by scale. The first passed with a worst median delta of +6.215%, dense
100,000 improvement of 47.862%, and 10,000+ lock-p95 reduction of at least 99.847%. The final
canonical 42-row output has 20 paired query workloads plus two schema-complete writer-wait rows;
stderr is empty.

| Workload | Current median | Implemented median | Delta | Current lock p95 | Implemented lock p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1,024 / 1 | 2,597 ns | 2,376 ns | -8.51% | 3,098 ns | 57 ns |
| 1,024 / 1,024 | 818,203 ns | 908,427 ns | +11.03% | 2,020,588 ns | 555 ns |
| 10,000 / 1 | 31,240 ns | 32,405 ns | +3.73% | 88,910 ns | 95 ns |
| 10,000 / 1,000 | 1,213,900 ns | 1,178,550 ns | -2.91% | 1,897,270 ns | 575 ns |
| 10,000 / 10,000 | 20,375,370 ns | 16,969,150 ns | -16.72% | 26,724,500 ns | 1,825 ns |
| 32,768 / 3,276 | 5,720,520 ns | 5,511,520 ns | -3.65% | 11,544,060 ns | 800 ns |
| 32,768 / 32,768 | 81,758,120 ns | 57,276,480 ns | -29.94% | 123,300,640 ns | 1,000 ns |
| 100,000 / 1,000 | 3,630,966 ns | 3,441,400 ns | -5.22% | 4,908,366 ns | 700 ns |
| 100,000 / 100,000 | 310,551,266 ns | 170,794,100 ns | -45.00% | 357,750,766 ns | 1,166 ns |

The worst median change across all 20 workloads is +11.027%, within the 15% gate. Every 10,000+
workload reduces lock-hold p95 by at least 99.893%. Dense 100,000-record total time improves 45.003%,
exceeding the 20% gate. Under the corrected writer-attempt handoff, writer-wait p95 falls from
362,889,400 ns to 66,000 ns (-99.982%), exceeding the 75% gate; the current baseline p95 remains in
the same long-lock scale as its 357,750,766 ns query lock p95. For 100,000 full matches, requested
allocation falls from
72,804,960 to 30,001,024 bytes (-58.79%) and peak additional allocation falls from 38,203,008 to
30,001,024 bytes (-21.47%). The 100,000/1 management case requests 232 additional transient bytes
but improves median time by 16.68%; no persistent memory is added.

Postimplementation evidence hashes:

- harness source: `ea4835c776d86d842f7edd27c85886577aec353ea507d94c4183bf44374ce820`;
- production profiler executable: `009a28aa0a2a3e37bbf335d96cf8be8588cad883bac406785defa49129040669`;
- raw postimplementation CSV: `ca89d8ce17f4ae06e6f531267816afd5fe694584757343f0c60304c61cc626d4`;
- profiled resource rlib: `a957b6f9039992726ca24b398f136b1085fb2eeb08a45a66b4b51b0632dd1493`;
- production-plus-timing-hooks source: `73aa603caaa4049824cacc1bcdde2236d921415ac2fa22d3853f058331ff3d28`.

These results establish CPU, allocation, lock-hold, and writer-delay behavior on this Windows host.
No hardware energy counter, production cadence trace, or cross-engine energy benchmark was captured,
so the slice makes no power-equivalence or whole-product acceptance claim. Managed Cargo validation
and consumer integration remain coordinator-owned acceptance work.

## Managed Cargo result

Coordinator job `03ec12d4aed34026a43ca913a6c901e8`, run
`2ef3bc71968740e5bbaea4a105d18cb7`, executed `cargo check -p zircon_runtime` against the current
workspace with the ephemeral D-drive target
`D:\cargo-targets\zircon-engine\frameworks01-r12-registry-export-review-20260827`. It ran from
2026-08-27 20:50:02 to 20:55:15 Asia/Shanghai and exited 101 before compiling this slice: the only
error was foreign E0599 at
`zircon_runtime_interface/src/runtime_api/session/editor_transform.rs:182`, where
`ZrByteSliceError` does not implement `Display`. The three warnings were also in foreign
`zircon_runtime_interface` sources. The 23,983-byte stderr SHA-256 is
`a9c4906395bf5ef7794208cda238bcd80fbf8fd7f912aec0eeb4a012ebdf26de`; stdout was empty. The job was
finished and released, its Cargo process tree is empty, and the D-drive target has been removed.
Frameworks01 does not edit or claim the interface error owner. This result therefore changes the
slice from managed-Cargo pending to foreign blocked, without weakening the exact projection or
independent-review evidence and without promoting M1.

## Independent review result

The final hash-gated review returned `C0 / I0 / M0, Ready`. It confirmed one canonical-display/id
order across both planner paths, complete registry fallback for projection inconsistency, the
writer-attempt contention handoff, the `10/10` and `120/0/2` test evidence, and all final profile
hashes and arithmetic. Managed Cargo remains the only acceptance item for this slice and is now
recorded as foreign current-source blocked rather than pending.

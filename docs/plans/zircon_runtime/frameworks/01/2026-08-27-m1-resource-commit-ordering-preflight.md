# M1 Resource Commit Ordered Staging Preflight

## Status

- Plan owner: Frameworks01
- Session: `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`
- State: `ordering_optimization_profiled / correctness_followup_source_repaired /
  independent_review_green / public_contract_documented / current_source_managed_full_library_green /
  zr_resource_warning_boundary_hard_cut_green / full_m1_acceptance_pending`
- Reviewed current HEAD: `ea35974cdf64068f6789010451d20bbf69e0a29d`
- Production input SHA-256: `586d8fd3c6443da4b653ed22eee4764805c6f36ebd85e438301d3e6531a2cfa3`
- Focused test input SHA-256: `46b6afb5d40278dcbaa1a5263244d4b833d90aee3b64d7bc653872a75862dbb0`
- Production output SHA-256: `823bea227221204fab81d84c0a70fc293a74ab9e6d32bd1e0cb4e2030c959ce8`
- Revision output SHA-256: `24114e07d84f1c08d3622055305884cfcaabf138663102ad4cac57808017259d`
- Focused test output SHA-256: `342f0ce085b35fbd6e66a67220b20cf797429dffcd6b0fe7e85e64febc4c8461`
- Scope: transaction-local staged-resource identity, first-touch ordering, and remove/re-add revision/state lineage
  inside `ResourceManager::prepare_commit` / `PreparedResourceMutation::commit`
- Excluded: public mutation API, registry representation, management/readiness algorithms,
  event retention, durable file transactions, physical `zr_resource` cutover, and product power claims

This record is deliberately written before production editing. It follows the required sequence: whole-owner and
consumer review, reference-engine review, controlled pre-profile, locked structural design, tests, implementation,
then exact post-profile and managed validation.

## Current Module And Consumer Review

The live Resource authority already follows the correct high-level transaction contract:

1. `commit_serial` admits one prepared mutation at a time and remains held across an upper-layer durable dependency
   transaction;
2. preflight reads one coherent authority snapshot and rejects locator collisions, implicit rename, kind drift,
   revision drift, and invalid state transitions without mutation;
3. apply takes the authority write lock, removes old registry identities before inserting new identities, updates
   management/readiness/payload/runtime state, and creates the receipt and events from the same staged state;
4. the authority lock is released before callbacks receive events, while `commit_serial` remains held until event
   publication completes;
5. event order is the first-touch order of distinct Resource IDs, independent of `HashMap` iteration order.

Production callers are not limited to one-record hot paths. Project open/full and targeted scans, project resource
reconciliation, builtin registration, imported-asset publication, close, relocation, and facade insertion all build
`ResourceMutationBatch`; project-wide publication can therefore contain thousands of distinct Resources. The local
transaction and concurrency suites also depend on exact rollback, first-touch event order, generation coalescing,
prepared-gate visibility, and stale-residency-token behavior.

The current transaction-local representation is structurally contradictory:

```text
preflight:
  HashMap<ResourceId, StagedResource { order, ... }>

apply:
  HashMap::into_values -> Vec<StagedResource>
  retain net-visible entries
  sort_by_key(order)
```

`order` exists only to reconstruct order that preflight already knew. For `K` distinct touched Resources, apply moves
every large staged entry out of hash buckets and then performs `O(K log K)` comparisons and swaps under the exclusive
authority write lock. `StagedResource` contains two inline `Option<ResourceRecord>` values; `ResourceRecord` itself
owns locators, dependency/diagnostic vectors, and three strings. The old focused benchmark compared stable with
unstable sort over 32,768 entries, so it optimized the sort implementation without questioning why commit sorts.

The full current-source review found no semantic need for arbitrary deletion or reordering inside preflight. A staged
identity is inserted once on first touch, then updated in place. This supports one insertion-ordered vector plus a
derived `ResourceId -> slot` index directly.

## Reference-Engine Review

Unreal Asset Registry is the primary reference because this is engine-scale resource authority, not a generic Rust
container exercise. `AssetRegistry.cpp` documents that registry mutations occur under the write lock, arbitrary
callbacks/events are forbidden inside that lock, and `FEventContext` defers publication until the top-level function
has released it. `AssetRegistryImpl.h` stores transaction-local asset events directly in an ordered
`TArray<TPair<FAssetData, EEvent>> AssetEvents`; it does not collect keyed values and sort them to reconstruct mutation
order. Asset identity lookup remains a separate keyed authority.

The applicable rule is the same one used by the ResourceEvent publication index: identity lookup and deterministic
order are separate responsibilities owned by the same transaction. Zircon must retain its stricter prepared commit,
rollback, generation, residency-token, and retained-event semantics; it should not copy Unreal's UObject or game-thread
surface.

Bevy's asset flow is a secondary check: mutations update keyed asset state and append events to an ordered queued-event
vector, then publish them as a batch. Its frame-lifetime queue does not replace Zircon's prepared transaction or
retained event stream, but it independently rejects a post-hoc sort as the normal way to recover mutation order.

## Pre-Change Profile

The first design-model run below is retained as rejected audit history, not acceptance evidence. Its modeled current
branch used `HashMap::with_capacity(operation_count)`, while the real pre-change production source used
`HashMap::new()` for staged IDs and locator overlays. That mismatch exaggerates repeated/random baseline allocation.
The later real-baseline structural, allocation, and public-API results supersede this table.

A D-drive Rust 1.94.1 release harness models the current container and the locked alternative with a 616-byte staged
entry. The size model represents the two inline ResourceRecord options plus locator/payload/bookkeeping footprint; it
does not claim exact ABI size. Each reported value is the median of 11 samples. The ordering cases construct the
container before timing and measure only its modeled `into_values + sort_unstable` versus ordered-vector finalization. The
full cases measure insert/update plus finalization. `black_box` retains observable work.

Artifacts:

- directory: `D:\zircon-frameworks01-r12-resource-commit-ordering-20260827`;
- source SHA-256: `941da2281b8f07e2819f1c955682eab41271ddcf66bb893f06432c8f8951a9c9`;
- executable SHA-256: `c01751ae25bfa200902c882c3f82b34cf1caaf0c5eff3989e5823e7fafcf521c`;
- CSV SHA-256: `94dadf6e9225cb3a2205b6f35f5b131c6495c83853ccfa1bf75b2fe4770cab33`;
- compiler: rustc `1.94.1` / LLVM `21.1.8`, `-C opt-level=3 -C target-cpu=native -C debuginfo=1`;
- host: Windows x86_64, AMD Ryzen 7 5800H, 8 cores / 16 logical processors.

| Scope | Operations | Distinct IDs | Pattern | Current median | Indexed Vec median | Reduction |
| --- | ---: | ---: | --- | ---: | ---: | ---: |
| ordering only | 64 | 64 | unique | 8.600 us | 0.500 us | 94.18% |
| ordering only | 1,000 | 1,000 | unique | 491.100 us | 0.500 us | 99.89% |
| ordering only | 10,000 | 10,000 | unique | 11.116 ms | 3.200 us | 99.97% |
| ordering only | 32,768 | 32,768 | unique | 40.020 ms | 168.500 us | 99.57% |
| full staging/finalize | 1 | 1 | ordered | 220 ns | 190 ns | 13.63% |
| full staging/finalize | 64 | 64 | ordered | 16.194 us | 3.936 us | 75.69% |
| full staging/finalize | 1,000 | 1,000 | ordered | 1.582 ms | 79.526 us | 94.97% |
| full staging/finalize | 10,000 | 10,000 | ordered | 20.300 ms | 3.172 ms | 84.37% |
| full staging/finalize | 32,768 | 32,768 | ordered | 85.583 ms | 13.372 ms | 84.37% |
| full staging/finalize | 100,000 | 64 | repeated | 3.291 ms | 3.137 ms | 4.68% |
| full staging/finalize | 100,000 | 4,096 | random | 32.256 ms | 11.185 ms | 65.32% |

The broad distinct-ID cases identify a structural ordering tax. The repeated-64 case is intentionally retained as a
negative control: when lookup/update work dominates and final `K` is small, the alternative is only 4.68% lower. The
single-item case does not regress. This rejects further tuning of the post-hoc sort and supports changing ownership of
order itself.

This isolated profile is not whole-commit, project-load, frame-time, energy, or Unreal parity evidence. Management and
readiness projection can still dominate a real project-wide commit and require their own profiles before any later
algorithm change.

## Locked Design

Replace the transaction-local map with one private container:

```text
StagedResources {
  index_by_id: HashMap<ResourceId, usize>,
  entries: Vec<StagedResource>,
}
```

- `entries` is the sole first-touch order authority;
- `index_by_id` is a verified derived lookup from ID to vector slot;
- first touch appends once and inserts its slot; later operations mutate the same slot;
- `StagedResource::order`, `next_order`, `HashMap::into_values`, and all ordering sorts are deleted;
- apply may retain net-invisible entries but never reorder remaining entries;
- initial vector/index/locator capacities are `min(operation count, 64)`; growth beyond that follows actual distinct
  Resource IDs and locators rather than total repeated operations;
- no public type, compatibility path, alias, event format, generation rule, or lock boundary changes.

Expected complexity changes from expected `O(1)` staging plus `O(K log K)` final ordering to expected `O(1)` staging
plus `O(K)` ordered traversal, with contiguous apply access. Space is `O(K + L)`, where `K` is the number of distinct
touched Resource IDs and `L` is the number of distinct locator-overlay entries. The large staged values live only in
the vector, while the ID hash table stores `(ResourceId, usize)`. The fixed 64-entry warm reserve is `O(1)`.

## TDD And Acceptance Gates

Implementation must first make the source contract RED, then satisfy all of these gates:

1. private ordered staging owns both the ID index and entry vector;
2. no transaction-local `order` field, `next_order`, `into_values`, `sort_by_key`, or `sort_unstable_by_key` remains;
3. repeated operations for one ID update one slot and keep the first-touch position;
4. mixed add/update/remove/rename batches publish events in first-touch order;
5. net-zero entries remain filtered without advancing management/readiness generations or publishing events;
6. collision, implicit rename, kind/revision/state failures remain atomic with no mutation;
7. prepared transaction visibility, event-after-state, and residency-token regressions remain green;
8. production and focused test owners remain below the 800-line soft budget;
9. exact post-profile uses the same harness/compiler/host and reports all cases and hashes;
10. 32,768 distinct full staging/finalize is at least 70% below current, 1,000 distinct is at least 80% below current,
    100,000/4,096 random is at least 50% below current, and single/repeated-64 cases do not regress by more than 15%.
11. a 100,000-operation repeated-ID batch must not eagerly reserve 100,000 staged entries or locator slots; exact
    allocation evidence must cover 32,768/32,768, 100,000/64, and 100,000/4,096 cases.

The production/test/future-record paths were admitted by ownership-transfer preview/apply fingerprint
`b6e0f7eaa4f1ee980294d26174337758256e7d601cccd15d11a68058ce6cf2d7` before any repository edit. This
preflight does not authorize milestone acceptance, Git commit, WeCom notification, product performance claims, or a
second Cargo job while the current package gate has 55 foreign compile errors.

## Implementation Result

The production implementation now matches the locked design:

- `StagedResources` owns `HashMap<ResourceId, usize>` plus `Vec<StagedResource>`;
- `StagedResource` owns its ID directly, so apply no longer re-derives identity from before/after records;
- first touch appends one entry and every later operation for the same ID mutates that slot;
- apply consumes the ordered vector and retains net-visible entries without sorting;
- the old `order`, `next_order`, `into_values`, `sort_by_key`, and `sort_unstable_by_key` path is gone;
- a module-private 64-entry initial-capacity limit preserves the measured small-batch warm reserve without allocating
  large staged storage for repeated operations;
- receipt maps reserve from staged cardinality, avoiding two additional growth ladders after apply;
- no public API, state transition, generation, publication, or commit-lock boundary changed.

At the ordering-optimization snapshot, the production owner was 605 physical lines and the focused test owner was
124 physical lines, both below the 800-line soft budget. Later correctness work has changed the current source and is
accounted for separately below; these historical profiles are not relabeled as measurements of that later snapshot.
Four focused tests cover the source contract, bounded initial capacity, repeated-ID first-touch slot ownership, and
observable mixed add/update/remove/rename publication order. The mixed test intentionally touches the update ID twice
and proves that the final event remains in its original first-touch position.

The first implementation incorrectly reserved the full operation count for the staged vector, ID index, and locator
overlay. Independent review classified this Important because a repeated-ID batch made eager allocation `O(N)` even
though live state is `O(K + L)`. The source-contract test was made RED against that implementation, then the bounded
capacity policy and allocation profile below were added. The rejected eager-reserve snapshot is not an integration
candidate.

## Exact Isolation Validation

The current-source `zr_resource` projection was rebuilt directly with Rust 1.94.1 against the managed dependency
pool, with all compiler and test artifacts under `D:`. This is an exact source isolation, not a Cargo acceptance gate.

- test executable SHA-256: `efdf8420bf261cfe4747e305e14b368ec4c90be27dfba463a3d612a6e9d33a7a`;
- focused ordering/capacity tests: `4 passed / 0 failed`;
- full projection: `112 passed / 0 failed / 3 ignored` in 3.98 seconds;
- ignored tests are the pre-existing managed-release performance gates;
- the projected crate emitted seven unused/dead-code warnings and no compile errors.

Final review rejected the first corrected evidence set because its projected old branch still used manual
`Vec::with_capacity + extend`, `sort_unstable_by_key`, and pre-sized receipt maps instead of the monolith's
`collect::<Vec<_>>()`, stable `sort_by_key`, and `HashMap::new()`. Those earlier structural, allocation, and paired
results are audit history only and are not acceptance evidence.

The final structural harness uses `HashMap::new()`, `collect::<Vec<_>>()`, and stable `sort_by_key` for the exact old
container and the bounded capacity policy for the new container. Source/executable/CSV SHA-256 values are
`5f6041b643c7405d6929102932242b3842b94cfc6edf4e8251f5d188a080f1ed`,
`8573694ef43b6334786a4cebd7d40a365147ba42bf3629b2b3a52a5803c98bfb`, and
`afbca7db47c9ce9811c0a5e788a082ebd02da171a58267f5b7345979f26832d2`. Full staging/finalize is
88.51% lower at 32,768 distinct IDs, 92.00% lower at 1,000 distinct IDs, and 79.68% lower at
100,000 operations/4,096 random IDs. The single-ID case is 34.60% lower; repeated-64 is 7.69% slower and remains
inside the locked 15% negative-control gate. All locked structural gates pass against the exact old container.

The exact allocation harness was corrected to report only allocation requests made during `ResourceManager::commit`.
It no longer reports live peak because the batch is allocated before the measurement window and consumed inside it.
Deallocations of pre-window pointers therefore cannot corrupt the retained metrics. It links exact pre-change source
SHA-256 `bc8891a08d44062492ad54ea15c5f18434f6646453025263a03e040a8cfcf2b3` / rlib SHA-256
`11f9764bba9a6750f7d521e6bbe547f040910a58aa9fe2e007aa445bb8e1ce21` against bounded-post rlib SHA-256
`0cbb5abb4f8697faef15448cc27b5a40e9a4b3715a45e873fe8a0574b20e997d`.
The real-pre projection is retained at
`D:\zircon-frameworks01-r12-resource-commit-real-pre-build-20260827\zircon_runtime\crates\zr_resource\src\manager\commit.rs`.
It was copied from `D:\zircon-frameworks01-r12-resource-snapshot-r4-applied`, then the two transaction-local maps,
staged-value collection, stable sort, and two receipt maps were restored from Git blob
`bf7b8d69e2aee5c3c5f92c8452cbcfa45b72be13`. After excluding the projection-only optimization-test module and
normalizing the physical-crate `use crate::{...}` path plus public `PreparedResourceMutation` type/commit-method
visibility, all 561 production lines match that blob and no commit algorithm or event semantic differs.

- harness SHA-256: `472d6864579f51fe1603b87b190d4f07f307be6e5c2aae34a8253c4076d6c2b3`;
- pre/post executable SHA-256: `6a15fe2e569195b8aac41801d200d08256cf9d4de4936bcc8dd9b4016f94f7ae` /
  `41c4b4e2d3a158021afeea1270f0102ec1a6da1e4624a60289e5a10a673cc036`;
- pre/post CSV SHA-256: `d2a722e0b67341c06df0aa16ae6ca7d66858b7857b10170d344da529b46e523e` /
  `b399f46762fb1650347e4372b5c71e87c32bf71f75be66548489a8b3b9db7bd5`.

| Operations | Distinct IDs | Requested pre/post | Reduction | Largest request pre/post | Reduction |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 32,768 | 32,768 | 246.20 / 179.40 MiB | 27.13% | 40.06 / 19.75 MiB | 50.70% |
| 100,000 | 64 | 9.35 / 9.15 MiB | 2.11% | 80.1 / 39.5 KiB | 50.71% |
| 100,000 | 4,096 | 41.90 / 32.29 MiB | 22.94% | 5.01 / 2.47 MiB | 50.70% |

The exact public-API timing harness links both real-pre and bounded-post rlibs into one process and alternates AB/BA
order for 12 paired samples per case. Source/executable/raw/summary SHA-256 values are
`0c77e4ae71c35b11022074e7d7c0aad37af5be2b4f1cf869baaca7b3bf6e2adf`,
`1db3aacd3fe30e1d0f98b469b793408981af9aa27f6866202a6046ded72bdaaa`,
`6b1378a0ea2022361b91816e830c4b56d29df1e90e2d9f5927579cafd65424a9`, and
`5dfc3e8d6f06f04a75fc588128f795c9437eb0f87db4219b1e8753a49ed8f93c`.

| Operations | Distinct IDs | Paired reduction median | Wins | AB / BA medians | Conclusion |
| ---: | ---: | ---: | ---: | ---: | --- |
| 1 | 1 | 9.88% | 8/12 | 10.81% / 5.26% | exploratory improvement |
| 64 | 64 | 28.93% | 12/12 | 28.93% / 28.32% | improvement |
| 1,000 | 1,000 | 30.16% | 10/12 | 25.80% / 33.63% | noisy improvement |
| 10,000 | 10,000 | 8.84% | 7/12 | 21.34% / -1.54% | inconclusive positive trend |
| 32,768 | 32,768 | 16.07% | 9/12 | 20.36% / 8.13% | noisy improvement |
| 100,000 | 64 | -3.90% | 5/12 | 11.46% / -10.26% | inconclusive |
| 100,000 | 4,096 | 15.38% | 7/12 | -2.29% / 26.48% | order-sensitive, inconclusive |

No paired median regresses beyond the locked 15% control gate, but the high-variance 32,768 and 100,000-operation
cases do not justify a product-wide speedup claim. The accepted evidence is structural complexity, exact allocation
reduction, preserved behavior, and no observed median regression above the gate. A quieter host and phase-timed or
sampling profile remain required before attributing residual cost or claiming whole-commit optimality.

The managed package check `433c40bf07154cbf9bc1d712f94dcf09` remains failed on 55 foreign current-source errors
across asset, builtin, render, task, platform, plugin, and scene owners. No new Cargo job was started for that snapshot.
Final independent exact-source review of the ordering optimization was Ready at `C0 / I0 / M0`; it does not cover the
later correctness changes below. Managed acceptance remains required before coordinator commit or WeCom notification.
Windows Performance Recorder privilege was unavailable, so this record makes no energy or engine-parity claim.

## 2026-08-29 Remove/Re-add Revision Lineage Correctness Follow-up

### Status

| Work item | Status | Evidence |
| --- | --- | --- |
| Whole commit/revision owner reread | complete | All staged operations, apply ordering, revision helper, transaction tests, and revision consumers rechecked |
| Unreal-led reference check | complete | Asset Registry event context records mutations under the lock and broadcasts after unlock; Bevy rejects stale slot generations |
| Remove/re-add CAS repair | implemented | Last effective record before removal remains the upsert revision/state baseline |
| First Ready revision repair | implemented | A zero-revision non-Ready record publishes its first Ready generation as revision 1 |
| Rust 2021 format and owner budgets | green | 625-line production owner, 31-line revision owner, 659-line transaction-test owner, all below the 800-line soft budget |
| Prior managed `readd` filter | green but superseded | coordinator job `4c58d6addbf6442f976dae46422c58f2` passed before the importer identity/exhaustion repair |
| Independent follow-up review | `C0 / I0 / M0`, Ready for managed Cargo | Reviewer matched all seven declared hashes, independently passed the 15-case architecture guard and exact Rust 2021 format, and performed no writes or Cargo |
| Public contract documentation | complete | archived r8 ownership transferred under fingerprint `e50d3ed1b99ebad7bf331af74853dda9f56b5e16051c3b1806391628217bab1a`; `core/resource.md` now distinguishes batch-local lineage from later independent re-add and documents fail-closed exhaustion |
| Current-source managed Resource compile/test | green | Production check job `a86b9a1cc126443db28241b511870863` emitted zero `zr_resource` warnings; final library job `1dcb8301b7b047d58f246c823fe6b42d` passed `167 / 0 / 3` with the test-only helper hard cut |
| Editor upward authority execution | blocked by foreign current source | job `41919258d25041c39d9c74d57d4c989f` stopped before Editor tests on `zircon_runtime_host` E0004 for the new `WorldQueryResult::TransformSnapshot` variant |
| Full package/M1 acceptance, commit, WeCom | pending | This targeted pass does not close the larger physical-cut and full-validation gates |

### Root cause and architecture decision

`Remove` cleared `StagedResource::record`. A following `UpsertReady` or `UpsertLazy` therefore observed no effective
record even though the intermediate absence was not externally visible. The final upsert could reset a changed Ready
record from revision 1 to revision 1, allowing an artifact prepared against the pre-batch revision to pass the later
`store_payload(expected_revision)` CAS. The same gap let `Error -> remove -> ordinary ready` bypass the explicit
recovery rule. Separately, `next_ready_revision` returned 0 when a zero-revision Error/Pending record first became
Ready without metadata changes; ready registry export intentionally rejects revision 0.

The correction keeps one transaction-local `removed_record_baseline`. Direct removal of the original record does not
clone it: the existing `before` value is reused. Only when earlier operations produced a different effective record
does the removal clone and retain that last record. A later upsert resolves its baseline in this order: current
effective record, changed record immediately before removal, transaction-entry `before`. Removal still clears the
final record, payload, and runtime state; it no longer erases revision/state lineage. Explicit
`StartReload -> remove -> ready` therefore remains legal, while removal alone cannot authorize Error recovery.
`next_ready_revision` maps a zero baseline to 1 before applying the semantic-change rule.

Independent review then found two additional revision-authority defects. `importer_id` was omitted from that semantic
identity even though it is a public persisted field; importer-only changes could publish a new record while retaining
the old importer's loaded payload and authorizing stale payload CAS. The identity comparison now includes
`importer_id`. The same review found unchecked `u64` increment at the public persisted maximum. Revision advance now
uses `checked_add` and returns typed `ResourceRegistryError::RevisionExhausted`; preflight propagates the error before
any authority mutation, event publication, or generation replacement.

This follows the applicable reference boundaries rather than copying their APIs. Unreal
`AssetRegistryImpl.h:770-788` keeps `FEventContext` mutations inside the authority lock and broadcasts them only after
the lock is dropped; `AssetRegistry.cpp:10834-10924` preserves remove/non-remove ordering at publication. Bevy
`bevy_asset/src/assets.rs:383-396,730-744` distinguishes add/modify and rejects stale/removed slot generations. Zircon
keeps its stricter fixed `ResourceId` plus record-revision CAS, so the non-visible batch-local remove cannot mint a
fresh revision lineage.

The repair adds no loop, scan, sort, lock, compatibility branch, or persistent index. Lookup remains expected `O(1)`
per staged operation. `Option<ResourceRecord>` adds fixed inline storage to every `StagedResource`; cloning the removed
record's owned strings/vectors occurs only when the record immediately before removal differs from the already-retained
transaction-entry record. `RevisionExhausted` is one new typed public error outcome, not a compatibility alias. This is
a correctness repair, not a latency, frame-time, power, or cross-engine performance claim. The earlier timing and
allocation profiles remain ordering-optimization evidence only; the current correctness snapshot has not been
re-profiled and is not described as exact-profile green.

### Evidence

Five focused regressions cover the missing contracts:

- `remove_then_ready_readd_preserves_revision_cas_lineage` proves the final revision advances, the event carries that
  revision, and the stale pre-batch revision cannot install an old payload;
- `explicit_reload_transition_survives_remove_and_readd` proves an explicit reload transition remains authorized and
  the first Ready record advances from revision 0 to 1;
- the extended importer-recovery regression proves `Error -> remove -> ordinary ready` fails atomically;
- `importer_identity_change_advances_revision_and_invalidates_old_payload` proves an importer-only update unloads the
  old payload, advances revision, and rejects the stale payload CAS;
- `ready_revision_exhaustion_rejects_the_entire_batch` proves a changed `u64::MAX` Ready record returns the typed
  exhaustion error without registry, generation, or event side effects.

The importer recovery regression leaves the Error record intact. Managed job
`4c58d6addbf6442f976dae46422c58f2` ran
`cargo test -p zr_resource --locked --lib readd` from 02:08:48 to 02:10:48 Asia/Shanghai against the current shared
source, exited 0, released at 02:10:57, and left no live process. Its target was
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
The same coordinator-built executable reported `3 passed / 0 failed / 165 filtered out` for `readd`; a direct exact
diagnostic run of the updated importer recovery test reported `1 passed / 0 failed / 167 filtered out`. Compiler
warnings were outside the three touched Rust owners; no new warning was introduced by this follow-up.

Current Rust SHA-256 values after the independent-review repairs are:

- `error.rs`: `99a2f12b1859d3744de4fde227ec7205bc673940347fee0a6f14252682bb69b0`;
- `manager/commit.rs`: `a41250387db16276837c94a4047bb988687a2c77a44a48cf1cbc78a38d9d489e`;
- `manager/revision.rs`: `a9c4f13a022a80c10b9ffe176c9e3e2562001f1ae59f4f6ceeed0f1bc2b49ac2`;
- `manager/tests/transaction.rs`: `24721c16b2e7fc5602fbce613cffbdaded116e29b8e9562c0424a11fb1e1d904`.

Rust 2021 format is green on those exact files. The first managed attempt for the repaired snapshot was rejected before
Cargo with `cargo_reuse_pool_busy`; it is neither a passing nor a failing test result. Independent re-review then
matched the four manager hashes above plus atomic transaction, atomic test, and architecture-guard hashes and returned
`C0 / I0 / M0`, Ready for managed Cargo validation. It confirmed that `importer_id` is part of Ready identity and that
`checked_add` exhaustion propagates before locator staging or authority apply. The reviewer independently ran the
15-case architecture guard and exact Rust 2021 format, both green, without starting Cargo or writing files.

The public `core/resource.md` contract was transferred from archived r8 to this session with preview fingerprint
`e50d3ed1b99ebad7bf331af74853dda9f56b5e16051c3b1806391628217bab1a` and apply request
`d05b3d9ba0e54d4f8b0a98b26a200d89`. It now states that an intermediate remove/re-add in one batch preserves the last
staged revision/state baseline, while a committed removal followed by a later independent add remains non-global, and
documents the typed exhaustion outcome.

Requests `ae6177392b9b41d6a666b9518bbef145` and `2f06dda7ce9248bcb4d69fa8d208d2a4` document coordinator
acquire contention and response-timeout recovery; neither started Cargo. The empty leased job created by the second
request was released only after the bound wrapper PID had naturally exited and the coordinator reported no live
processes. A fresh no-PID lease then ran through the coordinator-owned runner:

- job `7f6e2375e1064fc78cae255540ac4e2e`, run `cd33a5f667874bdca61bbeae9d50ec97`, exit 0;
- command `cargo test -p zr_resource --locked --lib
  concurrent_expected_missing_transaction_publication_has_exactly_one_winner`;
- compile 3m15s; managed behavior `1 passed / 0 failed / 169 filtered` in 0.22s;
- D-drive test executable SHA-256
  `8800bbbc17595aa13c4948d9dfa4bb400a13dfa3cb2f943d199ecec6ec7e2ea0`;
- immutable coordinator stdout/stderr hashes
  `3abb43d380415c55b353f415275b066b57fd07aa19758505a4d56f045c8a936a` /
  `d3ff3e089ceed2aa4a91002afe446298c167207f5b746e0c2672b9b5369ef53e`.

The exact executable then passed direct diagnostics for re-add (`3/0/167`), importer identity (`1/0/169`), revision
exhaustion (`1/0/169`), and the full library (`167 passed / 0 failed / 3 ignored`) in 2.26 seconds. Those direct runs
were diagnostic evidence and were superseded by the final managed full-library receipt below. The ignored cases are
the existing managed-release performance gates. The first managed compile reported existing current-source warnings
in `zircon_runtime_interface` and eight `zr_resource` crate-private/test projections, which triggered the physical
boundary hard cut below.

This follow-up is `source_repaired / independent_review_green / public_contract_documented /
current_source_managed_full_library_green / manager_behavior_managed_green`. It is not a Frameworks01 milestone
closeout by itself; the larger physical-cut, Failure-graph, exact attribution, and coordinator milestone gates remain.

## 2026-08-29 Physical Crate Warning Boundary Hard Cut

### Pre-edit owner review and locked change

The first current-source managed compile exposed eight `zr_resource` warning groups. Whole-crate consumer review shows
that they are not eight missing public APIs. Real `zircon_runtime` consumers already enter through the hidden
`zr_resource::assembly` surface. The warnings instead identify physical-split residue left in old aggregation layers:

- `atomic_file/mod.rs` re-exported `StagedPublicationError` even though callers consume the typed return value without
  naming it;
- `io/mod.rs` retained four crate-root aliases used only by `assembly::io` through their concrete atomic-file owner;
- `lib.rs` retained five crate-root aliases whose real cross-crate publication now comes directly from `assembly`;
- `ResourceManagementGeneration::ordered_pages_arc` has no consumer in production or tests.

The locked hard cut removes only those unused aliases and the zero-consumer method. It preserves the hidden assembly
exports, the curated public `io::{atomic_write, atomic_write_new}` facade, every internal helper that still has a real
consumer, all public Resource DTOs and errors, and all transaction/manager behavior. No compatibility alias replaces
the deleted crate-private routes. Complexity, allocation, persistent data, lock ordering, and filesystem algorithms do
not change.

### Final evidence and state

The final warning-boundary owner hashes are:

- `io/atomic_file/mod.rs`: `88b80eaed0c5d576db7978b1f780e2a4ae79a56408f52c48f5754efe37edcd93`;
- `io/mod.rs`: `357d790398cb124f8a41423ca340f1c11157b8c8da73e2c2134a10d896972b1f`;
- `lib.rs`: `4347e84f07ebd59dfb6167b75a85f982ab80bf2fb3b046e0192b6ad7ede3212d`;
- `event_stream.rs`: `5ab275d50d501e48b82b6133ac5c8d95ba6e53440da1637e1acbd4b17e212280`;
- `management_generation.rs`: `333e8d82759576fd8a10dfa236fe184b8b4b9caf08c50e4d917eb2c7aa62bf79`.

The first independent read-only review matched the original four-file hard cut before and after review and returned
`C0 / I0 / M0`, Ready for managed Cargo. It independently passed the Resource crate-boundary guard `5/5` and exact
Rust 2021 format, found zero product consumers of the retired root/io aliases or `ordered_pages_arc`, and confirmed
that real Runtime consumers use the hidden `assembly` projection while the public `io` facade remains curated.

Windows coordinator job `8b31357dce8a42f89e0c51212d3774fc` / run
`1cde4a6add6542b7916f6dc2bb16a5af` executed that test-target snapshot through the D-drive compatible pool from 03:40:38 to
03:41:39 Asia/Shanghai. `cargo test -p zr_resource --locked --lib` completed with
`167 passed / 0 failed / 3 ignored` in 3.27 seconds after a 53.73-second build. The job released with exit 0 and no
live process. That test executable is 5,412,352 bytes with SHA-256
`d901c0e372ecfa7b1c1714b402e3e224b335176a5121db0cbfea60969242ea14`; immutable stdout/stderr hashes are
`f41648a67514424b8086a441d6349ec83e0b2305c6cf4fa101a4d2fec64df90c` /
`43ea5f6b4a03a346e668d0d9ff18c136bdecc231940ccd39fa79147830075a78`.

That test-target compiler output contained zero `zr_resource` warning summaries, but the later Editor upward build
correctly compiled the normal Resource library and exposed three production-only warning groups. Whole-crate exact
consumer review showed that `ResourceEventLogEntries::{is_empty, values}`, `ResourceManagementRow::from_record`, and
`ResourceManagementGeneration::{from_rows, from_sorted_rows}` are used only by `#[cfg(test)]` modules. Production
continues through indexed event operations, `from_record_reusing_identity`, `from_parts`, and
`from_sorted_rows_with_hash_authority`; no test-support feature or cross-crate consumer uses the five helpers. The
hard cut therefore marks those five helpers `#[cfg(test)]` instead of retaining warning-only production surfaces.

Independent follow-up review matched the final `event_stream.rs` and `management_generation.rs` hashes above and
returned `C0 / I0 / M0`, Ready for managed Cargo. It rechecked all five consumers and their production replacements,
passed exact Rust 2021 format and the crate-boundary guard `5/5`, and performed no writes or Cargo.

The final normal-library receipt is coordinator check job `a86b9a1cc126443db28241b511870863` / run
`791f01d779254d56b153377a87b6d79a`. It ran `cargo check -p zr_resource --locked --lib` from 14:44:22 to 14:46:56
Asia/Shanghai, exited 0 after 2m31s, released with no live process, and emitted zero `zr_resource` warnings. Its
immutable stdout/stderr hashes are
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` /
`282b989fc798470bc169f042b8146438353f42c1cb6e3715e45c5bb09be771cb`.

After the requested shared-Cargo quiet window, final test job `1dcb8301b7b047d58f246c823fe6b42d` / run
`0b8fc25d8a52467d9c7013e24ace3972` ran the complete `zr_resource --lib` suite from 14:52:27 to 14:54:42
Asia/Shanghai. It exited 0, released with no live process, and returned `167 passed / 0 failed / 3 ignored` in 2.69
seconds after a 2m10s test-profile build. The 5,412,352-byte executable SHA-256 is
`86e85e0fe4f32107dac75cb119fb0547620a23ad9058cce45a08062576a91f8d`; immutable stdout/stderr hashes are
`b5e8bba492d79814147c1555394f9d1ca96ed16fb78094d79d806442ab2f1714` /
`161a40f0ae9e19fb3e66d05e36b4025514770853200b353808ee330d2e968f37`. Both final production and test targets
contain zero `zr_resource` warning summaries. Eleven detailed warnings and one summary remain in the foreign shared
dependency `zircon_runtime_interface`; this slice neither owns nor hides them.

Editor upward job `41919258d25041c39d9c74d57d4c989f` / run `b6657885fc6944009fcf4ad2d43dd75a`
did not reach any Editor authority test. It stopped on the single foreign current-source E0004 in
`zircon_runtime_host/src/foreign_output/item_count.rs:80`, where the match does not cover
`WorldQueryResult::TransformSnapshot`. The foreign owner hashes at diagnosis are
`2782a5a5a3533762b0a1d435b2b6faf1cd6b6dfce4b353901f0eb39aa1c4bf27` for `item_count.rs` and
`56f357dddd79e119ca894b195966658025731b67bc102377a386fda62a7aaa47` for
`zircon_runtime_interface/src/world_sync/query.rs`; this slice does not edit either owner. The full 15-case
conditional-write architecture guard passed again in 14.540 seconds, the final Resource crate-boundary guard passed
`5/5` in 9.128 seconds, and exact Rust 2021 format remains green. State is
`owner_review_complete / hard_cut_implemented / independent_review_green / production_check_green /
managed_full_library_green / zr_resource_warning_free / editor_upward_blocked_by_foreign_current_source /
full_m1_acceptance_pending`.

# M1 Resource Generation Object Identity Consumer Preflight

Date: 2026-08-31  
Plan: Frameworks 01  
Session: `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`  
Status: `resource_support_implemented / static_green / managed_build_green /
full_test_rerun_pending /
consumer_migrations_pending`

## Objective

Hard-cut Resource projection correctness away from wrapping numeric generations and revisions. A
published immutable object, and the immutable row objects it retains, are the in-process identity
authority. Numeric counters and fingerprints may describe or accelerate a projection, but they must
not decide cache reuse, ticket reuse, stale-result admission, or cross-module publication.

This preflight is required before I1 code changes. It records the whole current consumer union,
selects the object-identity contract, defines the paired publication boundary, and fixes ownership
gates. No source file listed below was edited by this preflight.

## Admission and evidence rules

- The exact current-source ResourceManagement/readiness release profile must run before production
  changes. The canonical managed-sccache Failure remains open until the unchanged origin command
  produces profile artifacts.
- The same workloads must run after the hard cut. Report p50, p95, MAD, allocation count, allocated
  bytes, peak live bytes, affected-closure count, edge visits, and exact source hashes.
- External RSS, I/O and power are separate measurements. Modeled `Arc` operation counts cannot be
  reported as elapsed-time, power, or bottleneck-elimination evidence.
- Cross-crate owners must claim or transfer their exact paths before mutation. Frameworks 01 does
  not absorb stale Asset, Render, resource-streamer, or Editor blobs into one mixed commit.
- There is no numeric compatibility overload, `unwrap_or(0)` identity fallback, reset API, or larger
  wrapping integer in the terminal contract.

## Current-source defect map

### Resource projection authority

`ResourceManagementGeneration` is already published as `Arc` and structurally shares immutable
pages and rows. However, its public `sequence: u64` is advanced with `wrapping_add(1)` in both sparse
and rebuilt publication paths. `ResourceManagementPage` copies that scalar into `generation`.
`ResourceMutationReceipt` copies management and readiness scalar generations even though the exact
published snapshots are available under the Resource authority write lock.

`ResourceReadinessGeneration` is also an immutable `Arc` graph. Unchanged readiness rows preserve
the same `Arc<ResourceReadinessRow>`, which is the correct per-resource semantic identity. The
implementation nevertheless advances both the global sequence and the per-row dependency revision
with `wrapping_add(1)`. A repeated finite value can therefore admit stale cache entries.

Current hashes:

- `management_generation.rs` `a0ad5609c863fbd75ef9d697c701f36976a61560e99ea6960a016eaf087ad4ba`;
- `readiness_generation.rs` `4830e32d4b730ee5ad4f435c1239a3bdc6d397250b3dfa6c0011880cda0045f8`;
- `manager/management_projection.rs` `c543c0fe41cf2c3f8ad1d59da2d1ec648354c071f5665f4a3139a12e4dd98fdc`;
- `manager/readiness_projection.rs` `9aac00f12030fff53fef7a23e388579c2b8a0099e9787cf300a4bfe4d8c525c9`;
- `mutation/receipt.rs` `b078e4b48be76d66c5703a34a7f3b911095f1b309a028fa183a2b06c8df44104`.

### Paired snapshot atomicity

`ResourceManager::management_generation()` and `readiness_generation()` each take and release the
authority read lock independently. A consumer that calls both can combine management from commit N
with readiness from commit N+1. Render residency currently accepts those two references as though
they form one source state.

The hard cut introduces a `ResourceProjectionSnapshot` captured by one authority read lock. It
contains the exact management and readiness `Arc` handles from one authority state. Every consumer
whose decision depends on both projections must accept this paired snapshot, not two independently
sampled arguments. The mutation receipt captures the same pair under the commit write lock.

### Asset projection consumers

`ProjectCatalogInputGeneration` is already an immutable `Arc` publication and reuses the exact Arc
when all catalog inputs are unchanged. Its sequence allocator is checked, but project-asset
aggregation discards the object identity and stores only `project_generation: Option<u64>` plus
`resource_generation: u64`. `is_for_generations` then compares those scalars.

The hard cut retains `Option<Arc<ProjectCatalogInputGeneration>>` and the Resource management
generation identity. Closed project remains `None`; an active project whose catalog happens to have
any repeated diagnostic value cannot compare equal to a different object.

Current hashes:

- `asset/project/catalog_input_generation.rs` `e844dd2af312b544953ed55baebbcefba3d5845de31e84062581e09055e98097`;
- `project_asset_manager/management.rs` `a73b259dfc5bbefe2dcba61953f6e6c688741cd4ac85c38acd00563a67f08e16`;
- `project_asset_manager/management_generation.rs` `de11ea80a5da5bd2a2b228b623ff502bf45dace593acd697a3ec24420acc055f`.

The management-generation file is currently unowned; `management.rs` is attributed to archived
Runtime51 work. Ownership matrix request `58e3a737ed704d04b3821ed2c80bb966` is the baseline. Asset must
establish one executable integration owner before editing. The wider project catalog CAS paths that
still compare `.sequence()` are an Asset lifecycle migration, not an excuse to retain a scalar
Resource compatibility path.

### Render residency consumers

`RenderAssetResidencyTicketSeed` and `RenderAssetResidencyTicket` are `Copy` values containing
`asset_revision`, `readiness_generation`, and `dependency_revision`. `seed.matches` uses those
finite values as the reuse authority. `resolve_ticket_seed` receives management/readiness as two
independent references and immediately erases both row identities.

The hard cut accepts one `ResourceProjectionSnapshot`, obtains exact management-row and
readiness-row identity handles for the resource, and retains those handles in the seed and ticket.
The ticket, release, pending and active state cease to be `Copy`; explicit cheap clones replace
implicit value copying. Reuse compares identity handles plus demand generation, device epoch, scope,
route, and the typed resource handle. Numeric revision fields may remain in diagnostics only.

Current hashes:

- `render_asset_residency/contract.rs` `bd7fbb201e4fc383f6d04efce1a7b6d1dc8a758e27fd3fa0f977c2ed333631a2`;
- `render_asset_residency/manager.rs` `8a155161c91a125654781be06111130750f94d5e9d93cd2308d05bd509e10b54`.

Ownership matrix request `93ab46083cd14a719965c186d5046f41` found the residency union added but
unattributed. Render must establish the owner before mutation.

### Shader and material streamer consumers

Shader preparation reads a single readiness snapshot but converts a row identity to
`dependency_revision`, returns that scalar through recursive preparation, stores it in
`PreparedShader`, and compares it in `shader_artifact_identity_is_current`. Material preparation
copies the same value into dependency snapshots and `PipelineKey`; missing rows become identity `0`.
That creates both rollover aliasing and a missing-vs-real-value alias.

The hard cut takes one readiness snapshot at traversal entry and threads opaque per-row identity
handles through shader imports, material dependency snapshots and local pipeline keys. Missing
identity is `None` and never a numeric sentinel. Identity handles implement process-local equality
and hashing from their retained Arc allocation, so local cache lookup remains O(1); they are not
serialized or used as durable asset IDs. Unchanged unrelated readiness rows retain their handles,
so a change in one shard does not invalidate every prepared shader.

Current hashes:

- `resource_streamer_ensure_shader_source.rs` `f8454b5037841964db18da48009b8f99de623e43d07ddd17c67845dc7fcb62a5`;
- `resource_streamer_ensure_material.rs` `1ee6278612debfa84fb83bbf81e4c5658d8660b5bb28d85516236455eae2c49b`.

Ownership matrix request `6dc4295739834cf28579a1772de81e49` found a mixed union: several owners are
attributed to active resolving-failure Session `01a019a5-b15f-7461-a1b0-ce4b6aa8e710`, while other
new files are unattributed. The full shader/material/pipeline key union must be enumerated and owned
before implementation. The separate model-cache `Arc<ModelAsset>` performance handoff is not part
of this identity slice.

### Editor workspace consumer

`AssetWorkspaceState` already retains `Arc<ResourceManagementGeneration>` and correctly uses
`Arc::ptr_eq` during resource synchronization. It then degrades the handle to `resource_sequence`
inside `AssetWorkspaceProjectionInput` and advances a projection generation with `wrapping_add(1)`.
The terminal Editor contract retains source identity handles in the input and publishes a new owned
projection identity when the output changes.

Current hash:

- `asset_workspace_state.rs` `63f97ee4ba3f299fac7796d2a99fa665ad2234df3ff044651a53e0e1466071fa`.

Ownership matrix request `8d8813976176471085c12ac28f9e40dd` found the prior Editor09 owner cancelled
with no live lease. Frameworks 01 must not recreate the earlier mixed-blob attribution. Editor09
must establish a legal current owner and integrate the whole workspace projection change.

## Selected Resource contract

### Opaque identities

The Resource crate publishes cloneable opaque identity values for:

1. the management generation;
2. the readiness generation;
3. a management row for one `ResourceId`;
4. a readiness row for one `ResourceId`.

Each identity retains the corresponding immutable Arc allocation. Equality and hashing are based on
the allocation while it is retained, not the pointed-to address after release, a payload hash, or a
numeric counter. Identities are in-process values: no serde, wire representation, persisted raw
pointer, or numeric conversion exists.

Whole-generation identities serve page provenance, receipts and global projections. Row identities
serve resource-local caches and tickets so unchanged rows remain reusable after unrelated graph
updates. Exposing a row identity does not expose readiness graph internals or mutable authority.

### Diagnostics are not identities

`sequence()` and `dependency_revision()` are removed from public correctness surfaces. If profiling
still requires publication/change counts, they move under explicitly named diagnostics and advance
without wrapping. Saturation is acceptable only for observation; no branch may use a saturated
count to admit, reject, reuse, order, or resume work. Dependency fingerprints remain private
accelerators and exact canonical dependency equality decides whether a row Arc is reused.

`ResourceManagementPage` carries the management-generation identity instead of `generation: u64`.
`ResourceMutationReceipt` carries the paired management/readiness identities captured after the
mutation under the authority write lock. Callers can prove that a page or receipt belongs to a
retained snapshot without reconstructing identity from a number.

### Complexity and memory

- generation and row identity clone/drop: O(1), one Arc reference operation;
- identity compare/hash: O(1);
- readiness invalidation remains O(affected closure + visited dependency edges);
- management sparse publication remains proportional to changed rows/shards;
- each retained ticket/cache key adds a constant number of pointer-sized handles;
- no global identity map, recycled slot allocator, manual ref-count, UUID allocation, or content
  rehash is introduced.

The plan does not claim that Arc traffic is faster than scalar copying. The correctness hard cut has
a constant memory cost that must be measured in the paired profile and residency/streamer benchmark.

## Reference-engine decision

Unreal Asset Registry collects an exact event context while holding its write authority and
broadcasts after unlock; its streamable/resource lifetime designs retain shared object handles.
Those patterns support immutable handle provenance plus lock-scoped capture. Bevy's strong handles
also demonstrate shared lifetime, but its recycled finite asset-index generation is not selected
because it reproduces the rollover class being removed.

Zircon keeps its existing COW projection topology. This slice changes identity transport and paired
capture, not page sizing, shard count, graph traversal, or cache eviction policy. Those algorithms
remain controlled by current-source profile evidence.

## RED and GREEN matrix

### Resource support RED

1. Force diagnostic publication/change counters to the same visible value for two distinct test
   generations; their generation and changed-row identities must remain different.
2. Publish an unrelated readiness-row change; an unchanged row identity must remain identical.
3. Change a dependency with otherwise equal metadata; the affected row identity must change.
4. Capture management/readiness around a paused commit; `projection_snapshot()` must return either
   the complete previous pair or complete next pair, never a mixed pair.
5. A mutation receipt and pages produced from its management snapshot must expose matching object
   identity without consulting a scalar sequence.
6. Static guards reject public `sequence()`/`dependency_revision()` correctness APIs,
   `wrapping_add`, numeric identity sentinels and compatibility overloads in the migrated union.

### Consumer GREEN

1. Project asset refresh reuses only when both retained catalog and Resource source handles are
   identical; closed and active-empty projects remain distinct.
2. Residency reuses a ticket for the same paired row identities and invalidates it when either
   relevant row object changes. An unrelated readiness update preserves the ticket.
3. Deliberately repeated diagnostic values cannot make a stale residency ticket current.
4. Shader import traversal and material pipeline lookup share unchanged per-row identities; missing
   readiness remains `None`; changing one dependency invalidates only its dependent closure.
5. Editor workspace input equality compares retained source identity; a new source object with a
   repeated diagnostic value still publishes a new projection identity.
6. No migrated ticket, seed, release or cache key relies on implicit `Copy` after it owns an identity
   handle.

## Execution order and ownership gates

1. Produce the exact current-source pre-change ResourceManagement/readiness profile artifacts.
2. Frameworks 01 implements only the Resource support contract and paired snapshot under its exact
   source leases. Run focused RED/GREEN and the full `zr_resource` suite.
3. Establish one legal Asset owner and hard-cut project-asset aggregation to retained source Arcs.
4. Establish one legal Render owner and migrate residency as one ticket/state/test union.
5. Coordinate the existing resource-streamer resolving-failure owner; migrate the complete
   shader/material/pipeline identity union without splitting a mixed blob.
6. Establish Editor09 ownership and migrate the asset-workspace projection input/output identity.
7. Run paired post-change Resource profile plus realistic residency and streamer cache benchmarks.
8. Obtain an independent cross-session review of correctness, ownership, measured memory cost and
   stale API absence before any milestone integration candidate.

## Current result

The whole I1 consumer architecture is now reviewed and locked. The unchanged pre-change
ResourceManagement profile subsequently completed under managed R9 job
`f2f3280096d64ca699bdd9c9e4800e97`. Its summary SHA-256 is
`1244bf20c9b30bf0fca4bd7f3bf850c7502a36c0f911823079718583923d05cd`; it contains 14 scenarios,
31 samples and 3 warmups per scenario.

The measured baseline sharpens the implementation target, but the whole-call-graph review rejects
one initial interpretation. `no_projected_change_100000` performs zero allocations at p50 51.1567
ms / p95 76.9357 ms because the harness deliberately replays 100,000 unchanged records directly
into private `ResourceManagementProjection::apply_delta`. The unique production caller filters
`before != record` before that call, so this row is a defensive misuse upper bound, not evidence of
a production no-change full-registry scan. `revision_100000_1` remains a valid sparse baseline at
p50 14.3 us / p95 17.2 us; `initial_build_100000` remains a valid construction baseline at p50
420.7386 ms with 410,112 allocations and 46,859,944 requested bytes. No page/shard/threshold change
is authorized from the synthetic row. A later optimization must start from transaction/commit or a
product trace. RSS/power remain unavailable and no speedup is claimed.

Frameworks01 has now implemented the owned Resource support layer. Management/readiness generations
and rows publish opaque retained `Arc` identities, pages and mutation receipts retain generation
identity, and `ResourceProjectionSnapshot` captures the exact management/readiness pair under one
authority lock. Public scalar `sequence()` / `dependency_revision()` correctness APIs and wrapping
identity advancement are gone; explicitly diagnostic counters saturate and are not used for cache
admission. Current source SHA-256 includes:

- `management_generation.rs`
  `016e75cff5ca999e076a0323b61310de5497fcd15b13c486bf7083432dcb24a5`;
- `readiness_generation.rs`
  `8441fc693344202651576d4828715f595ca99d370594f6cf26544fc01b3e762d`;
- `manager/resource_manager.rs`
  `f63c6a7fb927c7d911f6229a0021cff52040671841293737d0a3f658afc805da`;
- `mutation/receipt.rs`
  `2e1581e24fdb9f3a113696fdf9aad06dbaaaf9f950507870b707cd83316f27c4`.

Frameworks01 resource boundary guards, including object-identity, paired-snapshot and durable-I/O
contracts, are GREEN `14/14`. The added guard locks the production commit boundary to pass only
`before != record` deltas into management projection; the static owner SHA-256 is
`9EF5DB611439310E60320957B6754A6C9D4B44171C2C82095EAA6453EF5E503F`. The earlier
RuntimeInterface03 Clone-contract blocker is fixed by its exact owner and the current managed
validation now reaches `zr_resource`.

Managed job `a31f2a72cba34ced8b5dce40854359de` produced five owned compile diagnostics. The fixes keep
journal frame encoding transaction-private, use the hard-cut `ResourceScheme::Res` fallible locator
fixture, and remove a redundant mutable reborrow. Follow-up job
`6486a2ea6b664b2ba0130ab61193090b` is released exit `1`, but its production build is GREEN and it
linked current test binary SHA-256
`D22BC22C04975AF9FBFA96421F4D559A758FABB40DEC77637C643D96311BCAEE`. Direct execution reported
217 passed / 1 failed / 11 ignored across 229 tests; the sole failure was a source-contract test
still matching the old two-argument `apply_staged` call after event admission added the exact event
count. That assertion and one test-only must-use warning are fixed. A later rebuilt full run exposed
an I2 blocking-receiver lifetime deadlock before reaching the rest of the suite; the owned fix is
implemented and awaits focused/full Cargo validation behind foreign job
`42419d28b8254edf816b6b125bfa3eeb`. GREEN is not claimed yet.

The Asset/Render/resource-streamer/Editor consumer union remains cross-plan work and is not absorbed
into this support slice. The unchanged pre-cut profile is recorded above; the identical post-cut
profile, managed full-suite rerun, consumer integration, RSS/power evidence and independent review
remain pending. M1 stays `source infrastructure materially advanced / milestone_not_accepted`; no
coordinator milestone commit or WeCom completion message is authorized.

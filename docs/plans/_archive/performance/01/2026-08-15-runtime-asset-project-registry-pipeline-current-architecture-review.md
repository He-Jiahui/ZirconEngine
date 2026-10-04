---
related_code:
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/registry
  - zircon_runtime/src/asset/pipeline
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/StreamableManager.cpp
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/bevy/crates/bevy_asset/src/loader.rs
tests:
  - 118 of 118 current Rust files reconciled and reviewed
  - 16268 physical lines and 71 inline tests
  - path plus physical-line-count plus per-file SHA-256 manifest fingerprint 6a4e8be301be542dc596861191ce4287af4527f24f66340f7d10ed6cbd03de75
  - managed current-source Cargo and product WPR/xperf/RenderDoc/energy remain blocked by the non-runnable editor baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-15
---

# Runtime asset project/registry/pipeline current architecture review (2026-08-15)

## Scope freeze and method

This review freezes the current adjacent runtime asset vertical at **118/118 Rust files, 16,268
physical lines and 71 inline tests**. The manifest fingerprint is
`6a4e8be301be542dc596861191ce4287af4527f24f66340f7d10ed6cbd03de75`; it is SHA-256 over sorted
`path|physical-lines|file-sha256` rows joined with LF.

| Current module | Files | Physical lines | Tests | Static verdict |
|---|---:|---:|---:|---|
| `asset/project/**` | 41 | 7,150 | 28 | correct transaction and observation primitives exist, but project state remains a deep-cloned mutable aggregate and multi-change watch batches fall back to complete reconciliation |
| `asset/registry/**` | 16 | 1,861 | 3 | targeted APIs exist, but they clone whole indexes, scan broad reverse maps and serialize the complete registry for a single source mutation |
| `asset/pipeline/**` | 61 | 7,257 | 40 | watch ingress and worker-pool bounds are locally useful, but product residency is synchronous, the worker pool is not integrated into loading, and publication remains broad/unbounded |

Every current `.rs` file in the three roots was assigned to a module/function matrix, read and
reconciled against current production callers and inline tests. Modified and untracked files were
read as current source rather than compared only to HEAD. The roots contain extensive foreign
uncommitted work, so this pass made no source edit. DTO-only, error-only and test-support files were
still reviewed; they are grouped at folder level here to keep the acceptance index concise.

The approved-root defect in `tools/build-editor.ps1:130` still rejects valid D/E/F output roots
before Cargo. The current Pester result is 9 pass/6 fail from 15 tests. The latest managed editor and
focused runtime attempts therefore did not produce a current product executable. WPR 10.0.26100.8972,
xperf 10.0.26100.4188 and RenderDoc 1.44 are available, but running them against a stale executable
would not validate this source. Dynamic timings, power and GPU effects are explicitly
`not_measured`; these modules must remain outside `review.md`.

## Architecture verdict

The P0 defect is duplicated mutable authority, not an isolated slow loop. `ProjectManager` owns
paths, manifest, resource registry, asset indexes, package registry, importer/artifact state,
dependency state and catalog generation as one clonable value. Watch, explicit import, editor
queries and runtime loading repeatedly copy or scan that aggregate, prepare broad state off to the
side, then publish through channels that are not consistently bounded or outside locks. A two-file
save storm takes a qualitatively different full-project path from a one-file change.

The required hard-cut chain is:

`WatchIngressDelta -> RuntimeAssetMutationBatch -> Background Import/Dependency DAG ->
RegistryJournal + ArtifactManifest -> ImmutableAssetGeneration -> ResourceReadinessGeneration ->
Bounded ConsumerDelta`

`RuntimeAssetGenerationStore` is the only project/registry publication authority. Preparation may
run in parallel by source/dependency key. Commit is a short ordered generation check and pointer/slot
swap. Runtime and editor consumers hold immutable generation handles and resource leases; neither
receives a deep-cloned `ProjectManager`. Full reconciliation remains an explicit recovery operation
for overflow, lost events, root/schema changes or user-requested rebuild, not the normal batch path.

## Existing structures to preserve

The hard cut should retain current useful mechanisms rather than erase them indiscriminately:

- watch ingress caps raw entries, bytes and errors at 4,096 entries, 4 MiB and 64 errors, coalesces
  changes and marks overflow for reconciliation;
- durable file transactions stage and journal writes before project/resource publication;
- `ProjectGenerationObservation` already separates discovery, metadata, import, dependency,
  registry, serialization, resource, file-commit, install, apply, publish and recovery phases and
  records work counters;
- resource mutation batches provide an atomic prepare/commit boundary and incremental publication
  can update exact records;
- catalog records and shards already use `Arc`, and direct-predecessor catalog deltas are affected-only;
- the asset worker pool has single-flight payload sharing, bounded queue/waiter/completion count,
  retained-byte and age controls, cancellation, timers and non-blocking `Drop`.

These are valuable local primitives. They do not by themselves establish one immutable authority,
delta-proportional mutation, asynchronous product loading or frame-budgeted delivery.

## P0 current-source findings

### 1. `ProjectManager` value snapshots copy the complete mutable authority

`asset/project/manager/mod.rs:31-42` derives `Clone` for an aggregate containing the manifest,
`ResourceRegistry`, `AssetRegistryIndex`, package registry, importer, artifact store, shader
dependency index, catalog generation and task pool. `project_asset_manager/construction.rs:160-161`
returns `self.project_read().clone()`, and
`service_contracts/asset_manager_contract.rs:61-62` exposes that as
`current_project_snapshot`. Editor refresh, project/navigation/layout operations, runtime render,
scene, text and dynamic API callers consume this value-returning authority.

The watch path repeats the same design. `project_asset_manager/runtime.rs:416-442` captures broad
source records and clones the active project for every attempt; a superseded generation can repeat
the complete preparation up to the retry limit. `scan_and_import.rs:96-105` has another public-style
clone/prepare/replace transaction.

The hard cut is an immutable `Arc<RuntimeAssetGeneration>` plus narrow owner operations such as
`submit_source_delta`, `query_index` and `acquire_resource`. Delete `current_project_manager`,
`current_project_snapshot` and clone/replace mutation entry points in the replacing milestone. A
compatibility alias would preserve the algorithmic defect.

### 2. The normal watch algorithm is incremental only for exactly one non-rename event

`scan_and_import.rs:108-118` returns incremental mode only for a one-element slice containing
Added/Modified/Removed with no previous URI. `:181-182` declares rename unreachable because it must
enter complete reconciliation. Therefore two coalesced writes, a source plus metadata write, or a
rename take the full generation path even when every affected source is known.

`project_asset_manager/runtime.rs:403-447` holds one global `watch_refresh_gate` around all attempts,
clone, scan/import and later commit. Different source keys cannot prepare concurrently. The gate also
turns a retry on one generation into repeated work behind all later batches.

Replace this with a first-class `RuntimeAssetMutationBatch` containing add/modify/remove/rename
operations. Normalize and de-duplicate by canonical source key, form dependency-connected
components, then run independent components through the shared task DAG. Reconciliation is a
separate mode selected only by explicit loss-of-truth conditions.

### 3. One targeted source update still multiplies total-project work

`scan_and_import/targeted.rs:130-314` reads and hashes the source, loads metadata and imports it, then
clones the full `AssetRegistryIndex` through source-replacement preparation. It clones the resource
registry through `ResourceRegistry::begin_staging` (`core/resource/registry.rs:56-72`), including a
second identity map, and calls full registry persistence at `targeted.rs:283`. Targeted removal has
the same staging/persistence shape at `scan_and_import.rs:187-247`.

The registry's targeted removal/replacement clones the complete index
(`asset/registry/targeted.rs:11-74`). `AssetRegistryIndex` owns multiple maps/sets and diagnostics;
remove/update retains across broad referencer maps at `asset_registry_index.rs:138-208`.
Dependency-owner refresh de-duplicates with `Vec::contains` and filters the full diagnostic set.
Targeted import then searches diagnostics again per affected owner. `registry/persistence.rs:63-80`
collects, clones, sorts and pretty-serializes all entries on every targeted write.

The result is not K-proportional even when K=1. The target must stage a compact transaction log:
changed source records, index key diffs, dependency-edge diffs, artifact manifest records and
resource-slot changes. Append and fsync the transaction, publish the immutable generation, then
checkpoint/compact outside the interactive path. Full sorted JSON is an export/checkpoint operation,
not the commit record for one hot-reload mutation.

`asset/registry/incremental.rs` is a second legacy staged-registry implementation reached only by
tests/current public surface; it clones the registry, scans metadata, rebuilds dependency edges and
persists broadly. Delete it when the transaction owner lands instead of maintaining two algorithms.

### 4. Product resource loading is synchronous and returns deep payload clones

`project_asset_manager/loading/ensure_resident.rs:11-103` takes a per-ID stripe, prepares an artifact
read under the generation, then performs the artifact read synchronously before publishing the
payload. Graphics/render/scene/editor callers that use `load_*_asset` can therefore pay file I/O and
decode on their calling thread. `loading/load_typed.rs:22-25` then clones the complete typed asset;
the existing acquire path can return a `ResourceLease` without that clone.

`ProjectAssetManager::spawn_worker_pool_with_frame_sampler` at `construction.rs:57-69` constructs a
standalone pool, but production callers do not issue `AssetWorkerPool::request`; the manager stores
only the underlying task pool. Worker-pool bounds and tests therefore do not prove asynchronous
product residency.

`load_imported_asset.rs:42-75` also uses error-driven trial loads for coarse `AssetKind` values:
texture/icon and several UI payload families are tried in sequence. Runtime record identity needs a
stable payload type/decoder ID so one lookup selects one decoder without failed load chains.

Integrate residency into the shared Runtime11 task service. Request returns a typed generational
handle/ticket, duplicate keys join one flight, I/O/decode runs off frame threads, and completion
installs a ready slot under count/source/decoded/resident-byte and age budgets. Sync loading is an
explicit boot/commandlet policy only. Hard-cut steady-state `load_typed -> T` consumers to leases or
handles; do not preserve clone-returning aliases.

### 5. Publication invokes arbitrary work under locks and uses unbounded streams

`project_asset_manager/runtime.rs:232-245` holds the subscriber mutex while cloning every change,
sending to every subscriber and invoking wake callbacks. `subscribe_asset_changes_internal` at
`:277-284` creates an unbounded channel. `publish_project_generation` at `:287-295` broadcasts and
wakes before dropping the generation write guard. Watch-error publication also sends while holding
its subscriber mutex at `:298-304`, and its service subscription is unbounded.

This combines reentrancy/deadlock risk with unbounded memory and long generation fences. Publication
must snapshot bounded subscriber slots under a short lock, release both subscriber and generation
locks, then enqueue/coalesce and schedule callbacks. Every stream declares entry/byte/age policy,
sequence/gap semantics and overflow recovery. Consumers receive one shared immutable delta payload,
not one deep clone per subscriber.

### 6. Watch publication discards already prepared ready payloads

The explicit targeted-import resource path can take ready payloads, but
`project_asset_manager/runtime.rs:460-466` passes only updated records to
`prepare_incremental_project_resource_sync`. `resource_publication.rs:94-188` publishes those records
with `upsert_lazy`. Thus a successful watch import can retain prepared payloads until commit, discard
them with the candidate transaction, mark the resource lazy and make the next consumer re-read the
artifact synchronously.

The mutation transaction must carry the same decoded/ready payload identity through artifact commit,
project generation and resource publication. Whether the payload remains resident is then a
budget/priority decision, not an accidental difference between explicit import and watcher paths.

## P1 structural findings

### 7. Project activation and reconciliation serialize the import pipeline

`scan_and_import/full_generation.rs:80-378` records useful phases but processes discovered sources
through one sequential loop at `:129`. Discovery, metadata projection, hashing, artifact restore,
import, artifact preparation, dependency projection, full registry/catalog build and persistence
form one synchronous preparation. Cached restore reads the root artifact and
`restore_imported_artifact` reads every artifact URI again to validate availability
(`:154-170,381-428`), so unchanged activation can duplicate cached reads.

`ProjectAssetManager::open_prepared_project` completes full generation and full resource sync before
installation/publication, blocking its caller. Keep the atomic install semantics, but represent
discovery/hash/import/dependency work as bounded DAG tasks. Open can publish a valid project shell and
progressive immutable catalog/readiness generations, or use an explicit blocking boot policy; editor
interactive paths must never silently become the blocking variant.

### 8. Fixed catalog sharding reduces constants but not mutation complexity

`catalog_input_generation.rs:14-16` fixes the catalog at 64 hash-map shards. Targeted publication
clones 64 `Arc` pointers, then `Arc::make_mut` clones the whole touched shard. One record update is
therefore approximately O(N/64), still O(N). Direct-predecessor `delta_since` is affected-only, but a
skipped generation requires broad comparison. Delta sorting at `:517-527` also materializes locator
Strings.

Do not tune the shard count as the architectural fix. Use an append-only typed delta/journal plus an
immutable generation representation whose update complexity is measured. A persistent map, stable
slot arena plus immutable index generations, or chunked copy-on-write structure is acceptable only
after 1k/100k scale and read-locality measurements. Checkpoint/compaction bounds historical growth.

### 9. The unused worker pool still scans all completion entries on frequent operations

`worker_pool/completion.rs:333-366` scans both in-flight and completed maps and clones expired keys.
It runs on request (`worker_pool.rs:117`), cancel (`:233`), completion (`completion.rs:231`) and
maintenance (`:423`) despite each entry already owning an exact `TaskTimer`. Under a Q-request storm
this can approach O(Q^2) registry visits. Retained completion budgets cover payload bytes, but not
source buffers, decoded working set or executing-task memory.

This is secondary until the pool is integrated. Runtime11 should make timer callbacks remove exact
generation-safe keys, use lazy heap/wheel cleanup only if measurements require it, and account queued,
source, decoded, resident and completion bytes separately.

### 10. Query APIs expose full scans despite owning accelerator state

`AssetRegistryIndex` maintains path and dependency indexes, but `registry/query.rs` kind/tag/general
queries scan and sort broad entry sets. `AssetManagerContract::list_assets` and current-project URI
helpers also collect/sort complete views per call. Existing management projection work repeats typed
loads and record conversion.

The immutable generation owner needs UUID, locator, payload type, folder/package, dependency and
reverse-dependency accelerators plus paged/stable read models. Consumers query a generation or
subscribe to typed diffs; they do not rebuild a management view per poll. Add indexes only for
measured product queries, not speculatively for every possible tag.

## Lower-priority observations

- project path resolution has a necessary canonicalization/security boundary for external inputs;
  the existing virtual-URI report already limits any lexical fast path to scanner-owned root
  provenance and measurement, so path security is not traded for a speculative speedup;
- registry rebuild, full scan and sorted serialization remain valid for cold recovery/export. Their
  defect is use in common targeted/watch transactions, not their existence;
- builtin generation is normally one-time, but eviction lookup rebuilds the complete builtin vector
  and shader source is duplicated. Replace it with a static immutable ID table after the residency
  owner exists; this is not a reason to delay P0 work;
- locator String allocation in sort keys and small-vector de-duplication are real local costs, but
  optimizing them first would preserve whole-authority clone, synchronous I/O and full persistence.

## Reference-engine evidence and transferable rules

### Unreal Asset Registry: incremental accelerators and outside-lock events

- `AssetRegistryState.cpp:3445-3484` adds one asset while updating package, path, class and tag
  accelerators. `:3631-3810` updates only fields/indexes affected by the changed record.
- `AssetRegistryState.cpp:4040-4096` groups batch removal by accelerator and only uses parallel tasks
  at 100 or more keys, avoiding task overhead for small batches.
- `AssetRegistry.cpp:102-118` explicitly forbids arbitrary events/callbacks under the registry lock
  because they can re-enter and deadlock. `FEventContext` defers them outside the lock.
- `AssetRegistry.cpp:190-224` defines a per-frame registry processing allowance and a separate
  background limit. `:4803-4941` time-limits game-thread work and broadcasts deferred events after
  guarded processing.
- `AssetDataGatherer.cpp:4264-4295` delays cache saves across rapid additional mounts instead of
  rewriting the complete cache for every change. `:4300-4405` runs a low-priority, interruptible
  gather loop; `:4459-4473` supports background registry tick and `:4701+` stages local batches.

The transferable architecture is incremental indexed state, staged background gathering, short
ordered apply, deferred outside-lock notification and deliberate full checkpointing. Zircon should
not copy Unreal's containers or thresholds; it must measure its own task overhead and workload.

### Unreal Streamable Manager: single-flight async residency and callback budget

`StreamableManager.cpp:1931-1966` reuses existing loaded/outstanding streamable state. The normal path
queues an async load with priority at `:2035-2081`; synchronous behavior is reserved for explicit
initial/forced contexts. `:41-54,540-558` makes completion/cancel callback processing time-sliceable
with an explicit game-thread time limit.

This directly contradicts allowing ordinary `load_*_asset` calls to perform unknown file I/O/decode
on their caller. Zircon additionally needs byte/working-set budgets and generation cancellation,
because its payload shapes and shared Rust task pools differ from Unreal.

### Bevy cross-check: shared server handle, typed events and non-blocking load

Bevy is secondary evidence, not the architectural authority. Its `AssetServer` clone shares one
`Arc<AssetServerData>` (`bevy_asset/src/server/mod.rs:61-80`) rather than deep-cloning the asset
authority. `load` returns an existing/new strong typed handle and explicitly does not block or spawn
duplicate work for an already loaded path (`:320-365`). `AssetEvent<A>` carries a typed ID and
added/modified/removed/unused/loaded-with-dependencies state (`event.rs:47-96`). `LoadContext`
tracks direct dependencies and subassets (`loader.rs:374-417`) and exposes a parallel labeled-asset
construction path (`:421-450`).

The useful cross-check is shared authority plus typed handles/events/dependency context. Zircon still
needs stronger transaction durability, bounded publication and editor/runtime generation contracts.

## Required hard-cut architecture

### A. Runtime04 owns one asset generation store

`RuntimeAssetGenerationStore` owns project identity, source index, artifact manifest, catalog/query
indexes, dependency graph and publication sequence. Readers receive `Arc<RuntimeAssetGeneration>` or
stable generational slots. Mutations are journaled batches with exact add/modify/remove/rename and
edge/index diffs. Checkpoints are compacted asynchronously. No full aggregate value is cloneable.

### B. Runtime11 owns one keyed import/residency DAG

Keys include project generation, source/resource stable ID, source revision, importer/decoder type,
dependency fingerprint and output variant. Duplicate import/load requests join one flight. Admission
tracks count, priority, affinity, deadline, queued/source/decoded/resident/completion bytes and age.
Preparation never runs on the frame/UI thread; main-thread completion is a bounded slot swap plus
typed invalidation.

### C. Resource publication preserves ready identity

Artifact commit, registry generation and resource readiness are one ordered transaction. A prepared
payload may be retained or evicted according to the working-set policy, but watcher and explicit
import use the same path and publish the same type identity. Consumers acquire leases/handles; they
do not clone large payloads or trial several decoders.

### D. Consumer delivery is bounded and outside locks

One immutable delta payload is shared by subscribers. Each subscriber declares count/byte/age and
coalescing/gap policy. Subscriber snapshots, generation release and callbacks have a strict order:
capture short state, release locks/fence, enqueue, then schedule callbacks. Slow/reentrant consumers
cannot extend registry mutation or block unrelated subscribers.

### E. Editor consumes the same generation chain

This report is the runtime predecessor to PERF-MVP-637:

`RuntimeAssetSourceDelta -> EditorAssetIndexDelta -> UiAssetImportGeneration ->
UiAssetCompiledPreviewGeneration -> ThumbnailGeneration -> RetainedAssetSurfaceDelta`.

Editor refresh acquires immutable runtime generations/deltas. It does not clone `ProjectManager`,
read runtime source/meta/artifact files under editor locks or recapture/rebuild the full catalog.

## Dependency-ordered implementation milestones

| Milestone | Owner/dependency | Required result | Legacy deleted in the same milestone |
|---|---|---|---|
| A0 Measurement truth | build baseline, Runtime07 | current product runs; project-generation counters export source/meta/hash/artifact/index/edge/serialize/read/write/callback/lock/queue/byte truth | none; evidence only |
| A1 Authority and type contract | Plan02 M1, Runtime04 | immutable generation store, stable payload/decoder ID, transaction/delta schema, checkpoint format | deep-clone project snapshot APIs and duplicate registry incremental authority |
| A2 Mutation DAG | A1, Runtime11 | multi-source/rename batches, dependency-component parallel prepare, keyed single-flight, short ordered commit | exactly-one incremental predicate and global prepare serialization |
| A3 Persistence/publication | A1/A2 | append journal, bounded compaction/checkpoint, outside-lock bounded typed delivery | per-target full JSON commit and unbounded lock-held broadcasts |
| A4 Residency | A1/A2/A3, Runtime11/resource owner | async I/O/decode, leases, ready-payload continuity, byte/age/deadline budgets | ordinary synchronous clone-returning load and error-driven decoder trials |
| A5 Runtime/editor consumers | A1-A4, Editor09/Renderer | immutable generation/delta consumers; no broad capture/poll | old `current_project_snapshot`, list/rebuild and direct watcher/resource paths |
| A6 Dynamic acceptance | A0-A5 | scale, WPR/xperf, product, functional, RSS/energy and RenderDoc correlation gates pass | temporary counters only after archived evidence exists |

No local source fix was made in this review. Small edits such as changing the shard count, avoiding
one String allocation or parallelizing the existing full clone would make the current wrong owner
faster while preserving the dominant complexity. Source work begins only after A1 names the single
authority and protected owners accept the deletion boundaries.

## Quantitative validation matrix

Use deterministic fixture families at 1, 1k and 100k assets, stored only under an approved D/E/F
root. For each scale run cold open, unchanged warm open, one add/modify/remove, two coalesced writes,
rename, 32-event burst, 1% dependency-affecting change and forced reconciliation. Use 4 KiB, 4 MiB
and 256 MiB artifact classes and 1/8/64 subscribers.

| Domain | Required counters/trace | Acceptance shape |
|---|---|---|
| Mutation complexity | sources discovered/hashed/imported; records/index keys/edges/diagnostics visited; full scans; registry serialized/written bytes; journal/checkpoint bytes | ordinary K-change work is O(K + affected edges/index keys), full scan/JSON=0; rename is incremental |
| Authority/copy | ProjectManager/registry/index/catalog/payload clone count and bytes; generation/slot swaps | deep aggregate and payload clone=0 in product paths |
| Concurrency | task queue count/bytes/oldest age, single-flight joins, worker utilization, per-key/global lock wait/hold, retry/repeated work | independent keys prepare concurrently; commit fence is short; retries do not redo accepted components |
| Residency | artifact read/decode bytes and p50/p95/p99, caller thread, source/decoded/resident working set, lease hits/misses/evictions | frame/UI thread artifact read/decode=0; all working-set dimensions bounded |
| Publication | subscriber visits, shared payload owners, queue entries/bytes/age, drop/coalesce/gap, callback thread/time, callback-under-lock | payload owner=1; every queue bounded; callback-under-lock/generation-fence=0 |
| Activation | phase time/work counters, cache hit/read count, duplicate artifact reads, time-to-project-shell/time-to-ready | unchanged cache does not read each artifact twice; interactive publication is progressive/budgeted |

After the build-root failure is repaired, collect WPR CPU/File I/O/Disk I/O/CSwitch/ReadyThread and
xperf lock/thread attribution into `E:/Git/ZirconEngine/.artifacts/performance/<date>/`. Repeat each
scenario at least three times and report median plus range. Record hardware, power plan, build hash,
fixture fingerprint and profiler overhead.

RenderDoc is not a CPU asset-registry profiler. Use it only after texture/mesh/material residency is
changed, on the resulting stable F2/F4 frame, to correlate unexpected upload/copy/pass/pipeline work
and resource lifetime with CPU asset events. Do not claim GPU improvement from this static review.

Power comparison requires stable hardware/quality/workload and an available energy counter. Report
absolute energy/time and before/after distributions. Local Unreal/Bevy source constrains ownership
and algorithm direction; it cannot supply cross-machine numeric targets or justify a claim that
Zircon has reached another engine's power level.

## Static acceptance status

The 118/118 manifest and architecture review are complete. Dynamic acceptance is not complete:
current-source managed Cargo/product startup, WPR/xperf, scale counters, RSS/energy and any relevant
RenderDoc correlation are blocked or not yet run. The three module-level roots remain pending. No
milestone commit or WeCom completion message is permitted from this evidence alone.

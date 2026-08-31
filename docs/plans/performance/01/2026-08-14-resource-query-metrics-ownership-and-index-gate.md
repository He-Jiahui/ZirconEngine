---
related_code:
  - zircon_runtime/src/core/resource/management_generation.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_runtime/src/asset/project/catalog_input_generation.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling
  - tools/mvp/Write-RenderExtractBaselineReport.ps1
primary_reference:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
reference_example:
  - dev/UnrealEngine/Samples/Games/Lyra/Source/LyraEditor/Commandlets/ContentValidationCommandlet.cpp
secondary_references:
  - dev/bevy/crates/bevy_asset/src/assets.rs
  - dev/Fyrox/fyrox-resource/src/event.rs
status: proposed_pending_frameworks01_assembly_and_windows_baseline
created_at: 2026-08-14
---

# Resource Query Metrics Ownership And Index Decision Gate

## Scope

This record governs only resource-management query measurement and a possible future query-index
decision. It is not an accepted optimization result. No current-source Windows profile, product
trace, GPU capture, power trace, or before/after performance measurement exists for this path.

The path under review is:

```text
ProjectAssetManager::asset_ids_by_kind
  -> ResourceManager::management_generation
  -> ResourceManagementGeneration stable-order kind query
  -> ProjectAssetManager rich asset record projection
  -> optional scene entity projection
```

The current source facts are:

- `ProjectAssetManager` reads the published immutable resource generation instead of rescanning
  the mutable registry or sorting a caller-local id list.
- The generation uses 64 locator-sorted shards. Its scan/page merge retains one matching row per
  non-empty shard in a min-heap. A complete kind query therefore performs `O(shards + matching
  rows)` candidate advances plus `O(matching rows * log shards)` heap comparisons before the
  high-level rich-record work, in addition to cursor advances through excluded rows.
- `ProjectAssetManager` then loads typed assets and constructs model, mesh, scene, material, and
  shader management records. Scene entity records are derived from scene records. These are
  separate possible costs and must not be attributed to the shard merge without data.

Those facts identify measurement candidates, not a measured bottleneck.

### Static Cost Model And Index Invariant

This is an algorithm review, not a timing claim. Let `S = 64` be the fixed shard count, `N` the
published resource-row count, and `R` the rows matched by a complete query. Scan initialization
advances each shard cursor until its first matching row and inserts that candidate into a min-heap.
For an exhausted scan, direct candidate advances are bounded by `S + R`, heap work is
`O(R * log S)`, and excluded rows contribute up to `N - R` predicate/cursor advances. A page uses
the same merge until `offset + limit` candidates have been consumed, so a high-offset page can
perform nearly the same work as a full scan even when it returns a small visible page. Reducing or
retuning the fixed shard count changes the constant and heap depth; it does not remove the
filtered-row or high-offset growth paths.

The resource registry already owns the required locator identity invariant: its `id_by_locator`
map rejects an occupied locator for a different `ResourceId` during both staging and commit. A
future immutable `locator -> row` lookup index can therefore be a single-value map, provided it is
published from that same mutation result. It must not be inferred from the current shard search or
rebuilt by an Editor consumer. The direct lookup question is separate from ordered kind/state
pages: any proposed latter index must retain `(primary_locator, id)` order and be benchmarked
against its publication and retained-memory cost.

The first higher-level target remains the existing Runtime04 handoff, not a generalized resource
index: one `asset_management_record_sets` aggregate exhausts one unfiltered stable-order scan,
buckets the five managed kinds, and eagerly loads/builds their rich records. The overview, family-summary, status-index, and issue-index APIs
each invoke that aggregate again. A published immutable asset-management generation can eliminate
that stable-consumer repetition without changing generic resource query behavior. Only a baseline
that attributes material time or candidate work to the lower query path should promote a
resource-owned ordered query index to a separate implementation slice.

## Consumer Work To Measure Separately

Current source exposes two separate consumers of the immutable generation. They must not be
collapsed into one alleged "resource query" bottleneck.

- `ProjectAssetManager::asset_management_record_sets` builds Model, Mesh, Scene, Material, and
  Shader record sets separately, but obtains their IDs from one unfiltered stable-order scan and
  buckets only those five kinds. This avoids five independent filtered merge initializations and
  excluded-row walks while preserving the individual `asset_ids_by_kind` API for single-family
  callers. The source derives scene-entity records from the one scene-record projection inside
  that aggregate; preserve that behavior.
- `AssetWorkspaceState::build_snapshot` maps each visible catalog asset and the current selection
  through `ResourceManagementGeneration::row_by_locator`. The current lookup visits shard maps in
  sequence and performs a binary search in each visited shard. The activity/explorer pair shares
  one snapshot build and clones it, so do not count the clone as a second resource projection.

These are source-shape facts only. The aggregate scan count, visible asset count, folder-tree
work, typed asset load cost, resource lookup count, and UI snapshot cadence must be captured
independently before deciding whether the correct repair is a generation-owned index, a
generation-keyed consumer projection, or no change at all. A cache placed in Editor solely to
mask a resource lookup would duplicate owner truth and violate the selected architecture.

## Ownership Decision

Frameworks01 owns `zr_resource` and its physical migration order is fixed:

```text
zr_math / zr_resource -> zr_contracts -> zr_kernel -> zr_diagnostics
```

`zr_resource` may depend only on `zircon_runtime_interface::resource`; it must not depend on
`core::diagnostics`, recorder APIs, tracing aliases, or glob-imported higher layers. Query-local
scan/page work metrics belong to the resource generation because that owner observes candidate
checks, filtered rows, candidates consumed, returned rows, and matching counts exactly.

The resource assembly surface must expose only those immutable, query-local metric values through
the approved `#[doc(hidden)] zr_resource::assembly` boundary. It must not expose a profiler,
capture state, editor cache, or a mutable diagnostics callback.

The high-level adapter belongs outside `zr_resource`, at the asset-management consumer or a
diagnostics adapter selected by that consumer. It may convert a completed query's local metrics
into one recorder batch while an active profiling frame exists. The adapter owns the counter names
already consumed by `Write-RenderExtractBaselineReport.ps1`; resource code owns none of those
recorder writes. This retains one direction of dependency and avoids a second query cache in the
Editor.

## Reference Review

Unreal is the primary reference. `FAssetRegistryState` owns registry state and provides filtered
asset enumeration while retaining package, path, class, and tag indexes alongside that state
(`AssetRegistryState.h`, lines 749-780). Its own comments explicitly describe the tag-index
memory/query-speed tradeoff (`AssetRegistryState.h`, lines 25-36). The decision is therefore a
state-owner decision rather than an editor-side cache. Zircon should follow that owner placement
if an index becomes necessary.

The implementation confirms that this is not merely a declaration-level arrangement:
`FAssetRegistryState::AddAssetData` updates the primary asset set and its package, path, class,
and tag indexes through the same state-owner mutation (`AssetRegistryState.cpp`, lines 3445-3484).
Its package/path/class enumeration methods then look up the relevant owner-held index and iterate
only the resulting entries (`AssetRegistryState.h`, lines 898-1025). `GetAssets` delegates to the
state's filtered enumeration, whose private implementation selects an intersection strategy from
the actual filter-result sizes and complexity (`AssetRegistryState.cpp`, lines 1297-1339). The
specific Unreal cost constants and containers are not a Zircon implementation prescription; the
transferable rule is to make any index a transactionally maintained part of the authoritative
generation, and to choose query strategy from measured workload rather than an Editor cache.

The local Lyra sample confirms the consumer side of that separation:
`ContentValidationCommandlet` loads `AssetRegistry`, obtains `IAssetRegistry`, calls
`SearchAllAssets(true)`, and then validates project content (`ContentValidationCommandlet.cpp`,
lines 60-63 and 163-171). It does not build a second commandlet-owned catalog. This supports an
asset/scene consumer adapter for metrics but does not justify moving registry state, indexes, or
diagnostics dependencies out of `zr_resource`.

Bevy's `Assets<A>` owns dense/hash asset storage and queues lifecycle events in the same owner.
Fyrox's resource broadcaster keeps loaded, reloaded, added, and removed events at the resource
boundary. Both support resource-owned truth with consumers receiving published data or events;
neither supports an editor-owned duplicate registry or a lower-layer diagnostics dependency.

Fyrox also makes the publication boundary concrete: `ResourceManager` owns both the registry and
its `ResourceEventBroadcaster`, with a separate `ResourceIo` abstraction for virtual file-system
access (`fyrox-resource/src/manager.rs`, lines 88-100). The broadcaster's only event vocabulary is
loaded, reloaded, added, and removed (`fyrox-resource/src/event.rs`), and manager mutations publish
after the resource lifecycle transition (`manager.rs`, lines 1330-1359 and 1564-1618). Zircon's
immutable generation plus typed delta is the appropriate analogue because consumers require stable
pages and generation identity, not just a lossy lifecycle signal. This reference does not justify
adding a broad event fanout or moving diagnostics into `zr_resource`.

## Counter Contract

The high-level adapter must emit exactly one batch per completed query, with one frame association
and one timestamp for all values in that batch. The required names are retained for report
compatibility:

| Query | Counter names |
| --- | --- |
| Scan | `resource_management.scan.instances`, `matching_rows`, `rows_emitted`, `shard_candidate_checks`, `filtered_rows_skipped` |
| Page | `resource_management.page.instances`, `matching_rows`, `candidate_rows`, `rows_returned`, `shard_candidate_checks`, `filtered_rows_skipped` |

`candidate_rows` includes rows consumed by pagination offset. A query started outside an active
profiling frame must not create recorder I/O. A cloned/abandoned cursor must have explicit
ownership semantics before it is included in measurement; it must never silently double-count a
logical caller query.

`Write-ResourceManagementBaselineReport.ps1` is the reporting boundary for this matrix. It
consumes an immutable scale-plan JSON and a future observation JSON; it does not launch a product,
generate data assets, or manufacture counters. It validates the source fingerprint, plan SHA-256,
scenario inventory identity, every required repetition, the exact scan/page/workspace query shape,
a non-negative integral `frame_index` and `timestamp_us` for each observed query, and every
operation-specific counter. Until a trusted product observation producer binds a product receipt,
process identity, trace/frame evidence, collector, and machine/cache context to that input, the
report is fail-closed as `unverified` with reason `no-trusted-observation-producer`; structural
validation alone must not yield `measured`. Its JSON and Markdown outputs are atomically published
only below `E:\ZirconBuilds\mvp-resource-management-reports`. The report contains logical scenario
IDs and immutable digests, not project physical paths. It is not performance evidence.

The report must preserve independent `asset_management` and `asset_management_page` coverage.
Missing all counters is `not_emitted`; emitting only a subset is `partial`; scan data must never
make a page query appear measured.

## Current Instrumentation Gap

`Write-RenderExtractBaselineReport.ps1` and its focused fixture already validate the scan/page
counter schema and correctly classify missing or partial counter sets. That fixture is a report
consumer contract, not proof that a product emits the counters. The candidate
`ProjectAssetManager` adapter now opens `project_asset_manager.kind_query` and
`project_asset_manager.record_sets` spans only inside an active profiling frame, then writes the
five scan counters once after a kind scan completes. It reads crate-visible immutable scan metrics
and never introduces recorder access into `zr_resource`. The scan source contract is covered by
profiling-feature unit tests, but it remains `not_emitted` for product acceptance until a managed
Windows capture contains its actual counters; the page adapter remains absent, so
`asset_management_page` is still `not_emitted`.

The scale-specific report validator cannot change that status. It adds the missing link between a
declared 1/1k/100k workload plan and a later real observation, so a partial capture cannot be
mistaken for the full matrix. It must remain a consumer of raw profiler data; no fixture, test
mode, or synthesized observation may be used as a production measurement.

The adapter is deliberately outside `zr_resource`: after a complete scan, it reads the immutable
query-local values and emits one `record_counter_batch` containing the five scan counters. The
aggregate method emits its own `project_asset_manager.record_sets` scope. The current monolithic
crate can use crate-visible metrics; the approved `zr_resource::assembly` surface must carry the
same data when Frameworks01 completes the physical extraction. Do not make a report fixture or a
test-only counter producer stand in for that production boundary.

## Windows Measurement Gate

With the outer scan adapter wired, collect a managed current-source Windows baseline only.
Artifacts remain under an approved `D:`, `E:`, or `F:` root; no project data, capture output,
cache, or report is written to `C:`. Use project-relative resource locators and the existing
resolver; do not add a path prefix scheme. The page portion of the matrix remains blocked on its
own consumer adapter and must not be inferred from scan counters.

For each workload, capture at least three repetitions and report median/p95 for query wall time,
candidate checks, matching rows, returned rows, rich-record count, scene-entity count, product CPU
time, peak working set, and allocation/copy proxies where instrumented:

| Dimension | Values |
| --- | --- |
| Registry scale | 1, 1k, 100k resources |
| Query | each management kind; all-kind/state-filtered page where product code uses it |
| Page | offset/limit 0/50/1k plus a high-offset case |
| Change mode | cold open, stable generation, 1% resource change |
| Consumer | id-only query, rich-record list, scene-entity list |

Add two consumer-specific measurements to every applicable workload:

| Consumer path | Required independent measurements |
| --- | --- |
| Management aggregate | aggregate invocation count, five kind-query counts, per-kind matching/returned rows, typed-load attempts/successes, rich-record and scene-entity counts |
| Asset workspace snapshot | snapshot invocation count, catalog/folder/visible-asset counts, `row_by_locator` call count, per-call shard probes, selection lookup count, and activity/explorer clone count |

GPU timing and system power remain unavailable unless a supported GPU capture and an OS/device
power source identify their own sampling method. CPU counters cannot establish either measure, and
no comparison to another engine is valid until the workload and measurement method are equivalent.

## Index Decision Rules

Do not add an index merely because the 64-shard K-way merge exists. Make the decision from the
measured baseline:

1. If generation query time and candidate checks are insignificant beside rich-record or scene
   projection, optimize the measured high-level owner instead.
2. If stable-generation scans are material and candidate checks scale as predicted, add a compact
   immutable query index owned and published with the same resource generation. Start with the
   demonstrated query key, such as kind or `(kind, state)`, and retain deterministic locator order.
3. Compare index build time, generation memory, publish cost, query p50/p95, and RSS before and
   after. Reject an index that only moves work to generation publication or inflates memory without
   improving the measured user path.
4. Do not place the index in Editor, do not restore registry scans/caller sorting, and do not add a
   compatibility cache around the old query path.

The physical migration guard that rejects `core::resource -> core::diagnostics` is part of this
architecture, not a temporary test obstacle. Any implementation that violates it is incomplete.

## Catalog Authority Follow-Up

The existing Runtime04 handoff
`failure-2026-07-23-project-catalog-input-generation.md` remains the canonical failure record;
this section records current-source research only and does not create a second lifecycle.

Current Runtime source has progressed beyond the handoff's original absence: immutable
`ProjectCatalogInputGeneration` includes project metadata, package roots, resource records,
source paths and mtimes, meta documents, and artifact direct-reference payloads. It preserves its
`Arc` identity when all inputs compare equal and exposes an added/modified/removed/renamed delta.
The tests cover unchanged input, source touch with an unchanged digest, meta projection changes,
artifact references, package roots, and rename classification.

The consumer boundary has not converged yet. `DefaultEditorAssetManager::sync_from_project`
currently creates its separate `EditorAssetProjectSourceGeneration` by cloning every mutable
`ProjectManager::registry()` record, then compares two editor-owned maps. It does not reference
`ProjectCatalogInputGeneration`. Therefore an unchanged editor sync cannot use the Runtime
generation's `Arc` identity as the O(1) admission check; it first performs the registry capture
and map comparison. The subsequent full/incremental catalog projection is guarded correctly, but
the duplicate source generation remains a second authority-shaped data structure.

Targeted Runtime import/removal now uses `ProjectCatalogInputGeneration::publish_targeted`.
Catalog input records live in 64 `Arc<HashMap<...>>` shards; a targeted root replacement/removal
clones only the affected shard map and rebuilds only changed records. Unchanged records retain their
identity and direct-reference `Arc`, while a full project scan still constructs the complete
generation. The old `input_sources()` full-copy helper and targeted `registry.values()` publication
path are deleted. This is a source-shape repair, not a measured timing or memory result; measure it
independently from Editor refresh work.

### Targeted Snapshot Complexity Correction

The preceding source-shape statement must not be read as an `O(k)` targeted-publication claim.
`Arc::make_mut` clones the affected `HashMap` container before mutation. With a fixed 64-shard
layout, a one-root mutation copies the entries in its affected shard, approximately `N / 64` for an
even distribution. The records and direct-reference payloads remain shared through `Arc`, but the
map-entry work is still asymptotically `O(N)` for a constant number of touched roots. The direct
predecessor *delta construction* below is `O(k log k)` only after that snapshot has already been
published.

This is a material algorithm distinction. The current layout removes deep record rebuilding and
the retired full-source helper, but it is a constant-factor improvement rather than a persistent
map. It must not be used to claim bounded targeted publication, and no fixed shard count is an
acceptance substitute for a profiler result.

The workspace has no existing persistent-map dependency. Before changing the Runtime04 owner again,
compare only these two designs against the Windows baseline:

1. A bounded immutable overlay chain: each targeted generation owns just its changed/tombstoned
   records; lookup walks a bounded chain and a scheduled compaction rebuilds a full base only at a
   measured depth. This gives small publish work but changes lookup/iteration and compaction costs.
2. A proven persistent hash-trie dependency owned by Runtime04: publish and lookup become
   logarithmic structural sharing, at the cost of a new dependency and different memory profile.

Neither design is approved before measured scan/import publish, lookup, allocation, RSS and
compaction behavior. The public Runtime authority, direct-predecessor typed delta, skipped-version
full reconciliation, and Editor's future `Arc` admission rule remain invariant under either choice.

### Adjacent Generation Delta Decision

The predecessor-delta work started from a Runtime gap: structurally shared catalog records still
used a full current/previous comparison whenever the generation sequence changed. For a targeted
change this made the data publication path locally bounded while the immediate consumer delta
remained `O(all catalog roots)`. Runtime now retains the direct predecessor sequence and immutable
typed delta at publication, so an immediate consumer does not rescan all roots; a skipped-
generation comparison explicitly retains the full reconciliation fallback. The Editor has a
separate and larger unresolved instance of the same shape:
`EditorAssetProjectSourceGeneration::capture` clones every mutable registry row before it
performs its own full-map comparison. These are two distinct costs and must not be merged into one
claimed bottleneck without a capture.

Runtime04 retains an immutable typed delta and predecessor sequence in every published
`ProjectCatalogInputGeneration`. A targeted publish already knows the removed IDs and updated
root records, so after snapshot construction it can compare only that touched ID set against its
direct predecessor and publish added/modified/removed/renamed rows in `O(k log k)` for deterministic
ordering. A full scan may derive its delta by comparing complete inventories because its publication
itself is already a full-inventory operation. A consumer that compares the direct predecessor
receives the retained typed delta without scanning all roots; a skipped-generation consumer
explicitly falls back to a full comparison. This preserves correctness without adding an Editor
cache or retaining an unbounded generation history.

Before accepting this change, Windows measurements must separately capture catalog root count,
touched-ID count, touched-shard entry copies, Runtime publish/delta comparison count and wall time
for direct-predecessor versus skipped-generation comparisons, plus Editor registry rows cloned and
Editor delta comparison work. The acceptance hypothesis is that direct targeted *delta consumers*
scale with `k`, while snapshot publication and skipped generations retain their separately measured
costs. No wall-time, allocation, CPU, power, or cross-engine equivalence claim is made until those
managed captures exist.

The required structural direction is:

1. Make Editor09 retain the published `Arc<ProjectCatalogInputGeneration>` as its input identity
   and return before registry capture, meta/artifact projection, folder/reference rebuild, or
   shader IDE writes when that `Arc` is unchanged.
2. Build or patch editor catalog rows from the runtime generation's records and typed delta, not
   a separately captured mutable registry. Preserve the existing latest-started-wins publication
   fence.
3. Targeted Runtime publication now structurally shares unchanged catalog input records. Preserve
   that invariant while exposing its typed delta at the same authority boundary; do not move a full
   map clone into Editor or conceal it behind a consumer cache.
4. Measure Runtime publish record/source/meta/reference copies and Editor admission/projection
   separately for unchanged, one-percent change, rename, and package-root change workloads.

Unreal's `FAssetRegistryState` makes path/class/tag indexes part of the registry state and exposes
queries from that owned state; asset registry events then notify arbitrary consumers. This supports
the same direction: one published authority and consumer-specific projections, rather than an
Editor-owned reconstruction of mutable registry data.

## Management Projection Continuation

`failure-2026-07-22-asset-management-generation-projection.md` is the existing canonical
Runtime04 handoff for the rich management-record projection. It remains open; this section is
only a current-source design note and must not be used as a second handoff lifecycle.

`ProjectAssetManager::asset_management_record_sets_with_prepared_materials` now runs one
unfiltered stable-order resource scan, buckets model, mesh, scene, material, and shader IDs, and
then builds their typed records. Individual-family APIs retain their direct kind query. This
removes the aggregate's five independent merge initializations and repeated excluded-row walks
without adding a cache, index, or resource-layer diagnostics dependency. The aggregate shares the
scene records when deriving scene-entity rows, but it does not publish an immutable management
record generation; stable consumers can therefore still rebuild all five record families without a
resource mutation.

The current lifecycle has no hidden publication point that would make the rebuild safe to ignore.
`ProjectAssetManager` retains the project-generation fence, active project, resource manager, and
generation-wake subscribers, but no management-projection field. Its open, close, reimport, and
watch paths commit resource/project state and then call `publish_project_generation`; none installs
an immutable asset-management value before that broadcast. The present aggregate also accepts a
renderer-prepared `RenderMaterialManagementRecordSet`, so it cannot itself be the Runtime04-owned
authoritative snapshot: renderer-dependent material detail must be composed in the graphics
consumer after the asset projection is read.

The correct owner split is:

1. `core::resource` owns compact resource rows, order, narrow lookup data, and query-local work
   metrics. It must not own asset-specific rich records or diagnostics emission.
2. Runtime04 owns an immutable asset-management projection that is atomically derived from the
   published resource/catalog authority and carries its typed delta, family summaries, and stable
   query/page views.
3. Editor consumers retain and page that Runtime04 projection. They may not reconstruct it from
   the mutable registry or keep another full catalog cache.

The profiling contract records one completed aggregate scan and the typed record/scene-entity
projection work as separate counters. After Runtime04 publishes a projection, verify unchanged
polling has zero management-record rebuilds while a changed workload is bounded by the typed delta
plus the requested page. The single-scan assertion is a source-level work-count invariant, not a
timing, memory, GPU, or power result.

### Minimum Future Publication Cut

This is an implementation boundary for the existing Runtime04 handoff, not approval to bypass the
measurement gate. Once the baseline identifies stable management reconstruction as material, the
smallest coherent direction is:

1. Runtime04 owns an immutable, asset-only `AssetManagementGeneration`, keyed by the published
   resource/catalog generation identities and containing typed rows, family summaries, indexes,
   and an adjacent typed delta. It must exclude renderer-prepared material detail.
2. `ProjectAssetManager` installs that value after resource reconciliation and active-project
   replacement succeed, but before `publish_project_generation` wakes consumers. Close installs
   the corresponding empty generation at the same fence. The same ordering applies to open,
   targeted reimport, and watcher reconciliation; one lifecycle cannot keep a stale projection.
3. Public management summary/list/page queries read the retained generation and return its stable
   views. Renderer code adds `RenderMaterialManagementRecordSet` only when it composes a render
   view; Editor code consumes the Runtime projection and cannot rebuild a registry-shaped copy.
4. A direct-predecessor consumer reads the retained typed delta; skipped generations explicitly
   request reconciliation. No unbounded generation history or lower-layer diagnostics callback is
   permitted.

The release gate is evidence, not source shape: compare the current one aggregate scan plus typed
loads against the preceding five-scan implementation for unchanged, one-percent, rename, and page
workloads. Retain this implementation only if managed measurements confirm the user path is not
dominated elsewhere.

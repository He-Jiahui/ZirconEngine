---
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_editor/src/ui/retained_host/app/assets
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/shader/05-ide-and-authoring-dx.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/AssetThumbnail.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
tests:
  - 78 of 78 current Rust files reconciled and reviewed
  - 10186 physical lines and 62 inline tests
  - path plus physical-line-count plus per-file SHA-256 manifest fingerprint 671ed66386f72066c34c161734894c604bcfde6bec399d88ee847eb0708f8724
  - managed current-source Cargo and product WPR/xperf/RenderDoc/energy remain blocked by the non-runnable editor baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-15
---

# Editor asset refresh/preview current architecture review (2026-08-15)

## Scope freeze and method

This review freezes the current MVP editor asset refresh vertical at **78/78 Rust files, 10,186
physical lines and 62 inline tests**. The manifest fingerprint is
`671ed66386f72066c34c161734894c604bcfde6bec399d88ee847eb0708f8724`; it is SHA-256 over sorted
`path|physical-lines|file-sha256` rows joined with LF.

| Current module | Files | Physical lines | Tests | Verdict |
|---|---:|---:|---:|---|
| `ui/host/asset_editor_sessions/refresh` | 15 | 1,616 | 9 | generation queue is useful, but direct/save paths remain a second synchronous authority and commit recompiles on the main thread |
| `ui/host/editor_asset_manager` | 45 | 4,698 | 38 | change ingress is bounded, but every runtime refresh still clones/captures/scans broad project state and rebuilds global projections |
| `ui/asset_editor/preview` | 8 | 2,618 | 0 | resize dirtying is incremental, but stable presentation repeatedly rebuilds projection, mock fields and state/binding graphs |
| `ui/retained_host/app/assets` | 10 | 1,254 | 15 | event accumulation is bounded, but asset changes synchronously refresh the catalog and preview completion rediscovers visible rows through full chrome construction |

The July reports were used only for unchanged history. Every current file in the four roots was
reconciled, every modified/new file was reread, and production callers outside the roots were
followed through save, watcher, runtime project manager, retained host and shader IDE generation.
All four roots contain foreign uncommitted work, so the fingerprint is part of the evidence and no
source edit was made in this pass.

The approved-root defect in `tools/build-editor.ps1:130` still rejects valid D/E/F build roots before
Cargo. The latest Pester result is 9 pass/6 fail from 15 tests. Consequently there is no current
product executable for WPR/xperf or RenderDoc capture and no valid power comparison. This report
does not invent timing data and the four modules must remain outside `review.md`.

## Architecture verdict

The primary defect is split ownership, not a missing micro-optimization. One filesystem change can
be normalized by the watcher pipeline, rebuilt as a worker session, recompiled during main-thread
import commit, projected through a fully captured editor catalog, followed by a full reference/folder/
details generation, then produce an independent thumbnail completion which copies the complete
immutable catalog generation and asks the retained host to rebuild chrome to rediscover visible rows.
Save and direct editor actions can bypass the watcher generation and run another synchronous path.

The hard-cut target is one dependency-ordered generation chain:

`RuntimeAssetSourceDelta -> EditorAssetIndexDelta -> UiAssetImportGeneration ->
UiAssetCompiledPreviewGeneration -> ThumbnailGeneration -> RetainedAssetSurfaceDelta`

with the shader branch

`RuntimeAssetSourceDelta(shader) -> ShaderSourceArtifactGeneration ->
ShaderIdeEnvironmentGeneration`.

Each generation has one owner, an epoch plus source/dependency fingerprint, affected-only work,
keyed single-flight execution and immutable publication. Main/UI threads may swap a completed
generation and invalidate exact consumers; they may not read source/meta/artifact files, parse or
compile UI/shader documents, encode PNG, rebuild global indexes or wait for background work.

## What is already structurally useful

The replacement must preserve these current behaviors rather than discard them indiscriminately:

- the UI asset watcher has bounded raw ingress and a coalesced normalized-ID queue, one active batch,
  stale generation rejection and bounded exponential retry;
- editor asset change streams cap pending changes at 512 and collapse overflow to `CatalogChanged`;
- retained asset accumulation uses a 32 ms quiet period, 250 ms maximum deferral and 4,096-event cap;
- `PreviewScheduler` uses per-asset tokens, visible admission and a 64-job in-flight ceiling;
- `PreviewHost::rebuild_with_size` avoids same-size work and uses dirty roots for size changes;
- sprite atlas packing uses the proven `rectangle_pack` implementation and is currently an offline,
  non-product path, so it is not an MVP steady-frame optimization target.

These are local safety mechanisms. They do not establish a single source-to-presentation authority,
a main-thread budget, a decoded-byte budget or delta-proportional publication.

## P0/P1 current-source findings

### 1. UI asset refresh still has two authorities and worker completion recompiles on the main thread

`refresh/apply.rs:10-41` exposes synchronous `refresh_ui_asset_workspace_for_changes` and immediately
applies direct/import impact. Save calls it after filesystem write at
`asset_editor_sessions/save.rs:84-100`, then hydrates imports again. Editing, navigation and node
operations also reach this direct path, while watcher delivery uses `refresh/pipeline/**`.

The pipeline worker builds a complete editor session in `pipeline/job.rs:115`, including direct
preview compile. Commit then holds the dependency-generation lock from
`pipeline/commit.rs:30-209` and calls `replace_resolved_imports` at `:114` or `:180` while individual
session locks are held. `session/lifecycle.rs:581-655` reparses/reprojects and calls `compile_preview`
during `revalidate_with_palette_catalog`. Thus a directly changed asset can compile in the worker
and again during main-thread commit; import-only dependents compile on the main thread. Hydration has
the same dependency-lock plus session-revalidation shape at `asset_editor_sessions/hydration.rs:30-57`.

`pipeline/queue.rs:34-181` coalesces IDs but has no independent pending count/byte/age admission.
Stale or failed instances extend the entire batch change set at `pipeline/commit.rs:31-54,
103-198`, so one bad dependent can amplify broad requeue work. The required fix is not another
worker around commit: the worker artifact must already contain fully resolved imports, parsed IR,
compiled preview and dependency edges; commit must be a short generation check, atomic swap and
typed invalidation with no parse/compile or broad lock.

### 2. The editor catalog delta path remains O(N) capture plus O(N+E) global rebuild

`DefaultEditorAssetManager::refresh_from_runtime_project` obtains a value-returning runtime project
snapshot. `ProjectAssetManager::current_project_manager` is `self.project_read().clone()` at
`zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs:160-161`, and
`ProjectManager` itself derives `Clone` at `asset/project/manager/mod.rs:31-32`. This copies broad
project/registry ownership before editor delta work starts.

`project_sync/source_generation.rs:31-40` then clones the manifest, package assets and every registry
record into another generation. `delta_since` at `:42-80` walks both current and previous maps and
clones changed records; sorting at `:129-145` materializes locator Strings. Even the incremental
projection clones both full mutable maps at `sync_from_project.rs:77-88`.

After patching touched records, `sync_from_project.rs:134` always rebuilds `ReferenceGraph`.
`reference_graph.rs:14-47` recreates UUID/locator maps and visits every direct edge. Catalog publish
then calls `build_catalog_generation` at `sync_from_project.rs:149-157`; its implementation sorts all
records, rebuilds details for every asset and rebuilds all folders (`catalog_generation/build.rs:
14-65`). Folder construction scans all assets and depths, performs per-parent `Vec::contains`, then
sorts/reduces the entire tree (`folders.rs:18-100`). A one-asset change therefore remains coupled to
total catalog and reference size.

`project_sync/record_projection.rs:24-27` also loads `.zmeta` and imported artifacts synchronously per
touched ready resource. Shader-affecting changes invoke `write_shader_ide_env_for_project` at
`sync_from_project.rs:329-343` while the source commit gate is held (`:134-143`). File parsing,
artifact loading and shader IDE writes need TaskGraph artifacts, not a longer editor lock.

### 3. Thumbnail admission is count-only and every completion copies O(N) catalog rows

`editor_asset_manager/preview.rs:11,113-163` limits in-flight work to 64 and guards stale tokens, but
has no frame-time, decoded-byte, queue-byte, age or recent-access budget. A job clones the project,
record and catalog row at `request_preview_refresh.rs:81-119`. Image generation decodes source data,
resizes to 192x192 and synchronously writes PNG; placeholders construct and shade a 256x160 buffer,
resize it and write PNG (`preview.rs:60-109`). Sixty-four admitted decodes/encodes can therefore
compete for CPU, memory and storage without a byte or time envelope.

Completion takes a global publish gate and state write lock at
`request_preview_refresh.rs:208-249`. `EditorAssetCatalogGeneration::updated_asset` copies the entire
`assets` and `details_by_asset_index` slices for one row at `generation.rs:172-202`. A wave of K
thumbnails over N assets performs O(K*N) immutable-generation row copies even though the change
stream later coalesces notifications. Preview state must live in a keyed thumbnail slot/generation;
its completion should replace one slot and invalidate one visible row, not republish the catalog.

### 4. One asset event creates multiple retained epochs and preview completion rebuilds chrome

The retained accumulator is bounded, but `refresh_project_assets` synchronously calls the full
runtime-project refresh when `events.asset_changes` is nonempty
(`retained_host/app/assets/refresh.rs:17-30,84-113`). Catalog publication then emits another editor
asset event for a later batch. A single source change can therefore cause a source-refresh epoch and
another catalog/preview epoch instead of one typed delta transaction.

`refresh_visible_asset_previews` calls `build_chrome` to discover visible asset rows
(`retained_host/app/assets/refresh/snapshots.rs:49-57`; another call exists in `workspace.rs:32`).
The retained host should keep a generation-owned visible UUID set populated by layout/virtualization
changes. `PreviewChanged` then touches only the keyed preview slot and row/paint invalidation. This
matches the existing fast invalidation direction and removes full presentation construction from
thumbnail completion.

### 5. UI asset preview recomputes the same graphs during stable presentation

Every pane presentation calls `build_preview_projection` at
`session/presentation/preview.rs:35`. Inspector presentation calls `build_preview_mock_fields` and
then `build_preview_state_graph_items` again at `session/presentation/inspector.rs:59-65`, although
`build_preview_mock_fields` already builds and stores the same state graph at
`preview/preview_mock.rs:109-226`.

`build_preview_state_graph_items` (`preview_mock.rs:235-295`) walks nodes/properties, resolves
expressions/bindings and sorts results. Expression parsing materializes a char vector at
`mock_expression.rs:54`; mock override application clones the whole document at
`preview_mock.rs:576-589`. Projection, mock schema/suggestions and state/binding graph are not cached
by document/mock/selection/surface generation. The existing cached hit-index and dirty-root approach
is the right local model: produce one immutable presentation artifact keyed by those generations and
reuse its projection, mock fields, state graph and hit index until an input generation changes.

### 6. Lower-priority local costs are not the current optimization target

Reference sorting still allocates locator Strings in sort keys, and sprite-atlas packing retries
larger square sizes. These are valid profiling candidates but not justification for a source patch in
this slice: reference projection is being replaced by an incremental owner, and sprite atlas is an
offline path with no production caller. Optimizing either first would leave the P0 O(N) generation
and main-thread compile architecture intact.

## Unreal source evidence and transferable rules

### Asset registry and background gathering

- `AssetRegistryState.cpp:3445-3484` adds one record while updating package/path/class/tag
  accelerators. `:3631-3810` updates only changed key fields, tags and indexes; `:3771-3785`
  explicitly orders cheap checks before expensive equality.
- `AssetRegistryState.cpp:4040-4096` groups removals by accelerator and launches parallel tasks only
  at 100 or more keys, avoiding task overhead for small batches.
- `AssetDataGatherer.cpp:4316-4364` owns a background gather loop and sleeps when idle rather than
  busy-waiting. `:4367-4395` has an interruptible end-time boundary, `:4459-4473` moves registry tick
  off the game thread when permitted, and `:4701-4718` stages bounded local batch result arrays.

The transferable rule is one asset registry with incremental accelerators and staged background
gather/apply. Zircon should not copy Unreal containers or its fixed thresholds; thresholds must come
from Zircon task-overhead and workload measurements.

### Thumbnail budgeting and targeted invalidation

`AssetThumbnail.cpp:1950-1953` gives the pool a frame-time allowance and a real-time count budget.
Its tick at `:2114-2180` respects Slate expensive-task throttling, limits work to recently accessed
thumbnails, enforces both time and count, and retains retry state. `RefreshThumbnailFor` at
`:2892-2899` queues one unique thumbnail rather than rebuilding the content browser catalog.

Zircon additionally needs decoded/source/resident byte budgets because image jobs run through a Rust
job system and write disk artifacts. The relevant principle is multi-dimensional admission plus
targeted completion, not the exact Unreal pool size.

### Slate fast invalidation

`SlateInvalidationRoot.cpp:356-424` performs a slow rebuild only when required and otherwise paints
the fast path. `:1281-1405` processes explicit pre/attribute/prepass/post update lists and clears all
cached data only after slow-path fallback. Zircon's retained asset surface should therefore consume
typed dirty rows and a cached visible set; building chrome merely to locate rows contradicts both
the local retained model and the reference design.

## Required hard-cut architecture

### A. Runtime04 owns `RuntimeAssetSourceDelta`

`ProjectAssetManager` publishes immutable project/catalog generations and a typed change journal.
Each delta carries project epoch, asset slot/generation, locator/kind, old/new source revision and
changed dependency ranges. Editor code acquires an `Arc` generation handle; it never clones a
`ProjectManager` value or recaptures the whole registry to discover changes.

The runtime registry maintains UUID, locator, kind, folder and dependency/reverse-dependency indexes
incrementally. Add/update/remove/rename cost is proportional to changed records, affected edges and
folder depth. Full rebuild exists only for project activation, schema migration or explicit recovery.

### B. Runtime11 owns one keyed generation task service

UI import resolution, UI compile, thumbnail decode/encode, metadata/artifact reads and shader IDE
generation are dependency tasks in the shared scheduler. Keys include domain, project epoch, asset
slot/generation, source revision, dependency fingerprint and output variant. Duplicate requests join
one flight; stale work cancels before publication.

Admission has count, queued/source/decoded/resident bytes, age, priority, affinity and deadline.
Visible/recent thumbnail work outranks background work, but neither can starve save/import or block
the UI thread. Main-thread completion has its own count/time budget and contains only validation,
generation swap and typed invalidation.

### C. Editor09 owns delta-proportional authoring generations

`EditorAssetIndexGeneration` is a persistent/chunked or stable-slot projection over the runtime
catalog. Details, folder membership, reference edges and preview slots update by affected key/range.
Thumbnail state is not embedded in an array that must be copied for every completion. Published
handles are immutable and generation-tagged; mutable staging is private to a task.

UI assets use one `UiAssetCompiledPreviewGeneration` containing parsed source, resolved imports,
dependency edges, compiled preview/surface resources, diagnostics and presentation cache inputs.
Direct edits, watcher events, save and hydration all enqueue the same key; synchronous refresh APIs
do not survive the cut.

### D. EditorUI05/Runtime09 consume exact presentation deltas

The retained host stores visible asset UUIDs as part of the layout/virtualization generation.
Catalog row, preview row, folder, reference detail and UI asset pane invalidations are separate typed
domains. Stable paint consumes immutable handles; it performs no asset scan, chrome rebuild, preview
graph construction, filesystem access or compiler work.

UI asset pane presentation is cached by `(document_generation, import_generation,
mock_generation, selection_generation, surface_generation)`. State/binding graph is built once per
key and shared by mock fields and inspector; hit/projection caches use the same generation identity.

### E. Shader05 consumes shared source artifacts

Shader IDE environment generation is an asynchronous keyed artifact downstream of the canonical
shader source DAG established by PERF-MVP-636. It never performs synchronous writes under the editor
source gate, and plugin/material/IDE/preview do not maintain separate parse/dependency authorities.

## Dependency-ordered implementation milestones

1. **Instrumentation freeze:** add disabled-fast-path counters/scopes only; preserve current behavior
   and record baseline source fingerprint. Do not add another cache or private executor.
2. **Runtime asset journal:** Runtime04 publishes immutable catalog handle, typed deltas and
   incremental UUID/locator/kind/folder/dependency indexes. Replace value-returning project clone in
   editor consumers.
3. **Shared task artifacts:** Runtime11 exposes keyed single-flight DAG jobs, multi-dimensional
   budgets, cancellation/currentness and deadline shutdown. Editor jobs become adapters or are
   deleted in the same milestone.
4. **Editor index and UI compile generations:** Editor09 consumes runtime deltas, incrementally
   publishes rows/details/folders/references, and makes worker-built UI preview generations the only
   refresh result.
5. **Thumbnail and retained surface cut:** thumbnail results update keyed slots; EditorUI05/Runtime09
   retain visible UUIDs and invalidate exact rows/panes without chrome or catalog rebuild.
6. **Shader IDE branch:** Shader05/Render08 consume shared source generations and publish IDE files as
   bounded async artifacts.
7. **Hard deletion and product acceptance:** remove all legacy authorities below, run current-source
   Cargo and product traces, then update protected plan status and review/pending indexes.

## Required deletions in the replacing milestones

No alias, compatibility shim or dual-write period may keep these production paths alive:

- synchronous `refresh_ui_asset_workspace_for_changes` and save/direct bypasses;
- main-thread parse/compile in `replace_resolved_imports` commit/hydration paths;
- editor refresh through cloned `ProjectManager` plus full `EditorAssetProjectSourceGeneration::capture`
  and `delta_since` scans;
- per-delta full `ReferenceGraph::rebuild`, `build_catalog_generation` and folder/detail rebuild;
- O(N) `EditorAssetCatalogGeneration::updated_asset` publication for preview completion;
- preview completion `build_chrome` visibility discovery;
- synchronous `write_shader_ide_env_for_project` under the source-sync gate;
- duplicate state-graph construction and stable presentation recomputation without generation keys.

## Measurement and acceptance gates

### Required counters and scopes

Record per frame, generation and asset key:

- project-manager value clones, registry/source capture rows and bytes, delta map visits;
- catalog touched rows versus full rows, folder nodes, reference edges and generation bytes copied;
- UI source read/parse/import resolve/preview compile counts, worker/main split, retry/requeue IDs;
- dependency/session/source/publish lock wait and hold microseconds;
- thumbnail queued/in-flight/completed/cancelled/stale counts, source/decoded/resident bytes, decode/
  resize/PNG/fs time and main-thread apply time;
- retained chrome builds, visible-set rebuilds, row/folder/pane invalidations and full recomputes;
- preview projection/mock/state-graph builds, cache hits and cloned document/value bytes;
- shader IDE artifact submissions, source bytes, writes, stale cancellations and gate hold time.

Diagnostics disabled must add zero allocation, lock and per-row atomic RMW in steady state; enabled
counters use task/frame-local aggregation and bounded publication.

### Complexity and behavior matrix

Use projects with 1/1,000/10,000/100,000 assets; dependency degree 0/1/8/64; folder depth 1/8/32;
one/1%/100% changes; UI import fan-out 0/1/100/10,000; visible thumbnails 0/16/256/4,096; source
images 64 KiB/4 MiB/64 MiB; stable/edit/save/watcher/rename/remove/shader-change/storm/cancel/reload.

Required algorithmic outcomes:

- stable frame: project/registry clone, source/meta/artifact/PNG/IDE I/O, catalog/reference/folder
  rebuild, UI parse/compile, preview graph build and thumbnail publication row copy are all zero;
- one-asset update cost follows touched rows + affected dependency edges + folder depth, not N;
  increasing 1,000 to 10,000 unrelated assets must not produce a linear increase in visit/copy work;
- direct/save/watcher delivery for one source revision joins one keyed generation and publishes at
  most one current result; stale result publication is zero;
- main-thread commit contains no parse, compile, decode, PNG, filesystem or shader IDE work and obeys
  a configurable count/time budget; dependency/source/publish locks do not cover CPU-heavy work;
- thumbnail queue proves count, source/decoded/resident-byte, age and completion budgets; visible
  recent work is prioritized and one completion changes one keyed slot/row;
- preview mock/state graph is built at most once per input generation key; stable pane presentation
  has cache hit and zero graph/projection rebuild;
- functional save/import/retry/rename/remove/reference/preview/selection/hit-test/shader-IDE behavior
  and generation-currentness remain correct under cancellation and project reload.

### Product evidence after M0 is repaired

Build and stage only on an approved D/E/F `ZirconBuilds` child. Run at least three warm captures for
F0 editor startup/idle/exit and F4 asset-browser/UI-asset edit/save/thumbnail storm. WPR/xperf is the
CPU, scheduling, disk, context-switch, idle-wake, RSS and energy authority; local scopes provide
p50/p95/p99 attribution. RenderDoc and GPU timestamps verify that preview/asset changes do not add
unexpected frame draw/dispatch/upload/readback work and preserve pixels. Compare complexity and
thread/frame budgets to the cited Unreal behavior; do not claim equivalent absolute power or time
without same-machine, same-workload measurements.

## Current disposition

The 78/78 static review is complete and the structural direction is accepted as a plan input. No
source change is justified before Runtime04/11 ownership exists: a local HashMap, extra worker or
same-value guard would preserve the wrong authority and conflict with foreign dirty work. Dynamic
acceptance is blocked by the editor build baseline and current managed Cargo failures, so protected
owners must keep the four module folders in `pending.md`, not `review.md`. No milestone commit or
WeCom completion message is permitted from this evidence alone.

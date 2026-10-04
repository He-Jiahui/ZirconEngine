---
related_code:
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/assets
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
tests:
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/workbench/project
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/LayoutService.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor Workbench project asset presentation and persistence architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for asset presentation and layout-preset enumeration on host recompute; P1 for
  synchronous project save and versioned workspace/preset persistence.
- Accounting: retain `zircon_editor/src/ui/workbench/project/**` in `pending.md`. Do not add it to
  `review.md` before the generation cutovers and product traces below pass.
- Code disposition: no Rust source changed. Six focused source/test files contain foreign changes,
  and the Editor source tree is owned by an active session. The implementation owner must re-read
  and re-hash current source before editing.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/project/**` | 16/16 | 1,283 | 8 in-module | 46,805 | `fed12d140e743ebffc436d6e733b21e20140ca17fac1acb2eceae7b925636b43` |
| focused project/asset tests | 4/4 | 1,123 | 14 | 44,507 | `5876a8530f679c41d15411159b1dfce27262df876c359d9445305ec92d7a0edd` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 16 production files
and all four focused tests were read in full. Callers were traced through Editor snapshot/chrome,
retained-host recompute, asset refresh, layout persistence, project open/save and Runtime registry
enumeration.

## Module acceptance record

| module | current-source performance verdict |
|---|---|
| asset workspace state | Every aggregate Editor snapshot rebuilds and sorts folder topology, scans the full asset catalog, reparses parent paths with owned strings, rebuilds visible rows and selection, then deep-clones the complete result for the second surface. |
| project/workspace document | Startup reads settings, workspace and scene synchronously. Explicit save first reads the old workspace for compensation, writes pretty workspace JSON, writes the scene and performs synchronous post-commit import/catalog/watcher work. The compensation read is correctness work, not removable redundancy. |
| layout preset assets | User save/load is synchronous and preset files are not written atomically. More critically, every full or targeted pane recompute obtains preset names by cloning all project asset URIs, filtering/sorting/deduplicating them and decoding the config preset map. |
| path/facade helpers | Project-root canonicalization and URI-to-source resolution perform filesystem/index work. They are acceptable at command admission but must not become presentation-frame helpers. |
| focused tests | Tests cover small semantic fixtures, rollback and format errors. They do not cover large catalogs, stable zero-work, preset enumeration cadence, main-thread blocking, crash durability or bounded malformed input. |

## Structural bottlenecks

### P0: preset names enumerate the complete asset registry during presentation recompute

Both `collect_host_lifecycle_pane_payloads()` and the targeted
`collect_shell_content_pane_payloads()` call `runtime.preset_names()`. That path locks the Workbench
shell, calls `EditorUiHost::preset_names()`, asks Runtime for `current_project_asset_uris()`, clones
every primary locator in the registry, filters layout-preset URIs, allocates preset names, sorts and
deduplicates them, then loads and deserializes the complete config preset map and sorts again.

This work is not gated by menu visibility or a layout-preset generation. A hierarchy, Inspector,
asset, pointer or targeted shell-content recompute can therefore perform O(A) registry traversal and
allocation for A project assets while holding the shell access chain. The downstream Slint preset
model cache only detects equality after the scan, clones and decode have happened.

EditorUI08 and Optimize13 must consume one immutable `LayoutPresetCatalogGeneration`. Project asset
presets are maintained from Runtime04 prefix/type deltas; user config presets are maintained from a
config generation. Host recompute reads an Arc slice and generation receipt. Stable and unrelated
presentation changes perform exactly zero registry visits, URI clones, config decodes, sorts and
deduplications.

### P0: asset presentation recompiles catalog structure on every aggregate snapshot

`AssetWorkspaceState::build_snapshot()` performs the following work even when catalog, query,
folder, selection and resources are unchanged:

1. Lowercase the query.
2. Rebuild a parent-to-children hash map for every folder and sort every sibling list.
3. Scan all folders for direct children and scan all assets for the selected folder.
4. Derive each asset parent with `rsplit_once` plus a newly formatted `String`.
5. Lowercase display name, file name and locator on demand for search.
6. Clone visible row strings/diagnostics and probe resource state by locator.
7. Rebuild selected references/subassets/details and project overview strings.
8. Deep-clone the complete Activity snapshot into Explorer.

The adjacent-frame projection stamp records catalog/resource input changes but does not cache the
compiled output or skip any of the work above. Small tests use five assets and six folders, so they
cannot establish catalog-scale complexity. This is the producer-side continuation of
PERF-MVP-095/102/104, not a solved path.

Editor09/Optimize04 must publish a shared immutable asset-presentation generation containing folder
topology/order, direct asset membership, normalized search keys, type/resource presentation and
paged rows. Activity and Explorer retain only mode/view/utility/selection state and share content.
Stable input is O(1); search/filter cost is bounded by indexed candidates and visible ranges; a
single asset delta replaces only affected folder/index/row slots.

### P1: project save is synchronous and spans persistence plus projection repair

`save_project()` captures the complete current workspace, resolves the active project, writes the
workspace and scene, synchronously imports the default scene, refreshes the Editor asset catalog and
restarts the watcher. `save_to_project()` first reads the previous workspace bytes so a failed scene
write can restore the exact prior auxiliary document. Removing that read would weaken the tested
rollback contract; it is not a justified micro-optimization.

The structural problem is command-thread ownership. Scene serialization, JSON encoding, atomic
writes, import and catalog refresh can block input/present. Optimize02 must make scene/workspace a
typed participant in the shared generation-bound save coordinator: short immutable capture,
background encode/write, explicit commit/rollback receipt, then asynchronous projection repair. A
durable save remains successful if post-commit projection repair fails, with a visible retryable
state rather than a false save failure.

### P1: workspace and preset persistence lack one bounded durable schema path

Workspace load performs `exists()` followed by `read_to_string()` and unbounded JSON decode. It
checks only the outer format version; nested layout version and structural/resource limits are not
validated here. Named preset load checks neither its format version nor stored preset name before
returning the raw layout. Named preset save uses direct `fs::write`, unlike workspace's atomic writer.

These are primarily correctness and recovery defects, but malformed/deep/large documents also form
startup memory and CPU amplification. Optimize13 owns one bounded parse -> version dispatch ->
migration -> validation -> staged restore path, plus atomic preset writes and last-known-good/
quarantine behavior. Do not add a local parse cache before that authority is defined.

### P2: path resolution is acceptable only at explicit command boundaries

`project_root_path()` resolves an existing physical path, and layout preset source resolution asks
the project authority for an existing or primary source path. These operations are reasonable for
build/export, plugin or save admission. They require counters and source guards preventing reuse in
frame/pointer/projection paths. Cache keys must use the active project generation, not stale strings.

## Reference-engine evidence

- Unreal `SAssetView.cpp:1804-1823` compares backend filters before requesting a slow full refresh;
  `2132-2140` keeps slow backend and quick frontend refresh separate. `6794-6925` consumes item
  deltas, updates/removes exact retained items and records update duration/count. This supports a
  retained catalog generation with exact delta invalidation, not rebuilding folders and all visible
  rows from an aggregate snapshot.
- Unreal `SAssetView.cpp:131-136` derives text-filter batch size from worker count and caps each
  batch. This supports cancellable/budgeted large-catalog search after normalized/indexed keys are
  retained; it does not justify moving unbounded full scans to a worker.
- Unreal `TabManager.cpp:1164-1185` coalesces persistent-layout requests and defers the write for five
  seconds specifically to avoid resize hitches. `2678-2694` preserves unknown tabs or returns an
  explicit placeholder. This supports generation-coalesced persistence and placeholder-safe restore.
- Unreal `LayoutService.cpp:244-299` saves only a non-null named layout and loads a named/versioned
  layout with default fallback; `302-334` explicitly identifies older layout versions. This supports
  version dispatch and fallback before live mutation, not accepting raw preset layout data directly.
- Unreal `PackageAutoSaver.cpp:160-174,258-340` tracks dirty/saved/undo events and admits save work
  through explicit state/timing gates. This supports the shared save coordinator direction in
  Optimize02 rather than synchronous persistence embedded in presentation or layout owners.

These references establish ownership, invalidation and persistence patterns. They do not prove
Zircon timing, allocation, energy or crash-durability parity; identical-hardware product traces and
fault injection remain required.

## Required architecture cutover

1. Runtime04 publishes registry delta/prefix queries without cloning all asset URIs. Editor09 builds
   one immutable asset-presentation generation from those deltas.
2. Folder topology/order, direct membership, normalized search keys, resource/type presentation and
   visible paging are retained. Stable snapshot input performs zero scan/sort/path allocation.
3. Activity and Explorer share content and own only surface-local state. Selection/details are
   independent generations and do not duplicate visible catalog content.
4. Optimize13 publishes one `LayoutPresetCatalogGeneration` from project-asset and config changes.
   Full and targeted host recompute read its Arc slice; preset discovery never walks the registry.
5. EditorUI08 removes preset discovery and asset compilation from shell-held aggregate snapshot and
   pane-payload collection. One frame receipt coalesces domain generations.
6. Optimize02 moves project scene/workspace persistence into the shared asynchronous save
   coordinator while preserving tested compensation and post-commit repair semantics.
7. Optimize13 supplies bounded schema migration/validation, atomic named-preset writes, staged
   restore, last-known-good and quarantine. Unknown plugin views remain bounded placeholders.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Counters for asset snapshot builds, folder/asset visits, sorts, lowercase/path allocations, row/clone bytes, resource probes, preset registry visits/URI clones/config decodes/sorts, shell hold/wait and save stage time/bytes. | current source re-read |
| M1 | Shared asset-presentation generation and surface-local snapshots with stable/delta zero-work tests. | Editor09 + Runtime04 + Optimize04 |
| M2 | Layout-preset catalog generation and event-driven invalidation; remove registry enumeration/config decode from pane recompute. | Optimize13 + EditorUI08 |
| M3 | Asynchronous generation-bound project save participant and durable projection-repair receipt. | Optimize02 + Editor17 |
| M4 | Bounded workspace/preset schema, migration, atomic write, staged restore and fault-injection suite. | Optimize13 |
| M5 | Current-source Cargo/F4 plus WPR/ETW CPU, allocation, lock and package-power matrix; RenderDoc only for rendering-visible changes. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| asset stable/delta | assets/folders `1/1k/100k`, stable/query/filter/folder/selection/resource/1% delta | stable visits/sorts/path/lowercase/row clones/resource probes `=0`; delta work near changed candidates and visible page; second-surface content clone bytes `=0` |
| preset catalog | assets `1/1k/100k`, presets `0/1/100/10k`, full/targeted recompute and config/asset delta | unrelated recompute registry visits/URI clones/config decodes/sorts `=0`; one changed generation build; menu ordering/dedup/project-over-global behavior equivalent |
| save | scene/workspace `1KiB/1MiB/1GiB`, explicit/save-all/close, edit during save, import failure, disk full | UI capture bounded; encode/write/import off input thread; generation-bound terminal receipt; previous workspace/scene preserved on pre-commit failure; projection failure is retryable post-commit state |
| persistence | current/N-1/future/corrupt/deep/wide/unknown-plugin, crash at every write/rename/restore stage | bytes/depth/nodes/windows/tabs/payload bounded before full materialization; no partial live mutation; atomic preset/workspace files; LKG/quarantine/placeholder parity |
| product | F4 cold/warm/idle, asset/search/preset/layout/save storms, 31 runs | WPR/ETW CPU, allocation, lock hold/wait, input-to-pixel p50/p95/p99, RSS and package power on identical hardware/assets/settings; artifacts only on D/E/F |

RenderDoc is required only if the cutover changes UI/render resources, draw order or pixels. It
proves GPU event/resource and pixel parity, not CPU ownership, filesystem stalls, locks or power.

## Static gates executed

- Read 16/16 production files and four focused test files in full; reproduced the line/byte/test
  counts and both current-source fingerprints above.
- Traced aggregate asset snapshot construction, full/targeted pane recompute, Runtime registry URI
  enumeration, project open/save, workspace rollback and named layout preset save/load/import.
- Read the cited Unreal retained asset update, layout coalescing/version fallback and dirty-save
  admission source ranges.
- Preserved foreign changes in `editor_project_document.rs`, `editor_project_document_save.rs`,
  `layout_preset_assets.rs`, `project_root_path.rs`, `document_roundtrip.rs` and
  `asset_workspace.rs`.
- No Cargo lane, F4 launch, WPR/ETW, package-power or RenderDoc capture was run. Dynamic acceptance
  remains pending; RenderDoc is not yet applicable because no rendering-visible source changed.

## Completion rule

This module remains pending until M0-M5 pass against a current source fingerprint. Static review,
small semantic tests, downstream equality caches or a single warm timing are not acceptance. No
milestone commit or WeCom completion message is permitted before the quantified product evidence is
available.

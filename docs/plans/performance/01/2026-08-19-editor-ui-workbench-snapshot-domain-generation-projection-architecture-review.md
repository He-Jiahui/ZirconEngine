---
related_code:
  - zircon_editor/src/ui/workbench/snapshot
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
  - zircon_editor/src/ui/workbench/model
tests:
  - zircon_editor/src/tests/workbench/chrome_snapshot
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editing/history.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/performance/01/2026-08-14-resource-query-metrics-ownership-and-index-gate.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/InvalidateWidgetReason.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SDetailsViewBase.cpp
  - dev/UnrealEngine/Engine/Source/Developer/OutputLog/Private/SOutputLog.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor UI Workbench snapshot domain generation and projection architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for aggregate snapshot work under the shell mutex, asset surface duplication and
  layout/model re-materialization; P1 for Inspector schema/value projection, console replacement and
  discarded diagnostics projection.
- Accounting: retain `zircon_editor/src/ui/workbench/snapshot/**` in `pending.md`. Do not add the
  module to `review.md` before domain generations, stable zero-work gates and current-source product
  traces pass.
- Code disposition: no Rust source changed. The Editor source tree is owned by an active session;
  `console_output_snapshot.rs` and the focused `asset_workspace.rs` test contain foreign changes.
  The implementation owner must re-read and re-hash current source before editing.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/snapshot/**` | 39/39 | 2,110 | 4 in-module | 71,666 | `2eb3f12ccb7230794fe8829d02aac243c04e505be6ab38c94ed4de5d9fef241c` |
| focused snapshot/asset/hierarchy tests | 7/7 | 1,102 | 17 | 41,935 | `68c75daa35216b8674b675b730f4dbac2c282d0f78c9b77391929c6ccac3ee1a` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 39 production-scope
files and all seven focused test files were read in full. Production grouping is asset DTOs 12 files
/ 224 lines, data projection 14 / 1,573, root facade 1 / 29 and Workbench projection 12 / 284.

## Module acceptance record

| module | files/lines | current-source performance verdict |
|---|---:|---|
| asset snapshots | 12 / 224 | DTOs own nearly every string/vector. The producer builds one full Activity surface and deep-clones it into Explorer; a later registry projection then rewrites visible type metadata in both copies. |
| editor data/chrome | 8 / 1,218 | One aggregate request enters World, projects Inspector/assets/history/logs and then reconstructs layout maps and owned tabs. Bridge diagnostics are cloned into `EditorDataSnapshot` but discarded by `EditorChromeSnapshot`. |
| scene hierarchy | 4 / 268 | Positive boundary: hierarchy rows share the runtime artifact through `Arc`; generation-checked sparse fragments carry exact changed rows and explicit reflow. Selection overlay still rebuilds a `BTreeSet` on every aggregate snapshot. |
| console output | 1 / 337 | The final object is bounded and Arc-backed, but product construction formats all matching Activity Log rows before tail bounding. The private state console is built first and then overwritten. |
| Inspector projection | 1 / 87 plus builder | Dynamic component/schema/value objects are cloned under World access; reflected fields are searched linearly once per schema field, giving `O(F^2)` field matching per component. |
| Workbench layout snapshots | 12 / 284 | Every full build allocates recursive split boxes and clones tab title/icon/host/JSON/template payloads. Content kind is re-derived from descriptor strings rather than retained with the descriptor generation. |
| focused tests | 7 / 1,102 | Semantic output and 1,000-node hierarchy sharing are covered. Asset tests use five records and chrome tests use tiny layouts; no stable allocation/build-count or large-domain test exists. |

## Structural bottlenecks

### P0: full refresh crosses every presentation domain beneath one shell lock

`refresh_reflection_for_shell()` and `chrome_snapshot()` hold `WorkbenchShellStateData` while they
rebuild active capabilities, Inspector customization and field-editor containers, call the complete
`EditorState` snapshot, project asset types, rebuild Activity Log output, clone current layout/view
registries and construct `EditorChromeSnapshot`. Full reflection continues through command context,
`WorkbenchViewModel`, reflection model, route registration and final reflection snapshot before the
shell lock is released.

The retained host has useful paint-only, view-only, Workbench-projection and shell-content fast
paths. The slow path remains monolithic: a status, Inspector, asset or unrelated presentation change
can rebuild scene selection overlay, assets, logs and stable layout together. This is PERF-MVP-099's
domain-generation defect. The fix is not another aggregate cache. EditorUI08 must publish immutable
domain generations and one coalesced frame receipt; each consumer reads only the generations its
surface requires, and projection/publish work runs outside the shell lock.

### P0: one asset projection performs full catalog work and then deep-copies its result

`AssetWorkspaceState::build_snapshot()` lowercases the query, rebuilds and sorts the complete folder
tree, filters the catalog, clones every visible asset field and performs one resource locator lookup
per visible asset. `build_surface_snapshots()` then calls that once and executes `activity.clone()`;
the clone duplicates folder rows, visible asset rows, diagnostics, selection details, references and
subassets merely to change `surface_mode`, `view_mode` and `utility_tab`.

Afterward `project_asset_type_registry_for_shell()` walks both duplicated visible-asset vectors,
parses type IDs and recreates owned presentation strings. The existing PERF-MVP-095 result correctly
removed a second catalog scan/sort, but its current acceptance misses the still-linear second-surface
clone and duplicate registry rewrite. PERF-MVP-102 correctly removed full snapshots from stable
pointer paths; it does not make slow-path projection cheap.

Editor09/Optimize04 must own a shared immutable asset content generation: catalog rows, normalized
search keys, folder topology, resource status and type presentation are published once. Activity and
Explorer carry only small surface-local state and shared page/range handles. Selection and one asset
delta replace only their slots. An Editor consumer cache must not duplicate Runtime04 resource truth.

### P0: stable layout is repeatedly converted through three owned representations

The manager first returns owned descriptor and instance vectors. `EditorChromeSnapshot::build()`
collects them into two new hash maps, recursively allocates `DocumentWorkspaceSnapshot` boxes and
clones every tab payload. `WorkbenchViewModel::build_with_contributions_and_context()` then calls
`active_page_snapshot()`, which deep-clones the active page and its recursive workspace, clones the
drawer map, and creates another set of tab models with cloned IDs, titles and icons.

The downstream `HostChromeProjectionCache` can preserve final Slint models when value equality
matches, but it runs after this upstream allocation. Stable layout generation must therefore make
the snapshot and model stages zero-work, not merely let the final adapter rediscover equality. One
canonical layout generation should retain descriptor content kind, instance payload and tab geometry
identity; projections borrow or clone `Arc` handles. A data-only invalidation must not visit layout.

### P0: aggregate Editor snapshots ignore existing hierarchy delta ownership

The scene hierarchy path is the strongest part of this module. `SceneEntries` shares the runtime
`WorldInspectionArtifact` allocation, and `SceneInspectionHierarchyFragment` validates exact rows,
generation and structural/reflow rules. The retained bridge applies sparse row/selection changes and
uses a complete view only for explicit reflow, filtering or generation gaps.

The aggregate `EditorState::snapshot()` still recreates a selected-entity `BTreeSet` and a
`SceneEntries` wrapper whenever any other snapshot domain changes. Full chrome/reflection rebuilds
can therefore bypass the practical benefit of the sparse publication even when hierarchy is stable.
EditorUI08 should consume the already published hierarchy/selection generations directly. Stable
unrelated changes must produce zero hierarchy filtering, sorting or overlay allocation.

### P1: Inspector reflection clones schemas/components and matches fields quadratically

For the selected entity, `dynamic_components_for_entity()` clones every dynamic component ID, JSON
value and descriptor, then sorts the result. `reflect_schema()` clones the complete registration.
`reflect_fields()` materializes another owned field vector. The projection iterates visible schema
fields and linearly searches that response for each one, so a component with F fields performs
`O(F^2)` name comparisons before formatting and sorting properties.

Repeated calls through `customization.and_then(...)` also re-walk the same optional surface. A local
hash map would reduce one symptom, but the target remains Optimize05's versioned Inspector session:
Runtime reflection publishes schema slot/order and a field-value generation; the Editor consumes
typed slots, dirty paths and visible ranges. Schema/customization identity is retained across value
changes and large property lists are virtualized. No full World-owned component clone is required
for an unrelated chrome refresh.

### P1: console data is built twice and final tail bounding occurs too late

`EditorState::snapshot()` first clones its private bounded console output. Product host paths then
replace it with `activity_log_console_output()`, which snapshots all matching log records, constructs
activity views, formats/join all row text and builds level/jump arrays. Only
`ConsoleOutputSnapshot::activity()` applies the 256-logical-line tail, potentially allocating new
Arc slices after the broader work has already happened.

This is the same Optimize11 authority defect recorded in the state review. One diagnostic journal
must publish a bounded window/cursor generation. Stable frames perform zero scan/format, append work
is near the delta, filtering is cancellable/indexed, and the hidden private console is deleted.

### P1: chrome construction pays for data that it cannot expose

`EditorState::snapshot_with_inspector_customizations()` clones `EditorBridgeDiagnosticsSnapshot`,
including interface/status/diagnostic strings. `EditorChromeSnapshot::build()` moves every other
`EditorDataSnapshot` field but has no bridge-diagnostics field, so chrome/reflection slow paths pay
for a value they discard. Similar scalar callers already misuse the aggregate snapshot for
`project_path` as recorded in the state review.

Do not add a skip flag to the monolith. Split typed read models by domain. Runtime diagnostics reads
its generation only when the diagnostics view is visible; project identity, status, layout, assets,
Inspector and render each have narrow accessors/generation handles. Add a source guard that prevents
aggregate presentation snapshots from command and unrelated domain paths.

## Reference-engine evidence

- Unreal `InvalidateWidgetReason.h:14-67` distinguishes Layout, Paint, Volatility, ChildOrder,
  RenderTransform, Visibility, attribute registration and Prepass, explicitly calling Layout the
  expensive choice when only redraw is required. `SlateInvalidationRoot.cpp:299-339,1281-1370`
  queues invalid widgets by reason and builds a fast-path update list. This supports typed domain
  invalidation and exact dirty consumers rather than a full Workbench snapshot for every
  presentation change.
- Unreal `SAssetView.cpp:1800-1827,2132-2140` compares backend filters before requesting a slow full
  refresh and exposes a separate quick frontend refresh. Its list/tile widgets retain item identity;
  visible rows and relevant thumbnails are maintained by the view instead of deep-copying a complete
  catalog into two surfaces. This supports shared asset content generations plus surface-local state.
- Unreal `SDetailsViewBase.cpp:709-755,968-975,1290-1323` retains root property nodes/maps and defers
  force refresh to the next tick; its source comment says the deferral avoids multiple refreshes
  locking the editor for minutes in one frame. This supports a retained Inspector tree and
  coalesced schema/value invalidation, not schema/component reconstruction inside general chrome.
- Unreal `SOutputLog.cpp:940-1085` stores `NextPendingMessageIndex`, moves pending messages under a
  short critical section, reserves only the pending delta and appends rows from that cursor. This
  supports one journal/window owner and incremental append instead of formatting all rows before a
  tail cap.

These are source-level ownership and invalidation references. They do not establish Zircon timing,
energy or memory parity; identical-hardware product traces remain mandatory.

## Required architecture cutover

1. EditorUI08 replaces aggregate chrome ownership with immutable project/status, hierarchy,
   Inspector, asset, log, layout and render generations plus a coalesced frame receipt.
2. Projection work runs after short shell access. The shell lock captures generation handles and
   commits receipts; it does not cover World reflection, asset/log scans, model construction or
   reflection publication.
3. Editor09/Optimize04 publishes one asset content generation with normalized keys, folder topology,
   resource/type presentation and paged visible rows. Activity/Explorer share content and retain only
   small local state.
4. Layout descriptors retain content kind and shared payload/template identity. Stable layout does
   not rebuild hash maps, recursive boxes, active-page copies, drawer maps or tab models.
5. The retained hierarchy fragment generation becomes the sole product hierarchy projection.
   Aggregate snapshots do not reconstruct stable hierarchy selection overlays.
6. Optimize05 publishes retained typed Inspector schema/value/visible-row generations and removes
   component/schema/value cloning and quadratic field-name matching from chrome.
7. Optimize11 removes the private console and publishes journal cursor/window deltas. Tail bounding
   happens before formatting/materialization.
8. Runtime diagnostics and scalar command state use narrow generation accessors. Chrome cannot clone
   data it discards.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Counters for full editor/chrome/model/reflection builds, shell hold/wait, per-domain visits, asset rows/clones/registry rewrites/resource probes, layout payload bytes, Inspector component/schema/field comparisons and log rows/formatted bytes. | current source re-read |
| M1 | Domain generation/receipt contract and short-lock capture; unrelated domain changes have exact zero-work assertions. | EditorUI08 + Runtime09 |
| M2 | Shared asset content generation and surface-local state; no second-surface deep clone or duplicate type rewrite. | Editor09 + Optimize04 + Runtime04 |
| M3 | Shared layout/descriptor/instance generation through chrome and model; no stable recursive/tab/payload reconstruction. | EditorUI08 |
| M4 | Retained Inspector schema/value slots and single journal window; remove quadratic field matching and double console build. | Optimize05 + Optimize11 |
| M5 | Current-source Cargo, F4, WPR/ETW allocation/lock/CPU/power matrix and conditional RenderDoc parity for any rendering-visible cutover. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| domain isolation | status/layout/selection/Inspector/asset/log/render change alone and coalesced, 1/1k events | per unrelated domain visits/builds/clones `=0`; each changed domain builds at most once per frame/generation; projection outside shell lock |
| assets | catalog/folders/visible `1/1k/100k`, Activity/Explorer, stable/search/filter/selection/1% delta | one content projection; second-surface content clone bytes and duplicate type rewrites `=0`; stable scan/sort/resource probes `=0`; visible paging bounded |
| layout | tabs/descriptors/windows `1/100/10k`, stable/status-only/layout delta | stable map/box/page/drawer/tab/JSON clone bytes `=0`; one changed slot replaces only its generation; route/focus/placeholder parity |
| hierarchy | entities `1/1k/100k`, selection `0/1/10k`, stable/row/topology/filter/gap | stable and unrelated change hierarchy filter/sort/overlay alloc `=0`; row patch near delta; full reflow only for declared structural/filter/gap reasons |
| Inspector | components/fields `1/100/10k`, stable/value/schema/customization/selection | stable schema/component clone and field compare `=0`; value update near dirty visible slots; no `O(F^2)` field-name matching; mixed/stale parity |
| logs/diagnostics | records `1/100k/1M`, append/filter/stable, diagnostics hidden/visible | one journal owner; tail/page bound before format; stable scan/format `=0`; hidden bridge-diagnostics clone `=0` |
| product | F4 cold/warm/idle, status storm, layout storm, large assets/Inspector/log, 31 runs | WPR/ETW CPU, allocation, lock hold/wait, generations, RSS, input-to-pixel p50/p95/p99 and package power on identical hardware/assets/settings; artifacts on D/E/F |

RenderDoc is required only when the domain cutover changes UI/render resources, draw order or pixels.
It verifies GPU event/resource and pixel parity; it does not replace WPR/ETW for snapshot CPU,
allocation, locks or power.

## Static gates executed

- Read 39/39 production-scope files and seven focused tests; reproduced 2,110 production lines,
  71,666 bytes, four inline tests, 17 focused tests and fingerprint `2eb3f12ccb72...`.
- Traced both full construction paths from shell access through EditorData, asset registry/log
  replacement, Chrome, WorkbenchViewModel and reflection/retained host consumers.
- Traced asset surface construction through folder/filter/resource/type projections and confirmed
  one full build followed by one deep surface clone and two registry rewrite passes.
- Traced hierarchy artifact sharing and sparse fragment publication through the retained bridge;
  confirmed it is a positive delta boundary that aggregate snapshots still revisit.
- Read the cited Unreal Slate invalidation, Content Browser, PropertyEditor and Output Log primary
  sources plus current Optimize01/04/05/11 and resource-query owner records.
- `rustfmt --edition 2021 --check` passed for all 39 production-scope files and seven focused tests.
  Scoped `git diff --check`, 29/29 routed-path existence and the 329-plan coordinator audit with
  zero errors/warnings passed. The production fingerprint remains `2eb3f12ccb72...`.
- The documentation convention gate reports zero violations owned by these two records. The
  unrelated repository baseline remains 692 violations across 242 documents out of 2,725 scanned.
- Dynamic Cargo, scale counters, F4 launch, WPR/ETW, package power and rendering-visible RenderDoc
  evidence remain pending. This is not an accepted milestone, so no commit or WeCom notification is
  due.

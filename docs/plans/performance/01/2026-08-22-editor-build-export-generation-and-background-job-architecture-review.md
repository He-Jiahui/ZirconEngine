---
title: Editor build and export generation and background-job performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/** and build_export_wizard_panel.rs
priority: MVP-P0 editor build and export workflow
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate SListView and UATHelper
---

# Goal

Keep build/export catalog, job state and wizard presentation in separately versioned generations.
Stable catalog and wizard generations must cross the host boundary by shared identity, perform no
filesystem polling, and reuse retained rows. Only visible target rows may become host nodes. Export
execution must remain on the existing editor job system while bounded events patch exact stage,
output and control rows on the UI thread.

## Reviewed source

- owner files: build/export final pane conversion, target identity/actions/metrics/node/row
  construction, behavior tests and retained wizard conversion
- Rust files: 9/9
- current lines: 1,234
- current bytes: 44,172
- joined current source-bytes SHA256:
  `3f5e6ad78641991fc4b1dce04be4ed2232a673f40b02d5f69a05084d9bd3b791`
- pre-M1 lines: 1,210
- pre-M1 bytes: 43,421
- joined pre-M1 source-bytes SHA256:
  `5f6b5d9fa762e1f6213992b768c71ea4de4e87c45e703898aebcb0a436e012d9`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `build_export/mod.rs` | 73 | 2,685 | `47f8dc9c96ad71fbc993986b0c26a23758a3f74a2b92ed3ed31fe3fbee1c280c` |
| `build_export/target_rows/actions.rs` | 118 | 3,954 | `2a45da052e3d6f46095c6c3ca1dd20799498ba71341052431e15c71459c5fd34` |
| `build_export/target_rows/identity.rs` | 43 | 1,060 | `cfba539c74bbe21ce20c242d549af8a2f44457cebc7f94b1fc1d92af49598a6f` |
| `build_export/target_rows/metrics.rs` | 9 | 541 | `37be2f6e5dd97f54ef1febbb4cf5ee9de9872d9c7a93156b34d67ef8e3be6fe2` |
| `build_export/target_rows/mod.rs` | 108 | 3,435 | `50d6804ed004e433c9013f8c28c0e01f9c0fdaad04288eceead3eeb9e7a6f65b` |
| `build_export/target_rows/node.rs` | 18 | 574 | `0ea00a087a410bb24010390817641953a13e9a2815ab51e990277a2f33535604` |
| `build_export/target_rows/row.rs` | 127 | 4,307 | `c364572be2d478af38851e73007531a9168cdb39d089495298140038a387476d` |
| `build_export/tests.rs` | 434 | 16,323 | `5894dda1072935cab599b8299ecb4ba94b38aafea40e37b94ba5e228ca0cc0fb` |
| `build_export_wizard_panel.rs` | 304 | 11,293 | `f12e906b71b198680277bce9f26e95fddca58b2b873e908f2f9b60b940645e96` |

All nine files were read in full. Production ownership was followed through build/export payload
visibility, source-identity and rendered caches, preset loading, overlay application, wizard session
and `EditorJobSystem` submission, template/retained projection and host recompute/native-presenter
call sites. These related files are not counted in the 9/9 owner total.

## Existing foundations to retain

Preset parsing and target construction already have a base revision cache. Job overlays have an
independent generation and the rendered pane is cached by `(base_revision, overlay_generation)`.
The active wizard is a typed `ExportWizardPanelViewModel`, and execution is submitted through
`EditorJobSystem`; cancellation and bounded event polling update the view model instead of running
the export pipeline synchronously on the UI thread. The wizard template runtime is initialized once.
These are correct foundations. The required work is to move source change notification and retained
item identity through the remaining presentation boundary, not to add another worker pool or cache.

## Structural findings

### P0: a cache hit still polls every export source from the UI recompute path

`BuildExportProjectionCache::cached_base` calls `capture_source_identity` on every lookup. That path
performs metadata reads for the project manifest, export directory and every preset file before it
can reuse the cached target projection. A full host recompute collects the visible BuildExport pane
payload once, then native-window synchronization can call `build_export_pane_data` again for the same
generation. With P preset files and an active native presenter this permits up to `2(P + 2)` metadata
queries in one full recompute even when no export source changed.

Replace UI-time filesystem validation with a project/export source generation published by the
asset/file-watcher owner. A host recompute must compare an integer receipt only. A missed or
overflowed watcher event may schedule one explicit rescan outside the presentation phase; it must
not restore steady-state metadata polling.

### P0: cached wizard data is cloned and rebuilt through two retained representations

The payload cache returns a cloned `BuildExportPaneViewData`, including a cloned optional wizard
view model. Final conversion clones it again, projects the template, creates and lays out a shared
surface, builds a retained host projection, applies all state, appends slot nodes and then converts
every retained node into the wide `TemplatePaneNodeData` representation. Lists are cloned into new
models, drag payloads are joined and many strings are copied even when the wizard generation and
content size are unchanged.

The wizard session must publish a retained projection generation keyed by template, plan/job-state
and layout receipts. Pane conversion should reference that generation. Job events patch exact stage,
terminal-output and control items; geometry changes patch layout only. Delete the parallel wide-node
conversion after paint, hit, accessibility and profiling consume the same retained projection.

### P0: every target is represented as a copied DTO and nine host nodes

Even when the wizard path succeeds and target row nodes are not needed, final pane conversion maps
every target into another DTO and clones ten strings. The fallback target list retains that DTO model
and additionally expands every target into five base nodes plus four button nodes. Each target also
duplicates four allocated action IDs between the row action model and its buttons. There is no
visible range, so construction, hit artifacts and storage grow with all T targets rather than visible
rows V.

Publish one stable `BuildExportTargetItem` generation with immutable target identity and typed action
state. Generate/reuse only visible row widgets and let row actions reference the item identity. The
pane must choose wizard or target-list presentation before constructing final models so the inactive
representation costs zero.

### P1: target identity and row construction repeat bounded but avoidable work

The target-list fallback collects all targets, normalizes every platform in a first pass, normalizes
it again while building target IDs, clones every target ID into a count map and performs a fourth
occurrence pass. Row and action node vectors grow from zero despite the fixed four-action/nine-node
shape. This is not the structural bottleneck, but it is behavior-preserving M1 work: normalize each
platform once, retain duplicate-platform/profile/occurrence semantics, use a fixed four-action array
and reserve exact local node capacity.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/UATHelper/Public/IUATHelperModule.h`
- `dev/UnrealEngine/Engine/Source/Editor/UATHelper/UATHelperModule.cpp`

Unreal `SListView` starts generation at the current scroll offset and stops when the available view
area is filled. Its widget generator keeps item-to-widget identity, reuses an existing row when it is
still visible, and releases rows not seen in the generation pass (`SListView.h:978-1067`,
`1524-1690`). `STableViewBase::RequestLayoutRefresh` coalesces a pending refresh flag instead of
immediately regenerating rows (`STableViewBase.cpp:1393-1406`).

Unreal's UAT helper creates a serialized monitored UAT process, binds output/cancel/completion/failure
callbacks, launches the packager process, and dispatches UI notification updates back through graph
tasks (`IUATHelperModule.h:40-47`, `UATHelperModule.cpp:382-489`, `524-540`). Zircon's existing
`EditorJobSystem` follows the transferable execution boundary already; the missing parity is stable
item generations, coalesced refresh and visible row reuse on the presentation side.

## Target architecture

1. Publish `BuildExportSourceGeneration`, `BuildExportCatalogGeneration`,
   `BuildExportJobGeneration` and `BuildExportLayoutGeneration` as independent receipts.
2. Move manifest/export-directory/preset identity capture to the file watcher or a background rescan
   task. UI collection compares receipts only and never performs steady-state filesystem I/O.
3. Publish one shared target catalog with immutable target IDs. Overlay job state by target ID without
   cloning all catalog strings; stable catalog and unrelated job changes reuse rows.
4. Publish the wizard retained projection by template/plan/job/layout receipts. Patch stage/output/
   control items from bounded job events and remove the second wide-node representation.
5. Make the target list a virtualized item view. Paint, hit, accessibility and profiling consume the
   same visible range and retained row identity.
6. Select wizard versus list before final materialization. The inactive branch produces no copied
   DTOs, actions, nodes or hit artifacts.

Complexity targets:

- stable visible pane refresh: O(1), zero filesystem calls and zero target/wizard row reconstruction;
- one preset/source change: background O(P) rescan, one catalog generation publish;
- one job event: O(1) item lookup plus changed wizard/target rows;
- target-list scroll: O(V), where V is visible rows;
- inactive wizard/list representation: zero final nodes and zero copied target DTOs;
- final duplicate target/wizard presentation owners: zero.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| metadata calls during stable/full/scoped recompute | stable = 0 |
| base/rendered/wizard generation hits and misses | one miss per changed receipt |
| target strings/DTOs/actions/nodes copied or built | stable = 0; list scroll O(V) |
| wizard surfaces/layouts/retained/wide nodes | stable = 0; changed rows only |
| job event queue depth/drain budget and UI patch rows | bounded queue; patch changed rows |
| main-thread CPU, input-to-paint and event-to-paint | report median/p95/max |
| RSS, allocation bytes and package energy | report before/after on same fingerprint |

Matrix: targets 0/1/100/1,000; wizard slot entries 0/1/100/1,000; visible rows 1/8/32; stable
refreshes 1/1,000; one preset add/edit/delete; one job queued/running/cancel/complete transition;
terminal output bursts below/at/above the drain budget; embedded and native presenter paths. Capture
filesystem calls, generations, copies, allocations, nodes, visible visits, event queue depth, CPU,
latency, RSS and energy on one source/executable fingerprint.

Use managed Windows validation and WPR/ETW with all artifacts on D/E/F. RenderDoc is only required for
current-source pixel/draw parity of the BuildExport pane; it cannot prove filesystem, allocation or
job-event behavior.

## M1 result

Target projection now normalizes each platform once and retains the normalized key through duplicate
platform and target-ID construction. For T targets, platform-key character traversal and owned key
construction fall from 2T to T, removing T temporary `String` allocations while preserving platform,
profile and occurrence ordering. The four row actions now use a fixed array, removing one action
vector heap allocation per target. The complete target-node vector reserves 9T entries and each row
reserves its fixed nine-node shape, eliminating geometric growth reallocations under the existing
five-base-plus-four-action node contract.

M1 deliberately retains the final target DTO map, all-target node expansion, action-ID string owners,
source metadata polling and wizard dual projection. Those structural costs remain M2-M4 and must not
be hidden by the local allocation result.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add source metadata, generation hit/miss, copy/node, visible-row and event-drain counters; capture baseline. | scale-bound evidence on one fingerprint |
| M1 | Normalize target identity once, fix action cardinality and reserve exact local row capacity. | focused RED-to-GREEN contracts plus behavior parity |
| M2 | Publish watcher-owned source generation and remove UI-time metadata polling. | stable recompute filesystem calls = 0 |
| M3 | Publish shared target/job item generations and virtualize target rows. | stable rows = 0; scroll O(V) |
| M4 | Cache/patch wizard retained projection and delete the second wide-node representation. | changed rows only; one projection owner |
| M5 | Run managed scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 9/9 Rust files.
- Payload/cache, preset source, job session, recompute/native-presenter call chain and Unreal
  references: read.
- M1 source implementation: complete. Its focused performance contract moved RED 3/3 to GREEN 3/3.
- Combined owned performance contracts: passed, 26/26. Related fixture/row-patch contracts: passed,
  5/5. Changed Rust `rustfmt` and scoped diff check: passed.
- The broad editor performance-contract discovery ran 78 tests: 73 passed; three unrelated tests
  reference moved/missing files and two unrelated UI asset tests still detect `roots.clone()`.
  These foreign worktree failures were not modified or counted as M1 failures.
- Managed Rust behavior tests and M0-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; the focused command
  was rejected before Cargo launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.

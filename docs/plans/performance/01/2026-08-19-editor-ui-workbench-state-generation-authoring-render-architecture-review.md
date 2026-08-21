---
related_code:
  - zircon_editor/src/ui/workbench/state
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/core/gateway
  - zircon_runtime/src/core/framework/render/highlight_set.rs
  - zircon_runtime/src/core/framework/render/viewport_highlight_store.rs
tests:
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/editing/state
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/UserInterface/PropertyEditor/SPropertyEditorNumeric.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
  - dev/UnrealEngine/Engine/Source/Developer/OutputLog/Private/SOutputLog.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-19
---

# Editor UI Workbench state generation, authoring and render architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for World/transaction access amplification, Inspector string-form authority,
  render submission under shell lock and Play scene ownership; P1 for console/status convergence and
  small stable projections.
- Accounting: retain `zircon_editor/src/ui/workbench/state/**` in `pending.md`. Do not add it to
  `review.md` before the generation/transaction/Play/log cutovers and dynamic F4 gates pass.
- Code disposition: no Rust source changed. The active MVP00 session owns the Editor tree. Three
  scoped source files and one external test contain foreign changes; `editor_state_render.rs` changed
  while this review was running. The report discarded the disappeared highlight clone finding and
  anchored the later current source. Implementation must re-read and re-hash before editing.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/state/**` | 12/12 | 1,948 | 13 in-module | 73,168 | `d7d1c71378f719d81a00533df653c2a8d4cd1ebe77b64a39794732036995abf9` |
| focused external editing-state tests | 4/4 | 1,382 | 34 | 45,922 | `10ec7f0bb7e913c7ecde83e76bd9dd51721e9068c403a3494c7b62d0113cf267` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 12 production-scope
files and all four focused external test files were read in full. The current source includes foreign
changes in `console_history.rs`, its in-module tests, `editor_state_render.rs` and external
`state/viewport.rs`; it is not a clean-tree baseline.

## Module acceptance record

| module | files/lines | current-source performance verdict |
|---|---:|---|
| console history | 2 / 388 | Bounded to 256 logical lines, but every accepted append or filter change scans all retained messages and rebuilds the full text/level arrays. It is a hidden second authority later overwritten by Activity Log projection. |
| intent/transaction bridge | 1 / 352 | Rebuilds selection context and repeatedly crosses the authoring gateway. Batch command application inherits the per-command World lock amplification in PERF-MVP-600. |
| field updates | 1 / 101 | Dynamic-field typing reparses the string field ID, scans schema fields and performs a reflected read under World access before updating a second string draft map. |
| Play state | 1 / 107 | Captures a full owned `Scene`, selection and editor state; current product entry has already created another scene owner/source before this capture. |
| render state | 1 / 117 | Called while the Workbench shell mutex is held. Each dirty render builds/submits highlights, locks World for a full render packet and rebuilds the viewport HUD extract. |
| Inspector/selection | 1 / 388 | Applies every visible base/dynamic field to every selected node through owned string/reflection updates and repeated World command capture; selection sync reformats draft strings during gizmo movement. |
| viewport/gizmo | 1 / 323 | One pointer delta can enter World for input, preview write, transform recapture and Inspector/selection sync before render work; release adds state capture and transaction commit. |
| aggregate state/checkpoint | 1 / 144 | Inspector rollback checkpoint deep-copies selection and all string draft state, including bounded console history. `EditorState` aggregates unrelated authorities and makes full snapshot calls easy. |
| facade/helpers | 3 / 28 | Structural only; error helpers allocate owned strings on failure paths. |

## Structural bottlenecks

### P0: Inspector Apply multiplies fields, targets, World entries and commands

`apply_inspector_changes()` copies the complete selected set, then for every target rebuilds the four
base reflected updates with owned component/field strings. Dynamic drafts are read under one World
callback per target. Every base or dynamic update then calls `capture_scene_command()`, entering the
World again to create a command. The transaction engine later applies each command through the
per-command World access already recorded by PERF-MVP-600.

For N targets and K submitted fields, the Workbench layer alone performs N dynamic-preparation World
entries plus N*K command-capture entries; apply/revert adds another per-command sequence downstream.
It also applies the primary object's name, parent, translation and scale to every selected object even
when the user changed one unrelated property. Stable drafts have no dirty-path set, source generation,
mixed-value state or compare-and-set revision.

This is not fixed by reserving `commands`. Optimize05 must remove the string-form fields from
`EditorState` as authoring authority. A versioned `InspectorSession` owns typed property addresses,
target set, schema/selection/world generations, mixed values and exact dirty paths. One preflight
reads the required fields under one immutable generation, one transaction mutation lease applies the
typed batch, and one receipt publishes exact changed paths.

### P0: one gizmo pointer delta can cross World four times

During an active handle drag, `handle_viewport_input()` mutably enters World for input processing.
When a transform preview exists, `apply_gizmo_transform_preview()` enters again to write it,
`record_gizmo_transaction_step()` enters again to read the transform, and
`sync_selection_state()` enters again to read inspection artifacts, clone the selected name and
format six decimal strings. Release recaptures state and commits a command after the preview already
changed the World.

PERF-MVP-063 correctly requires transform-only before/current data, but the Workbench caller must also
stop reopening World and rebuilding Inspector strings. Editor03's interactive edit session should
hold one validated document/world lease per input batch, apply typed transform deltas, update a
generation-owned preview and commit one history record at the edge. Inspector consumes a throttled or
frame-coalesced typed delta; it does not reread and format the whole primary form per pointer event.

### P0: render submission performs cross-system work while the shell mutex is held

`EditorHostEventController::render_frame_submission()` locks the Workbench shell and calls state
submission. That path builds an owned highlight set, crosses the gateway, locks World for
`build_render_snapshot()` and rebuilds a one-command HUD extract containing new style strings and
formatted text. `build_render_snapshot()` constructs the scene render packet and interaction
overlays while both the shell mutex and the runtime World mutex can be held.

The retained host already avoids submission when render is not dirty, which is a useful gate. It does
not make changed-camera frames cheap: UI control state, selection, scene extract, interaction overlay
and HUD are still rebuilt through one shell-locked call. The target is a sealed frame generation:
short shell access captures immutable handles/generations, then scene/render preparation, gateway
delivery and UI serialization run without the shell lock. Stable sub-generations are shared rather
than rebuilt with every camera redraw.

### P0: selection highlights are canonicalized twice and equal generations replace storage

`EditorRuntimeHighlightSet::new()` collects the active selection into a `BTreeSet` and then a `Vec`.
The in-process gateway immediately passes those entities to `HighlightSet::new()`, which performs the
same `BTreeSet` canonicalization again. The dynamic-session path repeats the runtime-side sort after
FFI. `ViewportHighlightStore::submit()` rejects only older generations; an equal selection generation
still replaces the map entry under its mutex.

Therefore a camera/display redraw with unchanged selection can pay two O(S log S) sorts, two owned
entity vectors, gateway/FFI work and store mutation for S selected entities. Selection already has a
revision and an ordered `IndexSet`; publish a sealed highlight artifact only when selection,
viewport, outline policy or tint generation changes. Validate canonical order once at the owner
boundary. Equal generation plus identical attributes is a zero-work no-op.

### P0: two menu commands request full Editor snapshots for one path string

`SaveProject` reads `shell.state.snapshot().project_path`. `EnterPlayMode` performs the same full
snapshot to derive the project root. `EditorState::snapshot()` does not return a small state record:
it enters World, filters selection, projects hierarchy, builds Inspector including dynamic reflection,
projects asset surfaces, reads history and clones multiple UI fields.

Both calls happen with the shell lock already held. Commands must read a typed project/document
identity from the project session authority in O(1), not call an aggregate UI snapshot. Add a guard
that prevents command execution code from using full presentation snapshots for scalar state.

### P0: Enter Play creates multiple whole-scene owners before startup

The menu path first calls `project_scene()`, which returns `world.try_snapshot()` as an owned Scene,
then builds `PlaySceneSource`. `EditorState::enter_play_mode()` takes another full Scene snapshot for
its rollback checkpoint. The downstream process path serializes/materializes another artifact, as
already recorded by PERF-MVP-550. All of this is initiated while the Workbench shell command path is
active.

Optimize07's `PlaySessionAuthority` must capture one immutable authoring generation/artifact outside
the shell lock. Embedded/process backends share that identity; rollback retains document generation
and session ownership, not an unscoped Scene that can later overwrite another project. Serialize and
I/O are bounded, cancellable worker stages.

### P1: console/status has two stores and two full rebuild algorithms

Every accepted `EditorConsoleHistory::push_with_level()` scans the bounded deque twice and creates a
new joined String and level array. Inspector batch checkpoints deep-copy its message strings. Product
`editor_snapshot()` then replaces that result with `activity_log_console_output()`, which clones the
global log snapshot, creates views, formats/join all rows and builds levels/jump arrays before the
console window truncates. Status writes can therefore enter a hidden history that the product console
does not display.

Optimize11 already owns this defect. Delete the private console authority after migration. Status is
a small projection of the single diagnostic journal; Console consumes cursor/window deltas. Stable
frames do no log scan/format, new records append incrementally, and filter changes run a bounded,
cancellable indexed query.

### P1: current tests prove semantics but not scale or stable work

The 34 focused external tests cover selection, Play restore, gizmo rollback, highlight delivery and a
real GPU HUD pixel capture. They are valuable semantic gates. Most use the default tiny scene. The
highlight test checks sorted output once but not duplicate canonicalization or equal-generation skip;
the render test clones extract/UI 24 times as a glyph-settle fixture rather than measuring producer
allocation. Console tests preserve the hidden history. No test covers 100k selection, Inspector N*K
lock/command counts, 1M pointer deltas, large Play memory peaks or scalar menu access avoiding full
snapshot.

## Reference-engine evidence

- Unreal `LevelEditorViewport.cpp:3651-3850` starts one pending tracking transaction for a widget
  interaction, disables per-delta transaction modification overhead, records whether anything moved
  and closes the interaction at `TrackingStopped()`. This supports one interactive edit owner and one
  final history record, not repeated full state capture and World entry per delta.
- Unreal `SPropertyEditorNumeric.h:488-609` represents multi-value state as an unset optional,
  returns early for equal slider values, applies interactive steps as `InteractiveChange |
  NotTransactable`, and owns one `FScopedTransaction` from slider begin to end. Its per-object value
  path updates each target deliberately. This supports typed dirty property sessions and one
  continuous transaction rather than reapplying every form string.
- Unreal `PlayLevel.cpp:2332-2395` creates a dedicated PIE world/package with a PIE instance identity
  while retaining the Editor world. Zircon should keep its Rust/process isolation, but the reference
  rejects an unscoped rollback Scene that can overwrite whichever project happens to be current.
- Unreal `SOutputLog.cpp:940-1085,1606-1612` queues new messages, advances
  `NextPendingMessageIndex`, appends only pending rows on the next tick and reserves for that delta.
  Filtering may require broader work, but steady append is incremental. This supports Optimize11's
  journal cursor/window design over rebuilding two full string histories.

These sources establish transaction and ownership shape. They do not establish Zircon timing or
power parity; same-hardware product traces remain required.

## Required architecture cutover

1. Editor01/03 publishes a `SceneDocumentGeneration` and top-level mutation lease. Workbench code
   receives immutable read handles and typed commit receipts; it does not repeatedly call the gateway
   and World mutex inside one logical action.
2. Optimize05 replaces `name_field`, `parent_field`, transform/scale strings and dynamic draft map
   with a versioned `InspectorSession`. Submit exact typed dirty paths across one target set and one
   transaction.
3. Optimize03 replaces the current gizmo capture path with one `InteractiveEditSession` carrying
   frozen targets/basis, initial/current transform deltas and one edge commit/cancel. Inspector and
   render consume its generation delta.
4. EditorUI08 seals one `EditorFrameInput` from short shell access. Scene/render preparation,
   highlight gateway delivery and HUD/snapshot serialization occur outside the shell lock and share
   stable sub-generations.
5. Publish highlight artifacts only when selection/viewport/attribute generation changes. Remove
   duplicate canonicalization and make equal generation/attributes a no-op at producer and store.
6. Replace aggregate `state.snapshot()` scalar reads with project/document authority getters. Add a
   source guard and counter for full presentation snapshots requested from command execution.
7. Optimize07 captures one project-scoped Play artifact/session record and shares it across backends.
   Delete the unscoped Scene rollback owner after migration.
8. Optimize11 moves status and Console to one diagnostic journal/cursor model, then deletes
   `EditorConsoleHistory` and its rollback checkpoint field.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Add counters for shell/World/gateway/transaction entries and hold/wait, Inspector targets/fields/commands, selection sync formats, highlight sorts/bytes/submits/store replaces, full snapshots, scene owners and console rebuild bytes. | current source re-read |
| M1 | Remove aggregate snapshot reads from Save/Play; add O(1) project/document identity. Publish sealed frame inputs outside long shell work. | Editor01 + EditorUI08 |
| M2 | InspectorSession typed dirty paths and one transaction mutation lease; delete string-form authoring authority. | Optimize05 + Runtime reflection generation |
| M3 | InteractiveEditSession uses one input-batch World lease and one final transform history record; no per-delta Inspector full sync. | Optimize03 + M2 |
| M4 | Generation-owned highlight artifact and stable HUD/render sub-generations; duplicate sort and equal-generation replace removed. | M1 + Runtime09 |
| M5 | Single project-scoped Play artifact/session authority and hard removal of the Workbench Scene checkpoint. | Optimize07 |
| M6 | Single diagnostic journal/status/Console delta, then current-source Cargo, F4, WPR/ETW, power and conditional RenderDoc acceptance. | Optimize11 + M0-M5 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| Inspector | targets `1/100/10k`, fields `1/100/10k`, dirty `0/1/10%/100%`, mixed/missing/stale | World read/mutation lease at most one each per commit; commands/work near dirty targets*paths; unchanged paths write `=0`; failure atomic; no string field authority |
| gizmo | deltas `1/1k/1M`, targets `1/100/10k`, commit/cancel/capture-loss | World access bounded per input batch, not per helper; stable name/String formats `=0`; one final history record; cancel restores byte-equivalent committed generation |
| render shell | stable/camera/selection/scene/HUD changes at 60/120/240 Hz | shell lock hold excludes scene packet, World callback, gateway/FFI and serialization; stable sub-generation rebuild `=0`; changed frame one sealed input |
| highlight | selection `0/1/1k/100k`, stable/camera/change, in-process/dynamic | canonicalization at most once per changed selection; stable/equal generation sorts, alloc, gateway submit and store replace `=0`; output order/pixels unchanged |
| scalar command | Save/Play repeated `1/1k`, scene `1/1k/100k` entities | full Editor snapshot, hierarchy/Inspector/asset/log projection and World read `=0`; project identity lookup O(1) |
| Play | scene `1 KiB/64 MiB/1 GiB`, embedded/process, start/cancel/failure/project switch | one authoritative scene artifact identity; UI-thread serialization/I/O `=0`; shell/World lock hold independent of bytes; old session cannot mutate new project |
| logging | records `1/100k/1M`, append/filter/clear, visible rows `20/200` | one journal owner; stable scan/format `=0`; append work near delta; query/page bounded; hidden console history `=0` |
| product | F4 cold/warm/idle/gizmo/Inspector/Play/log storm, 31 runs | WPR/ETW CPU, allocation, lock, gateway, queue, RSS, input-to-pixel/commit p50/p95/p99 and package power on identical hardware/assets/settings; artifacts on D/E/F |

RenderDoc is required for the render/highlight/HUD cutover because it can change overlay order,
resources and visible output. Capture draw/event/resource and pixel parity on the same scene/settings.
WPR/ETW and explicit counters remain the authority for locks, CPU work and allocation.

## Static gates executed

- Read 12/12 production-scope files and 4/4 focused external tests; reproduced 1,948 production
  lines, 73,168 bytes, 13 inline tests, 34 external tests and current fingerprint `d7d1c713...`.
- Traced Inspector Apply through reflected reads, command capture and transaction application; traced
  gizmo pointer input through preview, transform recapture and selection/Inspector sync.
- Traced render submission from retained dirty gate through shell lock, highlight gateway/runtime
  store, World packet build and HUD extract. Confirmed duplicate highlight canonicalization and
  equal-generation replacement.
- Traced Save/Play scalar project-path reads through the full state snapshot; traced Play through two
  owned Scene snapshots before downstream materialization.
- Read the cited Unreal LevelEditorViewport, PropertyEditor, PlayLevel and OutputLog primary sources
  plus current Optimize03/05/07/11 owner plans.
- `rustfmt --edition 2021 --check` passed for all 12 production-scope and four focused external test
  files. Scoped `git diff --check`, 34/34 routed-path existence and
  `python -m tools.session_coordinator --repo-root . --json plan audit` passed. The production
  fingerprint remains `d7d1c713...` after the documentation write.
- The documentation convention gate reports zero violations owned by these two records. The
  unrelated repository baseline remains 692 violations across 242 documents out of 2,723 scanned.
- Dynamic Cargo, scale counters, F4 launch, WPR/ETW, package power and RenderDoc evidence remain
  pending. This is not an accepted milestone, so no commit or WeCom notification is due.

---
related_code:
  - zircon_editor/src/ui/workbench/state
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/UserInterface/PropertyEditor/SPropertyEditorNumeric.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
  - dev/UnrealEngine/Engine/Source/Developer/OutputLog/Private/SOutputLog.cpp
---

# Protected plan routing: Workbench state generation, authoring and render

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize03/05/07/11 and numbered owner plans are
protected or foreign dirty. The active MVP00 session owns the Editor source tree, and scoped source
changed during this review. This record routes the current 12/12-file evidence without editing those
authorities. Detailed evidence source:
`2026-08-19-editor-ui-workbench-state-generation-authoring-render-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-600 and PERF-MVP-063

Add the Workbench amplification chain. One gizmo pointer delta can enter World for input, preview
write, transform recapture and selection/Inspector string sync; release adds `NodeEditState` capture
and transaction application. Inspector Apply enters World once per target for dynamic preparation,
again per target-field command capture, then inherits per-command apply/revert World locks.

Required target: one document/world read generation and one top-level mutation lease per logical
input batch/transaction. Transform drag owns typed before/current transform only and commits one
history record. Per-delta name/parent/transform-string formatting and stable `NodeEditState` fields are
zero.

### PERF-MVP-567 and PERF-MVP-456

Add the current string-form Inspector authority. `EditorState` stores primary name/parent/transform/
scale and dynamic values as Strings, applies all fields to all selected targets, and rereads reflected
schema/value on typing and submit. Replace it with Optimize05's versioned `InspectorSession`, typed
property slots, mixed values and exact dirty paths over the shared inspection generation.

### PERF-MVP-096 and PERF-MVP-333

PERF-MVP-096's controller-clone fix remains valid. Current render submission still runs beneath the
Workbench shell mutex, enters World for a full render packet and rebuilds HUD data. Highlight sets are
canonicalized through a `BTreeSet` in Editor and again in Runtime; equal selection generations still
cross the gateway and replace runtime storage.

Required target: sealed frame inputs captured under short shell access; packet/gateway/UI work outside
the lock. Stable highlight generation performs zero sort/allocation/gateway/store work. Overlay
generation and GPU resources remain owned by PERF-MVP-333/Render owners.

### PERF-MVP-076 and PERF-MVP-099

Add the two scalar-command full snapshots. Save Project and Enter Play call the complete
`EditorState::snapshot()` only to read `project_path`, causing World hierarchy/Inspector/reflection,
asset, history and console projection under the shell lock. Commands must query O(1) project/document
identity; full presentation snapshot requests from command execution must be zero.

### PERF-MVP-550

Add the exact multi-owner sequence. Enter Play first clones Scene through `project_scene()` for
`PlaySceneSource`, then `EditorState::enter_play_mode()` clones Scene again for rollback before the
backend serializes/materializes its artifact. Replace all with one project-scoped immutable Play
generation shared by rollback/session/backends outside the shell lock.

### Optimize11 performance handoff

No new PERF ID is needed for console. The scoped evidence directly confirms E-LOG-P1-12/P1-19/P1-47:
private `EditorConsoleHistory` rebuilds complete text/levels on append, checkpoints deep-copy it, and
product snapshots overwrite it with another full Activity Log projection. Route implementation and
acceptance to Optimize11's single journal/cursor/window plan.

## Requested Optimize and owner updates

### Optimize03 + Editor03/05

Own one `InteractiveEditSession` and one transaction mutation lease for gizmo input. Selection and
Inspector consume typed generation deltas; pointer moves do not reopen World or rebuild primary form
strings. Multi-target transform, cancel/capture-loss and one-record commit remain semantic gates.

### Optimize05 + EditorUI08

Delete Workbench string-form authoring fields after `InspectorSession` migration. One typed request
contains target set, document/world/schema/selection revisions and exact dirty property paths. Full
state snapshot is not an Inspector service.

### Optimize07 + Editor04

Replace `EditorPlaySession.scene` and the earlier `project_scene()` clone with one project-scoped
Play artifact/session authority. Worker serialization/I/O is bounded and cancellable; late completion
cannot restore a Scene into a different project.

### Optimize11 + EditorUI08

Status becomes a small projection of one diagnostic journal. Console consumes windowed cursor deltas;
delete `EditorConsoleHistory`, the Inspector checkpoint copy and the full Activity Log rebuild path
after migration.

### Editor01 + Runtime09

Publish one sealed authoring/render frame generation. Shell lock only captures handles and receipts;
World projection, highlight gateway/FFI, HUD serialization and fanout occur outside it. Canonicalize
selection once and reject equal generation/attributes before runtime store mutation.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/workbench/state/**` with
  12/12 files, 1,948 lines, current fingerprint `d7d1c713...`,
  `source_recheck_required=true`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require one World/transaction lease per logical action,
  Inspector typed dirty paths, stable highlight zero-work, scalar command no-full-snapshot, one Play
  artifact, single diagnostic journal, current-source Cargo, F4, WPR/ETW, power and RenderDoc parity.
- Keep protected indexes module-level and concise; detailed evidence remains in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize03 + Editor03/05 | pointer batch World access bounded; per-delta stable String/NodeEditState clones `=0`; one final transaction record; cancel parity |
| Optimize05 + EditorUI08 | one typed Inspector request/lease/receipt; unchanged property writes `=0`; no string-form authoring authority; mixed/stale/multi-target parity |
| Optimize07 + Editor04 | one Play artifact identity; UI serialize/I/O `=0`; shell/World lock independent of bytes; no cross-project restore |
| Optimize11 + EditorUI08 | one journal owner; stable log scan/format `=0`; append delta/windowed query; hidden console history `=0` |
| Editor01 + Runtime09 | scene packet/gateway/HUD work outside shell lock; stable highlight sort/alloc/submit/store replace `=0`; RenderDoc/pixel parity |
| Performance01 | scale counters plus 31-run WPR/ETW, allocation, lock, gateway, latency, RSS and package-power matrix on identical hardware/assets/settings; D/E/F artifacts |

RenderDoc is mandatory for highlight/HUD/render cutover pixel and draw/resource parity. It does not
replace WPR/ETW evidence for CPU, lock, allocation or gateway behavior.

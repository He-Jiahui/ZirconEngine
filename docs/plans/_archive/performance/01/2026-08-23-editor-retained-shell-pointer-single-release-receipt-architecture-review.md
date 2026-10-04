---
title: Editor retained shell pointer single release receipt performance review
date: 2026-08-23
module: zircon_editor retained-host shell_pointer workspace docking
priority: MVP-P0 editor tab drag drop and drawer resize
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate application and docking drag operation
---

# Goal

Make one native drag or resize release produce one routed release receipt and consume the already
committed workbench projection. Remove release-time duplicate hit dispatch, full chrome/model
reconstruction and resize-move dispatch whose result is discarded. Preserve the current retained
multi-window drag surface until scale, WPR and allocation evidence can justify a different spatial
index.

## Reviewed source

- owner Rust files: 8/8
- physical lines: 1,355
- bytes: 49,354
- LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `29255d2c78e9e8469ee4918c188f2118ab3c48930253c89a12748cdc7af989b4`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`
- post-M0 owner Rust files: 8/8
- post-M0 physical lines: 1,359
- post-M0 bytes: 49,469
- post-M0 plus floating-scale M0 instrumentation lines: 1,376
- post-M0 plus floating-scale M0 instrumentation bytes: 50,139
- current owner manifest SHA256:
  `e728cd41be47120f60dee22befe5a2d06979161cd3b173f4e9bfa9a223054f7a`

All owner files under `ui/retained_host/shell_pointer` were read in full. The review also traced
native tab-drag and resize lifecycle callbacks, committed shell state, lifecycle recompute,
floating-window projection, workbench layout frames, app drag/drop and resize adapters, tab-drop
route resolution, resize dispatch, relevant retained-host tests and the prior 2026-07-17 report.

The old report is stale where it describes a mutex-backed immutable geometry snapshot and a full
drag-surface rebuild on every layout update. Current source uses `Arc<ArcSwap<DragHitGeometry>>`,
patches fixed topology in place, and rebuilds only when node geometry changes. It still scans and
allocates across all `9 + 5F` drag nodes on a stable-topology layout projection, so generation
gating remains pending rather than being reported as solved.

## Structural findings

### P0: pointer Up resolves the same drag target twice

`dispatch_drag_drop_from_pointer` first calls `sync_drag_target_group`, which invokes
`drag_route_at`. It then calls `resolve_drag_drop_route_from_pointer`, which invokes
`drag_route_at` again at the same coordinates. Each call constructs input metadata, advances the
bridge sequence, runs generic dispatch and converts reply effects through a route-intent map.

M0 must return the typed route from target-group synchronization and pass that exact release
receipt into route resolution. A release then performs one hit/dispatch, and UI group-key
publication remains changed-only.

### P0: release rebuilds the whole workbench model despite a committed projection

After the first release hit, route resolution calls `runtime.current_layout`, `build_chrome`,
`project_command_eval_snapshot`, locks commands and calls `WorkbenchViewModel::build_with_context`.
`CommittedShellState` already owns the exact `WorkbenchLayout` and `WorkbenchViewModel` used to
commit pointer frames, while `use_committed_pointer_layout` explicitly prohibits rebuilding the
editor tree inside native pointer callbacks.

M0 borrows committed layout/model and the committed componentized layout frames. It removes one
release-time layout snapshot, chrome build, command-evaluation snapshot, command lock and full
model build. Missing committed state rejects the drop instead of mixing new model identity with old
pointer geometry.

### P0: drawer move dispatches a captured route that the app discards

Native resize state already rejects inactive and unchanged moves. App state stores the captured
region and computes extent from the start coordinate. Nevertheless every active move dispatches
`update_resize` only to discard its route; the resize surface capture cannot change until release.
On Up, `finish_drawer_resize_capture` first calls that Move path and then dispatches Up, so one
native release becomes two generic dispatches.

M0 keeps Down/Up capture semantics but calculates move extent directly from `ActiveDrawerResize`.
Move dispatches become zero and Up dispatches become one. The final release coordinate is still
applied before the layout command.

### P0 correctness: rejected resize setup can leave mirror capture active until Up

`begin_resize` captures before the app validates a visible drawer frame and positive base extent.
If either validation fails, app resize state stays empty while mirror capture remains set until a
later native Up. M0 must explicitly cancel/release that captured route on either failure and cover
the cleanup with a focused contract.

### P1: stable pointer-layout projection is O(F) and allocates an O(F) patch vector

For stable floating-window topology, `patch_drag_surface` still recomputes geometry, allocates a
`Vec<Option<UiFrame>>`, compares all `9 + 5F` nodes and performs an `ArcSwap` publication when any
frame changes. `FloatingWindowProjectionBundle` is a newly built `HashMap` without a generation,
and `BuiltinWorkbenchWindowLayoutFrames` is Copy but has no committed identity token.

M1 must add one projection generation shared by committed frames, floating-window projection and
shell pointer layout. Exact generation equality should make an unchanged update O(1); changed
geometry remains O(F). Do not add a per-callback hash or compare another cloned window list.

### P1: drag surface duplicates typed window identity in several retained forms

For each floating window, the projection map, topology vector, geometry vector, five surface nodes,
formatted paths, closures and route-intent entries coexist. This may be acceptable for the current
small F, because the surface supplies overlap/z-order routing, but it has not been scale-validated.
M2 must compare retained-widget routing with a generation-owned direct spatial index at
`F = 0/1/8/64/256`; no authority deletion is allowed before behavior and cost evidence agree.

## Zircon and Unreal source basis

Direct Zircon source read:

- `app/host_lifecycle/tick.rs::use_committed_pointer_layout` requires pointer callbacks to use last
  committed frames and forbids an editor-tree rebuild in those callbacks.
- `app/committed_shell_state.rs` retains layout, chrome, model, geometry and componentized frames
  as one committed stage.
- native tab-drag move state rejects equal coordinates; native resize move rejects inactive and
  equal coordinates before invoking app callbacks.
- `shell_pointer/bridge.rs` routes drag and resize through separate retained surfaces; resize
  capture is established on Down and released on Up.

Direct Unreal source read:

- `SlateApplication.cpp::RoutePointerUpEvent` resolves one pointer-up path, resets drag-drop
  content before calling `OnDrop` to prevent reentrant double execution, and notifies release once.
- `SDockingTarget.cpp` publishes a typed `FDockTarget` on drag enter/leave and calls
  `OnUserAttemptingDock` directly on drop.
- `FDockingDragOperation.h/.cpp` stores `HoveredDockTarget`; `SetHoveredTarget` mutates it only when
  the typed target changes, and `OnDrop` consumes the existing drag operation instead of rebuilding
  the full editor layout/view model.
- Slate pointer capture routes Up to the existing captor path; capture is a session authority, not
  a reason to repeat target resolution on every app-level resize move.

The transferable design is one typed session receipt, changed-only hover publication, one release
route and reuse of committed layout identity.

## Target architecture

1. Native drag lifecycle emits one coordinate receipt per changed move and one Up receipt.
2. `sync_drag_target_group` returns the typed route that produced any group-key publication.
3. Drop resolution consumes that route and borrows committed layout/model/frames only.
4. Resize Down owns the captured region; app move arithmetic uses that region without redispatch.
5. Resize Up releases capture once, applies the final coordinate once and commits one command.
6. A committed pointer-layout generation makes identical lifecycle projection O(1) at M1.
7. Multi-window surface/index replacement waits for scale, WPR, allocation and parity evidence.

## Instrumentation and acceptance

Matrix: floating windows `0/1/8/64/256`; drag targets `document/edge/drawer/floating/empty`;
resize `left/right/bottom`; input `10/125/500 Hz`; release `inside/outside/reentrant`; projection
`stable/geometry/topology`; committed state `present/missing/stale`.

Acceptance requires:

- drag release generic route dispatches: `2 -> 1` at M0;
- release-time layout/chrome/context/command-lock/model builds: `1/1/1/1/1 -> 0` at M0;
- active resize move generic dispatches: `1 -> 0` at M0;
- resize Up generic dispatches: `2 -> 1` at M0;
- invalid resize setup leaves zero capture and zero active app resize state;
- stable group publication remains zero setters and no group-key allocation;
- identical layout projection becomes O(1), zero allocation and zero atomic publication at M1;
- p95 drag move below 0.05 ms at `F <= 64` and below 0.20 ms at `F = 256` on the accepted machine;
- p95 resize move below 0.02 ms, no queue growth and no per-move allocation;
- WPR shows no release-time chrome/model build and no main-thread backlog under 500 Hz storms;
- RenderDoc draw/pixel parity is required only if later surface/index changes alter render targets.

All executable, WPR, allocator, power and RenderDoc artifacts must remain on D/E/F and share the
same source/executable fingerprint.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Single release receipt; committed snapshot borrow; direct resize move arithmetic; capture cleanup. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | One committed pointer-layout/floating-projection generation and O(1) unchanged gate. | stable/geometry/topology counters and scale tests |
| M2 | Compare retained `9 + 5F` surface with generation-owned spatial index; cut only with evidence. | F-scale WPR/allocation/behavior parity |
| M3 | Storm, WPR, allocator, power and any required RenderDoc parity on one executable. | quantitative closeout |

## Validation state

- Owner review: complete, 8/8 current Rust files.
- Native lifecycle, committed shell state, projection/frame ownership, app adapters, route
  resolution, resize dispatch, relevant tests and Unreal sources: read and mapped.
- Architecture report: recorded before implementation.
- M0 implementation: applied. `sync_drag_target_group` now returns the exact typed route receipt;
  Up passes it into route resolution, which borrows committed layout/model/frames. Active resize
  Move performs only captured-region extent arithmetic; Up releases once and applies the final
  coordinate once. Failed drawer-frame/base validation cancels surface capture without a synthetic
  pointer event. The existing changed-only drag-group setter and committed WINDOW_METRICS resize
  path were preserved.
- Exact source operation delta: drag Up route dispatches `2 -> 1`; release-time layout snapshots,
  chrome builds, command-evaluation snapshots, command locks and workbench model builds each
  `1 -> 0`; active resize Move generic dispatches `1 -> 0`; resize Up generic dispatches `2 -> 1`;
  invalid setup capture lifetime `until native Up -> immediate cancel`. These are source/operation
  facts, not timing or power claims.
- Focused static contract:
  `tools/tests/test_editor_retained_shell_pointer_single_release_receipt_performance_contract.py`,
  102 lines, 4,039 bytes, SHA256
  `65421af2e19e44e32d3cf6e2d76ab61e3a0c5277bdaa409744ae88f9d93dd7cf`; RED 0/6 to GREEN
  6/6. A Rust behavior test also covers cancel-without-synthetic-event but has not executed.
- Retained-host Python performance contracts: GREEN 76/76. Broad current-worktree performance
  discovery: GREEN 255/255. Profile-path Pester: GREEN 3/3; UI profile output Pester: GREEN 45/45
  using E-drive temp roots. Rustfmt and scoped `git diff --check` passed.
- An earlier managed Cargo retry did not enter compilation. The coordinator returned
  `unmanaged_artifacts_detected` for unregistered shared artifacts rooted at
  `F:\cargo-targets\zircon-engine`; raw Cargo and deleting unrelated shared artifacts are not
  allowed bypasses.
- A later floating-scale M0 added feature-gated profile counters in `drag_surface.rs` for geometry
  resolves, floating-frame/node candidates, topology misses and unchanged reuse. Its focused
  contract is GREEN 4/4 and the broad performance suite is GREEN 265/265. This is observability,
  not M1 generation reuse or dynamic acceptance. The latest managed Cargo retry now fails earlier
  because current Session `019ffe1c-46d5-7933-97cb-65996b76f552` is archived; no compilation ran.
- This owner stays in `pending.md`; no timing, power, WPR, RenderDoc or completion claim exists.

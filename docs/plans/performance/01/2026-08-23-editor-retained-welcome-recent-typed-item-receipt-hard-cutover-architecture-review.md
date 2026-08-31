---
title: Editor retained welcome recent typed item receipt hard cutover performance review
date: 2026-08-23
module: zircon_editor retained-host welcome_recent_pointer
priority: MVP-P0 welcome startup recent-project open remove hover and scroll
status: source_reviewed_m0_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine SProjectBrowser and Slate table views
---

# Goal

Keep the native Welcome pane as the pane hit authority, preserve the current O(1) arithmetic row
lookup and O(V) native paint, and remove the editor-side two-node generic hit surface, route-owned
project-path strings, click-time full chrome/path projection, duplicate pointer state and unchanged
UI property publication. A recent-project action must carry a Copy item receipt and borrow the one
committed project path only at the template-command boundary.

## Reviewed source

- owner Rust files: 21/21
- physical lines: 689
- bytes: 26,195
- LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `e0607abfbeefd15ce87fb1041809709a6e321dbf39045035b85c211312c4ad1f`
- owning commit at review: `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`
- post-M0 owner Rust files: 15/15
- post-M0 physical lines: 417
- post-M0 bytes: 14,951
- post-M0 LF owner-relative-path-tab-raw-file-SHA manifest SHA256:
  `34130e18a8dd5996300189a1a1c220a7834f7b04a486d910d2dad149a382a654`

All owner files were read in full. The review also traced native Welcome callback routing,
committed pointer-layout lifecycle, workbench snapshot extraction, startup host construction,
shared template dispatch, route-intent ownership, root projection fallbacks, retained list tests,
native Welcome painting and the 2026-07-17 / 2026-07-31 reports.

Those older reports are stale where they describe N row hit nodes and a full surface rebuild on
every scroll. Current `rebuild_surface` creates only root and viewport nodes, scroll does not rebuild
the surface, `item_route_at_point` computes an index with `floor(content_y / row_pitch)`, and native
paint calls `welcome_recent_visible_row_count`. Current pointer hit cost is O(1), retained hit-node
count is 2 and native paint is O(V) in visible rows. M0 must preserve these properties.

## Structural findings

### P0: a committed native pane hit is sent through a second generic hit authority

Native `PanePointerRoute::Welcome` has already selected the committed pane and forwards local
coordinates. `WelcomeRecentPointerBridge` then dispatches through a two-node `UiSurface`,
`UiPointerDispatcher` and `EditorRouteIntentMap` only to recover `ListSurface`; the real row/action
hit is then computed arithmetically. The mirror owns neither row identity nor paint geometry and
adds a fallible dispatch, tree rebuild on geometry changes, route-map state and profiler counters.

M0 removes the generic surface and performs finite viewport validation plus the existing O(1) row
calculation directly.

### P0: every click rebuilds an O(N) path mirror before resolving one row

`welcome_recent_pointer_clicked` calls `runtime.chrome_snapshot()`, visits all recent rows, clones
every path into a new `Vec<String>`, deep-compares it through layout sync and only then resolves one
clicked item. This happens even though lifecycle recompute already committed the pointer layout and
the pointer callback explicitly promises to use committed geometry.

M0 uses the already committed bridge layout in input callbacks. O(N) path projection remains only
on an authoritative Welcome snapshot/layout publication. M1 must replace that residual path-only
mirror with generation-owned typed recent items shared by presentation, paint and input.

### P0: one path is cloned into two owned route layers and discarded on move

Arithmetic hit clones the selected `String` into `WelcomeRecentPointerRouteIntent`; conversion
moves it into `WelcomeRecentPointerRoute`. Move and scroll callers discard this route while still
paying the identity payload cost. Click only needs a path at the template binding boundary.

M0 makes hit, public route and dispatch Copy/index-only. Shared click validates the index against
the committed layout and borrows exactly one path before the command API creates its required owned
binding value. No path belongs in a pointer receipt.

### P0: bridge and host duplicate state and stable input republishes three properties

The bridge and `RetainedEditorHost` each own a `WelcomeRecentPointerState`. Each input callback
first synchronizes pane size and publishes scroll/hover/action, then dispatches and publishes the
same three properties again. Stable same-row motion, zero scroll and clamped overscroll therefore
perform redundant main-thread setter/invalidation work.

M0 makes the bridge the single interaction-state owner. Sync and dispatch return whether the
observable state changed; app adapters publish the three UI properties only on a changed receipt.

### P1: project identity lacks a presentation generation

An index is correct only for the exact recent-project ordering that produced it. Current callbacks
are synchronous, but a future asynchronous or delayed input path could resolve the same index
against a reordered list. M1 must publish a typed immutable recent-item allocation plus generation
across Welcome presentation, native paint and pointer receipts, and reject stale generations.

## Zircon and Unreal source basis

Direct Zircon source read:

- `app/host_lifecycle/tick.rs::use_committed_pointer_layout` states that native callbacks use the
  last committed bridge frames rather than rebuilding presentation inside input callbacks.
- `welcome_recent_pointer_bridge_project_route.rs` already owns the correct O(1) pitch/index hit.
- `host_contract/paint_workbench_renderer/welcome/recent_projects.rs` bounds row drawing through
  `welcome_recent_visible_row_count`; `rows.rs` visits only that visible prefix.
- `app/pointer_layout/welcome_recent.rs` is the lifecycle projection point and is the only M0 place
  allowed to rebuild the path layout after an authoritative Welcome projection changes.

Direct Unreal source read:

- `dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp`
  defines one `FProjectItem` containing `ProjectFile`, retains `TSharedPtr<FProjectItem>` in
  `STileView::ListItemsSource`, generates `SProjectTile` from that exact typed item, and passes it
  directly to double-click and selection callbacks.
- `SProjectBrowser.h` stores `TArray<TSharedPtr<FProjectItem>>` item sources and declares typed item
  handlers; it does not rebuild an all-path routing model for pointer events.
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h` generates from the
  scroll-derived item index only until the view is filled, reuses mapped widgets and releases unseen
  widgets.

The transferable architecture is a retained typed item source, view-bounded materialization and
typed/Copy input receipts. Zircon should not recreate project identity strings or a generic pane hit
tree inside a native pointer callback.

## Target architecture

1. Lifecycle recompute commits Welcome layout once per authoritative projection generation.
2. Native input validates the committed viewport and calculates row/action in O(1).
3. Pointer hit and public route contain only Copy `item_index` and action values.
4. Shared click borrows the indexed path once and creates ownership only at the command boundary.
5. `WelcomeRecentPointerBridge` is the sole pointer-state owner; dispatch and sync expose `changed`.
6. Stable move, zero scroll, boundary overscroll and stable pane-size sync publish zero UI setters.
7. Native paint remains O(V), and later M1 binds paint/input/command to one typed item generation.

## Instrumentation and acceptance

Matrix: rows `0/1/100/10K/100K`; visible rows `4/16/64`; operation
`projection/stable-sync/move/click/scroll/open/remove`; mutation `stable/add/remove/reorder/invalid`;
input `10/125/500 Hz`; receipt generation `current/stale`.

Acceptance requires:

- generic mirror dispatches per native Welcome callback: `1 -> 0` at M0;
- click-time chrome snapshots and all-row path clones: `1 + N -> 0` at M0;
- route-owned project-path clones: up to `2 -> 0` at M0;
- duplicated Welcome pointer state owners: `2 -> 1` at M0;
- stable callback UI setters: up to `6 -> 0` at M0;
- pointer hit remains O(1), native paint remains bounded by visible rows;
- open/remove route identity, hover, gap behavior, clamp and projection fallback remain equivalent;
- authoritative layout publication is O(N) at M0 and becomes O(1) shared typed-allocation handoff at
  M1;
- stale generation receipts are rejected at M1;
- p95 pointer routing below 0.01 ms at 100K rows with no row-count-correlated allocation;
- D/E/F WPR/allocator evidence contains no click-time chrome/path projection or mirror dispatch.

RenderDoc is relevant only to final Welcome pixel/draw parity because M0 changes neither geometry
nor renderer commands. WPR, allocator, executable and capture artifacts must remain on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Direct O(1) hit; Copy index/action route; borrowed one-path command target; single bridge state; changed-only publication; delete generic mirror authority. | focused RED/GREEN, Rustfmt, managed tests when available |
| M1 | One immutable typed recent-item allocation and generation across projection, paint and input; stale receipt rejection. | add/remove/reorder and stale-generation tests |
| M2 | Audit recent-project discovery/validation scheduling, sorting, persistence and invalidation for bounded worker/main-thread work. | scale and queue-age evidence |
| M3 | Run storm/WPR/allocator/power plus behavior and RenderDoc pixel/draw parity on one executable fingerprint. | quantitative closeout |

## Validation state

- Owner review: complete, 21/21 current Rust files.
- Native callback, lifecycle projection, snapshot extraction, shared dispatch, startup, tests and
  visible paint: read and mapped.
- Unreal Project Browser typed-item and Slate view-bounded generation source: read and mapped.
- Architecture report: recorded before implementation.
- M0 implementation: applied. The bridge now owns only committed layout, one interaction state and
  layout metrics. Click/move/scroll use direct finite viewport plus arithmetic row routing; Copy
  routes contain index/action only; shared click borrows one committed path at the required owned
  template-binding boundary. App callbacks no longer build a chrome snapshot or full path layout,
  and size/dispatch state changes are coalesced into one UI publication. Six generic
  surface/dispatcher/route-conversion owner files and the Welcome route-intent variant are deleted.
- Exact static owner delta: files `21 -> 15`, physical lines `689 -> 417` (-272, 39.5%), bytes
  `26,195 -> 14,951` (-11,244, 42.9%). Generic mirror dispatches are `1 -> 0` per callback;
  click-time chrome snapshots and all-row path clones are `1 + N -> 0`; route path clones are up to
  `2 -> 0`; pointer-state owners are `2 -> 1`; stable callback UI setters are up to `6 -> 0`.
  These are source/operation-count facts, not timing claims.
- Focused static contract:
  `tools/tests/test_editor_retained_welcome_recent_direct_receipt_performance_contract.py`, 180
  lines, 6,896 bytes, SHA256
  `fa698c575e76fe4adff3fad5d04d6b16623c34343dfacdb25a09df51cafea749`; RED 1/11 to GREEN
  11/11.
- Retained-host Python performance contracts: GREEN 70/70. Broad current-worktree performance
  discovery: GREEN 248/248. Profile-capture Pester: GREEN 45/45 using
  `E:\ZirconTemp\pester-welcome-m0`; critical source manifest is 147/147 unique paths with zero
  missing. Rustfmt and scoped `git diff --check` passed.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is archived and returns
  `cargo_session_not_executable`; raw Cargo is not an allowed bypass. A current-source executable,
  WPR/allocator/power capture and RenderDoc pixel/draw parity therefore remain pending.
- This owner stays in `pending.md`; no timing, power, WPR, RenderDoc or completion claim exists.

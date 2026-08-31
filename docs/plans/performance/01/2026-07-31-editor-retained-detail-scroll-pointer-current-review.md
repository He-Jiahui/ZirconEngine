---
related_code:
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer
  - zircon_editor/src/ui/retained_host/detail_pointer
  - zircon_editor/src/ui/retained_host/scroll_surface_host.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-control-index-and-virtual-row-sync.md
tests:
  - app inline performance guard inspected: 1
  - retained detail-pointer files/tests inspected: 5/10
  - direct rustfmt check: passed for app 4/4 and bridge 27/27
  - current-source managed Windows Cargo pending
  - boundary-scroll counters and WPR/Tracy trace pending
doc_type: implementation-evidence
status: superseded_by_2026-08-23_current_source_review
---

# Editor retained detail-scroll pointer current review (2026-07-31)

> Superseded on 2026-08-23 by
> `2026-08-23-editor-retained-detail-scroll-direct-change-receipt-hard-cutover-architecture-review.md`.
> The new review deletes the generic detail scroll surfaces and implements the changed-only
> publication that this report left pending. Use the newer source fingerprint for acceptance.

## Scope

`zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs` and its child directory are **4/4** clean Rust files, **118** physical lines, **1** inline performance test, with path+raw-content SHA-256 `7256ca827e81620ab0088f7649719b4871d46f154663fef6da1d3878e1ced9ff`. The shared `retained_host/detail_pointer/**` bridge is **27/27** clean Rust files, **485** physical lines, SHA-256 `51930799a92bc4e2b7d1f5925ee2a08d6d2d569086076e12b918ac48a93228d6`. All **31/31** files pass direct `rustfmt --check`.

The review also traced `ScrollSurfaceHostState`, `pointer_layout/detail_scrolls.rs`, source-window focus dispatch, and all **5/5** files / **10** tests in `retained_detail_pointer`.

## Findings

- PERF-MVP-110's implemented improvement remains present. Console, Inspector, and Asset Details share a two-node pointer surface; `handle_scroll` reads the runtime-mutated viewport offset and never calls `rebuild_surface`. Repeated size and identical layout/state have explicit no-op guards, fixed Asset Details section heights use constant arithmetic, and the route is a `Copy` enum.
- The app adapters only rebuild layout when callback surface size changes. Console reads the status line and Asset Details reads the committed `Arc<AssetWorkspaceSnapshot>` only on that resize path; stable scroll no longer takes a full editor/chrome snapshot.
- `ScrollSurfacePointerBridge::handle_scroll` correctly changes its state only when the clamped runtime offset differs. `ScrollSurfaceHostState::handle_scroll`, however, erases that information by returning `Result<(), String>` and unconditionally overwriting its state. All three app adapters then call a Slint scroll setter for every successful dispatch, including zero delta and already-clamped boundary scroll.
- This is not a new ownership problem. It strengthens PERF-MVP-171's existing `Ignored/Handled { damage }` requirement and PERF-MVP-110's interaction gate: the shared scroll result should carry `changed`, and the app should publish the single scroll property only when true. The adapter must not keep a second last-offset cache.
- Every scroll still calls `use_committed_pointer_layout`, so the three unchanged diagnostics writes remain under PERF-MVP-601. `focus_callback_source_window` clones one optional window id, but its same-window guard returns before chrome/model construction; only the first event after a real child-window switch dispatches `FocusView`, which is required behavior.
- Slow lifecycle sync still computes console text extent and applies scroll properties before bridge equality. That stable-generation projection cost remains owned by PERF-MVP-106 and the previously reviewed pointer-layout module, not by a detail-scroll local cache.

## Reference and target

Godot `dev/godot/scene/gui/range.cpp:172-180` preserves the clamped value but emits `value_changed` only when the value actually differs. Zircon should likewise preserve runtime scroll authority while returning a typed changed outcome to the host; equality must be decided at the owner that applied the scroll.

Acceptance should cover Console/Inspector/Asset Details, main/floating windows, pane sizes `0/1/4K`, content extents `0/viewport/10K`, and `1/1K/1M` deltas including zero, NaN/invalid handling, top/bottom overscroll, wheel/pan, and resize. Record dispatch/hit, focus model builds, surface rebuilds, state copies, Slint setters, invalidation/redraw/damage, queue age, and UI p50/p95. Stable/clamped scroll must have surface rebuild/path/tree alloc=0, focus model build=0, property setter=0, and redraw/damage=0; changed scroll may publish one setter and one local damage. Clamp, route, child-window focus, content extent, Cargo, F4, WPR/Tracy, and independent review remain required before moving this module to `review.md`.

# UI M7 Invalidation Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the M7 invalidation, performance, and Widget Reflector diagnostics slice for Zircon UI using Unreal Slate as the behavior reference.

**Architecture:** Keep `UiSurfaceFrame` as the runtime/editor spatial truth. Runtime work adds rebuild metrics and borrowed hit-grid queries; editor work tightens the existing invalidation root and retained softbuffer region path; diagnostics work extends the shared snapshot instead of inventing editor-only counters.

**Tech Stack:** Rust, Cargo, serde DTOs, `UiSurfaceFrame`, softbuffer native presenter, `.ui.toml` host projection, Unreal Slate reference files under `dev/UnrealEngine`.

---

## Execution Policy

- Work directly on `main`; do not create a branch or worktree.
- Do not commit unless the user explicitly asks.
- Preserve unrelated dirty UI work from active sessions.
- Follow Zircon milestone-first cadence: write code, tests, and docs during implementation slices; run compile/tests in the milestone testing stage.

## Reference Evidence

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h`: invalidation root owns cached draw data, hittest state, phase timings, and slow/fast paint decisions.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`: fast path repaints invalidated widgets/subtrees and removes covered descendants.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetList.cpp`: traversal-order widget ranges and sorted update lists justify recording dirty/rebuild counts separately from paint.
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h` and `Private/Input/HittestGrid.cpp`: pointer routing should query cached grid/path state, not clone or rebuild indexes per event.
- `dev/UnrealEngine/Engine/Source/Developer/SlateReflector/Private/Widgets/SWidgetReflector.cpp`: reflector snapshots should expose invalidation/debug state, paint/hittest data, and performance counters.

## File Structure

- `zircon_runtime/src/ui/tree/hit_test.rs`: add borrowed-grid hit testing so `UiSurfaceFrame` queries do not clone `UiHitTestGrid`.
- `zircon_runtime/src/ui/surface/frame_hit_test.rs`: switch surface-frame hit testing to the borrowed helper.
- `zircon_runtime/src/ui/surface/surface.rs`: expand `UiSurfaceRebuildReport` with dirty reasons, counts, and elapsed microseconds.
- `zircon_runtime_interface/src/ui/surface/diagnostics.rs`: add rebuild/invalidation and batch-break DTO fields to shared debug snapshots.
- `zircon_runtime/src/ui/surface/diagnostics.rs`: populate the extended debug DTOs from `UiSurfaceFrame`.
- `zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs`: new focused module for debug overlay frame calculation.
- `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs`: use the shared overlay frame helper instead of local geometry.
- `zircon_editor/src/ui/slint_host/host_contract/presenter.rs`: union overlay damage into region presents when diagnostics text changes without scheduling a second redraw.
- `zircon_editor/src/ui/slint_host/app/invalidation.rs`: expose the full structured invalidation snapshot.
- `docs/ui-and-layout/slate-style-ui-surface-frame.md`, `docs/editor-and-tooling/editor-workbench-shell.md`, and `tests/acceptance/ui-m7-invalidation-performance.md`: record behavior and evidence.

## Milestone M7.1: Runtime Invalidation Metrics And Borrowed Hit Grid

- Goal: make runtime rebuilds and `UiSurfaceFrame` hit queries measurable and avoid hit-grid clone overhead.
- In-scope behaviors: dirty reason summary, rebuilt node/command/hit-grid counts, elapsed microseconds, borrowed hit-test parity.
- Dependencies: existing `UiSurface::rebuild_dirty`, `UiHitTestIndex`, and `hit_test_surface_frame`.

### Implementation Slices

- [x] Add borrowed `UiHitTestIndex::hit_test_grid_arranged(...)` and make `hit_test_arranged(...)` delegate to it.
- [x] Update `hit_test_surface_frame(...)` to call the borrowed helper on `surface_frame.hit_grid`.
- [x] Extend `UiSurfaceRebuildReport` with `dirty_flags`, `dirty_node_count`, `arranged_node_count`, `render_command_count`, `hit_grid_entry_count`, `hit_grid_cell_count`, and elapsed microseconds for layout/arranged/hit/render phases.
- [x] Add runtime tests proving borrowed surface-frame hit test returns the same path as the surface index and reports rebuild metrics for render-only, layout dirty, and clean cached-count paths.

### Testing Stage: Runtime Metrics Gate

Run:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/tree/hit_test.rs" "zircon_runtime/src/ui/surface/frame_hit_test.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/tests/hit_grid.rs"
cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
```

### Exit Evidence

- `UiSurfaceFrame` hit testing no longer clones `UiHitTestGrid`.
- Focused runtime tests prove report counters and hit-test parity.

## Milestone M7.2: Editor Region Damage For Live Diagnostics Overlay

- Goal: keep the live top-right diagnostics overlay updated during region-only presents without creating a self-sustaining redraw loop.
- In-scope behaviors: overlay frame calculation is shared, changed overlay text expands the current damage only, unchanged overlay text does not expand damage, presenter diagnostics remain counter-based.
- Dependencies: `HostRefreshDiagnostics`, `SoftbufferHostPresenter`, `HostRedrawRequest`, and native painter clipped repaint.

### Implementation Slices

- [x] Create `painter/diagnostics_overlay.rs` with `debug_refresh_overlay_frame(top_bar, label)` and the matching presenter damage helpers.
- [x] Use that helper from `workbench.rs` for drawing the overlay.
- [x] Update `SoftbufferHostPresenter` to track the last diagnostics overlay text and union the overlay frame into the current region damage when the overlay text changes.
- [x] Add presenter unit tests for overlay damage expansion and unchanged-text no-op.

### Testing Stage: Editor Damage Gate

Run:

```powershell
rustfmt --edition 2021 --check "zircon_editor/src/ui/slint_host/host_contract/presenter.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs"
cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
```

### Exit Evidence

- Region repaint can update the diagnostics overlay in the current present.
- No new redraw is queued solely because the overlay text changed.

## Milestone M7.3: Shared Reflector Performance Snapshot

- Goal: extend the shared Widget Reflector-style snapshot with invalidation/performance and batch-break data that editor/runtime tools can display.
- In-scope behaviors: render batch break reason, hit-grid density, overdraw ratio, optional last rebuild report in `UiSurfaceFrame`.
- Dependencies: existing `debug_surface_frame(...)` and `UiSurfaceDebugSnapshot` DTOs.

### Implementation Slices

- [x] Add shared DTO fields for `UiSurfaceRebuildDebugStats` and `UiMaterialBatchDebugStat::break_reason`.
- [x] Store `last_rebuild_report` in `UiSurfaceFrame` and populate it from `UiSurface::surface_frame()`.
- [x] Populate snapshot rebuild stats and batch break reasons in `surface/diagnostics.rs`.
- [x] Add diagnostics tests asserting reflector snapshots include rebuild data and deterministic batch break reasons.

### Testing Stage: Reflector Gate

Run:

```powershell
rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/surface/diagnostics.rs" "zircon_runtime_interface/src/ui/surface/frame.rs" "zircon_runtime/src/ui/surface/diagnostics.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/tests/diagnostics.rs"
cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
```

### Exit Evidence

- Shared debug snapshot contains enough data for Widget Reflector M7 panels without editor-only coordinate interpretation.

## Milestone M7.4: Docs, Acceptance, And Final Scoped Validation

- Goal: record implementation, tests, Unreal evidence, and scoped validation results.
- In-scope behaviors: docs headers updated, acceptance file created, active session note updated or retired.
- Dependencies: M7.1-M7.3 implementation complete.

### Implementation Slices

- [x] Update `docs/ui-and-layout/slate-style-ui-surface-frame.md` for rebuild metrics, borrowed hittest, and debug snapshot fields.
- [x] Update `docs/editor-and-tooling/editor-workbench-shell.md` for diagnostics overlay region damage.
- [x] Create `tests/acceptance/ui-m7-invalidation-performance.md` with commands and evidence.
- [x] Update `.codex/sessions/20260506-0428-ui-m7-invalidation-performance.md` with current status.

### Testing Stage: Final Scoped Gate

Run:

```powershell
cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir "E:\zircon-build\targets\ui-m7" --message-format short --color never
```

### Exit Evidence

- All focused runtime/editor checks pass or exact blockers are recorded.
- Docs and acceptance records include implementation files, plan sources, tests, and remaining risks.

## Self-Review Notes

- Spec coverage: user requested full M7 invalidation/performance, and milestones cover runtime metrics, editor fast damage, shared reflector diagnostics, docs, and validation.
- Placeholder scan: no placeholder tasks remain; each milestone has exact files and commands.
- Type consistency: planned fields use `UiSurfaceRebuildReport`, `UiSurfaceRebuildDebugStats`, `HostRefreshDiagnostics`, and existing host/presenter names consistently.

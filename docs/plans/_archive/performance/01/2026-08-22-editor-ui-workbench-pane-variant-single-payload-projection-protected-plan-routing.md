---
title: Editor workbench pane variant single-payload projection protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-pane-variant-single-payload-projection-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/layouts/windows/workbench_host_window/{pane_projection.rs,pane_presentation.rs,pane_payload.rs,floating_windows.rs}`
- 4/4 Rust files source-reviewed; each selected pane currently builds all native payload variants and
rescans the full workbench for its tab snapshot; M1 limits construction to the selected kind, while
typed generation-owned payloads, one snapshot index, floating-window retention and M0/M2-M5
dynamic/profile/power/visual acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to editor pane/shell MVP work. Record per-kind builder calls, rows/bytes cloned,
snapshot entries visited, allocations, shell/floating builds, main-thread CPU, input-to-paint latency,
RSS and package energy. A selected pane must never execute an inactive domain builder.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of flat all-variant native-body ownership, fallback payload reconstruction and repeated
snapshot scans after all consumers use typed generation-owned pane artifacts and the shared index.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own `PaneProjectionArtifact`, one selected typed payload per pane generation and shared-owner shell
assembly. Stable panes must do zero builder/list/model work.

## `docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md`

Own the foreground-tab invariant across main docks, documents and floating windows: one active pane
content subtree per stack, matching Unreal `RefreshParentContent` behavior.

## `docs/plans/zircon_editor/editor_layout/06-floating-windows-and-design-parity.md`

Own generation-retained floating window identity, target-group strings, tab DTOs, active pane and
geometry. Unchanged windows must do zero remapping or string formatting.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Publish and coalesce exact per-pane content plus window/tab/geometry generations. A hierarchy update
must not invalidate timeline, plugin, export, animation or UI asset payloads.

## `docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`

Own the typed pane payload surface and removal of flat all-domain containers. Defaults are not an
accepted substitute for representing one active variant.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own immutable typed UI payload receipts and host conversion that consumes one selected payload
without reconstructing duplicate flat DTO forms.

## Acceptance handoff

The owner handoff requires 4/4 post-change fingerprints, managed focused and behavior tests, the full
scale matrix, current-source WPR/power artifacts on D/E/F, interaction and screenshot checks,
RenderDoc parity for GPU content, milestone commit and quantified WeCom notification. Shared ledgers
remain protected until then.

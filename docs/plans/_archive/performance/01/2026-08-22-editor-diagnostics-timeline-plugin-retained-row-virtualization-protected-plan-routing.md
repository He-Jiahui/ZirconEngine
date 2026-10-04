---
title: Editor diagnostics, timeline and plugin retained-row virtualization protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-diagnostics-timeline-plugin-retained-row-virtualization-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/{runtime_diagnostics.rs,performance_timeline.rs,module_plugins.rs}`
- 3/3 Rust files source-reviewed; diagnostics ordinary apply executes a temporary self-reflection
surface and second hit rebuild, timeline has bounded display but allocation-heavy full hotspot
analysis and duplicate final rows, and plugins cache source rows but expand every offscreen item into
3-11 nodes; M1 removes per-span hotspot keys/full percentile sorting and borrowed status/action
temporaries, while M0/M2-M5 dynamic/profile/power/visual acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to runtime diagnostics, performance timeline and plugin-manager MVP work. Record
temporary surfaces/rebuilds, hotspot key/string allocation and percentile work, total/visible/copied
rows, node builds/reuse, allocations, CPU, latency, RSS and energy across the specified scale matrix.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of ordinary diagnostic self-reflection, duplicate final pane DTO maps and full offscreen
node expansion after explicit capture receipts and virtualized item generations are live.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own retained pane item generations, list-window state, exact item refresh receipts and shared row
identity through the runtime/editor UI boundary.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own visible hit rows and scroll-only row-window changes. Offscreen diagnostic/timeline/plugin items
must not enter the hit tree.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry profile, plugin-catalog, debug-capture, scroll and geometry generations independently so a
stable source or scroll-only update cannot trigger full pane conversion.

## `docs/plans/zircon_runtime/runtime/05-frame-pipeline-profiling-and-logging.md`

Own one immutable cached hotspot report per profile generation and the no-per-span-key-clone,
selection-based percentile aggregation algorithm.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own explicit debug-capture receipts over committed surfaces and remove temporary self-reflection
surfaces from ordinary UI presentation.

## Acceptance handoff

The owner handoff requires 3/3 post-change fingerprints, managed focused and behavior tests, the full
row/span/plugin scale matrix, current-source WPR/power artifacts on D/E/F, interaction/screenshots,
RenderDoc parity where GPU output is relevant, milestone commit and quantified WeCom notification.
Shared ledgers remain protected until then.

---
title: Editor workbench debug reflector demand-driven virtualized snapshot protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-ui-workbench-debug-reflector-demand-driven-virtualized-snapshot-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` entry:

`zircon_editor/src/ui/workbench/debug_reflector` - 9/9 Rust files source-reviewed; production
Runtime Diagnostics cannot receive a real debug snapshot, so retained conversion builds and
reflects a synthetic pane surface, eagerly materializes all string rows and can run three full-tree
transactions; managed/profile/power acceptance remains pending.

Do not add the folder to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M1-M5 to `PERF-MVP-101`, `PERF-MVP-103` and `PERF-MVP-143`. Link Runtime snapshot detail
generation to `PERF-MVP-278`/`PERF-MVP-280`. Record hidden/visible subscription, source and synthetic
surface rebuilds, generation hits, row/string/overlay allocation bytes and pane hit rebuilds.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of `runtime_diagnostics_debug_surface_frame`, flat all-node label compatibility APIs
and any second snapshot authority after the generation-owned typed item source is committed. Do not
leave a compatibility path that silently rebuilds a synthetic surface.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own the Runtime Diagnostics subscription and pane projection cutover. A visible pane consumes one
real committed surface generation; hidden and unchanged generations do no reflector work. Pane hit
artifacts and all-node debug capture remain separate products.

## `docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md`

Own product behavior for Runtime Diagnostics and Widget Tree Debugger: virtualized hierarchical
rows, selection, expansion, search, snapshot/export controls and overlay toggles. Preserve feature
parity without eager all-row node materialization.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Runtime UI owns the immutable all-node debug snapshot, generation, stable node index and bounded
section/overlay artifacts. Editor consumers subscribe by generation and must not reconstruct a new
Runtime `UiSurface` from pane DTOs.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Route diagnostic subscription changes as coalesced generation receipts. Stable presentation,
resize and paint invalidations must not rebuild reflector rows or request another capture.

## Acceptance handoff

The owner handoff requires the 9/9 post-change fingerprint, managed focused tests, 1/100/1k/10k-node
visible/hidden traces, synthetic surface count, source rebuild/snapshot/row/clone/allocation counts,
current-source WPR and power artifacts on D/E/F, real-window screenshots, RenderDoc overlay parity,
milestone commit and quantified WeCom notification. Shared ledgers remain protected until then.

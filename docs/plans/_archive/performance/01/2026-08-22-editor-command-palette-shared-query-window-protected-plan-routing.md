---
title: Editor command palette shared-query-window protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-command-palette-shared-query-window-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{command_palette/**,selection_options/**}`
- 15/15 Rust files source-reviewed; a correct bounded catalog query window is still encoded as full
  commands plus filtered IDs and reparsed into legacy/structured/joined rows. M1 made the workbench
  consume one combined projection (focused contract GREEN 2/2; owned contracts GREEN 35/35).
  M0/M2-M5 typed generation/profile/power/interaction acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to the MVP command palette. Record query metrics, encoded/parsed bytes, indices,
rows/labels/joins, state patches, visible visits, allocations, CPU, latency, RSS and energy across the
specified matrix.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of generic command/filtered-ID properties, double projection and legacy option/join
owners after shared typed command row generations are live.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own one `CommandPaletteRowGeneration` through app, bridge, workbench, pane and native presenter
boundaries.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one visible command window across paint, hit, keyboard and accessibility with stable command IDs.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry query/catalog/MRU/window/focus/selection receipts independently and patch exact rows.

## `docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md`

Own typed command row bindings and removal of generic UiValue/TOML presentation round trips.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own retained typed searchable-list rows and category-specific fast updates shared by runtime/editor UI.

## Acceptance handoff

The owner handoff requires 15/15 post-change fingerprints, managed focused and behavior tests, the
full catalog/query/window matrix, current-source WPR/power artifacts on D/E/F, interaction/
screenshots, RenderDoc parity where GPU output is relevant, milestone commit and quantified WeCom
notification. Shared ledgers remain protected until then.

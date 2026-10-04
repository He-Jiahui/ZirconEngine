---
title: Editor selection option source generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-selection-option-source-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host selection_options/** + pane_option_projection.rs`
- 8/8 Rust files source-reviewed. No-options nodes still read twelve structured-state fields; option
  sources are cloned/parsed into multiple representations and rebuilt on interaction changes.
  M1 reduces no-options structured reads 12 -> 0 (focused contract GREEN 1/1; owned contracts GREEN
  50/50). M0/M2-M5 typed-source/patch/virtualization/profile/power acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to MVP menus/dropdowns/segmented controls. Record field reads, parse/owned bytes, visible
rows, dynamic patches, CPU/allocation/RSS/latency/context switches and power.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of duplicate raw/structured/options-text sources and string parsers after typed consumers
migrate.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own ordered selection/focus/press receipts and latest-wins hover/query patches on the UI thread.

## `docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`

Own compile-time typed option identity/label/static flags and declaration diagnostics.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own shared option generations plus virtualized/reused visible popup/menu/select rows.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own one option identity/index contract across paint, hit, keyboard, accessibility and event payloads.

## Acceptance handoff

The handoff requires 8/8 post-change fingerprints, managed behavior tests, the node/option/visible/
query/input matrix, current-source WPR/power artifacts on D/E/F, paint/hit/keyboard/accessibility
parity, milestone commit and quantified WeCom notification. Protected ledgers remain unchanged until
then.

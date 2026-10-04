---
title: Editor pane collection source-window protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-pane-collection-source-window-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{collection_fields/**,collection_projection/**,collection_window.rs}`
- 15/15 Rust files source-reviewed; virtualization currently slices after full string/typed-row
  materialization, editable arrays/maps recursively clone and validate all entries, and pagination
  lacks an explicit producer page-generation receipt. M1 now windows lazy string/typed-row iterators
  before owned DTO construction; M0/M2-M5 typed generation/profile/power/interaction acceptance
  remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to shared MVP editor collections. Record source scans, row/value bytes cloned,
materialized rows, type/validation/action work, visible visits, allocations, CPU, latency, RSS and
energy across the specified matrix.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of post-materialization slicing, recursive offscreen field conversion and duplicate
visible/page row authorities after typed collection generations are live.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own shared collection/page generations through pane payload, host and native presenter boundaries.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one visible collection index across paint, hit, keyboard, accessibility and profiling while
preserving original source index and stable row identity.

## `docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md`

Own typed editable collection schema/value generations and one-row edit/validation patches.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry source/filter/sort/page/viewport/schema/value receipts independently and coalesce exact row
updates.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own stable typed item sources, retained rows and viewport-before-generation scheduling shared by
runtime and editor UI.

## Acceptance handoff

The owner handoff requires 15/15 post-change fingerprints, managed focused and behavior tests, the
full collection/page/field scale matrix, current-source WPR/power artifacts on D/E/F, interaction/
screenshots, RenderDoc parity where GPU output is relevant, milestone commit and quantified WeCom
notification. Shared ledgers remain protected until then.

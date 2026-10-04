---
title: Editor popup, dialog and binding-route generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-popup-dialog-binding-route-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{binding_actions,showcase_actions,dialog,popup_actions,popup_frame}/**`
- 27/27 Rust files source-reviewed. Changed nodes repeatedly scan generic bindings, broad flat DTO
  projection builds popup/action/drop fields for unrelated components, closed content lacks a retained
  generation boundary, and placement has no work-area fit/flip authority. M1 hard-cuts two allocating
  route normalizers to one output-buffer owner (focused contract GREEN 2/2; owned contracts GREEN
  40/40). M0/M2-M5 compile/generation/profile/power/interaction acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to MVP direct input dispatch, dialogs and menus. Record binding visits, route
normalization bytes/allocations, popup rows/prepass visits, placement work, input latency and power.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of generic per-presentation binding scans, duplicate route normalizers, closed native
popup content and legacy menu/action label owners after compiled routes/generations are live.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own `CompiledNodeRoutes`, direct stable route slots and exact input dispatch without presentation-time
binding lookup.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own one `PopupPresentationGeneration` through workbench, retained host and native presenter.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry route/content/open/anchor/work-area/scale receipts independently and rebuild exact categories.

## `docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md`

Compile binding routes once and publish route-generation changes rather than rescanning generic rows.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own typed popup content generations, direct route descriptors and one work-area-aware placement
contract shared with editor UI.

## Acceptance handoff

The handoff requires 27/27 post-change fingerprints, managed route/geometry/interaction tests, the
full node/binding/menu/row-width matrix, current-source WPR/power artifacts on D/E/F, accessibility
and screenshot parity, RenderDoc popup/dialog draw parity, milestone commit and quantified WeCom
notification. Protected ledgers remain unchanged until then.

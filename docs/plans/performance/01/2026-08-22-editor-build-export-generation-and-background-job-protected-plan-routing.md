---
title: Editor build and export generation and background-job protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-build-export-generation-and-background-job-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/**` and
`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs`
- 9/9 Rust files source-reviewed; a stable cache hit still polls manifest/directory/preset metadata,
  cached wizard state is cloned and rebuilt through retained plus wide-node representations, and
  every target is copied to a DTO plus up to nine non-virtualized nodes. Existing EditorJobSystem
  execution is retained. M1 normalizes platform identity once per target, removes one action Vec per
  target and reserves the fixed 9T node shape; M0/M2-M5 dynamic/profile/power/interaction acceptance
  remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to the MVP editor BuildExport workflow. Record metadata calls, source/catalog/job/layout
generations, target strings/DTO/action/node copies, wizard surface/projection nodes, visible visits,
job queue/drain metrics, allocations, CPU, latency, RSS and energy across the specified matrix.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of UI-time source polling, duplicated target DTO/node models and the wizard retained-to-
wide conversion after generation receipts and shared visible item presentation are live.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own shared BuildExport catalog/job/wizard generations through pane payload, retained host and native
presenter boundaries. Stable generations must preserve identity instead of cloning pane models.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one target/wizard visible-row index for paint, hit, accessibility and profiling, with retained row
reuse and exact action identity.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry source, catalog, job, wizard and layout receipts independently. Coalesce refresh requests and
patch exact job-stage/output/control rows instead of rebuilding the pane.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own O(V) target/wizard pointer routing and ensure inactive presentation branches create no hit rows.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own persistent item generations, retained rows and visible-range scheduling shared across runtime
and editor UI consumers.

## `docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`

Preserve the existing background export boundary and own bounded job-event delivery, cancellation,
deadline and queue-depth instrumentation. Do not create a BuildExport-private worker pool.

## `docs/plans/zircon_plugins/09-export-publishing.md`

Own watcher/background-rescan publication of project manifest, export directory and preset source
generations. UI presentation must not poll filesystem metadata in steady state.

## Acceptance handoff

The owner handoff requires 9/9 post-change fingerprints, managed focused and behavior tests, the full
target/wizard/job scale matrix, current-source WPR/power artifacts on D/E/F, interaction/screenshots,
RenderDoc parity where GPU output is relevant, milestone commit and quantified WeCom notification.
Shared ledgers remain protected until then.

---
title: Editor pane component typed-generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-pane-component-typed-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_runtime_projection.rs` and
`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{mod.rs,host_template_node.rs,template_node_data/**}`
- 15/15 Rust files source-reviewed; stable panes still rebuild runtime surface/layout/host models,
  every node enters fourteen untyped component projectors over a shared attribute map, and all
  component families expand into one flat wide DTO. M1 removes one component clone per converted
  node and the generic fallback control-ID clone; M0/M2-M5 typed generation/profile/power/
  interaction acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to the shared MVP editor pane projection path. Record surface/layout/host conversions,
attribute lookups/parses, component family work, row/field bytes, category patches, allocations, CPU,
latency, RSS and energy across the specified matrix.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of untyped repeated component derivation, parallel runtime/surface conversion paths and
internal flat all-feature row ownership after typed generations are live.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own one shared `PaneComponentGeneration` through generic, console, inspector, showcase and native
presenter boundaries. Stable node identity must survive host projection.

## `docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md`

Own descriptor-driven visual/resource invalidation categories so style/theme changes patch only
affected node fields and never reparse semantic component attributes.

## `docs/plans/zircon_editor/editor_layout/11-data-binding-and-reactive-contract.md`

Own typed attribute/binding compilation and exact content/layout/interaction/visual dirty receipts.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry component semantic, layout, interaction, visual, resource/text and registry generations
independently; coalesce refresh and patch addressed rows.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own typed persistent component nodes, descriptor invalidation and retained fast-update scheduling
shared by runtime and editor UI.

## Acceptance handoff

The owner handoff requires 15/15 post-change fingerprints, managed focused and behavior tests, the
full node/attribute/component matrix, current-source WPR/power artifacts on D/E/F, interaction/
screenshots, RenderDoc parity where GPU output is relevant, milestone commit and quantified WeCom
notification. Shared ledgers remain protected until then.

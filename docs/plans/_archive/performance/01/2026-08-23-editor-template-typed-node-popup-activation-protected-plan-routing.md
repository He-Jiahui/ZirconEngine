---
title: Editor template typed node popup and activation protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-template-typed-node-popup-activation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep concise `pending.md` entries by module owner:

`zircon_editor retained-host host_contract/frame_geometry.rs + frame_geometry/**`
- 5/5 current Rust files source-reviewed; constant-time geometry foundation retained. Dynamic parity
  remains pending with the typed arranged generation cutover.

`zircon_editor retained-host host_contract/template_geometry* + template_geometry/**`
- 4/4 current Rust files source-reviewed. Pending single arranged-generation bounds ownership and
  cross-projection reuse evidence.

`zircon_editor retained-host host_contract/template_component_family* + template_component_family/**`
- 8/8 current Rust files source-reviewed. Pending typed descriptor compilation, one paint dispatch and
  hard deletion of hot-path string/control-id family fallback.

`zircon_editor retained-host host_contract/template_input_semantics* + template_input_semantics/**`
- 4/4 current Rust files source-reviewed. Pending typed input role shared by hit/focus/activation and
  current-source input parity.

`zircon_editor retained-host host_contract/template_activation_semantics* + template_activation_semantics/**`
- 6/6 current Rust files source-reviewed. Pending typed activation plan, single asset-route parse and
  pointer/table/action parity.

`zircon_editor retained-host host_contract/template_popup_layout* + template_popup_layout/**`
- 7/7 current Rust files source-reviewed; O(1) flip/clamp geometry retained. Pending one generation-
  owned popup layout/navigation artifact shared by paint/hit/dismiss/keyboard and scale acceptance.

Do not add any group to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M3 to MVP editor paint/input. Record descriptor builds/reuse/bytes, painter probes, hot-path
string classifications, unknown/fallback counts, popup builds, keyboard node/row visits and
allocations, geometry parity, CPU/context switches/power and source/workload fingerprints.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own hard deletion of sequential painter probing, hot-path control-id component inference and normal
current-generation popup full scans. No compatibility string dispatcher survives the cutover.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own immutable `TemplateNodeDescriptor` and `HostPopupLayoutArtifact` in the same presentation
generation used by paint, hit and input.

## `docs/plans/zircon_editor/editor_ui/05/failure-2026-07-17-template-projection-deep-copy-and-cache-generation.md`

Own compile/projection-time descriptor generation and invalidation with template document/pane model
generation; no frame-local or event-local descriptor cache.

## `docs/plans/zircon_editor/editor_layout/19-focus-and-navigation-model.md`

Own typed input/activation routes and popup keyboard navigation over the shared active-popup artifact,
including focus/select/disabled/separator and key-repeat behavior.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own runtime-level typed component/painter/input descriptor contracts and retained popup layout
authority so the editor does not permanently duplicate a string-based UI type system.

## Acceptance handoff

The handoff requires 34/34 post-change fingerprints, focused and managed Rust tests, node/kind/pane/
plugin/popup/input/placement/state/update/damage/scale matrices, same-executable WPR artifacts on D/E/F,
RenderDoc pixel/draw parity where applicable, milestone commit and quantified WeCom notification.
Protected ledgers remain unchanged until then.

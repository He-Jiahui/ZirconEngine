---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/icon.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/signals/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/signals.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/status_bar.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_shell.rs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry/metrics.rs
plan_sources:
  - user: 2026-07-14 continue component-first Unreal-style visual refinement with Runtime Text and screenshot evidence
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Private/Tree/SCurveEditorTreeFilterStatusBar.cpp
tests:
  - compact_status_signals_keep_full_runtime_text_inside_their_authored_widths
  - componentized_workbench_status_bar_adapts_primary_runtime_text_without_shrinking_fixed_controls
  - componentized_workbench_status_bar_prioritizes_primary_text_and_active_task_by_tier
  - cargo test -p zircon_editor --lib --locked paint_template_nodes::template_status_controls::tests --jobs 1 -- --nocapture --test-threads=1
  - status_control_component_visual_paints_signals_chips_icons_and_states
  - capture_status_control_component_visual_artifact
  - capture_blend_space_workspace_visual_artifacts
doc_type: module-detail
---

# Workbench Status Item

## Purpose

`WorkbenchStatusItem` is the compact marker-plus-label primitive used for Ready, Errors, Warnings, and Messages in the workbench status bar. Its native retained painter owns marker placement and Runtime Text geometry. `WorkbenchStatusBar` composes that primitive into one adaptive primary summary slot followed by tier-gated fixed diagnostic and tool controls.

## Layout Contract

The marker starts after one shared compact `gap_m` inset. Its size is also `gap_m`, and the authored component spacing supplies the marker-to-label gap. The text frame receives all remaining item width and is centered vertically with the host body-font line height.

This follows the Slate status-strip pattern: compact slots use one small outer padding before an auto-width marker/text pair. The earlier two-`gap_l` inset consumed 24 units before an 8-unit marker and 8-unit gap, leaving too little room for every authored status label even at the 1260 wide tier.

## Status Bar Composite Contract

The primary `WorkbenchStatusReady` item is the status bar's only horizontal `Stretch` child and owns a 160-unit minimum content lane. Visible Errors, Warnings, Messages, Grid, Snap, icon actions, and Zoom retain their authored fixed widths. The previous anonymous `status_fill` spacer has been removed: remaining width now belongs directly to the semantic primary summary instead of an empty layout node.

This is the same ownership shape used by Unreal Slate status strips: the informative summary receives the remaining horizontal lane while compact indicators/actions stay auto-width. It also keeps the Layout 15 boundary explicit—L2/L3 composition owns relative allocation; the L4 workbench window does not calculate pixel coordinates.

The existing shared workbench tiers define information priority without a status-specific host branch:

- Ultra/Narrow (420–640): primary summary and an optional active task remain; diagnostics and viewport tools collapse.
- Regular (641–1259): diagnostics join the primary summary; viewport tools remain collapsed. At the 641 lower bound an active task shrinks from 224 while its label/progress children shrink within matching bounds.
- Wide (1260+): all diagnostic, task, text-tool, and icon-tool controls are available.

Both idle and active-task matrices require the measured `Blend space opened` Runtime Text to fit at 420/480/481/640/641/900/1259/1260, preserve every visible fixed control width, prevent overlap, and pin the final visible item to the status-bar edge. The endpoint matrix proves the full shared tier intervals rather than only the three screenshot widths. This follows the component-first responsive rule: authored `responsive_min_tier` and bounded flex constraints decide visibility/width; L4 code does not calculate per-control pixel positions.

## Runtime Text Ownership

Status labels remain normal Runtime Text commands. Primitive regressions measure `Ready`, `No Errors`, `2 Warnings`, and `0 Messages`; the composite regression measures the dynamic primary summary with `measure_runtime_text_width`. Both add the shared raster clip guard and compare against final retained frames. No character-count estimate, feature-local font, bitmap label, or status-specific text renderer is introduced.

## Visual Validation

The old native geometry produced `signal_icon_left=24`; the new shared metric resolves to `8` with the production dense theme. The focused Runtime Text contract passes `1/1`, the full retained status-control group passes `29/29`, and the component screenshot shows complete signal labels.

The three primitive-slice Blend Space screenshots show complete `No Errors`, `2 Warnings`, and `1 Message` labels. The status-bar composite has now hard-cut the fixed primary slot plus dummy spacer into one semantic Stretch slot. Static ZUI parsing, topology inspection, and four-tier capacity checks are green. The former Runtime Text fixture blocker is resolved and a managed Windows build produced the current Editor test binary, but the full run timed out and exact execution exposed a test-only frame-owner mismatch; assertions now use the authored outer status region instead of the clipped internal root. Fresh current-source exact and screenshot evidence remain pending while unrelated Editor owner changes fail compilation before this test body.

## Constraints

- Marker inset, size, and gap come from shared host metrics or authored component projection.
- Text always uses Runtime Text measurement and painting.
- The status bar has exactly one Stretch child, and it is the semantic primary summary.
- The measured primary summary must fit from Ultra through Wide; secondary diagnostics/tools yield by shared tier before primary text.
- The task composite may compress only inside its authored 160–224 range; it must not force the primary lane below its minimum.
- Do not add control-id-specific marker coordinates or L4 window positions.
- Screenshot artifacts belong under `docs/tests/editor`; Cargo targets must remain free of editor visual evidence.

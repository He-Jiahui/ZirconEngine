# Layout15 Material state priority convergence

Plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
Milestone: M2
Status: accepted
Files: ["docs/plans/zircon_editor/editor_layout/15/2026-07-16-material-state-priority-convergence.md", "docs/superpowers/plans/2026-07-16-layout15-material-state-priority.md", "docs/tests/editor/editor-components-material-state-layer-900x360.png", "docs/zircon_editor/ui/retained_host/host_contract/paint_template_nodes/index.md", "docs/zircon_runtime/ui/v2.md", "zircon_editor/src/tests/host/retained_menu_pointer/material_state_layer_visual_screenshot.rs", "zircon_editor/src/tests/host/retained_menu_pointer/mod.rs", "zircon_editor/src/tests/host/retained_window/native_material_painter.rs", "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer/state.rs", "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/layers.rs", "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/surface.rs", "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/paint.rs", "zircon_runtime/src/ui/style.rs", "zircon_runtime/src/ui/tests/material_button_style.rs"]

## Scope delivered

- `MaterialStateLayerResolvedState` is the private retained owner for `disabled > pressed > dragging > focused > hovered > default`; opacity projection remains `0.08/0.10/0.10/0.16` and the public Button enum is unchanged.
- Runtime typed Button style keeps explicit authored interaction strings authoritative, then resolves booleans as loading, disabled, pressed, dragging, focused, hovered. Dragging intentionally folds to shared `ButtonInteractionState::Hover`.
- The specialized outlined/Workbench Button path now composes the existing shared state layer and ripple after the base surface. Named overlay/content helpers plus stable insertion produce base, state layer, ripple, selected indicator, content.
- The 900x360 component board uses relative four-column geometry and retained Runtime Text. It compares Hovered, Focused, Pressed+Focused, and Dragging+Focused against feature-off baselines and exports only to `docs/tests/editor`.

## Fresh testing evidence

| Milestone | Testing stage | Status | Evidence |
|---|---|---|---|
| M2 | M2-T current-source visual acceptance | passed | managed editor job `37b0965d5e7647bb8952c3adb523145d` exit 0; state-layer group 11 passed / 0 failed / 1 ignored; selected+ripple 1/1; capture 1/1; runtime priority 1/1 |

- Coordinator-managed Windows editor job `37b0965d5e7647bb8952c3adb523145d` completed exit 0 and produced current-source `D:/cargo-targets/zircon-engine/pool/841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025/debug/deps/zircon_editor-7cbf6e3f9c684171.exe` at 2026-07-17 06:42:41 +08:00.
- That binary ran the `state_layer` focused group: 11 passed, 0 failed, 1 ignored, including the private resolver, real Button feature-on/off structure, native mixed/winning/feature-off painter pairs, and the visual pixel guard. The selected+pressed+ripple DockTab order test separately passed 1/1. The ignored capture separately passed 1/1.
- Current runtime binary `zircon_runtime-4dc826a73923fd73.exe` ran `material_button_style_resolves_slint_state_layer_priority`: 1 passed, 0 failed. The lower runtime-interface drag-priority guard had already passed 1/1 on its current managed binary.
- The first managed editor attempt `71f1cb4adf414a20a0c900c03b32ef40` correctly exposed a stale native painter pure-base expectation (10 passed, 1 failed, 1 ignored). The replacement relational test was independently reviewed and the current-source rerun is green; no production behavior was weakened.
- `editor-components-material-state-layer-900x360.png` is 29106 bytes with SHA-256 `3A4FDB2BE4DE90F7E096DCC428812967A15718AEDF80E7239B1E3D813387E468`. Original-detail inspection found readable labels, balanced rounded surfaces/gaps, distinct focus/press treatment, stronger drag feedback, and no overlap or clipping. Exact-name scans under repository `target`, `D:/cargo-targets`, `E:/cargo-targets`, and `F:/cargo-targets` found 0 matches.
- The protected parent-plan topology/status update was committed separately through coordinator maintenance as `c617938ee73608179a8c3d69d04a2f5d1b84906c`; it is intentionally excluded from this business M2 `Files:` manifest.
- Scoped `rustfmt --edition 2021 --check` and `git diff --check` passed. No workspace manifest, lockfile, Runtime Text, Render18, or unrelated screenshot is part of this milestone manifest.

## Review

- M1 contract implementation passed specification review and independent quality review after corrections.
- M2 production/fixture implementation passed final specification review and independent quality review at Critical/Important/Minor `0/0/0`. The stale native painter regression repair separately passed specification review and independent quality review at `0/0/0`.
- Accepted residual risk: this is a focused component milestone, not a full editor or workspace suite. Runtime15's separate screen-space UI text `font_id_report` structure guard remains owned by its open failure record; it does not justify a Layout15 bypass or a false full-suite claim. The broader Layout15 and MVP editor/project/asset/play-mode goals remain active.

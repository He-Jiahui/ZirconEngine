---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens/chrome.rs
  - zircon_editor/assets/ui/editor/components/workbench/composites/chrome/workbench_panel_header.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_sample_weights.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/feedback/workbench_validation_log.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_composites.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace/composite_contracts.rs
implementation_files:
  - zircon_runtime_interface/src/ui/design_tokens/chrome.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/chrome/workbench_panel_header.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/test_editor_zui_workbench_panel_header_layout_contract.py
  - workbench_panel_header_exposes_compact_title_and_action_slots
  - blend_space_bottom_panels_compose_shared_panel_headers
  - cargo test -p zircon_editor --lib --locked zui_asset_governance -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked blend_space_workspace -- --test-threads=1
doc_type: module-detail
---

# Workbench Panel Header

`WorkbenchPanelHeader` is the shared compact header row for Workbench content panels. It replaces feature-local `HorizontalGroup` title bars without introducing another primitive or a panel-specific painter path.

## Composition Contract

The composite exposes one `title` slot and a multiple-child `actions` slot. Callers mount an existing Runtime Text component such as `WorkbenchCaption` or `WorkbenchSectionTitle` into the title slot and existing buttons, status values, or compact selectors into the action slot. The header does not own user-visible text, icon files, event routes, or local callback state.

Its root is a relative `HorizontalBox`: the title slot stretches, action children retain their authored bounded widths, and the row uses the independent 30 logical-pixel panel-header token. That height contains both the 28px dense title tier and the 30px compact action tier without clipping; pages do not publish a second header metric. The `workbench-panel-toolbar` class routes its surface through the existing token/painter path. The asset contains no raw RGB values, concrete font family, local typography override, or absolute position.

## Reference Basis

Unreal Starship separates `Header`, `Panel`, and `Recessed` colors, uses compact slim-toolbar padding, and places narrow ellipsis/actions at the trailing edge of header rows. Zircon follows that structure while keeping its own palette and Runtime Text pipeline. The accepted consumers are Preview Timeline, Sample Weights, Validation Log, Sample Grid, Blend Preview, and Weight Heatmap. The center-panel migration keeps SampleGrid/PreviewViewport/WeightHeatmap data and event ownership in their existing typed components; only the title/action chrome is shared.

## Validation Boundary

Source-level RED first confirmed the shared asset and all three consumers were absent. The implementation-stage checks parse all affected ZUI documents and verify the shared import, component use, title slot, action slot, relative height, and token inheritance.

The final Windows Editor lib-test build completed in a coordinator-managed retained target and emitted the current 3,157-test binary. The source composition and production-bridge slot/frame regressions pass `2/2`; the complete Blend Space group passes `16 passed / 0 failed / 1 ignored`; and ZUI governance passes `75/75`. The ignored screenshot route passes `1/1` and refreshes the 640x520, 900x620, and 1260x780 production-window artifacts under `docs/tests/editor`. Manual review accepts the center and bottom panel header alignment, Canvas trailing status containment, narrow-tier collapse, and wide-tier relative layout while retaining the broader whole-window visual work as active. Repository and D/E/F Cargo target scans contain zero matching Blend Space PNG files.

Mandatory review removed the last instance-local header gap/height overrides so the composite alone owns the 2px gap and 28–30px band. It also added a production-bridge regression that requires all three title nodes, plus the Preview Timeline action node, to expand under the shared header root and remain inside its inherited frame. The Runtime Text/SDF and Frameworks 05 transient compile drifts were repaired by their owners; the upward rebuild and all declared Editor gates now pass on current source.

The center-panel migration started with source-level RED missing four required shared-header declarations, then reached source GREEN with all four present and a 60-node TOML parse. Its production-bridge regression extends the same parent/frame assertions to Sample Grid, Blend Preview, and Weight Heatmap, including the Canvas trailing status action. Current-source Cargo, full governance, screenshot export, target-path scan, and manual three-tier visual acceptance all pass for this slice. The final hashes are 640 `B75F08CA3555DBA99D75939E20CDE478B19096EF94D4867406DC1521701CC832`, 900 `6CF6A2524B4DB068E4333FE29180F43041EF7095A811248D3C91568CB9F0A2EF`, and 1260 `F448EFB78808836ACC10530A55E292CCF2CC94738EC348C31D37B3880BEA4BDB`.

Mandatory review found that the first source contract embedded an LF-only cross-line substring and would fail after a Windows CRLF checkout. The final test parses the authored document as `toml::Value`, resolves each header through the `nodes` table, and verifies component identity plus each child node/slot mapping structurally. The production-bridge test remains the separate authority for expanded parent identities and final frames. Post-fix exact, complete Blend Space, and governance reruns pass, and independent rereview is Ready with no Critical or Important findings.

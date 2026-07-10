---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/compute.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/side_width_allocation.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/window_minimums.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/build.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/collapsed_constraints.rs
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/retained_host/ui/template_layout_context.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/shell_metrics.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/handle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header/side.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_drawer_breakpoints.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/layout_assertions.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/STabDrawer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabDrawer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabSidebar.cpp
  - docs/ui-and-layout/editor-workbench-designs/drawer-collapsed-state-spec.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo test -p zircon_runtime_interface --lib editor_design_tokens --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib workbench_layout_tiers_classify_reference_capture_widths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib workbench_breakpoint_defaults_are_sourced_from_design_tokens --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib compact_region_limits_follow_breakpoint_density_defaults --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib workbench_window_minimums_allow_reference_capture_sizes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib narrow_workbench_geometry_collapses_right_drawer_to_rail --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib --no-default-features side_dock_header_nodes_compact_inactive_tabs_inside_narrow_panel -- --exact --nocapture
  - cargo test -p zircon_editor --lib --no-default-features regular_workbench -- --nocapture
  - cargo test -p zircon_editor --lib --no-default-features side_width_allocation -- --nocapture
  - cargo test -p zircon_editor --lib window_scale_factor_defaults_to_one_and_filters_invalid_values --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-layout-tier-logical-0705 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib logical_width --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-layout-tier-logical-0705 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --lib --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-layout-tier-logical-0705 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never -- --ignored --test-threads=1 --nocapture
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
doc_type: module-detail
status: in_progress
---

# Workbench Autolayout Breakpoints

`layout_tier.rs` owns the Workbench shell breakpoint classification used by editor layout plan S15.5. The current accepted tiers are Ultra at 480 logical px and below, Narrow above 480 and up to 640, Regular above 640 and below 1260, and Wide at 1260 logical px and above. The defaults are projected from `EditorDensityTokens::workbench_dense()` and mirrored in `editor_tokens.zui`, so breakpoint thresholds, compact drawer clamps, bottom-region clamps, and minimum-window limits share one token source.

The tier API is explicit about units. Logical-width consumers call `workbench_layout_tier_for_logical_width(...)`; physical window consumers call the physical helpers with `scale_factor`, which first compute `physical_width / scale_factor`. The retained host stores `shell_scale_factor`, reads it through `UiHostWindow.window().scale_factor()`, and passes it into both shell geometry and the componentized Workbench bridge. The window contract now owns a normalized `window_scale_factor` with a default of 1.0, and the native winit event-loop sync writes `Window::scale_factor()` into that contract before retained-host recompute. This closes the previous R3/R4 path where 3840 physical pixels at 2.0 scale could be classified differently from 1920 physical pixels at 1.0 scale.

The first implemented consumer is the right drawer collapse rule. `compute_workbench_shell_geometry(...)` asks the tier owner whether the right drawer should be forced into the collapsed rail state. If so, `build_tool_region_state(...)` returns the existing side collapsed constraints from `collapsed_constraints.rs`, using the same `WorkbenchChromeMetrics::rail_width` as normal collapsed drawers. This avoids a second rail-width constant and keeps right-side splitters hidden through the existing expanded-state checks.

`geometry/region_frames.rs` now reads compact side and bottom defaults from the tier owner rather than owning local fallback constants. `geometry/window_minimums.rs` also consumes token-backed minimum-window limits, allowing the documented 640x420 and 900x620 screenshot captures while still clamping to lower Ultra limits at very small sizes.

The retained-host Workbench template bridge consumes the same tier rule in `drawer_layout.rs`. At narrow width it resolves the right drawer extent to the rail width, which makes the componentized right drawer shell and content roots collapse to zero visible width. This keeps rendered template content aligned with the autolayout geometry; without the bridge change, the geometry would be rail-sized while the surface still painted the full Inspector shell.

Regular width now has a second, joint budget rule rather than two independent side clamps. `minimum_document_width_fraction` is defined once in `EditorDensityTokens` and authored as `--minimum-document-width-fraction`; both raw autolayout geometry and the retained-host drawer bridge reserve that fraction for the document after accounting for visible rails and separators. `side_width_allocation.rs` owns the shared deterministic policy: release width from the larger side first until the side panels are balanced, then shrink both proportionally only if the central document still needs space. The 900-wide Assets drawer therefore keeps about half of the shell for the document instead of allowing the two drawers to reduce it to roughly 40 percent.

The side-dock header consumes the final panel width through the runtime text measurement path. It reserves the active tab's complete preferred label first, keeps other tabs as 32px icon slots while budget remains, and removes overflow slots from the projected node tree. At 900 width the active Assets label stays readable; at 640 the active Project label stays readable while lower-priority tabs compact; at 1260 the wider dock restores additional labels. The tab order and hit frames remain relative to the resolved slot sequence, with no screenshot-coordinate branches.

The same tier source drives follow-up consumers outside raw shell geometry. `page_tabs/metrics.rs` treats Ultra and Narrow as the aggressive overflow tier for main page tabs, and `retained_host/ui/template_layout_context.rs` maps Ultra and Narrow to `layoutNarrow` so table rows can drop low-priority columns under the same responsive contract.

The design reference is Unreal's Slate drawer pattern: side tabs/rails stay available while heavy drawer content opens separately and should not keep squeezing the central editor area when space is scarce. Zircon currently implements only the deterministic geometry/content-shell part of that behavior. Overlay drawer interaction remains owned by the docking/window plans.

Current evidence:

- `workbench_layout_tiers_classify_reference_capture_widths` locks 420/480/640/900/1260 as Ultra/Narrow/Regular/Wide boundary behavior.
- `workbench_breakpoint_defaults_are_sourced_from_design_tokens` locks breakpoint and compact defaults to central density tokens.
- `compact_region_limits_follow_breakpoint_density_defaults` locks side and bottom compact clamps to the same token-backed defaults.
- `workbench_window_minimums_allow_reference_capture_sizes` locks the lowered minimum-window contract for 640x420 and 900x620 captures.
- `narrow_workbench_geometry_collapses_right_drawer_to_rail` locks the right region to rail width and removes the right splitter at 640 while preserving a regular right drawer at 900.
- `componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width` locks the retained-host bridge so right drawer shell/content frames disappear at 640 and return at 900.
- `regular_workbench_geometry_reserves_token_backed_document_width` and `componentized_regular_workbench_reserves_half_width_for_the_document` lock both geometry paths to the same token-backed central document reserve.
- The two `side_width_allocation` tests lock the larger-side-first and no-op-when-fit behavior; `side_dock_header_nodes_compact_inactive_tabs_inside_narrow_panel` locks the real five-tab 226px case so the active label stays complete while overflow tabs disappear.
- `capture_m3_gui_acceptance_visual_artifacts` refreshes the M3 Workbench screenshots under `docs/tests/editor/`; the latest 900x620 capture is `editor-window-m3-workbench-900x620.png`. Screenshot output stays outside Cargo `target`.
- `window_scale_factor_defaults_to_one_and_filters_invalid_values` locks the retained-host window scale contract default, valid write path, and invalid-value fallback.
- `tier_uses_logical_width_consistent_across_scale`, `scaled_workbench_geometry_uses_logical_width_for_right_drawer_collapse`, and `componentized_workbench_layout_collapses_right_drawer_shell_by_logical_width_under_scale` lock the logical-width cutover. The 2026-07-05 focused Cargo run passed 3/3 with existing warning noise only.
- The 2026-07-05 M3 screenshot rerun passed 1/1 and refreshed `editor-window-m3-svg-icon-scale-small-640x420.png`, `editor-window-m3-workbench-900x620.png`, and `editor-window-m3-svg-icon-scale-large-1260x780.png` under `docs/tests/editor/`. SHA256 evidence: small `1EA0D63A88BF18C1312F712D0D6147E675AED6DA20F11C913A2810D81B3F1432`, workbench `A57E93CD530BDA66AAB5B18DA102C276EAE654C12BDA57654E1C6C0506C430AB`, large `73BF5465BF3C272DA27296EE25E9B16FC1B0A66ABA3E306A02EEA77A15B98E7F`. Target screenshot scan returned no matching editor/workbench screenshots.
- The 2026-07-10 regular document-reserve rerun passed dock-header 5/5, side allocation 2/2, regular geometry/bridge 2/2, narrow/logical collapse 4/4, and M3 capture 1/1. Manual review accepts active-label readability and central document reserve at all three tiers. Current SHA256 evidence: 900 Assets drawer `2F5319A7FB5292E07409C306DA0C07C813EBD01F35D8BE1F14D29488BA104F08`, 640 small `B4DB2C7287DDE00D43E48D3982AD05E497288268CC23BB7B43C5AD4D50480161`, 1260 large `35DFA7C49E69E307BFB00DFF9A58BD59C34E3C1DCA407DDB9C2193E52B215602`. Repository and external target scans found zero matching PNG outputs.

Remaining S15.5 work is explicit: broader authored window chrome composition, overlay drawer interaction, and additional complex-list/popup visual parity remain separate work. Breakpoint thresholds, compact defaults, joint regular-width document reserve, responsive side tabs, table/list narrow context, lowered window minimums, Ultra tier classification, and the code-side scale-factor plumbing are implemented, focused-Cargo verified, and refreshed through the M3 screenshot harness.

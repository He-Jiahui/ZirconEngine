---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.v2.ui.toml
  - zircon_editor/src/ui/workbench/autolayout/layout_tier.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/compute.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/window_minimums.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/build.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/collapsed_constraints.rs
  - zircon_editor/src/ui/workbench/page_tabs/metrics.rs
  - zircon_editor/src/ui/retained_host/ui/template_layout_context.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_drawer_breakpoints.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/STabDrawer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabDrawer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabSidebar.cpp
  - docs/ui-and-layout/editor-workbench-designs/drawer-collapsed-state-spec.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo test -p zircon_runtime_interface --lib editor_design_tokens --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib workbench_layout_tiers_classify_reference_capture_widths --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib workbench_breakpoint_defaults_are_sourced_from_design_tokens --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib compact_region_limits_follow_breakpoint_density_defaults --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib workbench_window_minimums_allow_reference_capture_sizes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib narrow_workbench_geometry_collapses_right_drawer_to_rail --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never -- --ignored --test-threads=1 --nocapture
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never --message-format short
doc_type: module-detail
status: in_progress
---

# Workbench Autolayout Breakpoints

`layout_tier.rs` owns the Workbench shell breakpoint classification used by editor layout plan S15.5. The current accepted tiers are Ultra at 480 px and below, Narrow above 480 and up to 640, Regular above 640 and below 1260, and Wide at 1260 px and above. The defaults are projected from `EditorDensityTokens::workbench_dense()` and mirrored in `editor_tokens.v2.ui.toml`, so breakpoint thresholds, compact drawer clamps, bottom-region clamps, and minimum-window limits share one token source.

The first implemented consumer is the right drawer collapse rule. `compute_workbench_shell_geometry(...)` asks the tier owner whether the right drawer should be forced into the collapsed rail state. If so, `build_tool_region_state(...)` returns the existing side collapsed constraints from `collapsed_constraints.rs`, using the same `WorkbenchChromeMetrics::rail_width` as normal collapsed drawers. This avoids a second rail-width constant and keeps right-side splitters hidden through the existing expanded-state checks.

`geometry/region_frames.rs` now reads compact side and bottom defaults from the tier owner rather than owning local fallback constants. `geometry/window_minimums.rs` also consumes token-backed minimum-window limits, allowing the documented 640x420 and 900x620 screenshot captures while still clamping to lower Ultra limits at very small sizes.

The retained-host Workbench template bridge consumes the same tier rule in `drawer_layout.rs`. At narrow width it resolves the right drawer extent to the rail width, which makes the componentized right drawer shell and content roots collapse to zero visible width. This keeps rendered template content aligned with the autolayout geometry; without the bridge change, the geometry would be rail-sized while the surface still painted the full Inspector shell.

The same tier source drives follow-up consumers outside raw shell geometry. `page_tabs/metrics.rs` treats Ultra and Narrow as the aggressive overflow tier for main page tabs, and `retained_host/ui/template_layout_context.rs` maps Ultra and Narrow to `layoutNarrow` so table rows can drop low-priority columns under the same responsive contract.

The design reference is Unreal's Slate drawer pattern: side tabs/rails stay available while heavy drawer content opens separately and should not keep squeezing the central editor area when space is scarce. Zircon currently implements only the deterministic geometry/content-shell part of that behavior. Overlay drawer interaction remains owned by the docking/window plans.

Current evidence:

- `workbench_layout_tiers_classify_reference_capture_widths` locks 420/480/640/900/1260 as Ultra/Narrow/Regular/Wide boundary behavior.
- `workbench_breakpoint_defaults_are_sourced_from_design_tokens` locks breakpoint and compact defaults to central density tokens.
- `compact_region_limits_follow_breakpoint_density_defaults` locks side and bottom compact clamps to the same token-backed defaults.
- `workbench_window_minimums_allow_reference_capture_sizes` locks the lowered minimum-window contract for 640x420 and 900x620 captures.
- `narrow_workbench_geometry_collapses_right_drawer_to_rail` locks the right region to rail width and removes the right splitter at 640 while preserving a regular right drawer at 900.
- `componentized_workbench_layout_collapses_right_drawer_shell_at_narrow_width` locks the retained-host bridge so right drawer shell/content frames disappear at 640 and return at 900.
- `capture_m3_gui_acceptance_visual_artifacts` refreshes the M3 Workbench screenshots under `docs/tests/editor/`; the latest 900x620 capture is `editor-window-m3-workbench-900x620.png`. Screenshot output stays outside Cargo `target`.

Remaining S15.5 work is explicit: popup anchor spacing still needs tokenization, and the next visual pass should continue from primitive component chrome into authored page-tab/window chrome composition. Breakpoint thresholds, compact defaults, table/list narrow context, lowered window minimums, and Ultra tier classification are now implemented and covered by focused tests.

# Global UI Material, Responsive Layout, And Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every editor/runtime `.ui.toml` surface follow the same Material-style component, responsive layout, intrinsic text sizing, native painter, and live refresh diagnostics contracts instead of fixing only Asset Browser.

**Architecture:** Keep generated Slint UI out of the shipping path. `.ui.toml` assets define component usage, Material tokens, and responsive layout intent; `zircon_runtime::ui` computes intrinsic sizes and arranged geometry; `zircon_editor::ui::slint_host` projects that geometry into the Rust-owned host contract; the native painter consumes projected frames and diagnostic counters without recomputing layout or adding per-screen coordinate tables.

**Tech Stack:** Rust, Cargo, TOML `.ui.toml` assets, `UiSurfaceFrame` arranged geometry, softbuffer native presenter, Slint Material reference assets under `dev/slint/ui-libraries/material`, Unreal Slate invalidation/layout precedent, Markdown docs.

---

## Execution Policy

- Work directly on `main` in the existing checkout.
- Do not create worktrees or feature branches.
- Do not commit unless the user explicitly asks for a commit.
- Preserve unrelated dirty work from active sessions. Do not revert, format, or rewrite files outside the current milestone ownership unless this plan explicitly lists them.
- Use the `zirconEngine` milestone-first cadence: implementation slices may add code, tests, and docs, but compile/build/unit-test execution belongs to the named testing stage for each milestone unless a blocker requires earlier evidence.
- Apply support-first debugging when an upper-layer UI snapshot fails: inspect shared layout/material/text support first before adding a screen-specific workaround.
- Do not restore generated `.slint` UI modules. All visible editor/runtime UI must continue through `.ui.toml -> runtime layout -> host projection -> native painter`.
- Do not add Rust coordinate tables or screen-specific dispatch branches for Material controls. If a control needs different behavior, express it through `.ui.toml` component descriptors, layout metadata, style classes, binding metadata, or shared runtime contracts.
- Before each milestone implementation, refresh `.codex/plans` and `.codex/sessions` with the cross-session coordination script and update `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md` or a replacement note with the active milestone.

## Global Scope Rule

This plan intentionally covers all current `.ui.toml` surfaces, not only Asset Browser. Treat the following inventory as the minimum global UI surface set that must conform before the plan is complete:

```text
zircon_editor/assets/ui/editor/*.ui.toml
zircon_editor/assets/ui/editor/host/*.ui.toml
zircon_editor/assets/ui/editor/windows/*.ui.toml
zircon_editor/assets/ui/runtime/*.ui.toml
zircon_editor/assets/ui/theme/*.ui.toml
```

Current concrete files discovered during planning include:

```text
zircon_editor/assets/ui/editor/material_meta_components.ui.toml
zircon_editor/assets/ui/theme/editor_material.ui.toml
zircon_editor/assets/ui/theme/editor_base.ui.toml
zircon_editor/assets/ui/editor/component_showcase.ui.toml
zircon_editor/assets/ui/editor/asset_browser.ui.toml
zircon_editor/assets/ui/editor/assets_activity.ui.toml
zircon_editor/assets/ui/editor/animation_editor.ui.toml
zircon_editor/assets/ui/editor/binding_browser.ui.toml
zircon_editor/assets/ui/editor/component_widgets.ui.toml
zircon_editor/assets/ui/editor/console.ui.toml
zircon_editor/assets/ui/editor/editor_widgets.ui.toml
zircon_editor/assets/ui/editor/hierarchy.ui.toml
zircon_editor/assets/ui/editor/inspector.ui.toml
zircon_editor/assets/ui/editor/layout_workbench.ui.toml
zircon_editor/assets/ui/editor/preview_state_lab.ui.toml
zircon_editor/assets/ui/editor/project_overview.ui.toml
zircon_editor/assets/ui/editor/theme_browser.ui.toml
zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml
zircon_editor/assets/ui/editor/welcome.ui.toml
zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml
zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
zircon_editor/assets/ui/editor/workbench_menu_popup.ui.toml
zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml
zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml
zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml
zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
zircon_editor/assets/ui/editor/host/build_export_desktop_body.ui.toml
zircon_editor/assets/ui/editor/host/console_body.ui.toml
zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml
zircon_editor/assets/ui/editor/host/inspector_body.ui.toml
zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml
zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml
zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml
zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
zircon_editor/assets/ui/editor/host/workbench_bottom_dock_header.ui.toml
zircon_editor/assets/ui/editor/host/workbench_document_dock_header.ui.toml
zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
zircon_editor/assets/ui/editor/host/workbench_menu_chrome.ui.toml
zircon_editor/assets/ui/editor/host/workbench_page_chrome.ui.toml
zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
zircon_editor/assets/ui/editor/host/workbench_side_dock_header.ui.toml
zircon_editor/assets/ui/editor/windows/asset_window.ui.toml
zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml
zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml
zircon_editor/assets/ui/runtime/inventory_dialog.ui.toml
zircon_editor/assets/ui/runtime/pause_dialog.ui.toml
zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml
zircon_editor/assets/ui/runtime/runtime_hud.ui.toml
zircon_editor/assets/ui/runtime/settings_dialog.ui.toml
```

## Current Repository Baseline

- Material meta components already exist in `zircon_editor/assets/ui/editor/material_meta_components.ui.toml` and carry layout metadata such as `layout_padding_left`, `layout_min_height`, `layout_icon_size`, and Material classes.
- `zircon_runtime/src/ui/layout/pass/material.rs` already centralizes Material layout metric measurement for Button, IconButton, ToggleButton, Checkbox, InputField, TextField, ListRow, ComboBox, RangeField, NumberField, and Switch.
- `zircon_runtime/src/ui/layout/pass/measure.rs` already calls `measure_material_content(...)` and still has fallback button padding for non-Material controls.
- `zircon_editor/assets/ui/theme/editor_material.ui.toml` already contains Material-like color tokens and class rules, but `editor_base.ui.toml` and many layout assets still use legacy `.editor-shell`, `.panel`, fixed sizes, or no Material style import.
- `zircon_editor/src/ui/slint_host/host_contract/presenter.rs` already tracks `present_count`, `full_paint_count`, `region_paint_count`, and `painted_pixel_count` internally and writes diagnostic logs, but the top-right overlay receives a static string through `HostWindowShellData.debug_refresh_rate`.
- `HostRedrawRequest` and retained `SoftbufferHostPresenter` already support full vs region redraw and backbuffer region repaint.
- `HostInvalidationRoot` already has invalidation masks and counters for layout, presentation, render, paint-only, hit-test, and window metrics requests.
- Active sibling sessions are touching event routing, material layout foundation, native text/input, and shared Slate layout/hit-grid files. This plan must advance through coordinated milestones rather than broad opportunistic edits.

## Reference Alignment

- Primary UI toolkit reference: `dev/slint/ui-libraries/material` for Material component metrics, token names, state layers, and control density.
- Primary invalidation/layout reference: Unreal Slate (`FSlateInvalidationRoot`, arranged geometry, hit grid, widget path) via the existing `.codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md` and `.codex/plans/Editor 绘制与鼠标事件优化计划.md`.
- Rust/editor split reference: Fyrox-style editor/runtime separation. Runtime layout and shared contracts stay in `zircon_runtime` / `zircon_runtime_interface`; editor host diagnostics and native presentation stay in `zircon_editor`.

## Global UI Contracts

Every `.ui.toml` surface must satisfy these contracts when the plan is complete:

1. Material import contract: editor authoring surfaces import `res://ui/theme/editor_material.ui.toml`; runtime demo/game UI either imports the same Material theme or a documented runtime Material theme that maps the same token names.
2. Component contract: interactive controls use Material meta components or carry equivalent Material classes/layout props. Plain `Button`, `TextField`, `IconButton`, `ListRow`, `ComboBox`, `Switch`, and similar native controls are allowed only when wrapped by a Material meta component or explicitly documented as chrome-only exceptions.
3. Layout contract: fixed pixel width/height is allowed only for chrome rails, icon squares, status bars, or intentional bounded dialogs. Main panels, content bodies, lists, tables, toolbar rows, and editor/runtime dialogs must use stretch, min/preferred/max, scroll, or responsive container behavior.
4. Text contract: labels/buttons/text fields derive desired size from shared text measurement plus Material padding/min-size. Native painter may clip to the arranged frame but must not be the first layer that makes text fit.
5. Render contract: SVG/icon/image assets resolve through the existing visual asset path; missing assets use deterministic fallback only as a visible failure marker.
6. Diagnostics contract: top-right overlay shows live counters derived from presenter/invalidation state, not static text.
7. Damage contract: hover, press, viewport image, text cursor, scroll, and overlay counter changes prefer local `HostRedrawRequest::Region`; layout/data changes are the only path to full presentation rebuild.

## Affected Files By Layer

### Shared Runtime Layout And Contracts

- Modify `zircon_runtime_interface/src/ui/layout/scroll.rs` only if a new responsive container enum is needed.
- Modify `zircon_runtime_interface/src/ui/layout/mod.rs` only to re-export a new layout contract type.
- Modify `zircon_runtime/src/ui/template/build/parsers.rs` if `.ui.toml` needs new `layout.container.kind` parsing such as `WrapBox`.
- Modify `zircon_runtime/src/ui/template/build/layout_contract.rs` only to merge and carry new layout contract fields.
- Modify `zircon_runtime/src/ui/layout/pass/measure.rs` for shared desired-size behavior.
- Modify `zircon_runtime/src/ui/layout/pass/material.rs` for Material metric coverage and global control support.
- Modify `zircon_runtime/src/ui/layout/pass/arrange.rs`, `axis.rs`, and `child_frame.rs` if responsive wrapping or breakpoint behavior needs arrangement support.
- Add or modify tests under `zircon_runtime/src/ui/tests/material_layout.rs`, `shared_core.rs`, and `text_layout.rs`.

### Editor UI Assets And Theme

- Modify `zircon_editor/assets/ui/theme/editor_material.ui.toml` as the global Material token/style owner.
- Modify `zircon_editor/assets/ui/theme/editor_base.ui.toml` to either import/delegate to Material tokens or shrink to compatibility chrome tokens only.
- Modify `zircon_editor/assets/ui/editor/material_meta_components.ui.toml` as the global Material component asset.
- Modify all `.ui.toml` surfaces listed in the Global Scope Rule so they import Material style and use Material classes/components consistently.
- Add `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs` and register it in `zircon_editor/src/tests/ui/boundary/mod.rs` to enforce global `.ui.toml` conformance.
- Modify `zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs` or add focused host/template tests only when projection coverage is needed.

### Editor Native Host Diagnostics And Painter

- Create `zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs` for DTOs and formatting if live overlay state cannot stay small inside existing data modules.
- Modify `zircon_editor/src/ui/slint_host/host_contract/data/host_components.rs` to replace or augment `debug_refresh_rate` with structured diagnostic text fields if needed.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`, `shell_presentation.rs`, and `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs` to project live diagnostics instead of static text.
- Modify `zircon_editor/src/ui/slint_host/host_contract/presenter.rs` to expose a small `HostPresenterDiagnostics` snapshot without leaking softbuffer internals.
- Modify `zircon_editor/src/ui/slint_host/app/invalidation.rs` to expose counters that the overlay can summarize.
- Modify `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs` only for overlay rendering and Material-aware host chrome colors that cannot come from template nodes yet. This file is near 1000 lines; if new overlay helpers grow beyond a small function, extract a focused `painter/diagnostics_overlay.rs` module and wire it from `painter/mod.rs`.
- Add or extend tests in `zircon_editor/src/tests/host/slint_window/shell_window.rs`, `native_host_contract.rs`, or a new `native_diagnostics.rs` module.

### Documentation And Acceptance

- Update `docs/ui-and-layout/runtime-ui-component-showcase.md` for Material component/global surface conformance.
- Update `docs/ui-and-layout/shared-ui-core-foundation.md` for text measurement/responsive layout contracts.
- Update `docs/ui-and-layout/slate-style-ui-surface-frame.md` for invalidation, damage, and arranged geometry authority.
- Update `docs/editor-and-tooling/editor-workbench-shell.md` for live top-right diagnostics and native host rendering behavior.
- Update `docs/assets-and-rendering/runtime-ui-graphics-integration.md` only when visual asset or painter contracts change.
- Create or update `tests/acceptance/global-ui-material-responsive-diagnostics.md` with the exact validation inventory and results.

## Milestone 0: Coordination And Baseline Inventory

- Goal: Freeze a global UI inventory and active-session boundary before implementation begins.
- In-scope behaviors: all `.ui.toml` surfaces are enumerated; active conflicting sessions are read; this plan is referenced from the active session note.
- Dependencies: existing `.codex/plans`, `.codex/sessions`, and current dirty worktree.

### Implementation Slices

- [x] Run the coordination context script before editing:

```powershell
.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4
```

- [x] Read active session notes that touch `zircon_runtime/src/ui`, `zircon_editor/assets/ui`, `zircon_editor/src/ui/slint_host`, and `docs/ui-and-layout`.
- [x] Update `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md` so `Current Step` says `Global UI Material/responsive/diagnostics convergence`, and add this plan file to `related_plans`.
- [x] Generate a current `.ui.toml` inventory with `Glob` or equivalent and compare it against the Global Scope Rule above.
- [x] If a new `.ui.toml` file exists that is not listed above, add it to this plan and the acceptance inventory before implementation.

### Testing Stage: Baseline Gate

- [x] No Cargo build is required for this milestone.
- [x] Record the coordination script timestamp and the active conflicting sessions in the session note.
- [x] Record the final `.ui.toml` inventory in `tests/acceptance/global-ui-material-responsive-diagnostics.md`.

### Exit Evidence

- The active session note names the global scope and this plan.
- The acceptance file contains the current `.ui.toml` inventory.
- No source behavior changes are made in this milestone.

## Milestone 1: Live FPS And Refresh Diagnostics Overlay

- Goal: Replace the static top-right debug marker with live refresh diagnostics based on presenter and invalidation counters.
- In-scope behaviors: FPS estimate, present count, full paint count, region paint count, painted pixel count, invalidation slow-path count, render rebuild count, paint-only request count, and a stable overlay string updated through the native host contract.
- Dependencies: existing `SoftbufferHostPresenter` counters, `HostInvalidationRoot`, `HostWindowShellData.debug_refresh_rate`, and `workbench.rs::draw_debug_refresh_rate_marker(...)`.

### Implementation Slices

- [x] Create `zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs` with a focused DTO if keeping this state in `presenter.rs` would mix formatting and softbuffer presentation. Use this shape:

```rust
use std::time::Instant;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostRefreshDiagnostics {
    pub present_count: u64,
    pub full_paint_count: u64,
    pub region_paint_count: u64,
    pub painted_pixel_count: u64,
    pub slow_path_rebuild_count: u64,
    pub render_rebuild_count: u64,
    pub paint_only_request_count: u64,
    first_present_at: Option<Instant>,
    last_present_at: Option<Instant>,
}

impl Default for HostRefreshDiagnostics {
    fn default() -> Self {
        Self {
            present_count: 0,
            full_paint_count: 0,
            region_paint_count: 0,
            painted_pixel_count: 0,
            slow_path_rebuild_count: 0,
            render_rebuild_count: 0,
            paint_only_request_count: 0,
            first_present_at: None,
            last_present_at: None,
        }
    }
}

impl HostRefreshDiagnostics {
    pub(crate) fn record_present(&mut self, painted_pixels: u64, full_paint: bool, region_paint: bool) {
        let now = Instant::now();
        if self.first_present_at.is_none() {
            self.first_present_at = Some(now);
        }
        self.last_present_at = Some(now);
        self.present_count = self.present_count.saturating_add(1);
        if full_paint {
            self.full_paint_count = self.full_paint_count.saturating_add(1);
        }
        if region_paint {
            self.region_paint_count = self.region_paint_count.saturating_add(1);
        }
        self.painted_pixel_count = self.painted_pixel_count.saturating_add(painted_pixels);
    }

    pub(crate) fn with_invalidation_counts(
        mut self,
        slow_path_rebuild_count: u64,
        render_rebuild_count: u64,
        paint_only_request_count: u64,
    ) -> Self {
        self.slow_path_rebuild_count = slow_path_rebuild_count;
        self.render_rebuild_count = render_rebuild_count;
        self.paint_only_request_count = paint_only_request_count;
        self
    }

    pub(crate) fn fps(&self) -> Option<f32> {
        let start = self.first_present_at?;
        let end = self.last_present_at?;
        let seconds = end.duration_since(start).as_secs_f32();
        (seconds > 0.0).then_some(self.present_count as f32 / seconds)
    }

    pub(crate) fn overlay_text(&self) -> String {
        format!(
            "FPS {:.1} | present {} | full {} | region {} | pixels {} | slow {} | paint-only {}",
            self.fps().unwrap_or(0.0),
            self.present_count,
            self.full_paint_count,
            self.region_paint_count,
            self.painted_pixel_count,
            self.slow_path_rebuild_count,
            self.paint_only_request_count,
        )
    }
}
```

- [x] Wire `diagnostics.rs` from `zircon_editor/src/ui/slint_host/host_contract/mod.rs` or the narrow parent module that already owns host-contract exports. Keep the root file structural.
- [x] Modify `SoftbufferHostPresenter::present(...)` so it records whether the repaint was full or region and exposes `diagnostics_snapshot()` returning `HostRefreshDiagnostics` or a plain snapshot struct.
- [x] Modify `HostInvalidationRoot` to expose the current `slow_path_rebuilds`, `render_rebuilds`, and `paint_only_requests` through a small snapshot method instead of parsing `stats_summary()`.
- [x] Add a host-level method that merges presenter and invalidation snapshots into `HostWindowShellData.debug_refresh_rate` before repaint. If the existing lifecycle cannot access both states at one point, store the formatted string in host globals immediately after `present(...)` and use the latest string on the next frame.
- [x] Preserve the static fallback string only for no-present-yet startup: `FPS 0.0 | present 0 | full 0 | region 0`.
- [x] Add focused tests covering:
  - Overlay text changes after two recorded presents.
  - Region presents increment `region_paint_count` and not `full_paint_count`.
  - Full repaint increments `full_paint_count`.
  - Invalidation paint-only counter appears in overlay text.
  - Existing top-right marker snapshot test still verifies visible pixels.

### Testing Stage: Diagnostics Gate

Run targeted formatting for changed files:

```powershell
rustfmt --edition 2021 --check "zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs" "zircon_editor/src/ui/slint_host/host_contract/presenter.rs" "zircon_editor/src/ui/slint_host/app/invalidation.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs" "zircon_editor/src/tests/host/slint_window/shell_window.rs"
```

If `diagnostics.rs` is not created, omit it from the command.

Run focused tests:

```powershell
cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo test -p zircon_editor --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
```

### Debug / Correction Loop

- If overlay text is stale by one frame, document and test the one-frame delay only if the next present reliably shows updated data. Prefer same-frame update if lifecycle access permits it.
- If tests cannot instantiate `SoftbufferHostPresenter`, test `HostRefreshDiagnostics` directly and keep one existing painter snapshot test for pixel visibility.
- If touching `workbench.rs` grows it beyond the current overlay helper, extract `diagnostics_overlay.rs` instead of adding another section to the 900+ line painter file.

### Exit Evidence

- The overlay text is live and counter-based.
- Focused diagnostics and top-right overlay tests pass.
- Documentation records that the overlay is derived from presenter/invalidation counters.
- Accepted in `tests/acceptance/global-ui-material-responsive-diagnostics.md` with formatting, snapshot, diagnostics, and editor library compile evidence. The final compile check passed with pre-existing warnings only.

## Milestone 2: Shared Text And Material Measurement For All Controls

- Goal: Every Material-capable control across all surfaces derives desired size from shared text measurement plus Material metrics, avoiding painter-side text clipping fixes.
- In-scope behaviors: Button, IconButton, ToggleButton, Checkbox, InputField, TextField, ListRow, ComboBox, RangeField, NumberField, Switch, menu item, tab, table row, and runtime dialog controls.
- Dependencies: existing `measure_material_content(...)`, `measure_text(...)`, `UiTemplateNodeMetadata`, and Material meta-component tokens.

### Implementation Slices

- [ ] Extend `supports_material_layout(...)` in `zircon_runtime/src/ui/layout/pass/material.rs` to cover all Material meta component root `type` values and common native roles used by current `.ui.toml` files. The target list must include:

```text
Button
IconButton
ToggleButton
Checkbox
InputField
TextField
ListRow
ComboBox
RangeField
NumberField
Switch
MenuItem
Tab
TableRow
Label
```

- [ ] Keep `Label` Material sizing conservative: only apply Material padding/min-height when it has at least one `layout_*` attribute; plain labels without Material layout attributes should keep current text-only measurement.
- [ ] Update `MaterialLayoutMetrics::has_layout_attribute(...)` if new keys are required for menu/tab/table rows. Reuse existing keys first:

```text
layout_padding_left
layout_padding_right
layout_padding_top
layout_padding_bottom
layout_spacing
layout_min_width
layout_min_height
layout_icon_size
layout_leading_slot_width
layout_trailing_slot_width
```

- [ ] Update `zircon_editor/assets/ui/editor/material_meta_components.ui.toml` so Material menu items, tabs, table rows, progress/spinner labels, and text edit roots include layout metrics when they present visible text or interactive rows.
- [ ] Add runtime tests in `zircon_runtime/src/ui/tests/material_layout.rs` for:
  - Long button text expands desired width and is not clipped by default frame.
  - Icon-only button keeps square Material size.
  - Menu item text uses list-row min height and horizontal padding.
  - Tab label uses control height and text width plus padding.
  - Plain non-Material label remains text-only.
- [ ] Add or update editor host projection tests to prove Component Showcase Material controls carry frames at least as wide as their labels after shared layout.

### Testing Stage: Text/Material Gate

Run targeted formatting:

```powershell
rustfmt --edition 2021 --check "zircon_runtime/src/ui/layout/pass/material.rs" "zircon_runtime/src/ui/layout/pass/measure.rs" "zircon_runtime/src/ui/tests/material_layout.rs" "zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs"
```

Run focused runtime/editor tests:

```powershell
cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
```

### Debug / Correction Loop

- If a button still clips, inspect the runtime arranged frame before changing painter text bars.
- If a Material test passes only because of fixed width in `.ui.toml`, remove the fixed width and fix shared measurement instead.
- If non-Material labels grow unexpectedly, restrict Material sizing to nodes with explicit `layout_*` attributes.

### Exit Evidence

- Shared runtime tests prove text intrinsic size plus Material padding across all in-scope controls.
- Component Showcase projection proves controls receive non-collapsed arranged frames.
- No painter-side text fitting workaround is added for this milestone.

## Milestone 3: Responsive Layout Primitives And Global Surface Rules

- Goal: All UI surfaces adapt to available size using shared layout primitives instead of screen-specific fixed dimensions.
- In-scope behaviors: responsive horizontal-to-wrapped rows, stretch-based main content, bounded dialogs, scrollable overflow, fixed-size exceptions, and a global asset conformance test.
- Dependencies: stable text/material measurement from Milestone 2 and current runtime layout pass.

### Implementation Slices

- [ ] Add a global UI asset conformance test file `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs` and register it in `zircon_editor/src/tests/ui/boundary/mod.rs`.
- [ ] In that test, enumerate all `.ui.toml` files under `zircon_editor/assets/ui/editor`, `zircon_editor/assets/ui/editor/host`, `zircon_editor/assets/ui/editor/windows`, and `zircon_editor/assets/ui/runtime`.
- [ ] The conformance test must check:
  - Every non-theme UI asset imports `res://ui/theme/editor_material.ui.toml` or imports another asset that imports it.
  - Main roots use stretch width and height unless the file is a popup/menu/dialog intentionally bounded by min/preferred/max.
  - Plain interactive native controls are either Material meta component roots or have `material-*` classes and `layout_*` metrics.
  - Fixed width/height values are allowed only for chrome rails, icon buttons, status bars, known splitter/header rows, or bounded dialogs.
  - Every scrollable/list/table-heavy surface has a `ScrollableBox` or explicit bounded viewport region.
- [ ] If existing runtime layout lacks wrapping, add a `WrapBox` container to `zircon_runtime_interface/src/ui/layout/scroll.rs`:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiWrapBoxConfig {
    pub horizontal_gap: f32,
    pub vertical_gap: f32,
    pub item_min_width: f32,
}

pub enum UiContainerKind {
    Free,
    Container,
    Overlay,
    Space,
    HorizontalBox(UiLinearBoxConfig),
    VerticalBox(UiLinearBoxConfig),
    ScrollableBox(UiScrollableBoxConfig),
    WrapBox(UiWrapBoxConfig),
}
```

- [ ] If `WrapBox` is added, update `zircon_runtime/src/ui/template/build/parsers.rs` to parse:

```toml
[node.layout.container]
kind = "WrapBox"
horizontal_gap = 8.0
vertical_gap = 8.0
item_min_width = 180.0
```

- [ ] If `WrapBox` is added, update `measure.rs` and `arrange.rs` so children wrap into multiple rows based on available width. Keep the implementation pure runtime; do not add Asset Browser-specific layout code.
- [ ] Update all global `.ui.toml` surfaces to use the shared responsive primitives. For large files, edit by section and keep the intent mechanical:
  - Toolbars: stretch main text/search fields, fixed only icon/action buttons.
  - Sidebars/nav: bounded min/preferred/max or scrollable list.
  - Content grids/cards: WrapBox or stretch+scroll.
  - Dialogs: bounded panel with stretch root overlay and scrollable body.
  - Runtime HUD/dialogs: stretch root, bounded panels, Material classes.
- [ ] Remove or replace legacy `editor_base.ui.toml` imports where the surface should be Material. If `editor_base.ui.toml` remains, make it import/delegate Material tokens instead of defining a separate visual language for normal controls.

### Testing Stage: Responsive Gate

Run targeted formatting for runtime layout changes if any:

```powershell
rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/layout/scroll.rs" "zircon_runtime_interface/src/ui/layout/mod.rs" "zircon_runtime/src/ui/template/build/parsers.rs" "zircon_runtime/src/ui/template/build/layout_contract.rs" "zircon_runtime/src/ui/layout/pass/measure.rs" "zircon_runtime/src/ui/layout/pass/arrange.rs" "zircon_runtime/src/ui/tests/material_layout.rs" "zircon_runtime/src/ui/tests/shared_core.rs" "zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs" "zircon_editor/src/tests/ui/boundary/mod.rs"
```

Run focused tests:

```powershell
cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
```

### Debug / Correction Loop

- If global conformance fails on a legitimate fixed chrome size, add a narrow allowlist entry in the test with a reason string naming the control and why it is fixed.
- If several surfaces need the same exception, add or adjust a shared Material meta component instead of broadening the allowlist.
- If responsive wrapping fails, test `WrapBox` in runtime first before changing any `.ui.toml` surface.

### Exit Evidence

- Global conformance test covers all `.ui.toml` surfaces in the inventory.
- Responsive layout tests pass at shared runtime level.
- Asset Browser is covered as one member of the global inventory, not as a special Rust path.

## Milestone 4: Global Material Theme And Native Painter Consistency

- Goal: All surfaces visibly share one Material-style visual language through tokens/classes/component projection, including state layers and native painter fallback paths.
- In-scope behaviors: color tokens, surface variants, text tones, validation levels, border/radius, hover/pressed/focus/selected states, icon/image tint, popup/menu/list/table/tab visuals, runtime dialogs.
- Dependencies: global surface conformance from Milestone 3 and existing visual asset painter path.

### Implementation Slices

- [ ] Normalize `zircon_editor/assets/ui/theme/editor_material.ui.toml` token names so all Material classes use one palette:

```text
material_surface
material_surface_inset
material_surface_hover
material_surface_pressed
material_surface_selected
material_surface_disabled
material_accent
material_accent_soft
material_border
material_text
material_text_muted
material_text_disabled
material_warning
material_error
material_popup
material_track
material_focus_ring
```

- [ ] Add or update stylesheet rules for all Material classes emitted by `material_meta_components.ui.toml`.
- [ ] Update `material_meta_components.ui.toml` roots so every component emits a stable Material class, and stateful controls emit `hovered`, `pressed`, `focused`, `selected`, `checked`, or `disabled` metadata matching their existing parameter set:

```text
material-control
material-button
material-icon-button
material-toggle-button
material-checkbox-row
material-outlined-field
material-list-item
material-menu-item
material-tab-impl
material-text-edit
material-runtime-dialog
```

- [ ] Update runtime dialog `.ui.toml` files to import Material theme and use Material classes/components:
  - `zircon_editor/assets/ui/runtime/inventory_dialog.ui.toml`
  - `zircon_editor/assets/ui/runtime/pause_dialog.ui.toml`
  - `zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml`
  - `zircon_editor/assets/ui/runtime/settings_dialog.ui.toml`
  - `zircon_editor/assets/ui/runtime/runtime_hud.ui.toml`
- [ ] Update editor host surfaces and windows to use Material style imports/classes where they currently use legacy `editor_base` only.
- [ ] If native painter fallback colors in `workbench.rs` visibly conflict with Material surfaces, extract host chrome colors into a small `painter/theme.rs` module. Keep this limited to shell chrome fallback; template-rendered controls should still use projected visual fields.
- [ ] Add tests that render representative surfaces and assert Material pixels appear in Component Showcase, Asset Browser, one host pane body, and one runtime dialog.

### Testing Stage: Theme Gate

Run targeted formatting if Rust painter/test files change:

```powershell
rustfmt --edition 2021 --check "zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs" "zircon_editor/src/tests/host/slint_window/shell_window.rs" "zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs"
```

Run focused tests:

```powershell
cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib runtime_dialog --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
```

If no existing `runtime_dialog` filter exists after implementation, replace it with the exact test filter created for runtime dialog Material rendering.

### Debug / Correction Loop

- If a surface still looks legacy, inspect projected classes/style tokens first; do not patch painter color constants for a single screen.
- If Material style rules do not project into `TemplatePaneNodeData`, fix style resolution/projection before adding per-control painter branches.
- If runtime UI intentionally needs a different color palette, add a runtime Material theme using the same token names rather than bypassing Material tokens.

### Exit Evidence

- Global conformance test proves all assets import Material theme or a documented equivalent.
- Representative render tests show Material pixels for editor and runtime surfaces.
- No plain legacy visual language remains on normal controls.

## Milestone 5: Damage, Invalidation, And Performance Acceptance

- Goal: Prove the global UI path keeps Slate-style fast repaint behavior while Material/responsive changes are active.
- In-scope behaviors: hover same-target idle, region damage for hover/press/viewport image/diagnostics overlay, full redraw for layout/window/data changes, no repeated template load/compile on idle repaint.
- Dependencies: live diagnostics, global layout, global theme, existing `HostRedrawRequest`, `HostInvalidationRoot`, presenter backbuffer, and viewport image fast path.

### Implementation Slices

- [ ] Add or extend native host tests so 100 same-target mouse moves do not increment presentation rebuild counters and do not request full frame updates.
- [ ] Add tests so diagnostics overlay text changes request only top-bar region repaint when no layout changed.
- [ ] Add tests so viewport image update repaints only viewport content region.
- [ ] Add tests so Material hover/press state damages old/new control frames only.
- [ ] Add or update diagnostic log assertions only where deterministic; prefer direct counter assertions over log-string matching.
- [ ] Update `tests/acceptance/global-ui-material-responsive-diagnostics.md` with actual evidence commands and remaining accepted risks.

### Testing Stage: Performance Gate

Run focused tests:

```powershell
cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib native_viewport_image --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
```

### Debug / Correction Loop

- If same-target hover rebuilds presentation, inspect `HostInvalidationMask` source before changing pointer tests.
- If overlay changes force full repaint, check `HostRedrawRequest::merge(...)` and top-bar damage calculation before changing painter behavior.
- If viewport image update triggers presentation dirty, fix `viewport_image_redraw.rs` / host globals paint-only path, not the viewport painter.

### Exit Evidence

- Tests prove repeated idle pointer movement avoids presentation rebuild.
- Tests prove local visual changes request region damage.
- Live overlay counters demonstrate region/full paint split.

## Milestone 6: Documentation And Final Validation

- Goal: Record the final global UI contract, validation evidence, and remaining risks without claiming workspace-wide green unless it is actually run.
- In-scope behaviors: docs headers updated, acceptance file completed, scoped validation run, active session note retired or handed off.
- Dependencies: Milestones 1-5 implemented and focused tests passing.

### Implementation Slices

- [ ] Update `docs/ui-and-layout/runtime-ui-component-showcase.md` with global Material component and surface conformance rules.
- [ ] Update `docs/ui-and-layout/shared-ui-core-foundation.md` with text intrinsic measurement, Material metric support, and responsive container behavior.
- [ ] Update `docs/ui-and-layout/slate-style-ui-surface-frame.md` with damage/invalidation acceptance and region repaint behavior.
- [ ] Update `docs/editor-and-tooling/editor-workbench-shell.md` with live top-right diagnostics, presenter counters, and native host overlay behavior.
- [ ] Update `docs/assets-and-rendering/runtime-ui-graphics-integration.md` only if this plan changes visual asset behavior beyond already-documented SVG/image support.
- [ ] Update `tests/acceptance/global-ui-material-responsive-diagnostics.md` with exact command output summaries and any non-reproducible or environmental failures.
- [ ] Update `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md` with completion or move it to archive if no handoff remains.

### Testing Stage: Final Scoped Gate

Run formatting where practical. If `cargo fmt --all --check` fails on unrelated active-session files, record the exact files and run targeted `rustfmt --check` on this plan's touched files instead.

```powershell
cargo fmt --all --check
```

Run scoped package checks and focused suites:

```powershell
cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never
cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture
```

Workspace-level validation is optional for this plan unless the implementation touches shared public API or workspace manifests. If shared public API changes in `zircon_runtime_interface`, run:

```powershell
cargo test --workspace --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui-workspace" --message-format short --color never
```

### Debug / Correction Loop

- If editor tests fail because another active session edited the same files, refresh coordination context before changing shared code.
- If workspace validation fails outside UI/runtime/editor touched files, record it as unrelated unless the failure is proven to depend on this plan's changes.
- If any milestone lacks direct coverage, do not mark the global UI convergence complete; reopen the relevant milestone.

### Exit Evidence

- All scoped checks and focused tests listed above pass or exact unrelated blockers are recorded.
- Docs and acceptance files list implementation files, plan sources, tests, and remaining risks.
- Active session note is archived or explicitly handed off.

## Global Acceptance Checklist

- [ ] Top-right overlay shows live FPS/present/full/region/pixel/invalidation counters.
- [ ] All `.ui.toml` surfaces in editor, host, windows, runtime, and theme folders are inventoried.
- [ ] Global conformance test enforces Material imports/classes/layout metrics for all UI assets.
- [ ] Asset Browser is responsive because it follows shared global rules, not because of Rust special casing.
- [ ] Component Showcase demonstrates Material controls with correct intrinsic text sizing and style.
- [ ] Runtime dialogs use the same Material token vocabulary as editor surfaces.
- [ ] Shared runtime tests prove text measurement plus Material padding for every supported control family.
- [ ] Responsive layout behavior is in shared runtime layout primitives.
- [ ] Native painter consumes arranged frames and projected visual fields; it does not compute screen-specific Material layout.
- [ ] Region damage remains the fast path for pointer/viewport/diagnostic visual changes.
- [ ] Documentation and acceptance records are updated before closeout.

## Self-Review Notes

- Spec coverage: the user requested full continuation of live FPS, responsive layout, Material polish, and button/text measurement, and clarified this must apply to all interfaces, not just Asset Browser. Milestones 1-5 cover those requirements globally.
- Red-flag scan: this plan intentionally avoids disallowed marker terms; any conditional command states exactly when to omit non-created files or replace a filter with the exact test created.
- Type consistency: `HostRefreshDiagnostics`, `HostInvalidationRoot`, `HostRedrawRequest`, `UiContainerKind`, `WrapBox`, and Material `layout_*` names are used consistently across milestones.
- Scope check: this is broad but not decomposed into unrelated products; all milestones share one UI pipeline and converge on a single global UI contract.

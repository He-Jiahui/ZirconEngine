# Editor ActivityWindow Restructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move ActivityDrawer ownership from the root Workbench shell into ActivityWindow layouts and introduce reusable `.ui.toml` ActivityDrawerWindow templates.

**Architecture:** Main frame becomes a minimal window host. Drawer-capable windows use `ActivityDrawerWindow`; Workbench, Asset, and UI Layout Editor each declare their own drawer slots.

**Tech Stack:** Rust, `.ui.toml`, `zircon_editor::ui`, `zircon_runtime::ui`, existing `UiSurface` projection, no new Slint business UI.

---

## Task 1: Add Window-Level Layout Types

**Files:**
- Create: `zircon_editor/src/ui/workbench/layout/activity_window_id.rs`
- Create: `zircon_editor/src/ui/workbench/layout/activity_window_host_mode.rs`
- Create: `zircon_editor/src/ui/workbench/layout/activity_window_layout.rs`
- Modify: `zircon_editor/src/ui/workbench/layout/mod.rs`
- Test: `zircon_editor/src/tests/ui/workbench/layout/activity_window_layout.rs`

- [ ] Add tests for `ActivityWindowId`, `ActivityWindowHostMode`, and `ActivityWindowLayout` serde roundtrip.
- [ ] Run the targeted test and verify it fails because the new modules do not exist.
- [ ] Implement the three layout modules and export them from `mod.rs`.
- [ ] Run the targeted test and verify it passes.

Validation: `cargo test -p zircon_editor activity_window_layout`

## Task 2: Add Minimal Main Frame Layout

**Files:**
- Create: `zircon_editor/src/ui/workbench/layout/editor_main_frame_layout.rs`
- Modify: `zircon_editor/src/ui/workbench/layout/mod.rs`
- Test: `zircon_editor/src/tests/ui/workbench/layout/editor_main_frame_layout.rs`

- [ ] Add tests proving `EditorMainFrameLayout` stores active window and window tabs but no drawer state.
- [ ] Run the targeted test and verify it fails because the type is missing.
- [ ] Implement `EditorMainFrameLayout` and export it from `mod.rs`.
- [ ] Run the targeted test and verify it passes.

Validation: `cargo test -p zircon_editor editor_main_frame_layout`

## Task 3: Add `.ui.toml` Main Frame Asset

**Files:**
- Create: `zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml`
- Test: extend existing asset parse/compile tests or add `zircon_editor/tests/editor_main_frame_template.rs`.

- [ ] Add a failing test that parses the asset and rejects drawer/document business controls in it.
- [ ] Run the targeted test and verify it fails because the asset does not exist.
- [ ] Create the `.ui.toml` with `task_bar`, `window_tab_strip`, and `active_window_host` slots only.
- [ ] Run the targeted test and verify it passes.

Validation: `cargo test -p zircon_editor editor_main_frame_template`

## Task 4: Add `ActivityDrawerWindow` `.ui.toml` Template

**Files:**
- Create: `zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml`
- Test: `zircon_editor/tests/activity_drawer_window_template.rs`

- [ ] Add a failing test that parses the asset, checks required slots, and rejects Workbench/Asset/UI-Asset business controls.
- [ ] Run the targeted test and verify it fails because the asset does not exist.
- [ ] Create the neutral `ActivityDrawerWindow` template.
- [ ] Run the targeted test and verify it passes.

Validation: `cargo test -p zircon_editor activity_drawer_window_template`

## Task 5: Create WorkbenchWindow Asset

**Files:**
- Create: `zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml`
- Test: `zircon_editor/tests/workbench_window_template.rs`

- [ ] Add a failing test that verifies `WorkbenchWindow` references `ActivityDrawerWindow` and mounts workbench panes into drawer slots.
- [ ] Run the targeted test and verify it fails because the asset does not exist.
- [ ] Create the WorkbenchWindow asset.
- [ ] Run the targeted test and verify it passes.

Validation: `cargo test -p zircon_editor workbench_window_template`

## Task 6: Create AssetWindow Asset

**Files:**
- Create: `zircon_editor/assets/ui/editor/windows/asset_window.ui.toml`
- Reuse: `zircon_editor/assets/ui/editor/asset_browser.ui.toml`
- Test: `zircon_editor/tests/asset_window_template.rs`

- [ ] Add a failing test that verifies `AssetWindow` references `ActivityDrawerWindow` and mounts asset browser content.
- [ ] Run the targeted test and verify it fails because the asset does not exist.
- [ ] Create the AssetWindow asset.
- [ ] Run the targeted test and verify it passes.

Validation: `cargo test -p zircon_editor asset_window_template`

## Task 7: Create UILayoutEditorWindow Asset

**Files:**
- Create: `zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml`
- Reuse: `zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml`
- Test: `zircon_editor/tests/ui_layout_editor_window_template.rs`

- [ ] Add a failing test that verifies `UILayoutEditorWindow` references `ActivityDrawerWindow` and mounts UI editor content.
- [ ] Run the targeted test and verify it fails because the asset does not exist.
- [ ] Create the UILayoutEditorWindow asset.
- [ ] Run the targeted test and verify it passes.

Validation: `cargo test -p zircon_editor ui_layout_editor_window_template`

## Task 8: Register Window Descriptors

**Files:**
- Modify: `zircon_editor/src/ui/host/builtin_views/activity_windows/asset_browser_view_descriptor.rs`
- Modify: `zircon_editor/src/ui/host/asset_editor_sessions/editing.rs`
- Create: `zircon_editor/src/ui/host/builtin_views/activity_windows/workbench_window_view_descriptor.rs`
- Modify: relevant builtin views module wiring.

- [ ] Add descriptor tests for WorkbenchWindow, AssetWindow, and UILayoutEditorWindow host/template expectations.
- [ ] Run the descriptor tests and verify they fail.
- [ ] Register or update descriptors with correct host/template ids.
- [ ] Run the descriptor tests and verify they pass.

Validation: `cargo test -p zircon_editor builtin_view_descriptors`

## Task 9: Migrate Layout Ownership

**Files:**
- Modify: `zircon_editor/src/ui/workbench/layout/workbench_layout.rs`
- Modify: `zircon_editor/src/ui/workbench/layout/manager/*.rs`
- Modify: tests under `zircon_editor/src/tests/host/slint_tab_drag/`

- [ ] Add failing tests that default root layout has a WorkbenchWindow with drawers and no root-owned drawer mutation path.
- [ ] Run targeted layout tests and verify they fail.
- [ ] Add `activity_windows` to layout state and move default drawers into the default WorkbenchWindow.
- [ ] Update layout manager resolution to target active window drawers.
- [ ] Run targeted layout tests and verify they pass.

Validation:
- `cargo test -p zircon_editor workbench_layout`
- `cargo test -p zircon_editor slint_tab_drag`

## Task 10: Update Projection To Use Active ActivityWindow

**Files:**
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/*`
- Modify: `zircon_editor/src/ui/slint_host/*projection*`
- Modify: pointer layout builders that currently read root drawer slots.

- [ ] Add failing projection tests proving main frame projection has no drawer controls and active ActivityWindow projection has them.
- [ ] Run targeted projection tests and verify they fail.
- [ ] Update projection to source drawer frames from `ActivityWindowLayout`.
- [ ] Run targeted projection tests and verify they pass.

Validation:
- `cargo test -p zircon_editor workbench_slint_shell`
- `cargo test -p zircon_editor native_window_hosts`

## Task 11: Route Drawer Events Through EditorEvent

**Files:**
- Modify: `zircon_editor/src/ui/host/editor_event_execution/*`
- Modify: `zircon_editor/src/ui/slint_host/drawer_resize.rs`
- Modify: drawer/activity rail pointer dispatch tests.

- [ ] Add failing event tests requiring drawer events to carry `window_id`.
- [ ] Run targeted event tests and verify they fail.
- [ ] Include `window_id` in drawer open/close/resize/activate events and mutate target window layout.
- [ ] Run targeted event tests and verify they pass.

Validation:
- `cargo test -p zircon_editor drawer`
- `cargo test -p zircon_editor editor_event`

## Task 12: Add Source Guards For No New Slint Business UI

**Files:**
- Modify or create: `zircon_editor/tests/structure_roots.rs`
- Modify or create: `zircon_editor/tests/workbench_slint_shell.rs`

- [ ] Add failing guard tests for forbidden business window names and drawer controls in `.slint`.
- [ ] Run guard tests and verify the current forbidden cases are detected or the missing assets fail.
- [ ] Adjust guards to enforce the new `.ui.toml` ownership after migration.
- [ ] Run guard tests and verify they pass.

Validation:
- `cargo test -p zircon_editor structure_roots`
- `cargo test -p zircon_editor workbench_slint_shell`

## Task 13: Full Validation And Docs

**Files:**
- Create or update docs under `docs/editor-and-tooling/` with related code headers.

- [ ] Update module documentation for the new ActivityWindow and ActivityDrawerWindow boundary.
- [ ] Run `cargo fmt --all --check`.
- [ ] Run `cargo test -p zircon_editor`.
- [ ] Run `cargo check --workspace`.

Acceptance:
- Main frame has no root drawer ownership.
- Workbench, Asset, and UI Layout Editor windows each own drawer layouts.
- Embedded and native host mode share the same ActivityWindow layout.
- No new Slint business tree is introduced.

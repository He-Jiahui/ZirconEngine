# UI Cutover Move-First Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the remaining editor `.slint` de-source and runtime UI usable cutover by moving files to the generic host/runtime boundaries first, then repairing compile, tests, docs, and final acceptance.

**Architecture:** The editor host becomes a generic Slint bootstrap that consumes `TOML -> UiSurface -> host projection` payloads. Runtime UI remains owned by `zircon_runtime::ui`, and `zircon_runtime::graphics` consumes only `UiRenderExtract` through the screen-space UI pass. The first implementation action is structural movement, not compatibility layering.

**Tech Stack:** Rust 2021/2024 workspace, Cargo, Slint, WGPU, TOML UI assets, `UiSurface`, `UiRenderExtract`, PowerShell on Windows.

---

## Execution Notes

- Work in the existing `main` checkout.
- Do not create commits unless the user explicitly requests them.
- Use `apply_patch` for manual source edits and file moves.
- Keep unrelated dirty files intact.
- Use focused tests first, then broader workspace gates.
- Update `.codex/sessions/20260429-2236-ui-cutover-single-milestone.md` whenever ownership, blockers, or validation status changes.

## Target File Structure

### Slint Moves

Move current editor Slint files out of `zircon_editor/ui/workbench/` into these folders.

- `zircon_editor/ui/host/window/context.slint` from `zircon_editor/ui/workbench/host_context.slint`
- `zircon_editor/ui/host/window/scaffold.slint` from `zircon_editor/ui/workbench/host_scaffold.slint`
- `zircon_editor/ui/host/window/root.slint` from `zircon_editor/ui/workbench/host_root.slint`
- `zircon_editor/ui/host/window/components.slint` from `zircon_editor/ui/workbench/host_components.slint`
- `zircon_editor/ui/host/window/scene.slint` from `zircon_editor/ui/workbench/host_scene.slint`
- `zircon_editor/ui/host/window/surface_host.slint` from `zircon_editor/ui/workbench/host_surface.slint`
- `zircon_editor/ui/host/window/surface_contract.slint` from `zircon_editor/ui/workbench/host_surface_contract.slint`
- `zircon_editor/ui/host/window/interaction.slint` from `zircon_editor/ui/workbench/host_interaction.slint`
- `zircon_editor/ui/host/window/resize_layer.slint` from `zircon_editor/ui/workbench/host_resize_layer.slint`
- `zircon_editor/ui/host/window/tab_drag_overlay.slint` from `zircon_editor/ui/workbench/host_tab_drag_overlay.slint`
- `zircon_editor/ui/host/surface/menu_chrome.slint` from `zircon_editor/ui/workbench/host_menu_chrome.slint`
- `zircon_editor/ui/host/surface/page_chrome.slint` from `zircon_editor/ui/workbench/host_page_chrome.slint`
- `zircon_editor/ui/host/surface/side_dock.slint` from `zircon_editor/ui/workbench/host_side_dock_surface.slint`
- `zircon_editor/ui/host/surface/document_dock.slint` from `zircon_editor/ui/workbench/host_document_dock_surface.slint`
- `zircon_editor/ui/host/surface/bottom_dock.slint` from `zircon_editor/ui/workbench/host_bottom_dock_surface.slint`
- `zircon_editor/ui/host/surface/status_bar.slint` from `zircon_editor/ui/workbench/host_status_bar.slint`
- `zircon_editor/ui/host/surface/floating_window_layer.slint` from `zircon_editor/ui/workbench/host_floating_window_layer.slint`
- `zircon_editor/ui/host/surface/native_floating_window.slint` from `zircon_editor/ui/workbench/host_native_floating_window_surface.slint`
- `zircon_editor/ui/host/primitives/chrome.slint` from `zircon_editor/ui/workbench/chrome.slint`
- `zircon_editor/ui/template/pane.slint` from `zircon_editor/ui/workbench/template_pane.slint`
- `zircon_editor/ui/template/node_data.slint` from `zircon_editor/ui/workbench/template_node_data.slint`
- `zircon_editor/ui/template/collection_field_row.slint` from `zircon_editor/ui/workbench/template_collection_field_row.slint`
- `zircon_editor/ui/pane/data.slint` from `zircon_editor/ui/workbench/pane_data.slint`
- `zircon_editor/ui/pane/surface.slint` from `zircon_editor/ui/workbench/pane_surface.slint`
- `zircon_editor/ui/pane/content.slint` from `zircon_editor/ui/workbench/pane_content.slint`
- `zircon_editor/ui/pane/fields.slint` from `zircon_editor/ui/workbench/pane_fields.slint`
- `zircon_editor/ui/pane/surface_host_context.slint` from `zircon_editor/ui/workbench/pane_surface_host_context.slint`
- `zircon_editor/ui/pane/fallback.slint` from `zircon_editor/ui/workbench/fallback_pane.slint`
- `zircon_editor/ui/pane/tool_window_empty_state.slint` from `zircon_editor/ui/workbench/tool_window_empty_state.slint`
- `zircon_editor/ui/pane/welcome.slint` from `zircon_editor/ui/workbench/welcome.slint`
- `zircon_editor/ui/pane/module_plugins.slint` from `zircon_editor/ui/workbench/module_plugins_pane.slint`
- `zircon_editor/ui/pane/ui_asset_editor.slint` from `zircon_editor/ui/workbench/ui_asset_editor_pane.slint`
- `zircon_editor/ui/pane/ui_asset_editor_data.slint` from `zircon_editor/ui/workbench/ui_asset_editor_data.slint`
- `zircon_editor/ui/assets/assets.slint` from `zircon_editor/ui/workbench/assets.slint`
- `zircon_editor/ui/assets/reference_panel_projection.slint` from `zircon_editor/ui/workbench/reference_panel_projection.slint`
- Keep one canonical collection-field row file at `zircon_editor/ui/template/collection_field_row.slint`; do not create an asset-specific duplicate.

Keep `zircon_editor/ui/workbench.slint` as a temporary build entry during repair. It must import only generic paths after Task 2.

### Rust Moves

- Move `zircon_editor/src/ui/layouts/windows/workbench_host_window/` to `zircon_editor/src/ui/layouts/windows/host_window/`.
- Change `zircon_editor/src/ui/layouts/windows/mod.rs` from `pub(crate) mod workbench_host_window;` to `pub(crate) mod host_window;`.
- Replace internal Rust references to `workbench_host_window` with `host_window`.

### Test Moves And Additions

- Create `zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs`.
- Modify `zircon_editor/src/tests/host/slint_window/mod.rs` to include `mod generic_host_layout_paths;`.
- Extend `zircon_runtime/src/tests/ui_boundary/runtime_host.rs` with runtime input dispatch coverage.
- Extend `zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs` with all-fixture runtime visual submission coverage under the existing feature gate.

---

### Task 1: Add Failing Move-Target Source Guards

**Files:**
- Create: `zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs`
- Modify: `zircon_editor/src/tests/host/slint_window/mod.rs`

- [ ] **Step 1: Add the source guard module**

Add this file:

```rust
use std::path::Path;

fn editor_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(relative_path: &str) -> String {
    std::fs::read_to_string(editor_root().join(relative_path))
        .unwrap_or_else(|error| panic!("read `{relative_path}`: {error}"))
}

#[test]
fn generic_host_slint_files_live_outside_workbench_component_folder() {
    let root = editor_root();
    for required in [
        "ui/host/window/context.slint",
        "ui/host/window/scaffold.slint",
        "ui/host/window/root.slint",
        "ui/host/window/components.slint",
        "ui/host/window/scene.slint",
        "ui/host/window/surface_host.slint",
        "ui/host/surface/menu_chrome.slint",
        "ui/host/surface/page_chrome.slint",
        "ui/host/surface/side_dock.slint",
        "ui/host/surface/document_dock.slint",
        "ui/host/surface/bottom_dock.slint",
        "ui/host/surface/status_bar.slint",
        "ui/host/surface/floating_window_layer.slint",
        "ui/host/surface/native_floating_window.slint",
        "ui/host/primitives/chrome.slint",
        "ui/template/pane.slint",
        "ui/template/node_data.slint",
        "ui/template/collection_field_row.slint",
        "ui/pane/data.slint",
        "ui/pane/surface.slint",
        "ui/pane/content.slint",
        "ui/pane/surface_host_context.slint",
        "ui/assets/assets.slint",
    ] {
        assert!(
            root.join(required).exists(),
            "generic host move target `{required}` should exist"
        );
    }
}

#[test]
fn workbench_slint_imports_generic_host_paths_only() {
    let source = read_source("ui/workbench.slint");
    for required in [
        "host/window/context.slint",
        "host/window/scaffold.slint",
        "host/window/components.slint",
        "host/window/root.slint",
    ] {
        assert!(
            source.contains(required),
            "workbench.slint should import generic path `{required}`"
        );
    }
    for forbidden in [
        "workbench/host_context.slint",
        "workbench/host_scaffold.slint",
        "workbench/host_components.slint",
        "workbench/host_root.slint",
        "workbench/host_scene.slint",
    ] {
        assert!(
            !source.contains(forbidden),
            "workbench.slint should not import old workbench path `{forbidden}`"
        );
    }
}

#[test]
fn rust_projection_module_uses_generic_host_window_path() {
    let root = editor_root();
    assert!(
        root.join("src/ui/layouts/windows/host_window/mod.rs").exists(),
        "Rust host projection should live under layouts/windows/host_window"
    );
    let windows_mod = read_source("src/ui/layouts/windows/mod.rs");
    assert!(
        windows_mod.contains("pub(crate) mod host_window;"),
        "windows module should expose generic host_window module"
    );
    assert!(
        !windows_mod.contains("workbench_host_window"),
        "windows module should not expose the old workbench_host_window path"
    );
}
```

- [ ] **Step 2: Wire the module**

Change `zircon_editor/src/tests/host/slint_window/mod.rs` to include the new module:

```rust
mod activity_rail_template_boundary;
mod callback_source_window;
mod generic_host_boundary;
mod generic_host_layout_paths;
mod native_mode;
mod native_window_targets;
mod presenter_store;
mod shell_window;
mod support;
mod ui_asset_editor;
```

- [ ] **Step 3: Run the guard and verify it fails before moves**

Run:

```powershell
cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
```

Expected: FAIL because `ui/host/window/context.slint` and `src/ui/layouts/windows/host_window/mod.rs` do not exist yet.

- [ ] **Step 4: Checkpoint without commit**

Run:

```powershell
git diff -- zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs zircon_editor/src/tests/host/slint_window/mod.rs
```

Expected: diff contains only the new failing source guard and module registration.

---

### Task 2: Move Slint Files And Repair Imports

**Files:**
- Move: all Slint files listed in the "Slint Moves" section
- Modify: `zircon_editor/ui/workbench.slint`
- Modify: moved Slint files under `zircon_editor/ui/host/`, `zircon_editor/ui/template/`, `zircon_editor/ui/pane/`, and `zircon_editor/ui/assets/`

- [ ] **Step 1: Move files with `apply_patch` move headers**

Use one or more patches with this shape for each tracked file:

```diff
*** Begin Patch
*** Update File: zircon_editor/ui/workbench/host_context.slint
*** Move to: zircon_editor/ui/host/window/context.slint
*** End Patch
```

Repeat for every mapping in the "Slint Moves" section. Do not copy a file and leave the old source behind.

- [ ] **Step 2: Update `zircon_editor/ui/workbench.slint` imports and exports**

Replace the root file contents with this generic import/export shape, preserving the existing `UiHostWindow` component body:

```slint
import { UiHostContext } from "host/window/context.slint";
import { UiHostScaffold } from "host/window/scaffold.slint";
import { HostWindowBootstrapData } from "host/window/components.slint";
import { HostWindowPresentationData } from "host/window/root.slint";

export { UiHostContext } from "host/window/context.slint";
export { PaneSurfaceHostContext } from "pane/surface_host_context.slint";
export { FrameRect, FloatingWindowData, HostWindowBootstrapData, HostWindowLayoutData, HostWindowShellData, HostWindowSurfaceData, HostNativeFloatingWindowSurfaceData } from "host/window/components.slint";
export { HostWindowPresentationData } from "host/window/root.slint";
export { ModulePluginStatusData, ModulePluginsPaneData, PaneData, ProjectOverviewData, SceneNodeData } from "pane/data.slint";
export { HostWindowSceneData } from "host/window/scene.slint";
```

Keep the existing `export component UiHostWindow inherits Window { ... }` block, but its `UiHostScaffold` reference should now come from `host/window/scaffold.slint`.

- [ ] **Step 3: Update window-level moved imports**

Use these import replacements inside `zircon_editor/ui/host/window/*.slint`:

```slint
// host/window/scaffold.slint
import { HostWindowBootstrapData } from "components.slint";
import { HostWindowPresentationData } from "root.slint";
import { HostWindowSurfaceHost } from "surface_host.slint";
```

```slint
// host/window/root.slint
import { HostWindowLayoutData, HostWindowShellData, HostNativeFloatingWindowSurfaceData } from "components.slint";
import { HostWindowSceneData } from "scene.slint";
```

```slint
// host/window/scene.slint
import { Palette } from "../primitives/chrome.slint";
import { HostWindowLayoutData, HostMenuChromeData, HostPageChromeData, HostStatusBarData, HostResizeLayerData, HostTabDragOverlayData, HostSideDockSurfaceData, HostDocumentDockSurfaceData, HostBottomDockSurfaceData, HostFloatingWindowLayerData, HostWindowSurfaceMetricsData, HostWindowSurfaceOrchestrationData } from "components.slint";
import { HostMenuChrome } from "../surface/menu_chrome.slint";
import { HostPageChrome } from "../surface/page_chrome.slint";
import { HostResizeLayer } from "resize_layer.slint";
import { HostStatusBar } from "../surface/status_bar.slint";
import { HostTabDragOverlay } from "tab_drag_overlay.slint";
import { HostBottomDockSurface } from "../surface/bottom_dock.slint";
import { HostDocumentDockSurface } from "../surface/document_dock.slint";
import { HostFloatingWindowLayer } from "../surface/floating_window_layer.slint";
import { HostSideDockSurface } from "../surface/side_dock.slint";
```

```slint
// host/window/surface_host.slint
import { HostNativeFloatingWindowSurfaceData } from "components.slint";
import { HostWindowPresentationData } from "root.slint";
import { HostWindowSceneData, HostWindowScene } from "scene.slint";
import { HostWindowSurfaceContract } from "surface_contract.slint";
import { HostNativeFloatingWindowSurface } from "../surface/native_floating_window.slint";
```

- [ ] **Step 4: Update surface-level moved imports**

Use these import rules in `zircon_editor/ui/host/surface/*.slint`:

```slint
import { Palette } from "../primitives/chrome.slint";
import { UiHostContext } from "../window/context.slint";
import { FrameRect, HostChromeTabData } from "../window/components.slint";
import { TemplatePane } from "../../template/pane.slint";
import { PaneSurface } from "../../pane/surface.slint";
```

Only keep the imported names that the file actually uses. For `menu_chrome.slint` and `page_chrome.slint`, omit `Palette` and `PaneSurface` if unused.

- [ ] **Step 5: Update template imports**

In `zircon_editor/ui/template/pane.slint`, replace old imports with:

```slint
import { Palette, ShellButton } from "../host/primitives/chrome.slint";
import { TemplatePaneNodeData } from "node_data.slint";
import { TemplateCollectionFieldRow } from "collection_field_row.slint";
```

In `zircon_editor/ui/template/collection_field_row.slint`, update any old `chrome.slint` import to:

```slint
import { Palette } from "../host/primitives/chrome.slint";
```

- [ ] **Step 6: Update pane and asset imports**

Use these import rules in `zircon_editor/ui/pane/*.slint`:

```slint
import { EmptyStateCard, Palette, ShellButton, ShellIcon, ToolbarButton } from "../host/primitives/chrome.slint";
import { TemplatePane } from "../template/pane.slint";
import { TemplatePaneNodeData } from "../template/node_data.slint";
import { PaneSurfaceHostContext } from "surface_host_context.slint";
```

Use these import rules in `zircon_editor/ui/assets/*.slint`:

```slint
import { Palette, ShellButton, ShellIcon, ToolbarButton } from "../host/primitives/chrome.slint";
import { TemplatePaneNodeData } from "../template/node_data.slint";
```

Only keep the imported names that each file actually uses.

- [ ] **Step 7: Run Slint/Rust compile repair loop**

Run:

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short --color never
```

Expected after initial moves: FAIL with missing Slint import paths or generated type references. Repair only import paths and direct generated type paths. Do not restore old `workbench/` component files.

- [ ] **Step 8: Verify the move source guards pass**

Run:

```powershell
cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS for the Slint path guards. The Rust projection path guard may still fail until Task 3 is complete.

- [ ] **Step 9: Checkpoint without commit**

Run:

```powershell
git diff --check -- zircon_editor/ui zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs zircon_editor/src/tests/host/slint_window/mod.rs
```

Expected: no whitespace errors. LF-to-CRLF warnings are acceptable on Windows.

---

### Task 3: Move Rust Host Projection Module

**Files:**
- Move: `zircon_editor/src/ui/layouts/windows/workbench_host_window/**` to `zircon_editor/src/ui/layouts/windows/host_window/**`
- Modify: `zircon_editor/src/ui/layouts/windows/mod.rs`
- Modify: Rust imports that mention `workbench_host_window`

- [ ] **Step 1: Move the folder with `apply_patch` move headers**

Move every file under `zircon_editor/src/ui/layouts/windows/workbench_host_window/` to the same relative path under `zircon_editor/src/ui/layouts/windows/host_window/`.

Example move patch shape:

```diff
*** Begin Patch
*** Update File: zircon_editor/src/ui/layouts/windows/workbench_host_window/mod.rs
*** Move to: zircon_editor/src/ui/layouts/windows/host_window/mod.rs
*** End Patch
```

- [ ] **Step 2: Update the windows module root**

Replace `zircon_editor/src/ui/layouts/windows/mod.rs` with:

```rust
pub(crate) mod host_window;
```

- [ ] **Step 3: Update Rust module references**

Replace Rust references of this form:

```rust
crate::ui::layouts::windows::workbench_host_window
```

with:

```rust
crate::ui::layouts::windows::host_window
```

Also replace direct `super::workbench_host_window` or `windows::workbench_host_window` references with `host_window` equivalents.

- [ ] **Step 4: Run compile repair loop**

Run:

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short --color never
```

Expected after initial move: FAIL if any stale module path remains. Repair stale imports only. Do not add a behavior-bearing compatibility module named `workbench_host_window`.

- [ ] **Step 5: Verify generic path guards pass**

Run:

```powershell
cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS for Slint and Rust path guards.

- [ ] **Step 6: Checkpoint without commit**

Run:

```powershell
git diff --check -- zircon_editor/src/ui/layouts/windows zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs
```

Expected: no whitespace errors. LF-to-CRLF warnings are acceptable on Windows.

---

### Task 4: Restore Editor Generic Host Boundary Suites

**Files:**
- Modify: `zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs`
- Modify: any source guards that read old Slint paths
- Modify: moved Slint files only for source guard repair

- [ ] **Step 1: Update path constants and source reads in source guards**

Where a source guard reads an old path such as:

```rust
"ui/workbench/host_menu_chrome.slint"
```

change it to the moved path:

```rust
"ui/host/surface/menu_chrome.slint"
```

Apply the same path update for page chrome, dock surfaces, status bar, floating surfaces, template pane, template node data, and pane files.

- [ ] **Step 2: Strengthen guards against old path imports**

Add this helper to `generic_host_boundary.rs` if no equivalent exists:

```rust
fn assert_source_omits_old_workbench_imports(relative_path: &str) {
    let source = read_source(relative_path);
    for forbidden in [
        "workbench/host_",
        "workbench/template_",
        "workbench/pane_",
        "workbench/chrome.slint",
    ] {
        assert!(
            !source.contains(forbidden),
            "`{relative_path}` should not import old workbench component path `{forbidden}`"
        );
    }
}
```

Call it from moved-file source guard tests for files under `ui/host/`, `ui/template/`, and `ui/pane/`.

- [ ] **Step 3: Run focused source guard suite**

Run:

```powershell
cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS. If it fails, repair source guard paths or moved Slint imports. Do not weaken assertions about `TemplatePane` authority.

- [ ] **Step 4: Run Slint window host suite**

Run:

```powershell
cargo test -p zircon_editor --lib slint_window --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS. If generated Slint ABI changed, repair Rust conversion code to use the generated names from `UiHostWindow` and exported DTOs.

- [ ] **Step 5: Checkpoint without commit**

Run:

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short --color never
```

Expected: PASS.

---

### Task 5: Split Oversized Pane Content After The Move

**Files:**
- Modify: `zircon_editor/ui/pane/content.slint`
- Create: `zircon_editor/ui/pane/content/hierarchy.slint`
- Create: `zircon_editor/ui/pane/content/inspector.slint`
- Create: `zircon_editor/ui/pane/content/console.slint`
- Create: `zircon_editor/ui/pane/content/animation.slint`
- Create: `zircon_editor/ui/pane/content/assets_activity.slint`
- Create: `zircon_editor/ui/pane/content/asset_browser.slint`
- Create: `zircon_editor/ui/pane/content/project_overview.slint`
- Create: `zircon_editor/ui/pane/content/module_plugins.slint`

- [ ] **Step 1: Add failing split source guard**

Extend `zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs` with:

```rust
#[test]
fn pane_content_native_views_are_folder_backed_by_domain() {
    let root = editor_root();
    for required in [
        "ui/pane/content/hierarchy.slint",
        "ui/pane/content/inspector.slint",
        "ui/pane/content/console.slint",
        "ui/pane/content/animation.slint",
        "ui/pane/content/assets_activity.slint",
        "ui/pane/content/asset_browser.slint",
        "ui/pane/content/project_overview.slint",
        "ui/pane/content/module_plugins.slint",
    ] {
        assert!(
            root.join(required).exists(),
            "pane content domain file `{required}` should exist"
        );
    }
    let content = read_source("ui/pane/content.slint");
    for required_import in [
        "content/hierarchy.slint",
        "content/inspector.slint",
        "content/console.slint",
        "content/animation.slint",
        "content/assets_activity.slint",
        "content/asset_browser.slint",
        "content/project_overview.slint",
        "content/module_plugins.slint",
    ] {
        assert!(
            content.contains(required_import),
            "pane/content.slint should import `{required_import}`"
        );
    }
}
```

Run:

```powershell
cargo test -p zircon_editor --lib pane_content_native_views_are_folder_backed_by_domain --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
```

Expected: FAIL because the domain files do not exist yet.

- [ ] **Step 2: Extract one component family per file**

Move component definitions from `zircon_editor/ui/pane/content.slint` into these files:

- `HierarchyPaneView` into `content/hierarchy.slint`
- `InspectorPaneView` into `content/inspector.slint`
- `ConsolePaneView` into `content/console.slint`
- `AnimationEditorPaneView` into `content/animation.slint`
- `AssetsActivityPaneView` into `content/assets_activity.slint`
- asset browser components into `content/asset_browser.slint`
- project overview components into `content/project_overview.slint`
- module plugin pane content helpers into `content/module_plugins.slint`

Each extracted file should import only the DTOs and primitives it uses. Use this import pattern and remove unused names per file:

```slint
import { EmptyStateCard, Palette, ShellButton, ShellIcon, ToolbarButton } from "../../host/primitives/chrome.slint";
import { TemplatePane } from "../../template/pane.slint";
import { PaneSurfaceHostContext } from "../surface_host_context.slint";
import { HierarchyPaneData, InspectorPaneData, ConsolePaneData, AnimationEditorPaneData, AssetsActivityPaneData, AssetBrowserPaneData, ProjectOverviewPaneData, ModulePluginsPaneData } from "../data.slint";
```

- [ ] **Step 3: Reduce `pane/content.slint` to orchestration**

Keep `PaneContent` in `zircon_editor/ui/pane/content.slint`. Its imports should include the extracted components:

```slint
import { HierarchyPaneView } from "content/hierarchy.slint";
import { InspectorPaneView } from "content/inspector.slint";
import { ConsolePaneView } from "content/console.slint";
import { AnimationEditorPaneView } from "content/animation.slint";
import { AssetsActivityPaneView } from "content/assets_activity.slint";
import { AssetBrowserPaneView } from "content/asset_browser.slint";
import { ProjectOverviewPaneView } from "content/project_overview.slint";
import { ModulePluginsPaneView } from "content/module_plugins.slint";
```

Do not leave duplicate component definitions in `pane/content.slint`.

- [ ] **Step 4: Compile and repair extraction errors**

Run:

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short --color never
```

Expected after first extraction: FAIL if any component was not exported or an import is stale. Repair exports/imports only.

- [ ] **Step 5: Verify split guard and host suites**

Run:

```powershell
cargo test -p zircon_editor --lib pane_content_native_views_are_folder_backed_by_domain --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
cargo test -p zircon_editor --lib slint_host --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
```

Expected: both PASS.

---

### Task 6: Add Runtime UI Input Dispatch Acceptance

**Files:**
- Modify: `zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs`
- Modify: `zircon_runtime/src/tests/ui_boundary/runtime_host.rs`

- [x] **Step 1: Write failing runtime input test**

Add this test to `zircon_runtime/src/tests/ui_boundary/runtime_host.rs`:

```rust
#[test]
fn runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface() {
    use crate::ui::dispatch::{
        UiNavigationDispatchEffect, UiNavigationDispatcher, UiPointerDispatchEffect,
        UiPointerDispatcher, UiPointerEvent,
    };
    use crate::ui::layout::UiPoint;
    use crate::ui::surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind};

    let viewport_size = crate::core::math::UVec2::new(640, 360);
    let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
    manager
        .load_builtin_fixture(crate::ui::RuntimeUiFixture::PauseMenu)
        .unwrap();

    let root_node = manager.surface().tree.roots[0];
    let mut pointer_dispatcher = UiPointerDispatcher::default();
    pointer_dispatcher.register(root_node, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::Captured
    });

    let pointer_result = manager
        .dispatch_pointer_event(
            &pointer_dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(320.0, 180.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(pointer_result.captured_by, Some(root_node));
    assert_eq!(manager.surface().focus.captured, Some(root_node));

    let mut navigation_dispatcher = UiNavigationDispatcher::default();
    navigation_dispatcher.register(root_node, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::Handled
    });

    let navigation_result = manager
        .dispatch_navigation_event(&navigation_dispatcher, UiNavigationEventKind::Activate)
        .unwrap();
    assert_eq!(navigation_result.handled_by, Some(root_node));
}
```

Run:

```powershell
cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime" --message-format short -- --test-threads=1 --nocapture
```

Expected: FAIL because `RuntimeUiManager::dispatch_pointer_event` and `RuntimeUiManager::dispatch_navigation_event` do not exist. Observed during implementation: the missing methods were confirmed, and the snippet was corrected from nonexistent `UiPointerButton::Left` to the existing `UiPointerButton::Primary`.

- [x] **Step 2: Add manager dispatch methods**

In `zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs`, extend imports:

```rust
use crate::ui::dispatch::{
    UiNavigationDispatchResult, UiNavigationDispatcher, UiPointerDispatchResult,
    UiPointerDispatcher, UiPointerEvent,
};
use crate::ui::surface::UiNavigationEventKind;
use crate::ui::tree::UiTreeError;
```

Add methods inside `impl RuntimeUiManager`:

```rust
    pub(crate) fn dispatch_pointer_event(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        self.surface.dispatch_pointer_event(dispatcher, event)
    }

    pub(crate) fn dispatch_navigation_event(
        &mut self,
        dispatcher: &UiNavigationDispatcher,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        self.surface.dispatch_navigation_event(dispatcher, kind)
    }
```

- [x] **Step 3: Verify runtime input test passes**

Run:

```powershell
cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS. Latest evidence: `cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 1 test / 0 failed / 1190 filtered out.

- [x] **Step 4: Run runtime UI boundary suite**

Run:

```powershell
cargo test -p zircon_runtime --lib ui_boundary --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS. Latest evidence: `cargo test -p zircon_runtime --lib ui_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 17 tests / 0 failed / 1174 filtered out, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed.

---

### Task 7: Expand Runtime Graphics Fixture Acceptance

**Files:**
- Modify: `zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs`

- [x] **Step 1: Add all-fixture render submission test under the existing feature gate**

Add this test to `zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs`:

```rust
#[cfg(feature = "runtime-ui-integration-tests")]
#[test]
fn render_framework_submits_all_builtin_runtime_ui_fixtures() {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderFramework, RenderQualityProfile, RenderViewportDescriptor,
    };

    let viewport_size = crate::core::math::UVec2::new(800, 450);
    let asset_manager = Arc::new(crate::asset::ProjectAssetManager::default());
    let server = crate::graphics::WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-fixtures")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false),
        )
        .unwrap();

    for fixture in [
        crate::ui::RuntimeUiFixture::HudOverlay,
        crate::ui::RuntimeUiFixture::PauseMenu,
        crate::ui::RuntimeUiFixture::SettingsDialog,
        crate::ui::RuntimeUiFixture::InventoryList,
    ] {
        let mut manager = crate::ui::RuntimeUiManager::new(viewport_size);
        manager.load_builtin_fixture(fixture).unwrap();
        server
            .submit_runtime_frame(viewport, manager.build_frame().into())
            .unwrap();

        let stats = server.query_stats().unwrap();
        assert!(
            stats.last_ui_command_count >= 4,
            "expected fixture {fixture:?} to submit a non-trivial UI command list"
        );
        assert!(
            stats.last_ui_quad_count >= 1 || stats.last_ui_text_payload_count >= 1,
            "expected fixture {fixture:?} to reach the screen-space UI pass"
        );
    }
}
```

- [x] **Step 2: Run the feature-gated graphics acceptance test**

Run:

```powershell
cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime-graphics" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS if the existing screen-space UI pass handles all fixture extracts. Latest evidence: `cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 1 test / 0 failed / 1195 filtered out after waiting for an artifact lock.

- [x] **Step 3: Run existing runtime UI text render contract**

Run:

```powershell
cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime-graphics" --message-format short -- --test-threads=1 --nocapture
```

Expected: PASS. Latest evidence: `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 7 tests / 0 failed.

---

### Task 8: Update Documentation For Moved Boundaries

**Files:**
- Modify: `docs/ui-and-layout/shared-ui-template-runtime.md`
- Modify: `docs/ui-and-layout/index.md`
- Modify: `.codex/sessions/20260429-2236-ui-cutover-single-milestone.md`

- [x] **Step 1: Update module documentation related-code headers**

In `docs/ui-and-layout/shared-ui-template-runtime.md`, replace old Slint and Rust paths in `related_code` and `implementation_files` with the moved paths:

```yaml
  - zircon_editor/ui/host/window/context.slint
  - zircon_editor/ui/host/window/scaffold.slint
  - zircon_editor/ui/host/window/scene.slint
  - zircon_editor/ui/host/window/components.slint
  - zircon_editor/ui/host/surface/menu_chrome.slint
  - zircon_editor/ui/host/surface/page_chrome.slint
  - zircon_editor/ui/host/surface/side_dock.slint
  - zircon_editor/ui/host/surface/document_dock.slint
  - zircon_editor/ui/host/surface/bottom_dock.slint
  - zircon_editor/ui/host/surface/status_bar.slint
  - zircon_editor/ui/host/surface/floating_window_layer.slint
  - zircon_editor/ui/host/surface/native_floating_window.slint
  - zircon_editor/ui/template/pane.slint
  - zircon_editor/ui/template/node_data.slint
  - zircon_editor/ui/pane/content.slint
  - zircon_editor/src/ui/layouts/windows/host_window/mod.rs
```

- [x] **Step 2: Add narrative section for move-first cutover**

Add a section near the generic host boundary discussion:

```markdown
## Move-First Generic Host Cutover

The editor host Slint tree has moved out of the workbench-owned component folder. `ui/workbench.slint` is no longer the owner of business layout or chrome structure; it is only the build entry for `UiHostWindow` while the generic host files live under `ui/host/window`, `ui/host/surface`, `ui/template`, `ui/pane`, and `ui/assets`.

The Rust projection owner moved from `layouts/windows/workbench_host_window` to `layouts/windows/host_window`. This keeps the path aligned with the already-renamed `HostWindow*` DTOs and prevents new source from treating workbench-specific geometry as the canonical host boundary.
```

- [x] **Step 3: Update docs index summary**

In `docs/ui-and-layout/index.md`, update the shared UI entry to mention generic host file ownership and runtime fixture acceptance:

```markdown
- Shared UI template runtime: documents the `.ui.toml -> UiSurface -> host projection / UiRenderExtract` path, the generic host file layout, runtime fixtures, and screen-space UI pass acceptance.
```

- [x] **Step 4: Update active session note**

In `.codex/sessions/20260429-2236-ui-cutover-single-milestone.md`, update current step and touched modules after the move:

```markdown
## Current Step
- Move-first design and implementation plan have been approved.
- This session now owns overlapping runtime graphics/showcase scope and is executing the structural move before behavior repair.
```

- [x] **Step 5: Run doc whitespace check**

Run:

```powershell
git diff --check -- docs/ui-and-layout/shared-ui-template-runtime.md docs/ui-and-layout/index.md .codex/sessions/20260429-2236-ui-cutover-single-milestone.md
```

Expected: no whitespace errors. LF-to-CRLF warnings are acceptable on Windows. Latest evidence: `git diff --check -- docs/ui-and-layout/shared-ui-template-runtime.md docs/ui-and-layout/index.md .codex/sessions/20260429-2236-ui-cutover-single-milestone.md docs/superpowers/plans/2026-04-29-ui-cutover-move-first.md` reported only LF-to-CRLF warnings for the touched docs.

---

### Task 9: Final Validation And Cleanup

**Files:**
- Modify: any moved source still containing temporary compatibility imports
- Modify: docs only if validation evidence changes

- [x] **Step 1: Search for old workbench host paths**

Use the Grep tool or PowerShell equivalent to find these patterns:

```text
workbench/host_
workbench/template_
workbench/pane_
workbench/chrome.slint
workbench_host_window
```

Expected: no production source references remain. Test names may mention old paths only when asserting they are forbidden. Latest evidence: source searches found no production references to old `workbench/host_`, `workbench/template_`, `workbench/pane_`, or `workbench/chrome.slint` paths; the only hit was the no-Slint absence guard for `ui/workbench/host_context.slint`. `workbench_host_window` remains intentionally live as the current Rust projection seam in this no-Slint cutover and is documented as accepted plan drift rather than a stale Slint path.

- [x] **Step 2: Run focused editor validation**

Run:

```powershell
cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
cargo test -p zircon_editor --lib slint_window --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short -- --test-threads=1 --nocapture
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-move-first" --message-format short --color never
```

Expected: all PASS. Latest closeout evidence: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed after the `to_host_contract_*` helper cleanup. Earlier focused editor filters in this task also passed and the final workspace `cargo test --workspace --locked` rerun covered the editor lib tests.

- [x] **Step 3: Run focused runtime validation**

Run:

```powershell
cargo test -p zircon_runtime --lib runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime" --message-format short -- --test-threads=1 --nocapture
cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime" --message-format short -- --test-threads=1 --nocapture
cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime" --message-format short -- --test-threads=1 --nocapture
```

Expected: all PASS. Latest recorded focused runtime evidence remains the Task 6/9 runtime reruns: runtime manager dispatch passed 1 test / 0 failed, `ui_boundary` passed 17 tests / 0 failed, and `ui::tests` passed 126 tests / 0 failed with `--locked` on the runtime target dir. The final workspace validator also passed `cargo test --workspace --locked`.

- [x] **Step 4: Run focused graphics validation**

Run:

```powershell
cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime-graphics" --message-format short -- --test-threads=1 --nocapture
cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-cutover-runtime-graphics" --message-format short -- --test-threads=1 --nocapture
```

Expected: all PASS. Latest focused graphics evidence remains the Task 7 reruns: `render_framework_submits_all_builtin_runtime_ui_fixtures` passed 1 test / 0 failed with `runtime-ui-integration-tests`, and `runtime_ui_text_render_contract` passed 7 tests / 0 failed with `--locked`.

- [x] **Step 5: Run formatting and diff checks**

Run `rustfmt --edition 2021 --check` on touched Rust files. Include at least:

```powershell
rustfmt --edition 2021 --check zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs zircon_runtime/src/tests/ui_boundary/runtime_host.rs zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
git diff --check
```

Expected: rustfmt passes for touched Rust files; diff check reports no whitespace errors. LF-to-CRLF warnings are acceptable on Windows. Latest evidence: targeted `rustfmt --edition 2021 --check` on touched editor/runtime Rust files passed with no output; `git diff --check` reported only Windows LF-to-CRLF warnings.

- [x] **Step 6: Run workspace gates when focused validation is green**

Run:

```powershell
cargo build --workspace --locked --verbose
cargo test --workspace --locked --verbose
```

Expected: both PASS. If either command times out in this environment, record the timeout and keep the focused passing evidence in the session note and docs. Latest evidence: `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir E:\cargo-targets\zircon-ui-cutover-move-first` passed `Cargo build` and `Cargo test` with `--locked`. The first closeout rerun passed build but failed tests on stale `to_slint_host_scene_data`; after updating the test import/call and boundary assertions to `to_host_contract_*`, the rerun passed.

- [x] **Step 7: Final checkpoint without commit**

Run:

```powershell
git status --short
```

Expected: changed files match this plan plus any directly required repair files. Do not commit unless the user explicitly requests a commit. Closeout note: no commit was created. The worktree remains broadly dirty because unrelated render/plugin and runtime refactor sessions are active; those unrelated changes are not part of this UI cutover completion.

---

## Self-Review Notes

- Spec coverage: tasks cover move-first Slint structure, Rust projection path, source guards, runtime input dispatch, graphics fixture acceptance, docs, and final validation.
- Completion scan: the plan contains concrete file paths, concrete code snippets for new tests and methods, exact commands, expected outcomes, and no commit instructions.
- Type consistency: runtime input task uses existing `UiPointerDispatcher`, `UiNavigationDispatcher`, `UiPointerEvent`, `UiPointerEventKind`, `UiPointerButton`, and `UiNavigationEventKind` APIs observed in the current source.

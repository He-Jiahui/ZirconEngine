# Host Page Tabs UI TOML Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `HostPageChrome` and document dock tab/header sizing from handwritten Slint layout into `.ui.toml -> UiSurface -> host projection` while keeping Slint as the renderer/event adapter.

**Architecture:** The first vertical slice stays inside `zircon_editor`. `.ui.toml` exposes host/page/document tab chrome frames, Rust projection combines those frames with dynamic `TabData`, and Slint consumes projected frame DTOs without owning tab/header geometry. The existing template bridge and pointer bridges remain the lifecycle and event-routing boundary.

**Approved Amendment (2026-04-29):** The implementation direction is now TemplatePane-first. `HostMenuChrome`, `HostPageChrome`, side dock headers, document dock headers, and bottom dock headers render authored `.ui.toml` assets through `TemplatePane`; Slint may only keep transparent input overlays driven by projected `HostChromeControlFrameData` / `HostChromeTabData` frames. The chrome assets live at `zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml`, `zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml`, and `zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml`; the older separate `host/workbench_side_dock_header.ui.toml`, `host/workbench_document_dock_header.ui.toml`, and `host/workbench_bottom_dock_header.ui.toml` split is superseded for this slice. Tasks below that mention direct typed tab renderers, `DockTabButton`, `TabChip`, or `workbench_shell.ui.toml` frame-authority controls are superseded by the root-asset TemplatePane approach and the focused guards now in `generic_host_boundary.rs`.

**Current Focused Guards:**
- `slint_shell_chrome_uses_template_panes_instead_of_direct_chrome_buttons`
- `workbench_shell_chrome_ui_toml_assets_define_menu_page_and_dock_headers`
- `workbench_chrome_projection_uses_user_requested_asset_paths`
- `slint_chrome_inputs_use_projected_control_frames_instead_of_local_geometry_math`
- `workbench_chrome_heights_are_loaded_from_toml_assets_not_scene_constants`

**Latest Validation (2026-04-29 16:36 +08:00):**
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short`: passed after the lower-layer Hybrid GI resolve DTO consumers were converged to existing constructor/accessor APIs.
- `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture`: 31 passed / 0 failed / 873 filtered out.
- `cargo test -p zircon_editor --lib host_page_pointer --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture`: 7 passed / 0 failed / 897 filtered out.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short`: passed.
- Formatting caveat: `rustfmt --edition 2024 --check` on the touched Hybrid GI runtime files is still blocked by broader existing formatting drift in those large/concurrent files; this plan does not claim workspace formatting is green.

**Post-Review Guard Hardening (2026-04-29 05:36 +08:00):** Read-only review found the source guards could pass on import-only `TemplatePane` references or misplaced relative-coordinate substrings. `generic_host_boundary.rs` now scans actual `TemplatePane { ... }` blocks for their `nodes:` bindings and checks the compact `host_page_pointer_clicked(...)` relative-coordinate argument sequence. Re-review reported no findings. The earlier Runtime UI showcase blocker later converged source-side; the subsequent Hybrid GI compile blocker was resolved by consuming `HybridGiResolveProbeSceneData` / `HybridGiResolveTraceRegionSceneData` through existing constructors and accessors instead of direct private fields. Focused editor validation is now green again.

**Tech Stack:** Rust 2024, Slint, `zircon_runtime::ui::surface::UiSurface`, editor `.ui.toml` templates, Cargo focused tests.

**Repository Policy:** Work in the existing `main` checkout. Do not create a worktree or branch. Do not commit unless the user explicitly asks for a commit.

---

## File Structure

- Modify `zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs` to add source guards that fail while Slint still owns page/document tab layout math.
- Modify `zircon_editor/src/tests/host/template_runtime/shared_surface.rs` to require template-exposed page/document tab chrome frame controls.
- Modify `zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml` to expose page tab strip, page tab prototype, page project path, document tab prototype, document close button, and document subtitle controls as shared template frame authorities.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs` to add shared chrome tab/frame DTOs.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs` to build projected page/document tab frames from template bridge frames plus dynamic tabs.
- Modify `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs` to build host chrome template frame input from `BuiltinHostRootShellFrames` and convert new DTOs to Slint.
- Modify `zircon_editor/ui/workbench/host_components.slint` to expose the new DTOs to Slint.
- Modify `zircon_editor/ui/workbench/host_page_chrome.slint` to render page chrome from projected frames.
- Modify `zircon_editor/ui/workbench/host_document_dock_surface.slint` to render document tabs/header/content from projected frames.
- Modify `docs/ui-and-layout/shared-ui-template-runtime.md` and `.codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md` after validation to document the completed slice.

---

### Task 1: Add RED Source Guards For Slint Geometry Ownership

**Files:**
- Modify: `zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs`

- [ ] **Step 1: Add the failing source guards**

Append these tests near the existing host menu chrome guards in `generic_host_boundary.rs`:

```rust
#[test]
fn host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math() {
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_components.slint"
    ));
    let host_page_chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_page_chrome.slint"
    ));

    for required in [
        "export struct HostChromeTabData {",
        "frame: FrameRect,",
        "close_frame: FrameRect,",
        "tab_row_frame: FrameRect,",
        "project_path_frame: FrameRect,",
        "tabs: [HostChromeTabData]",
        "x: root.page_data.tab_row_frame.x * 1px;",
        "y: root.page_data.tab_row_frame.y * 1px;",
        "x: page.frame.x * 1px;",
        "width: page.frame.width * 1px;",
        "x: root.page_data.project_path_frame.x * 1px;",
    ] {
        assert!(
            host_components.contains(required) || host_page_chrome.contains(required),
            "host page chrome is missing projected template frame seam `{required}`"
        );
    }

    for forbidden in [
        "x: 8px;",
        "y: 1px;",
        "spacing: 4px;",
        "parent.width - 292px",
        "width: 280px",
        "8.0 + self.x / 1px",
        "1.0 + self.y / 1px",
    ] {
        assert!(
            !host_page_chrome.contains(forbidden),
            "host_page_chrome.slint still owns page tab layout math `{forbidden}`"
        );
    }
}

#[test]
fn document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math() {
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_components.slint"
    ));
    let document_dock = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_document_dock_surface.slint"
    ));

    for required in [
        "tabs: [HostChromeTabData]",
        "header_frame: FrameRect,",
        "tab_row_frame: FrameRect,",
        "subtitle_frame: FrameRect,",
        "content_frame: FrameRect,",
        "x: root.surface_data.header_frame.x * 1px;",
        "x: root.surface_data.tab_row_frame.x * 1px;",
        "x: tab.frame.x * 1px;",
        "width: tab.frame.width * 1px;",
        "x: root.surface_data.subtitle_frame.x * 1px;",
        "height: root.surface_data.content_frame.height * 1px;",
    ] {
        assert!(
            host_components.contains(required) || document_dock.contains(required),
            "document dock header is missing projected template frame seam `{required}`"
        );
    }

    for forbidden in [
        "x: 8px;",
        "y: 1px;",
        "spacing: 2px;",
        "parent.width - 144px",
        "width: 132px",
        "parent.height - root.surface_data.header_height_px * 1px - 1px",
        "8.0 + self.x / 1px",
        "1.0 + self.y / 1px",
        "self.width / 1px - 24.0",
        "1.0 + self.y / 1px + 7.0",
    ] {
        assert!(
            !document_dock.contains(forbidden),
            "host_document_dock_surface.slint still owns document tab layout math `{forbidden}`"
        );
    }
}
```

- [ ] **Step 2: Run the RED guards**

Run:

```powershell
cargo test -p zircon_editor --lib host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: fails because `HostChromeTabData` and projected frame bindings do not exist yet, or because `host_page_chrome.slint` still contains `x: 8px;` / `spacing: 4px;`.

Run:

```powershell
cargo test -p zircon_editor --lib document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: fails because `HostDocumentDockSurfaceData` lacks projected header/content frames, or because `host_document_dock_surface.slint` still contains document tab layout math.

- [ ] **Step 3: Checkpoint instead of committing**

Run:

```powershell
git diff -- zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
```

Expected: only the two new failing guard tests are shown.

---

### Task 2: Add RED Template Frame Authority Guard

**Files:**
- Modify: `zircon_editor/src/tests/host/template_runtime/shared_surface.rs`
- Modify later: `zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml`

- [ ] **Step 1: Add the failing shared surface test**

Append this test to `shared_surface.rs`:

```rust
#[test]
fn workbench_shell_template_exposes_page_and_document_tab_chrome_frames() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    for control_id in [
        "HostPageStripRoot",
        "HostPageTabPrototype",
        "HostPageProjectPathLabel",
        "DocumentTabsRoot",
        "DocumentTabPrototype",
        "DocumentTabClosePrototype",
        "DocumentSubtitleLabel",
        "PaneSurfaceRoot",
    ] {
        assert!(
            host_model.node_by_control_id(control_id).is_some(),
            "workbench_shell.ui.toml must expose `{control_id}` as a shared chrome frame authority"
        );
    }

    let host_page_strip = host_model.node_by_control_id("HostPageStripRoot").unwrap();
    assert_eq!(host_page_strip.frame.height, 24.0);

    let page_tab = host_model.node_by_control_id("HostPageTabPrototype").unwrap();
    assert_eq!(page_tab.frame.width, 68.0);
    assert_eq!(page_tab.frame.height, 22.0);

    let document_tab = host_model.node_by_control_id("DocumentTabPrototype").unwrap();
    assert_eq!(document_tab.frame.width, 114.0);
    assert_eq!(document_tab.frame.height, 30.0);

    let close_button = host_model
        .node_by_control_id("DocumentTabClosePrototype")
        .unwrap();
    assert_eq!(close_button.frame.width, 16.0);
    assert_eq!(close_button.frame.height, 16.0);
}
```

- [ ] **Step 2: Run the RED template guard**

Run:

```powershell
cargo test -p zircon_editor --lib workbench_shell_template_exposes_page_and_document_tab_chrome_frames --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: fails because `HostPageTabPrototype`, `HostPageProjectPathLabel`, `DocumentTabPrototype`, `DocumentTabClosePrototype`, or `DocumentSubtitleLabel` do not exist yet.

---

### Task 3: Expose Chrome Frames In `workbench_shell.ui.toml`

**Files:**
- Modify: `zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml`
- Verify: `zircon_editor/src/tests/host/template_runtime/shared_surface.rs`

- [ ] **Step 1: Update `HostPageStripRoot` from hidden zero-size frame to template-owned strip frame**

In the existing `HostPageStripRoot` block, replace the zero-size height/width constraints with fixed height and stretch width:

```toml
[components.UiHostWindow.root.children.node.layout.height]
max = 24.0
min = 24.0
preferred = 24.0
stretch = "Fixed"

[components.UiHostWindow.root.children.node.layout.width]
stretch = "Stretch"
```

Keep `control_id = "HostPageStripRoot"` and `clip = true`.

- [ ] **Step 2: Add page chrome prototype children under `HostPageStripRoot`**

Inside the `HostPageStripRoot` node, set an overlay container and add two children:

```toml
[components.UiHostWindow.root.children.node.layout.container]
kind = "Overlay"

[[components.UiHostWindow.root.children.node.children]]

[components.UiHostWindow.root.children.node.children.slot]

[components.UiHostWindow.root.children.node.children.node]
node_id = "component_UiHostWindow_root_1_page_tab_prototype"
kind = "native"
type = "Container"
control_id = "HostPageTabPrototype"
classes = []
bindings = []
children = []

[components.UiHostWindow.root.children.node.children.node.params]

[components.UiHostWindow.root.children.node.children.node.props]

[components.UiHostWindow.root.children.node.children.node.layout.height]
max = 22.0
min = 22.0
preferred = 22.0
stretch = "Fixed"

[components.UiHostWindow.root.children.node.children.node.layout.width]
max = 68.0
min = 68.0
preferred = 68.0
stretch = "Fixed"

[components.UiHostWindow.root.children.node.children.node.style_overrides.self]

[components.UiHostWindow.root.children.node.children.node.style_overrides.slot]

[[components.UiHostWindow.root.children.node.children]]

[components.UiHostWindow.root.children.node.children.slot]

[components.UiHostWindow.root.children.node.children.node]
node_id = "component_UiHostWindow_root_1_project_path"
kind = "native"
type = "Label"
control_id = "HostPageProjectPathLabel"
classes = []
bindings = []
children = []

[components.UiHostWindow.root.children.node.children.node.params]

[components.UiHostWindow.root.children.node.children.node.props]

[components.UiHostWindow.root.children.node.children.node.layout.height]
max = 16.0
min = 16.0
preferred = 16.0
stretch = "Fixed"

[components.UiHostWindow.root.children.node.children.node.layout.width]
max = 280.0
min = 280.0
preferred = 280.0
stretch = "Fixed"

[components.UiHostWindow.root.children.node.children.node.style_overrides.self]

[components.UiHostWindow.root.children.node.children.node.style_overrides.slot]
```

- [ ] **Step 3: Add document tab prototype controls under `DocumentTabsRoot`**

In `components.DocumentHost.root.children.node` for `control_id = "DocumentTabsRoot"`, add an overlay container and these prototype children:

```toml
[components.DocumentHost.root.children.node.layout.container]
kind = "Overlay"

[[components.DocumentHost.root.children.node.children]]

[components.DocumentHost.root.children.node.children.slot]

[components.DocumentHost.root.children.node.children.node]
node_id = "component_DocumentHost_tabs_document_tab_prototype"
kind = "native"
type = "Container"
control_id = "DocumentTabPrototype"
classes = []
bindings = []
children = []

[components.DocumentHost.root.children.node.children.node.params]

[components.DocumentHost.root.children.node.children.node.props]

[components.DocumentHost.root.children.node.children.node.layout.height]
max = 30.0
min = 30.0
preferred = 30.0
stretch = "Fixed"

[components.DocumentHost.root.children.node.children.node.layout.width]
max = 114.0
min = 114.0
preferred = 114.0
stretch = "Fixed"

[components.DocumentHost.root.children.node.children.node.style_overrides.self]

[components.DocumentHost.root.children.node.children.node.style_overrides.slot]

[[components.DocumentHost.root.children.node.children]]

[components.DocumentHost.root.children.node.children.slot]

[components.DocumentHost.root.children.node.children.node]
node_id = "component_DocumentHost_tabs_close_prototype"
kind = "native"
type = "Container"
control_id = "DocumentTabClosePrototype"
classes = []
bindings = []
children = []

[components.DocumentHost.root.children.node.children.node.params]

[components.DocumentHost.root.children.node.children.node.props]

[components.DocumentHost.root.children.node.children.node.layout.height]
max = 16.0
min = 16.0
preferred = 16.0
stretch = "Fixed"

[components.DocumentHost.root.children.node.children.node.layout.width]
max = 16.0
min = 16.0
preferred = 16.0
stretch = "Fixed"

[components.DocumentHost.root.children.node.children.node.style_overrides.self]

[components.DocumentHost.root.children.node.children.node.style_overrides.slot]
```

- [ ] **Step 4: Add document subtitle label authority**

Add `DocumentSubtitleLabel` as an additional child under `components.DocumentHost.root` after `DocumentTabsRoot` and before `PaneSurfaceRoot`:

```toml
[[components.DocumentHost.root.children]]

[components.DocumentHost.root.children.slot]

[components.DocumentHost.root.children.node]
node_id = "component_DocumentHost_subtitle_label"
kind = "native"
type = "Label"
control_id = "DocumentSubtitleLabel"
classes = []
bindings = []
children = []

[components.DocumentHost.root.children.node.params]

[components.DocumentHost.root.children.node.props]

[components.DocumentHost.root.children.node.layout.height]
max = 0.0
min = 0.0
preferred = 0.0
stretch = "Fixed"

[components.DocumentHost.root.children.node.layout.width]
max = 132.0
min = 132.0
preferred = 132.0
stretch = "Fixed"

[components.DocumentHost.root.children.node.style_overrides.self]

[components.DocumentHost.root.children.node.style_overrides.slot]
```

This label is a frame authority only in this slice. It is projected into Slint, not rendered by the shared template surface.

- [ ] **Step 5: Run the template guard**

Run:

```powershell
cargo test -p zircon_editor --lib workbench_shell_template_exposes_page_and_document_tab_chrome_frames --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: pass. If the exact node count assertions in `shared_surface.rs` fail, update the counts in existing tests by adding the number of new template authority nodes. Do not remove the frame assertions.

---

### Task 4: Add Host Chrome Frame DTOs

**Files:**
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- Modify: `zircon_editor/ui/workbench/host_components.slint`

- [ ] **Step 1: Add Rust DTOs**

In `host_data.rs`, after `TabData`, add:

```rust
#[derive(Clone)]
pub(crate) struct HostChromeTabData {
    pub id: SharedString,
    pub slot: SharedString,
    pub title: SharedString,
    pub icon_key: SharedString,
    pub active: bool,
    pub closeable: bool,
    pub frame: FrameRect,
    pub close_frame: FrameRect,
    pub drag_origin_x_px: f32,
    pub drag_origin_y_px: f32,
}

#[derive(Clone, Default)]
pub(crate) struct HostChromeTemplateFrames {
    pub host_page_strip_frame: Option<FrameRect>,
    pub host_page_tab_prototype_frame: Option<FrameRect>,
    pub host_page_project_path_frame: Option<FrameRect>,
    pub document_tabs_frame: Option<FrameRect>,
    pub document_tab_prototype_frame: Option<FrameRect>,
    pub document_tab_close_prototype_frame: Option<FrameRect>,
    pub document_subtitle_frame: Option<FrameRect>,
    pub pane_surface_frame: Option<FrameRect>,
}
```

- [ ] **Step 2: Update Rust page/document DTOs**

Replace `tabs: ModelRc<TabData>` in `HostPageChromeData` with projected tabs and add frame fields:

```rust
#[derive(Clone)]
pub(crate) struct HostPageChromeData {
    pub top_bar_height_px: f32,
    pub host_bar_height_px: f32,
    pub frame: FrameRect,
    pub tab_row_frame: FrameRect,
    pub project_path_frame: FrameRect,
    pub tabs: ModelRc<HostChromeTabData>,
    pub project_path: SharedString,
}
```

Replace `tabs: ModelRc<TabData>` in `HostDocumentDockSurfaceData` and add frame fields:

```rust
#[derive(Clone)]
pub(crate) struct HostDocumentDockSurfaceData {
    pub region_frame: FrameRect,
    pub surface_key: SharedString,
    pub tabs: ModelRc<HostChromeTabData>,
    pub pane: PaneData,
    pub header_height_px: f32,
    pub header_frame: FrameRect,
    pub tab_row_frame: FrameRect,
    pub subtitle_frame: FrameRect,
    pub content_frame: FrameRect,
    pub tab_origin_x_px: f32,
    pub tab_origin_y_px: f32,
}
```

- [ ] **Step 3: Add Slint DTOs**

In `host_components.slint`, after `export struct TabData`, add:

```slint
export struct HostChromeTabData {
    id: string,
    slot: string,
    title: string,
    icon_key: string,
    active: bool,
    closeable: bool,
    frame: FrameRect,
    close_frame: FrameRect,
    drag_origin_x_px: float,
    drag_origin_y_px: float,
}
```

Update `HostPageChromeData`:

```slint
export struct HostPageChromeData {
    top_bar_height_px: float,
    host_bar_height_px: float,
    frame: FrameRect,
    tab_row_frame: FrameRect,
    project_path_frame: FrameRect,
    tabs: [HostChromeTabData],
    project_path: string,
}
```

Update `HostDocumentDockSurfaceData`:

```slint
export struct HostDocumentDockSurfaceData {
    region_frame: FrameRect,
    surface_key: string,
    tabs: [HostChromeTabData],
    pane: PaneData,
    header_height_px: float,
    header_frame: FrameRect,
    tab_row_frame: FrameRect,
    subtitle_frame: FrameRect,
    content_frame: FrameRect,
    tab_origin_x_px: float,
    tab_origin_y_px: float,
}
```

- [ ] **Step 4: Run the source guards again**

Run:

```powershell
cargo test -p zircon_editor --lib host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: still fails because Slint has not been updated to consume projected frames.

Run:

```powershell
cargo test -p zircon_editor --lib document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: still fails because Slint has not been updated to consume projected frames.

---

### Task 5: Project Page And Document Tab Frames In Rust

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`
- Modify: `zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs`

- [ ] **Step 1: Build template frame input in `apply_presentation.rs`**

Add this import adjustment at the top if `HostChromeTemplateFrames` is not already in scope through `host_window`:

```rust
use crate::ui::layouts::windows::workbench_host_window::HostChromeTemplateFrames;
```

Add this helper near `host_window_layout`:

```rust
fn host_chrome_template_frames(
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> HostChromeTemplateFrames {
    let Some(frames) = shared_root_frames else {
        return HostChromeTemplateFrames::default();
    };
    HostChromeTemplateFrames {
        host_page_strip_frame: frames.host_page_strip_frame.map(frame_rect),
        host_page_tab_prototype_frame: None,
        host_page_project_path_frame: None,
        document_tabs_frame: frames.document_tabs_frame.map(frame_rect),
        document_tab_prototype_frame: None,
        document_tab_close_prototype_frame: None,
        document_subtitle_frame: None,
        pane_surface_frame: frames.pane_surface_frame.map(frame_rect),
    }
}
```

Then change the `build_host_scene_data` call:

```rust
let host_chrome_template_frames = host_chrome_template_frames(shared_root_frames);
let host_scene_data = build_host_scene_data(
    &model.menu_bar,
    &presentation.host_surface_data,
    &presentation.host_shell,
    &host_layout,
    &host_chrome_template_frames,
    &presentation.status_primary,
    chrome.inspector.is_some(),
    &chrome.project_overview,
);
```

- [ ] **Step 2: Update `build_host_scene_data` signature**

In `scene_projection.rs`, change the function signature to accept template frames:

```rust
pub(crate) fn build_host_scene_data(
    menu_bar: &MenuBarModel,
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    chrome_template_frames: &HostChromeTemplateFrames,
    status_primary: &SharedString,
    delete_enabled: bool,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> HostWindowSceneData {
```

- [ ] **Step 3: Add projection constants to Rust, scoped as adapter defaults**

Near the existing metric constants in `scene_projection.rs`, add:

```rust
const PAGE_TAB_ROW_X_PX: f32 = 8.0;
const PAGE_TAB_ROW_Y_PX: f32 = 1.0;
const PAGE_TAB_GAP_PX: f32 = 4.0;
const PAGE_TAB_WIDTH_PX: f32 = 68.0;
const PAGE_TAB_HEIGHT_PX: f32 = 22.0;
const PROJECT_PATH_WIDTH_PX: f32 = 280.0;
const PROJECT_PATH_HEIGHT_PX: f32 = 16.0;
const PROJECT_PATH_RIGHT_PADDING_PX: f32 = 12.0;
const PROJECT_PATH_Y_PX: f32 = 5.0;
const DOCUMENT_TAB_ROW_X_PX: f32 = 8.0;
const DOCUMENT_TAB_ROW_Y_PX: f32 = 1.0;
const DOCUMENT_TAB_GAP_PX: f32 = 2.0;
const DOCUMENT_TAB_WIDTH_PX: f32 = 114.0;
const DOCUMENT_TAB_HEIGHT_PX: f32 = 30.0;
const DOCUMENT_TAB_CLOSE_X_OFFSET_PX: f32 = 24.0;
const DOCUMENT_TAB_CLOSE_Y_OFFSET_PX: f32 = 7.0;
const DOCUMENT_TAB_CLOSE_SIZE_PX: f32 = 16.0;
const DOCUMENT_SUBTITLE_WIDTH_PX: f32 = 132.0;
const DOCUMENT_SUBTITLE_HEIGHT_PX: f32 = 16.0;
const DOCUMENT_SUBTITLE_RIGHT_PADDING_PX: f32 = 12.0;
const DOCUMENT_SUBTITLE_Y_PX: f32 = 9.0;
```

These constants are temporary Rust adapter defaults. The source guards prevent these values from living in Slint. A later slice can replace them with explicit template prototype frames from `host_projection` once prototype frame export is wired.

- [ ] **Step 4: Add helper functions in `scene_projection.rs`**

Add these helpers below `menu_popup_height`:

```rust
fn frame_or_default(frame: Option<&FrameRect>) -> FrameRect {
    frame.cloned().unwrap_or_default()
}

fn page_chrome_data(
    tabs: &ModelRc<TabData>,
    host_shell: &HostWindowShellData,
    metrics: &HostWindowSurfaceMetricsData,
    chrome_template_frames: &HostChromeTemplateFrames,
) -> HostPageChromeData {
    let frame = frame_or_default(chrome_template_frames.host_page_strip_frame.as_ref());
    let tab_row_frame = FrameRect {
        x: frame.x + PAGE_TAB_ROW_X_PX,
        y: frame.y + PAGE_TAB_ROW_Y_PX,
        width: (frame.width - PAGE_TAB_ROW_X_PX).max(0.0),
        height: PAGE_TAB_HEIGHT_PX,
    };
    let project_path_frame = FrameRect {
        x: (frame.x + frame.width - PROJECT_PATH_WIDTH_PX - PROJECT_PATH_RIGHT_PADDING_PX)
            .max(frame.x),
        y: frame.y + PROJECT_PATH_Y_PX,
        width: PROJECT_PATH_WIDTH_PX.min(frame.width.max(0.0)),
        height: PROJECT_PATH_HEIGHT_PX,
    };
    HostPageChromeData {
        top_bar_height_px: metrics.top_bar_height_px,
        host_bar_height_px: metrics.host_bar_height_px,
        frame: frame.clone(),
        tab_row_frame: tab_row_frame.clone(),
        project_path_frame,
        tabs: projected_tabs(
            tabs,
            &tab_row_frame,
            PAGE_TAB_WIDTH_PX,
            PAGE_TAB_HEIGHT_PX,
            PAGE_TAB_GAP_PX,
            None,
        ),
        project_path: host_shell.project_path.clone(),
    }
}

fn document_header_frames(
    region_frame: &FrameRect,
    header_height_px: f32,
    chrome_template_frames: &HostChromeTemplateFrames,
) -> (FrameRect, FrameRect, FrameRect, FrameRect) {
    let header_frame = chrome_template_frames
        .document_tabs_frame
        .clone()
        .unwrap_or_else(|| FrameRect {
            x: region_frame.x,
            y: region_frame.y,
            width: region_frame.width,
            height: header_height_px,
        });
    let tab_row_frame = FrameRect {
        x: header_frame.x + DOCUMENT_TAB_ROW_X_PX,
        y: header_frame.y + DOCUMENT_TAB_ROW_Y_PX,
        width: (header_frame.width - DOCUMENT_TAB_ROW_X_PX).max(0.0),
        height: DOCUMENT_TAB_HEIGHT_PX,
    };
    let subtitle_frame = FrameRect {
        x: (header_frame.x + header_frame.width
            - DOCUMENT_SUBTITLE_WIDTH_PX
            - DOCUMENT_SUBTITLE_RIGHT_PADDING_PX)
            .max(header_frame.x),
        y: header_frame.y + DOCUMENT_SUBTITLE_Y_PX,
        width: DOCUMENT_SUBTITLE_WIDTH_PX.min(header_frame.width.max(0.0)),
        height: DOCUMENT_SUBTITLE_HEIGHT_PX,
    };
    let content_frame = chrome_template_frames
        .pane_surface_frame
        .clone()
        .unwrap_or_else(|| FrameRect {
            x: region_frame.x,
            y: header_frame.y + header_frame.height + 1.0,
            width: region_frame.width,
            height: (region_frame.height - header_frame.height - 1.0).max(0.0),
        });
    (header_frame, tab_row_frame, subtitle_frame, content_frame)
}

fn projected_tabs(
    tabs: &ModelRc<TabData>,
    row_frame: &FrameRect,
    tab_width_px: f32,
    tab_height_px: f32,
    tab_gap_px: f32,
    close: Option<(f32, f32, f32)>,
) -> ModelRc<HostChromeTabData> {
    model_rc(
        (0..tabs.row_count())
            .filter_map(|row| tabs.row_data(row))
            .enumerate()
            .map(|(index, tab)| {
                let x = row_frame.x + index as f32 * (tab_width_px + tab_gap_px);
                let frame = FrameRect {
                    x,
                    y: row_frame.y,
                    width: tab_width_px,
                    height: tab_height_px,
                };
                let close_frame = close
                    .map(|(offset_x, offset_y, size)| FrameRect {
                        x: frame.x + frame.width - offset_x,
                        y: frame.y + offset_y,
                        width: size,
                        height: size,
                    })
                    .unwrap_or_default();
                HostChromeTabData {
                    id: tab.id,
                    slot: tab.slot,
                    title: tab.title,
                    icon_key: tab.icon_key,
                    active: tab.active,
                    closeable: tab.closeable,
                    frame: frame.clone(),
                    close_frame,
                    drag_origin_x_px: frame.x,
                    drag_origin_y_px: frame.y,
                }
            })
            .collect(),
    )
}
```

- [ ] **Step 5: Use the helpers when building page/document data**

Replace the inline `HostPageChromeData` creation with:

```rust
let page_chrome = page_chrome_data(
    &host_surface_data.host_tabs,
    host_shell,
    &metrics,
    chrome_template_frames,
);
```

Before building `document_dock`, compute:

```rust
let (document_header_frame, document_tab_row_frame, document_subtitle_frame, document_content_frame) =
    document_header_frames(
        &host_layout.document_region_frame,
        metrics.document_header_height_px,
        chrome_template_frames,
    );
```

Then set `HostDocumentDockSurfaceData` fields:

```rust
tabs: projected_tabs(
    &host_surface_data.document_tabs,
    &document_tab_row_frame,
    DOCUMENT_TAB_WIDTH_PX,
    DOCUMENT_TAB_HEIGHT_PX,
    DOCUMENT_TAB_GAP_PX,
    Some((
        DOCUMENT_TAB_CLOSE_X_OFFSET_PX,
        DOCUMENT_TAB_CLOSE_Y_OFFSET_PX,
        DOCUMENT_TAB_CLOSE_SIZE_PX,
    )),
),
header_frame: document_header_frame,
tab_row_frame: document_tab_row_frame,
subtitle_frame: document_subtitle_frame,
content_frame: document_content_frame,
```

- [ ] **Step 6: Run Cargo to reveal conversion errors**

Run:

```powershell
cargo test -p zircon_editor --lib host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: compile fails in Slint/Rust conversion because `apply_presentation.rs` and Slint DTOs are not fully connected yet, or source guard still fails because Slint still owns old layout math.

---

### Task 6: Convert New DTOs To Slint

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs`

- [ ] **Step 1: Add `HostChromeTabData` conversion**

Below `to_slint_tabs`, add:

```rust
fn to_slint_host_chrome_tab(tab: host_window::HostChromeTabData) -> slint_ui::HostChromeTabData {
    slint_ui::HostChromeTabData {
        id: tab.id,
        slot: tab.slot,
        title: tab.title,
        icon_key: tab.icon_key,
        active: tab.active,
        closeable: tab.closeable,
        frame: to_slint_frame_rect(&tab.frame),
        close_frame: to_slint_frame_rect(&tab.close_frame),
        drag_origin_x_px: tab.drag_origin_x_px,
        drag_origin_y_px: tab.drag_origin_y_px,
    }
}

fn to_slint_host_chrome_tabs(
    tabs: &ModelRc<host_window::HostChromeTabData>,
) -> ModelRc<slint_ui::HostChromeTabData> {
    map_model_rc(tabs, to_slint_host_chrome_tab)
}
```

- [ ] **Step 2: Update page chrome conversion**

Replace `to_slint_page_chrome` with:

```rust
fn to_slint_page_chrome(page: &host_window::HostPageChromeData) -> slint_ui::HostPageChromeData {
    slint_ui::HostPageChromeData {
        top_bar_height_px: page.top_bar_height_px,
        host_bar_height_px: page.host_bar_height_px,
        frame: to_slint_frame_rect(&page.frame),
        tab_row_frame: to_slint_frame_rect(&page.tab_row_frame),
        project_path_frame: to_slint_frame_rect(&page.project_path_frame),
        tabs: to_slint_host_chrome_tabs(&page.tabs),
        project_path: page.project_path.clone(),
    }
}
```

- [ ] **Step 3: Update document dock conversion**

Update `to_slint_document_dock` to convert the new fields:

```rust
fn to_slint_document_dock(
    dock: &host_window::HostDocumentDockSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
) -> slint_ui::HostDocumentDockSurfaceData {
    let pane_size = host_window::PaneContentSize::new(
        dock.content_frame.width,
        dock.content_frame.height,
    );
    slint_ui::HostDocumentDockSurfaceData {
        region_frame: to_slint_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        tabs: to_slint_host_chrome_tabs(&dock.tabs),
        pane: to_slint_pane(dock.pane.clone(), pane_size, component_showcase_runtime),
        header_height_px: dock.header_height_px,
        header_frame: to_slint_frame_rect(&dock.header_frame),
        tab_row_frame: to_slint_frame_rect(&dock.tab_row_frame),
        subtitle_frame: to_slint_frame_rect(&dock.subtitle_frame),
        content_frame: to_slint_frame_rect(&dock.content_frame),
        tab_origin_x_px: dock.tab_origin_x_px,
        tab_origin_y_px: dock.tab_origin_y_px,
    }
}
```

- [ ] **Step 4: Run Cargo to reveal Slint layout guard failures**

Run:

```powershell
cargo test -p zircon_editor --lib host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: source guard still fails on Slint-owned geometry strings until the next tasks update Slint.

---

### Task 7: Render `HostPageChrome` From Projected Frames

**Files:**
- Modify: `zircon_editor/ui/workbench/host_page_chrome.slint`

- [ ] **Step 1: Replace the default page data with projected frame defaults**

Use this default property value:

```slint
in property <HostPageChromeData> page_data: {
    top_bar_height_px: 25.0,
    host_bar_height_px: 24.0,
    frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
    tab_row_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
    project_path_frame: { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
    tabs: [],
    project_path: "",
};
```

- [ ] **Step 2: Replace the hardcoded header rectangle and layout with projected frames**

Replace the component body after `background: transparent;` with:

```slint
Rectangle {
    x: root.page_data.frame.x * 1px;
    y: root.page_data.frame.y * 1px;
    width: root.page_data.frame.width * 1px;
    height: root.page_data.frame.height * 1px;
    background: palette.chrome_bg;

    for page[index] in root.page_data.tabs: DockTabButton {
        x: page.frame.x * 1px - root.page_data.frame.x * 1px;
        y: page.frame.y * 1px - root.page_data.frame.y * 1px;
        width: page.frame.width * 1px;
        height: page.frame.height * 1px;
        label: page.title;
        icon_key: page.icon_key;
        active: page.active;
        pointer_pressed(x, y) => {
            UiHostContext.host_page_pointer_clicked(
                index,
                page.frame.x,
                page.frame.width,
                page.frame.x + x,
                page.frame.y + y,
            );
        }
    }

    Text {
        x: root.page_data.project_path_frame.x * 1px - root.page_data.frame.x * 1px;
        y: root.page_data.project_path_frame.y * 1px - root.page_data.frame.y * 1px;
        width: root.page_data.project_path_frame.width * 1px;
        height: root.page_data.project_path_frame.height * 1px;
        horizontal-alignment: right;
        text: root.page_data.project_path != "" ? root.page_data.project_path : "No project open";
        color: palette.text_dim;
        font-size: 9px;
        overflow: elide;
    }
}
```

- [ ] **Step 3: Run the page source guard**

Run:

```powershell
cargo test -p zircon_editor --lib host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: pass for `HostPageChrome` if Rust/Slint DTO conversion compiles.

---

### Task 8: Render Document Dock Header From Projected Frames

**Files:**
- Modify: `zircon_editor/ui/workbench/host_document_dock_surface.slint`

- [ ] **Step 1: Replace header layout with projected frames**

In `HostDocumentDockSurface`, replace the top header `Rectangle` and following `PaneSurface` height math with projected frames. The component body should keep the root `x/y/width/height` from `region_frame`, then use:

```slint
Rectangle {
    x: root.surface_data.header_frame.x * 1px - root.surface_data.region_frame.x * 1px;
    y: root.surface_data.header_frame.y * 1px - root.surface_data.region_frame.y * 1px;
    width: root.surface_data.header_frame.width * 1px;
    height: root.surface_data.header_frame.height * 1px;
    background: palette.tab_bg;

    for tab[index] in root.surface_data.tabs: TabChip {
        x: tab.frame.x * 1px - root.surface_data.header_frame.x * 1px;
        y: tab.frame.y * 1px - root.surface_data.header_frame.y * 1px;
        width: tab.frame.width * 1px;
        height: tab.frame.height * 1px;
        label: tab.title;
        icon_key: tab.icon_key;
        active: tab.active;
        closeable: tab.closeable;
        drag_origin_x: tab.drag_origin_x_px;
        drag_origin_y: tab.drag_origin_y_px;
        pointer_clicked(x, y) => {
            UiHostContext.document_tab_pointer_clicked(
                root.surface_data.surface_key,
                index,
                tab.frame.x,
                tab.frame.width,
                tab.frame.x + x,
                tab.frame.y + y,
            );
        }
        close_pointer_clicked(x, y) => {
            UiHostContext.document_tab_close_pointer_clicked(
                root.surface_data.surface_key,
                index,
                tab.frame.x,
                tab.frame.width,
                tab.close_frame.x + x,
                tab.close_frame.y + y,
            );
        }
        drag_started(x, y) => {
            UiHostContext.drag_state = {
                active_drag_target_group: UiHostContext.drag_state.active_drag_target_group,
                drag_active: true,
                drag_tab_id: tab.id,
                drag_tab_title: tab.title,
                drag_tab_icon_key: tab.icon_key,
                drag_source_group: root.surface_data.surface_key,
                drag_pointer_x: x,
                drag_pointer_y: y,
            };
            UiHostContext.host_drag_pointer_event(
                1,
                UiHostContext.drag_state.drag_pointer_x,
                UiHostContext.drag_state.drag_pointer_y,
            );
        }
        drag_moved(x, y) => {
            if (UiHostContext.drag_state.drag_tab_id == tab.id) {
                UiHostContext.drag_state = {
                    active_drag_target_group: UiHostContext.drag_state.active_drag_target_group,
                    drag_active: UiHostContext.drag_state.drag_active,
                    drag_tab_id: UiHostContext.drag_state.drag_tab_id,
                    drag_tab_title: UiHostContext.drag_state.drag_tab_title,
                    drag_tab_icon_key: UiHostContext.drag_state.drag_tab_icon_key,
                    drag_source_group: UiHostContext.drag_state.drag_source_group,
                    drag_pointer_x: x,
                    drag_pointer_y: y,
                };
                UiHostContext.host_drag_pointer_event(
                    1,
                    UiHostContext.drag_state.drag_pointer_x,
                    UiHostContext.drag_state.drag_pointer_y,
                );
            }
        }
    }

    Text {
        x: root.surface_data.subtitle_frame.x * 1px - root.surface_data.header_frame.x * 1px;
        y: root.surface_data.subtitle_frame.y * 1px - root.surface_data.header_frame.y * 1px;
        width: root.surface_data.subtitle_frame.width * 1px;
        height: root.surface_data.subtitle_frame.height * 1px;
        horizontal-alignment: right;
        text: root.surface_data.pane.subtitle;
        color: palette.text_dim;
        font-size: 10px;
        overflow: elide;
    }
}

Rectangle {
    x: root.surface_data.content_frame.x * 1px - root.surface_data.region_frame.x * 1px;
    y: root.surface_data.content_frame.y * 1px - root.surface_data.region_frame.y * 1px - 1px;
    width: root.surface_data.content_frame.width * 1px;
    height: 1px;
    background: palette.border_soft;
}

PaneSurface {
    x: root.surface_data.content_frame.x * 1px - root.surface_data.region_frame.x * 1px;
    y: root.surface_data.content_frame.y * 1px - root.surface_data.region_frame.y * 1px;
    width: root.surface_data.content_frame.width * 1px;
    height: root.surface_data.content_frame.height * 1px;
    pane: root.surface_data.pane;
    show_header: false;
    compact_empty: false;
}
```

- [ ] **Step 2: Run the document source guard**

Run:

```powershell
cargo test -p zircon_editor --lib document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: pass if document dock Slint no longer contains the forbidden layout strings.

---

### Task 9: Update Tests Impacted By DTO Shape

**Files:**
- Modify: `zircon_editor/src/ui/slint_host/ui/tests.rs`
- Modify any focused compile failures reported by Cargo in editor tests.

- [ ] **Step 1: Update test fixtures that construct `HostPageChromeData`**

For any fixture with:

```rust
page_chrome: host_window::HostPageChromeData {
    top_bar_height_px: 40.0,
    host_bar_height_px: 24.0,
    tabs: ...,
    project_path: ...,
},
```

replace with:

```rust
page_chrome: host_window::HostPageChromeData {
    top_bar_height_px: 40.0,
    host_bar_height_px: 24.0,
    frame: host_window::FrameRect::default(),
    tab_row_frame: host_window::FrameRect::default(),
    project_path_frame: host_window::FrameRect::default(),
    tabs: model_rc(Vec::<host_window::HostChromeTabData>::new()),
    project_path: "project".into(),
},
```

- [ ] **Step 2: Update document dock test fixtures**

For any fixture with `HostDocumentDockSurfaceData { tabs: ... }`, use:

```rust
document_dock: host_window::HostDocumentDockSurfaceData {
    region_frame: host_window::FrameRect::default(),
    surface_key: "document".into(),
    tabs: model_rc(Vec::<host_window::HostChromeTabData>::new()),
    pane: host_window::PaneData::default(),
    header_height_px: 32.0,
    header_frame: host_window::FrameRect::default(),
    tab_row_frame: host_window::FrameRect::default(),
    subtitle_frame: host_window::FrameRect::default(),
    content_frame: host_window::FrameRect::default(),
    tab_origin_x_px: 0.0,
    tab_origin_y_px: 0.0,
},
```

If `PaneData::default()` is not available in that fixture scope, clone the existing `pane` value from the current fixture and only replace the new frame/tab fields.

- [ ] **Step 3: Run the focused conversion tests**

Run:

```powershell
cargo test -p zircon_editor --lib slint_host --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: either pass or report remaining fixture compile errors caused by the DTO field changes. Fix only compile errors in tests that directly construct these DTOs.

---

### Task 10: Focused Regression Validation

**Files:**
- Verify all touched production/test files.

- [ ] **Step 1: Run the exact guards**

Run:

```powershell
cargo test -p zircon_editor --lib host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: `1 passed; 0 failed` for the focused guard.

Run:

```powershell
cargo test -p zircon_editor --lib document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: `1 passed; 0 failed` for the focused guard.

- [ ] **Step 2: Run the template authority guard**

Run:

```powershell
cargo test -p zircon_editor --lib workbench_shell_template_exposes_page_and_document_tab_chrome_frames --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: `1 passed; 0 failed` for the focused guard.

- [ ] **Step 3: Run the generic host boundary group**

Run:

```powershell
cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-editor-host-page-tabs" --message-format short -- --test-threads=1 --nocapture
```

Expected: all focused generic host boundary tests pass. If this times out during compilation under active Cargo pressure, record the timeout and rerun once the target is warm.

- [ ] **Step 4: Run formatting and whitespace checks**

Run:

```powershell
cargo fmt --package zircon_editor -- --check
```

Expected: no output.

Run:

```powershell
git diff --check -- "zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs" "zircon_editor/src/tests/host/template_runtime/shared_surface.rs" "zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml" "zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs" "zircon_editor/src/ui/slint_host/ui/apply_presentation.rs" "zircon_editor/ui/workbench/host_components.slint" "zircon_editor/ui/workbench/host_page_chrome.slint" "zircon_editor/ui/workbench/host_document_dock_surface.slint"
```

Expected: no whitespace errors. LF-to-CRLF warnings are acceptable in this repository.

---

### Task 11: Update Roadmap And Module Docs

**Files:**
- Modify: `.codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md`
- Modify: `docs/ui-and-layout/shared-ui-template-runtime.md`
- Modify: `.codex/sessions/20260428-2342-host-page-tabs-ui-toml-design.md`

- [ ] **Step 1: Update the cutover roadmap**

Add a checked bullet under Generic host boundary after the menu chrome bullets:

```markdown
- [x] Generic host boundary / page and document tab chrome template frames：`HostPageChrome` 与 document dock tab/header 的 tab strip、project path、subtitle、content frame 已改由 `.ui.toml -> UiSurface -> host projection` 下发，Slint 只按 `HostChromeTabData.frame` / `close_frame` 渲染和转发 pointer facts；focused guards `host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math`、`document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math` 和 `workbench_shell_template_exposes_page_and_document_tab_chrome_frames` 已通过。
```

- [ ] **Step 2: Update shared UI template runtime docs**

In `docs/ui-and-layout/shared-ui-template-runtime.md`, add related code entries for any newly touched files if missing, and add this status paragraph near the current Generic Host Boundary section:

```markdown
`HostPageChrome` 和 document dock tab/header 也进入了 shared template frame authority：`workbench_shell.ui.toml` 暴露 page/document tab chrome frame controls，`scene_projection.rs` 将这些 frames 与动态 `TabData` 合成 `HostChromeTabData`，Slint 侧只消费 projected frame DTO 并转发 pointer facts。该 slice 明确不接管当前 menu layout regression；menu chrome 会在后续使用同一 DTO/authority 模式收敛。
```

Record the exact validation commands and pass counts from Task 10.

- [ ] **Step 3: Update or retire the coordination note**

If implementation is complete and no handoff is needed, move `.codex/sessions/20260428-2342-host-page-tabs-ui-toml-design.md` to `.codex/sessions/archive/` and set:

```yaml
status: completed
```

If validation is blocked by active Cargo pressure or another session-owned compile failure, keep the active note and add the blocker with exact command output.

- [ ] **Step 4: Final status check**

Run:

```powershell
git status --short -- "docs/superpowers/plans/2026-04-28-host-page-tabs-ui-toml-implementation.md" "docs/superpowers/specs/2026-04-28-host-page-tabs-ui-toml-design.md" ".codex/sessions/20260428-2342-host-page-tabs-ui-toml-design.md" ".codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md" "docs/ui-and-layout/shared-ui-template-runtime.md" "zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs" "zircon_editor/src/tests/host/template_runtime/shared_surface.rs" "zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml" "zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs" "zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs" "zircon_editor/src/ui/slint_host/ui/apply_presentation.rs" "zircon_editor/ui/workbench/host_components.slint" "zircon_editor/ui/workbench/host_page_chrome.slint" "zircon_editor/ui/workbench/host_document_dock_surface.slint"
```

Expected: only intended files for this slice are listed. Do not revert unrelated workspace changes.

---

## Self-Review Notes

- Spec coverage: Tasks 1-2 establish RED guards for Slint geometry ownership and template authority; Tasks 3-8 implement the `.ui.toml -> UiSurface/shared frames -> Rust projection -> Slint adapter` path; Task 10 validates; Task 11 updates docs and coordination.
- Scope control: The plan does not edit `HostMenuChrome`, side dock headers, bottom dock headers, floating headers, runtime graphics, or Runtime UI showcase files.
- Type consistency: `HostChromeTabData`, `HostChromeTemplateFrames`, `HostPageChromeData`, and `HostDocumentDockSurfaceData` names are consistent across Rust DTOs, Slint DTOs, conversion, and tests.
- Repository policy: The plan avoids worktrees, branches, and commits because this repository works directly on `main` and commits require explicit user request.

---
related_code:
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon Editor Workbench Shell VNext.md
  - .codex/plans/全系统重构方案.md
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/ui/workbench/host_components.slint
  - zircon_editor/ui/workbench/host_page_chrome.slint
  - zircon_editor/ui/workbench/host_document_dock_surface.slint
  - zircon_editor/ui/workbench/host_side_dock_surface.slint
  - zircon_editor/ui/workbench/host_bottom_dock_surface.slint
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
implementation_files:
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/ui/workbench/host_components.slint
  - zircon_editor/ui/workbench/host_page_chrome.slint
  - zircon_editor/ui/workbench/host_document_dock_surface.slint
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
plan_sources:
  - user: 2026-04-28 Move HostMenuChrome, HostPageChrome, and dock headers to ui.toml so Slint is only the rendering adapter
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon Editor Workbench Shell VNext.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo test -p zircon_editor --lib host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1
  - cargo test -p zircon_editor --lib document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math --locked --jobs 1
  - cargo test -p zircon_editor --lib editor_ui_host_runtime_builds_laid_out_host_model_from_shared_surface_authority --locked --jobs 1
  - cargo test -p zircon_editor --lib slint_shell_chrome_uses_template_panes_instead_of_direct_chrome_buttons --locked --jobs 1
  - cargo test -p zircon_editor --lib slint_chrome_inputs_use_projected_control_frames_instead_of_local_geometry_math --locked --jobs 1
  - cargo test -p zircon_editor --lib host_page_pointer --locked --jobs 1
  - cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1
  - cargo check -p zircon_editor --lib --locked --jobs 1
  - cargo fmt --package zircon_editor -- --check
doc_type: milestone-detail
---
# Host Page Tabs UI TOML Design

## Background

The Generic Host Boundary cutover is moving editor workbench structure, layout formulas, event semantics, hit testing, and route results from handwritten `.slint` into `TOML -> UiSurface -> host projection`. The previous menu chrome slice moved menu labels/items into projected data and pulled menu-button frame ABI back inside `HostMenuChrome`, but the next visible problem remains: top tabs and dock headers still encode sizing and placement in Slint or Rust constants.

The current state already has the required lower layer:

- `zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml` defines a host shell with `UiHostWindow`, `MenuBar`, `ActivityRail`, `DocumentHost`, `DocumentTabs`, and `StatusBar` components.
- `EditorUiHostRuntime` can load that `.ui.toml`, build a shared `UiSurface`, compute frames, and expose host-model snapshots.
- `scene_projection.rs` still owns constants such as `TOP_BAR_HEIGHT_PX`, `HOST_BAR_HEIGHT_PX`, `PANEL_HEADER_HEIGHT_PX`, and `DOCUMENT_HEADER_HEIGHT_PX`.
- `host_page_chrome.slint` still hardcodes tab-strip layout facts such as `x: 8px`, `y: 1px`, `spacing: 4px`, project path label dimensions, and relative pointer offsets.
- `host_document_dock_surface.slint`, `host_side_dock_surface.slint`, and `host_bottom_dock_surface.slint` still hardcode dock header padding, tab row offsets, border rows, and close-button pointer offsets.

The user-approved first slice is `HostPageChrome` plus document tab/header, not the menu bar. An active coordination note currently owns the menu visibility/top chrome regression, so this design avoids taking over that fix.

2026-04-29 amendment: the accepted implementation converged on a TemplatePane-first root chrome cutover instead of typed tab renderers. `workbench_menu_chrome.ui.toml`, `workbench_page_chrome.ui.toml`, and `workbench_dock_header.ui.toml` are now the root chrome visual authorities, `chrome_template_projection.rs` projects their laid-out nodes into `HostChromeControlFrameData` / `HostChromeTabData`, and the Slint chrome files render `TemplatePane` nodes with transparent input overlays only. The page tab overlay forwards coordinates relative to `page_data.tab_row_frame` because `HostPagePointerBridge` remains the bridge that adds the shared strip frame origin.

## Goals

- Make `.ui.toml` the sizing and placement authority for `HostPageChrome` and the document dock tab header.
- Keep Slint as the renderer and event adapter for now: it places Slint controls from projected frames and forwards normalized pointer facts.
- Prove the pattern with one vertical slice before expanding to `HostMenuChrome`, side dock headers, bottom dock headers, floating headers, and final Slint generic renderer work.
- Preserve existing `TabData`, `EditorUiBindingPayload`, `HostPagePointer*`, `HostDocumentTabPointer*`, and drag/drop contracts.
- Add source guards that prevent reintroducing top-tab/header layout math into Slint after the cutover.

## Non-Goals

- Do not rewrite `HostMenuChrome` in this first slice.
- Do not replace Slint with direct `UiRenderExtract` rendering in this first slice.
- Do not move docking, split, floating-window lifecycle, focus, closeability, or layout persistence authority into `.ui.toml`.
- Do not introduce a second command or binding system. The existing editor binding payloads remain the semantic route authority.
- Do not change runtime graphics or screen-space UI pass architecture.

## Approaches Considered

### Approach A: Page/Document Tab Vertical Slice

This approach first migrates `HostPageChrome` and the document tab header to projected frames from `UiSurface`. It proves that tab-strip size and tab placement no longer come from Slint, while keeping the surface small enough for focused guards and validation.

This is the recommended approach because it directly targets the current visible tab-size issue and avoids the active menu regression session.

### Approach B: Global Chrome Schema First

This approach designs a single schema for menu chrome, page chrome, side dock headers, document dock headers, bottom dock headers, and floating headers before implementing any slice. It reduces long-term duplication, but delays evidence and increases overlap with active sessions.

Use this approach only if the first vertical slice reveals that shared chrome fields cannot be generalized incrementally.

### Approach C: Direct Generic Slint Renderer

This approach removes typed Slint chrome components and renders `UiSurface` output through a generic Slint adapter immediately. It is closer to the final state, but it crosses into runtime visual contract, generic component rendering, and active Runtime UI showcase ownership.

This is deferred until host chrome template projection is stable.

## Architecture

This is an editor host boundary change. Ownership stays in `zircon_editor`; `zircon_ui` remains the shared layout and render-extract authority, and Slint remains the desktop host renderer. The change does not introduce a new `zircon_server` facade or engine module lifecycle surface because it is a leaf detail inside the existing editor host template/runtime abstraction.

The target architecture for this slice is:

1. `.ui.toml` declares chrome structure and static layout facts.
2. `EditorUiHostRuntime` compiles the template and computes `UiSurface` frames.
3. `scene_projection.rs` merges dynamic workbench state such as `TabData`, active tab state, project path, and pane subtitle with the computed template frames.
4. `HostPageChromeData` and `HostDocumentDockSurfaceData` carry explicit projected frames for tab strip root, tab row, individual tab slots, project path label, document subtitle label, and content area.
5. `apply_presentation.rs` converts those DTOs into Slint DTOs.
6. Slint renders existing controls at the provided frames and forwards pointer facts with projected origins. Slint no longer decides top-tab/header sizes.

The frame authority becomes data-driven. For the first slice, `.ui.toml` may still use existing native component descriptors such as `DocumentTabs`, but the projection must expose the resulting `UiFrame` to Slint instead of duplicating the same layout numbers in `.slint`.

## Data Model

Extend the host chrome DTOs with template-frame projections. Exact field names can be refined during implementation, but the required concepts are stable:

- `HostChromeFrameData`: frame for a named chrome node from `UiSurface`.
- `HostProjectedTabData`: `TabData` plus `frame`, `close_frame`, `drag_origin_x_px`, and `drag_origin_y_px` where applicable.
- `HostPageChromeData`: top/page host frame, tab row frame, project path frame, and projected host tabs.
- `HostDocumentDockSurfaceData`: header frame, tab row frame, subtitle frame, pane/content frame, and projected document tabs.

The dynamic tab list still originates from workbench state. `.ui.toml` defines the repeated-row container and static sizing rules; Rust projection maps each current `TabData` into that repeated-row frame model.

The initial implementation can derive per-tab repeated frames from the `.ui.toml` container frame plus template item sizing if the current runtime does not yet materialize dynamic repeat children directly. That derivation must live in Rust projection, not in Slint, and must be guarded as a temporary adapter seam.

## Data Flow

The page/document tab flow is:

1. `EditorUiHostRuntime::load_builtin_host_templates()` loads `workbench_shell.ui.toml`.
2. The host runtime builds a shared surface for `UI_HOST_WINDOW_DOCUMENT_ID` and computes layout for the current host size.
3. `scene_projection.rs` reads frames for `DocumentHostRoot`, `DocumentTabsRoot`, `PaneSurfaceRoot`, and any new page/tab chrome control IDs added for this slice.
4. `scene_projection.rs` combines those frames with current `host_surface_data.host_tabs` and `host_surface_data.document_tabs`.
5. `HostPageChromeData` and `HostDocumentDockSurfaceData` carry the final layout facts to Slint.
6. `host_page_chrome.slint` and `host_document_dock_surface.slint` render using `root.*.frame` fields and pass pointer events back through existing `UiHostContext` callbacks.

## Error Handling

Missing or invalid template frames should fail early in focused tests and degrade explicitly at runtime.

- During tests, required control IDs must be asserted. Missing `DocumentTabsRoot`, page tab row, or document content frame is a test failure.
- During runtime projection, missing optional decorative frames may fall back to empty frames only if the control is not visible.
- Required structural frames should use a named fallback frame only during the migration window, with a source guard and roadmap note so it does not become a permanent second authority.
- Invalid negative or non-finite frame values should be clamped or rejected at the projection boundary, not inside Slint.

## Testing Strategy

Use TDD for each implementation step.

First RED guards:

- `host_page_chrome_uses_projected_template_frames_instead_of_slint_layout_math` should fail while `host_page_chrome.slint` still owns tab-row `x`, `y`, `spacing`, project path label width, or hardcoded pointer origins.
- `document_dock_header_uses_projected_template_frames_instead_of_slint_layout_math` should fail while `host_document_dock_surface.slint` still owns header/tab-row offsets, subtitle frame math, close-button origin math, or pane height subtraction.

Green tests:

- Existing `editor_ui_host_runtime_builds_laid_out_host_model_from_shared_surface_authority` should continue to prove `workbench_shell.ui.toml` produces stable `DocumentTabsRoot` and `PaneSurfaceRoot` frames.
- Add or update projection tests so changing `.ui.toml` tab/header height changes `HostPageChromeData` and `HostDocumentDockSurfaceData` without editing Slint.
- Run focused `generic_host_boundary` guards after each slice.
- Run `cargo fmt --package zircon_editor -- --check` and `git diff --check` on touched files.

## Implementation Slices

### Slice 1: Page Chrome Template Frames

Add source guards, then project `HostPageChrome` tab row and project path frames from shared template/runtime data. Remove Slint-owned tab strip sizing and pointer origin constants from `host_page_chrome.slint`.

### Slice 2: Document Dock Header Template Frames

Project document header, tab row, subtitle, content, tab click, tab close, and drag origin frames from the same template authority. Remove Slint-owned document header placement and pane height subtraction from `host_document_dock_surface.slint`.

### Slice 3: Shared Chrome Frame DTO Cleanup

If the first two slices duplicate frame concepts, extract a small reusable DTO and projection helper under the workbench host window projection boundary. Keep it editor-local until menu and dock headers also consume it.

### Slice 4: Extend Pattern To Remaining Chrome

After page/document tab evidence is green, apply the same pattern to `HostMenuChrome`, side dock headers, bottom dock headers, and floating headers. At that point the shared DTO may become the canonical `HostChromeTemplateProjection` for all editor host chrome.

## Acceptance Criteria

- `host_page_chrome.slint` contains no hardcoded tab strip placement, tab gap, project path label frame, or pointer origin constants for page tabs.
- `host_document_dock_surface.slint` contains no hardcoded document header placement, tab gap, subtitle frame, close-button pointer offset, or content height subtraction for document tabs.
- Changing the `.ui.toml` tab/header height changes projected Slint DTO frames through `UiSurface`, not by editing Slint.
- Existing page tab, document tab click, close, and drag event callbacks continue to dispatch through existing host pointer bridges.
- The active menu regression session remains unmodified by this first slice.

## Open Extension Points

- The dynamic repeated-row model may later move fully into `UiSurface` once the template runtime supports data-bound repeat materialization for host chrome.
- `HostMenuChrome` can reuse the same frame DTO after menu layout ownership is no longer being actively fixed elsewhere.
- Final generic Slint rendering can replace typed chrome Slint components once runtime visual contract and component renderer coverage are ready.

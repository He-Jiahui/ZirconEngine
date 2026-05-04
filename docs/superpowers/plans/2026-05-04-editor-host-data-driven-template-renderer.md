# Editor Host Data-Driven Template Renderer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the exported editor's skeletal native host paint path with a two-stage renderer: first a data-driven CPU workbench renderer, then an authored template/render-extract renderer that consumes existing `.ui.toml -> UiSurface -> host projection` data.

**Architecture:** Keep the Rust-owned `UiHostWindow` and `softbuffer` native presenter. Milestone 1 keeps rendering inside `zircon_editor::ui::slint_host::host_contract` and consumes `HostWindowPresentationData.host_scene_data` plus pane/tab/template-node DTOs. Milestone 2 introduces a reusable host-contract render command adapter/backend so `TemplatePaneNodeData` and runtime-style `UiRenderCommand` data share the same primitive drawing path.

**Tech Stack:** Rust 2021, `slint::ModelRc` DTO traversal, `softbuffer` CPU presentation, existing `zircon_runtime_interface::ui::surface` render command contracts, existing `zircon_editor` host-contract snapshot tests, existing `tools/zircon_build.py` export path.

---

## Repository Baseline

- Current checkout policy: work directly in the existing `main` checkout; do not create worktrees or feature branches.
- Current exported editor state: `UiHostWindow::run()` owns a native `winit` event loop and `SoftbufferHostPresenter` writes a CPU-painted frame, but `painter.rs` draws only top-level shell blocks and marker bars.
- Current evidence: exported logs show valid asset lookup, template loading, presentation data, native window creation, and presenter frames. The remaining display gap is render consumption of authored scene data.
- Current data already available to the native host: `HostWindowPresentationData.host_scene_data` contains menu chrome, page chrome, status bar, left/right/document/bottom dock surfaces, floating windows, pane body nodes, tab frames, and authored `TemplatePaneNodeData` lists.
- Current lower shared runtime path already available: `UiRenderExtract` contains `UiRenderCommand { kind, frame, clip_frame, z_index, style, text_layout, text, image, opacity }` and runtime UI extraction tests cover visual contract fields.

## File Structure

- Modify `zircon_editor/src/ui/slint_host/host_contract/mod.rs`: route the existing `painter` file through a folder-backed renderer module if implementation growth requires it.
- Replace `zircon_editor/src/ui/slint_host/host_contract/painter.rs` with a narrow orchestration module or keep it as the public paint entry while moving behavior to child modules.
- Create `zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs`: own `HostRgbaFrame` and byte accessors.
- Create `zircon_editor/src/ui/slint_host/host_contract/painter/geometry.rs`: own visible-frame checks, pixel clipping, frame translation, and intersection helpers.
- Create `zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs`: own filled rect, border, separator, text-bar marker, and primitive draw helpers.
- Create `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs`: consume `HostWindowPresentationData.host_scene_data` and draw data-driven menu/page/status/dock/floating/pane surfaces.
- Create `zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs`: draw `TemplatePaneNodeData` collections with relative origins, role/style variants, labels, buttons, panels, selected/disabled states, clipping, and deterministic fallback text bars.
- Create `zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs` during Milestone 2: draw runtime-style render commands through the same primitive backend.
- Modify `zircon_editor/src/ui/slint_host/host_contract/presenter.rs`: keep consuming `paint_host_frame`; only adjust imports if the painter becomes folder-backed.
- Modify `zircon_editor/src/ui/slint_host/host_contract/window.rs`: keep `take_snapshot()` and native redraw using the same paint entry; only adjust imports if the painter becomes folder-backed.
- Modify `zircon_editor/src/tests/host/slint_window/shell_window.rs`: add focused snapshot regressions for data-driven scene DTO consumption and later template-renderer consumption.
- Modify `docs/editor-and-tooling/editor-workbench-shell.md`: document the two-stage native host renderer, source data, tests, and acceptance evidence.
- Update `.codex/sessions/20260504-1816-editor-authored-template-renderer.md`: record plan path, milestone state, and validation evidence.

## Architecture Note

- Owner crate and role: `zircon_editor` owns the exported editor native host and authoring-shell presentation. `zircon_runtime_interface` remains the neutral render command contract owner. No new root crate, manager facade, service registry, or runtime-world authority is introduced.
- Boundary depth: Milestone 1 is a leaf renderer inside the existing Rust-owned host-contract abstraction; it does not change public workspace architecture. Milestone 2 adds an adapter/backend seam inside the host-contract renderer so future authored template nodes and runtime-style render commands reuse primitives instead of adding one-off workbench branches.
- Reference alignment: Slint provides item-rendering traversal precedent, Fyrox provides Rust editor/runtime UI renderer shape, and Bevy provides extract/list/render separation. The local landing zone remains the current `zircon_editor` host-contract data plus `zircon_runtime_interface::ui::surface` command model.
- Lower-layer invalidators: if M2 fails, check template node projection, `UiRenderExtract` command shape, style resolution, clip/intersection, and primitive drawing before special-casing workbench panes.

## Milestone 1: Data-Driven CPU Workbench Renderer

### Goal

Make the exported editor native host visibly consume populated host scene data instead of drawing only skeletal layout blocks.

### In-Scope Behaviors

- Draw root shell background, top menu bar, host page bar, center band, status bar, and visible dock regions from `HostWindowPresentationData.host_scene_data` first, with current `host_layout` fallback only when scene frames are missing.
- Draw menu chrome from `HostMenuChromeData.template_nodes` and `menus` labels.
- Draw host page chrome from `HostPageChromeData.template_nodes`, `tab_frames`, `tabs`, and `project_path`.
- Draw left/right dock rails from `HostSideDockSurfaceData.rail_nodes`, `rail_button_frames`, active rail control id, and tabs.
- Draw side/document/bottom dock headers from `header_nodes`, `tab_frames`, `tabs`, subtitles, and active tab state.
- Draw document viewport area using `document_dock.content_frame`, `document_dock.pane.kind`, and `document_dock.pane.viewport` toolbar labels when the pane is `Scene` or `Game`.
- Draw pane body previews for Hierarchy, Inspector, Console, Assets, Asset Browser, Project Overview, Build Export, Module Plugins, UI Asset, Animation, and empty panes by using their existing `TemplatePaneNodeData` lists or pane labels.
- Draw floating windows from `HostFloatingWindowLayerData.floating_windows`, including frame, header nodes, tab frames, and active pane body.
- Preserve zero-size safety: a `0x0` frame returns empty bytes and invalid/negative/non-finite child frames are skipped.
- Preserve snapshot determinism: tests sample colors and structural differences rather than using OS font rendering or image assets.

### Dependencies

- Existing native run loop and `softbuffer` presenter.
- Existing `HostWindowPresentationData` DTOs and `slint::ModelRc` traversal.
- Existing template-node projection into host contract data.
- Existing snapshot harness in `HostWindowHandle::take_snapshot()`.

### Implementation Slices

- [x] Convert `painter.rs` into a folder-backed module before adding data-driven behavior, keeping `paint_host_frame(width, height, presentation)` and `HostRgbaFrame` as the only import surface needed by `window.rs` and `presenter.rs`.
- [x] Move RGBA frame storage and accessors to `painter/frame.rs` without changing behavior.
- [x] Move pixel clipping, visible-frame checks, and frame translation helpers to `painter/geometry.rs`.
- [x] Move filled rect, border, separator, and deterministic text-bar drawing to `painter/primitives.rs`.
- [x] Add `painter/template_nodes.rs` that draws `TemplatePaneNodeData` lists relative to a supplied origin and clip frame, mapping roles/variants to stable colors and drawing text bars from node text/control id.
- [x] Add `painter/workbench.rs` that draws scene-level surfaces in z-order: shell background, menu chrome, page chrome, center band/docks, pane headers, pane bodies, floating windows, resize/splitter overlays, status bar.
- [x] Keep the old skeletal fallback in the workbench renderer only for missing scene data so tests with minimal presentation remain valid.
- [x] Add snapshot regression `rust_owned_host_window_snapshot_consumes_host_scene_data` that builds a `HostWindowPresentationData` with menu/page/status/dock/template nodes and proves sampled pixels differ at menu, page tab, activity rail, dock header, viewport, status, and floating header locations.
- [x] Add snapshot regression `rust_owned_host_window_snapshot_reflects_pane_template_nodes` that injects visible template nodes into a pane body and proves the pane body changes relative to an otherwise identical empty pane.
- [x] Update docs and session note with Milestone 1 implementation state before entering the testing stage.

### Testing Stage: M1 Renderer Snapshot And Export Check

- [x] Run `rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/window.rs zircon_editor/src/ui/slint_host/host_contract/presenter.rs zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs zircon_editor/src/ui/slint_host/host_contract/painter/geometry.rs zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs zircon_editor/src/tests/host/slint_window/shell_window.rs`.
- [x] Run `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_consumes_host_scene_data --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_reflects_pane_template_nodes --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_contains_editor_chrome_pixels --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m1 --message-format short --color never`.
- [x] Run `cargo test -p zircon_editor --lib rust_owned_host_window_run_uses_native_event_loop --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m1 --message-format short --color never`.
- [x] Run `cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --target-dir E:\zircon-build\targets\editor-renderer-m1-app --locked --jobs 1 --message-format short --color never`.
- [x] Run `python tools\zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug`.
- [x] Smoke-run `E:\zircon-build\ZirconEngine\zircon_editor.exe` long enough to confirm it stays alive and writes `editor_host_presenter` records with populated scene data.
- [x] Run `git diff --check -- zircon_editor/src/ui/slint_host/host_contract zircon_editor/src/tests/host/slint_window/shell_window.rs docs/editor-and-tooling/editor-workbench-shell.md docs/superpowers/plans/2026-05-04-editor-host-data-driven-template-renderer.md .codex/sessions/20260504-1816-editor-authored-template-renderer.md`.
- [x] Debug/correction loop: if an upper-layer snapshot fails, inspect lower support in this order: `ModelRc` traversal, frame visibility/clipping, origin translation, primitive drawing, template-node style mapping, then workbench surface ordering.

### Exit Evidence

- Focused snapshot tests prove scene DTOs and pane template nodes affect native host pixels.
- Existing run-loop and non-blank snapshot regressions still pass.
- Editor binary check and export build pass with existing warnings recorded rather than hidden.
- Exported editor remains alive in smoke and logs presenter frames.
- Docs and session note record M1 evidence and remaining M2 scope.

## Milestone 2: Authored Template Render-Extract Renderer

### Goal

Make authored template visual data the normal native host rendering path by adapting `TemplatePaneNodeData` and runtime-style `UiRenderCommand` data into one shared CPU primitive backend.

### In-Scope Behaviors

- Convert `TemplatePaneNodeData` into host render commands or command-like draw records that preserve frame, text, role, style hints, corner radius, border width, selected/hovered/pressed/disabled states, clip, opacity defaults, and z-order from traversal order.
- Draw `UiRenderCommandKind::Quad`, `Text`, `Image`, and `Group` through `painter/render_commands.rs` without depending on OS fonts or GPU resources.
- Map `UiResolvedStyle.background_color`, `foreground_color`, `border_color`, `border_width`, `corner_radius`, `text_align`, and `opacity` into deterministic CPU drawing.
- Treat images/icons as deterministic placeholders keyed by `UiVisualAssetRef`; do not load image files or introduce new asset lifetime rules in this milestone.
- Preserve clipping: child commands and template nodes must not paint outside their surface clip/content frame.
- Preserve ordering: lower z-index and earlier list commands paint before higher z-index or later commands.
- Preserve unsupported data behavior: unknown roles, malformed colors, non-finite frames, and zero opacity are skipped or drawn with explicit fallback colors without panics.
- Keep `workbench.rs` as an orchestration layer that collects surface origins and feeds template/render-command draw lists rather than hand-authoring each control's visuals.

### Dependencies

- Milestone 1 primitive backend, geometry clipping, and scene surface traversal.
- Existing `zircon_runtime_interface::ui::surface` render command contracts.
- Existing `.ui.toml -> UiSurface -> UiRenderExtract` projection evidence in `zircon_runtime` and editor view projection code.

### Implementation Slices

- [x] Add `painter/render_commands.rs` with a narrow function that draws an ordered slice of command-like records using existing primitives and clip helpers.
- [x] Add style color parsing for `#rgb`, `#rgba`, `#rrggbb`, and `#rrggbbaa` strings; invalid color strings fall back to stable role colors and are covered by tests.
- [x] Add a `TemplatePaneNodeData -> HostPaintCommand` adapter in `painter/template_nodes.rs`, keeping node traversal and style mapping out of `workbench.rs`.
- [x] Route menu/page/status/dock header/rail/pane body node drawing through the command backend instead of direct template-node ad hoc drawing.
- [x] Add explicit z-order ordering for mixed surface primitives: shell/chrome backgrounds, template node commands, pane content placeholders, floating windows, resize/drag overlays, status bar.
- [x] Add focused regression `rust_owned_host_window_snapshot_renders_template_node_styles` covering panel, label, button, selected, disabled, border, and clipped child cases.
- [x] Add focused regression `rust_owned_host_window_snapshot_respects_template_node_order_and_clip` covering overlapping nodes, z/order by traversal, and clipping to a dock/pane body.
- [x] Add focused regression `rust_owned_host_painter_draws_runtime_render_commands` that constructs `UiRenderCommand` values directly and proves quad/text/image placeholders paint through the backend.
- [x] Update docs and session note with Milestone 2 implementation state before entering the testing stage.

### Testing Stage: M2 Template Renderer And Export Acceptance

- [x] Run `rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs zircon_editor/src/ui/slint_host/host_contract/painter/geometry.rs zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs zircon_editor/src/tests/host/slint_window/shell_window.rs`.
- [x] Run `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_renders_template_node_styles --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m2 --message-format short --color never`.
- [x] Run `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_respects_template_node_order_and_clip --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m2 --message-format short --color never`.
- [x] Run `cargo test -p zircon_editor --lib rust_owned_host_painter_draws_runtime_render_commands --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m2 --message-format short --color never`.
- [x] Re-run the M1 snapshot regressions in the same target dir.
- [x] Run `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m2-runtime --message-format short --color never -- --test-threads=1 --nocapture` if runtime render command contracts are touched or imported behavior changes. Not required for this slice because runtime contracts were consumed but not changed.
- [x] Run `cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --target-dir E:\zircon-build\targets\editor-renderer-m2-app --locked --jobs 1 --message-format short --color never`.
- [x] Run `python tools\zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug`.
- [x] Smoke-run `E:\zircon-build\ZirconEngine\zircon_editor.exe` long enough to confirm it stays alive, writes presenter records, and no asset/template diagnostics report missing built-in assets.
- [x] Run `git diff --check -- zircon_editor/src/ui/slint_host/host_contract zircon_editor/src/tests/host/slint_window/shell_window.rs docs/editor-and-tooling/editor-workbench-shell.md docs/superpowers/plans/2026-05-04-editor-host-data-driven-template-renderer.md .codex/sessions/20260504-1816-editor-authored-template-renderer.md`.
- [x] Debug/correction loop: if command rendering fails, validate lower layers in this order: color parsing, clip intersection, z/order sorting, primitive fill/border drawing, text placeholder drawing, template-node adapter, then workbench surface collection.

### Exit Evidence

- Template-node style, order, and clip snapshot tests pass.
- Runtime-style render command backend test passes.
- M1 data-driven scene snapshot tests remain green through the new backend.
- Editor binary check, export build, and smoke evidence pass with any existing warnings recorded.
- Docs describe the renderer path as authored template/render-command driven, with skeletal fallback only for missing data.

## Final Acceptance Evidence

- The exported editor no longer relies on a skeletal CPU chrome-only painter for normal visibility.
- `HostWindowPresentationData.host_scene_data` is the renderer's primary source for editor shell surfaces.
- Authored `.ui.toml` template projections and runtime-style render commands share a primitive CPU backend in the native host path.
- Generated Slint modules remain deleted and are not reintroduced.
- No compatibility alias, feature branch, or worktree is created.

## Self-Review

- Spec coverage: two-stage order, current baseline, file ownership, renderer boundaries, tests, export smoke, docs, and session coordination are covered.
- Placeholder scan: no `TBD`, `TODO`, or unspecified validation commands remain.
- Type consistency: plan uses existing `HostWindowPresentationData`, `HostWindowSceneData`, `TemplatePaneNodeData`, `UiRenderCommand`, `UiRenderExtract`, `UiResolvedStyle`, and `UiRenderCommandKind` names.
- Scope check: graphics/runtime GPU UI pass, component drawer, particles, virtual geometry, HGI, plugin lanes, and final `.slint` business tree deletion are intentionally out of scope for this two-stage native host renderer milestone.

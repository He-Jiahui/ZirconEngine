# Hub Editor SVG Resources Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate and wire SVG image resources for the Hub and Editor reference visuals.

**Architecture:** SVG files are repository-owned static assets. Hub uses Slint `Image` surfaces plus a small Rust `view_model::media` child module to load bundled icons and fallback covers, while Editor receives unreferenced SVG preview and shell icon assets for future retained-host/template consumption.

**Tech Stack:** Rust, Slint, SVG, Cargo.

---

## File Structure

- Create `zircon_hub/assets/brand/zircon-mark.svg` for the Hub title-bar and project badge mark.
- Create `zircon_hub/assets/icons/nav/*.svg` for the Hub navigation rail.
- Create `zircon_hub/assets/icons/actions/*.svg` for Quick Actions.
- Create `zircon_hub/assets/icons/status/*.svg` for title-bar status pills.
- Create `zircon_hub/assets/covers/project-*.svg` for bundled project cover fallbacks.
- Create `zircon_editor/assets/preview/editor-scifi-room.svg` for the Editor screenshot-style preview resource.
- Create `zircon_editor/assets/icons/zircon_editor_shell/**` for screenshot-derived Editor toolbar, activity rail, scene hierarchy, inspector, controls, status, and viewport icons.
- Modify `zircon_hub/ui/shared.slint` to expose image fields and render image icons.
- Modify `zircon_hub/ui/projects.slint` to render bundled badge/action icons when present.
- Modify `zircon_hub/src/app/view_model.rs` only to delegate media loading to a child module.
- Create `zircon_hub/src/app/view_model/media.rs` to own bundled SVG paths and `slint::Image` loading.
- Update `docs/zircon_hub/index.md` so the Hub docs mention the static SVG resource pack and fallback cover flow.

## Milestone 1: Asset Pack And Hub Wiring

### Implementation Slices

- [x] Create the Hub SVG asset directories and files listed in the design spec. Keep SVGs ASCII, self-contained, and free of external references.
- [x] Create `zircon_editor/assets/preview/editor-scifi-room.svg` as an unreferenced preview resource.
- [x] Update Slint shared structs: add `icon-image: image` to `NavItemData`, `QuickActionData`, and `HeaderStatusData`.
- [x] Update `BrandMark`, `NavButton`, `StatusPill`, and `QuickActionButton` rendering to prefer SVG `Image` content while preserving current text fallback fields.
- [x] Add `zircon_hub/src/app/view_model/media.rs` with functions for nav icon, action icon, status icon, project fallback cover, and project cover resolution.
- [x] Update `zircon_hub/src/app/view_model.rs` to call `media` for icon images and project covers without adding long media helper sections to the root view-model file.
- [x] Update `docs/zircon_hub/index.md` headers and body to document the SVG resource pack, fallback behavior, and validation commands.

### Testing Stage: Hub SVG Resource Validation

- [x] Run `cargo fmt -p zircon_hub --check`.
- [x] Run `cargo check -p zircon_hub --locked`.
- [x] If either command fails, fix the lowest owning layer first: Slint syntax/data shape, Rust generated bindings/type errors, then formatting.
- [x] Record final command results in the closeout response.

## Self-Review

- Spec coverage: covers Hub assets, Editor preview asset, Slint data fields, Rust media module, project cover fallback, docs, and validation.
- Placeholder scan: no `TBD`, `TODO`, or unspecified implementation steps remain.
- Type consistency: Slint data fields are named `icon-image`; Rust generated fields should map to `icon_image`.

## Milestone 2: Editor Shell SVG Resource Expansion

### Implementation Slices

- [x] Create `zircon_editor/assets/icons/zircon_editor_shell/toolbar` icons for top command-bar actions and transform tools.
- [x] Create `zircon_editor/assets/icons/zircon_editor_shell/activity` icons for the left activity rail.
- [x] Create `zircon_editor/assets/icons/zircon_editor_shell/scene` icons for hierarchy rows and row controls.
- [x] Create `zircon_editor/assets/icons/zircon_editor_shell/inspector` icons for property panel sections and controls.
- [x] Create `zircon_editor/assets/icons/zircon_editor_shell/controls` icons for UI component lab controls.
- [x] Create `zircon_editor/assets/icons/zircon_editor_shell/status` icons for alert/status rows.
- [x] Create `zircon_editor/assets/icons/zircon_editor_shell/viewport` icons for viewport overlays and status-bar controls.
- [x] Document the icon pack in `docs/zircon_editor/assets/editor-shell-svg-resources.md`.

### Testing Stage: Editor SVG Resource Inventory

- [x] Count the generated SVG files under `zircon_editor/assets/icons/zircon_editor_shell`.
- [x] Check the generated SVG files contain no external `href` or `url(http...)` references.
- [x] Record final inventory results in the closeout response.

## Milestone 3: Reference-Informed Engine Style Icon Expansion

### Implementation Slices

- [x] Inspect the reference icon/resource vocabularies in `dev/Fyrox/editor/resources`, `dev/godot/scene/theme/icons`, and Unreal Engine resource folders.
- [x] Generate `zircon_editor/assets/icons/zircon_engine_style/assets` icons for common asset browser/resource types.
- [x] Generate `zircon_editor/assets/icons/zircon_engine_style/scene` icons for hierarchy, scene object, and component concepts.
- [x] Generate `zircon_editor/assets/icons/zircon_engine_style/graph` icons for visual scripting, behavior, and shader graph concepts.
- [x] Generate `zircon_editor/assets/icons/zircon_engine_style/tools` icons for editor transform/selection helper tools.
- [x] Generate `zircon_editor/assets/icons/zircon_engine_style/runtime` icons for preview, simulation, profiling, and diagnostics controls.
- [x] Generate `zircon_editor/assets/icons/zircon_engine_style/build` icons for plugin/module/package/deploy/build pipeline concepts.
- [x] Document the reference routing, style rules, and generated icon categories in `docs/zircon_editor/assets/engine-style-svg-resources.md`.

### Testing Stage: Engine Style SVG Resource Inventory

- [x] Count the generated SVG files under `zircon_editor/assets/icons/zircon_engine_style`.
- [x] Check the generated SVG files contain no external `href`, raster `<image>`, or `url(http...)` references.
- [x] Record final inventory results in the closeout response.

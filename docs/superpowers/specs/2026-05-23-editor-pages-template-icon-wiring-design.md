---
related_code:
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml
  - zircon_editor/assets/icons/editor_pages/**
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
implementation_files:
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/performance_timeline_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
plan_sources:
  - user: 2026-05-23 continue by wiring production Editor templates to editor_pages icons
  - user: 2026-05-25 complete live Editor visual rendering and 16px readability validation for wired editor_pages icons
  - docs/superpowers/specs/2026-05-21-editor-svg-polish-ui-mapping-design.md
  - docs/superpowers/plans/2026-05-21-editor-svg-polish-ui-mapping.md
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
tests:
  - production template parse/project checks for edited v2 UI TOML assets
  - mapped editor_pages path existence checks for all wired icon/value props
  - gap-row preservation scan for controls without current editor_pages coverage
  - editor_pages asset inventory remains 204 SVG files with A 61, B 54, C 89
  - retained-host 16px readability test: cargo test -p zircon_editor editor_pages_template_icons_have_readable_16px_raster_footprints --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture
  - live editor build and screenshot: cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation; target/visual-layout/editor-live-window-900x620.png
doc_type: milestone-detail
---

# Editor Pages Template Icon Wiring Design

## Goal

Wire current production Editor `*.v2.ui.toml` template icon usages to the already validated `zircon_editor/assets/icons/editor_pages` SVG pack where the mapping document has an accepted `direct` or `near` recommendation. This turns the prior advisory map into real template asset references without changing Editor layout, routes, retained-host behavior, Rust registries, atlas logic, or the `editor_pages` icon inventory.

## Scope

The wiring pass covers only production Editor templates that were inventoried in `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`. It updates `Icon` and `IconButton` props in those templates when the mapping confidence is `direct` or `near`.

Rows marked `gap` remain unchanged. Those controls still use their current generic or Ionicons-style icon names until a future icon-coverage pass creates specific assets or a UI-design pass changes their role.

Deferred demo, showcase, and Material component lab surfaces remain unchanged. They are not production shell chrome for this pass.

## Architecture Boundary

This is an editor-authoring asset wiring change. Ownership stays in `zircon_editor` because the affected templates and `editor_pages` icons describe Editor shell and pane controls. Runtime contracts, shared UI DTOs, retained-host Rust projection code, icon atlas code, and asset loader behavior remain untouched unless validation proves that the existing resolver cannot load `editor_pages` paths from template metadata.

The planned icon string form is repository-asset relative to the editor icon root, for example `editor_pages/workbench/menu/open-project.svg`. The retained-host image resolver already searches under `zircon_editor/assets/icons` for icon names, so this form avoids absolute paths and does not require a new resolver branch.

## Wiring Rules

- Preserve every existing `control_id`, component name, layout, event route, label, class list, and visual style prop unless the prop is the icon path being wired.
- For `IconButton` nodes, replace only the `icon` prop for `direct` and `near` mappings.
- For static `Icon` nodes that currently include both `icon` and `value`, set both to the same `editor_pages/...svg` path when the row is wired. This keeps retained-host and runtime metadata paths aligned for those explicit static icons.
- Do not wire `gap` rows: `SetTransformSpace`, `SetPreviewSkybox`, `AlignView`, `SelectItem`, `SetViewMode`, `TriggerAction`, and `CreateProject` stay on their current icon names.
- Do not add, remove, rename, or rewrite SVG files during this pass.
- Do not change Rust dynamic chrome projection paths such as menu, page-tab, or dock-tab runtime-generated icons in this milestone.

## Expected Wired Coverage

The implementation should wire 39 production template usages: 22 `direct` rows and 17 `near` rows. The 7 `gap` rows remain deferred and should be documented as intentionally unwired.

The mapping document should be updated from purely advisory language to record the current wiring state: `direct` and `near` rows are wired in templates, while `gap` rows are still blockers for complete migration.

## Validation

Acceptance requires evidence for these checks:

- Edited production templates still parse and project through existing Editor UI template tests.
- Every wired `editor_pages/...svg` path exists under `zircon_editor/assets/icons`.
- The seven `gap` controls remain unchanged from their prior icon names.
- The `editor_pages` inventory remains exactly 204 SVG files, with A 61, B 54, and C 89.
- SVG contract scans still report no forbidden references and no non-ASCII content.
- No Rust source, atlas code, icon registry, `.zui` demo, or deferred demo `.v2.ui.toml` file is modified by this wiring pass.

## Visual Validation Addendum

The follow-up visual pass validates the accepted wiring without changing the original template-wiring scope. `editor_pages_template_icons_have_readable_16px_raster_footprints` renders the unique wired `editor_pages` icon paths through the retained-host painter at 16 x 16 px and rejects blank, collapsed, or full-slot silhouettes. The retained-host screenshot gate also covers small and large SVG icon scaling artifacts under `target/visual-layout`.

Live Editor validation required a real `zircon_editor` binary. The app/runtime fixes stay within existing ownership: `zircon_runtime/src/lib.rs` exposes the manifest-specific runtime-profile helper APIs from the crate root, and `zircon_app/src/entry/builtin_modules.rs` keeps the optional project manifest available for the feature-registration resolver path. These changes support live validation and provider-aware profile bootstrap; they do not add icon registry behavior, atlas behavior, or new SVG assets.

## Residual Risks

The follow-up visual pass proves retained-host 16 px raster readability for the currently wired icon set and captures a live Editor smoke screenshot. It still does not prove that every icon is the final art direction or that every future theme/background keeps the same contrast; those remain normal visual-design review concerns rather than blockers for this wiring pass.

---
related_code:
  - zircon_editor/assets/icons/editor_pages/**
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
  - docs/zircon_editor/assets/icon-resource-audit.md
implementation_files:
  - zircon_editor/assets/icons/editor_pages/**
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
plan_sources:
  - user: 2026-05-21 continue improving Editor SVGs; choose A + C: polish existing icons, then prepare UI mapping
  - docs/superpowers/specs/2026-05-20-editor-page-function-svg-icons-design.md
  - docs/superpowers/plans/2026-05-20-editor-page-function-svg-icons.md
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
tests:
  - visual/semantic audit checklist for 204 SVG files under zircon_editor/assets/icons/editor_pages
  - asset inventory: 204 SVG files under zircon_editor/assets/icons/editor_pages after polish
  - external-reference scan: no href/image/remote-url references under zircon_editor/assets/icons/editor_pages
  - ASCII scan: no non-ASCII content under zircon_editor/assets/icons/editor_pages
  - UI mapping inventory: production Editor template icon references mapped or marked as deferred gaps
doc_type: milestone-detail
---

# Editor SVG Polish And UI Mapping Design

## Goal

Improve the existing `editor_pages` SVG pack before any UI integration, then prepare a documented mapping from current Editor template icon usage to the new page/function icon paths. This work keeps UI behavior unchanged: no `*.v2.ui.toml`, `.zui`, Rust icon registry, atlas generation, or retained-host runtime code is modified.

## Scope

This pass includes two approved tracks:

- **A. Visual polish:** audit and selectively improve the 204 existing SVGs under `zircon_editor/assets/icons/editor_pages`.
- **C. UI mapping preparation:** document how current Editor template icon roles should map to `editor_pages` paths, including gaps where no precise replacement exists yet.

This pass intentionally excludes broad new coverage. New icon creation is allowed only when the polish audit finds a clear quality defect in an existing `editor_pages` icon and the replacement keeps the same file path and meaning. It does not add a new page family, new feature category, or direct UI reference.

## Current Context

`docs/zircon_editor/assets/editor-page-function-svg-resources.md` documents the generated 204-icon page/function taxonomy. The previous acceptance gate confirmed A/B/C counts of 61, 54, and 89 icons, no external asset references, and ASCII-only content.

Current Editor UI templates still use generic `icon = "..."` names and a small number of explicit `value = "ionicons/...svg"` references. Examples include `workbench_shell.v2.ui.toml`, `scene_viewport_toolbar.v2.ui.toml`, `asset_surface_controls.v2.ui.toml`, `workbench_activity_rail.v2.ui.toml`, and `workbench_dock_header.v2.ui.toml`. The mapping pass prepares a clean replacement policy but does not perform replacement.

## A. Visual Polish Design

The polish pass should review every `editor_pages` SVG, but only edit icons with concrete quality problems. The objective is to keep the pack stable while removing weak assets before downstream UI adoption.

### Audit Criteria

- **Semantic fit:** the glyph should clearly match its filename and page/function directory.
- **Pair directionality:** directional pairs such as input/output, import/export, push/pull-like metaphors, and dock-left/dock-right must read correctly.
- **16 px readability:** important shapes should survive reduction to compact toolbar size.
- **Family consistency:** icons inside a function group should share visual weight and base metaphors without becoming indistinguishable.
- **Palette consistency:** colors should stay within the documented cool gray, Zircon cyan, amber, red, violet, and green palette unless an existing file already uses an accepted local variant.
- **Accessibility metadata:** every SVG should keep `role="img"` and a useful `aria-label`.
- **SVG contract:** every SVG should remain inline, ASCII-only, `viewBox="0 0 24 24"`, and free of `<image>`, external `href`, `<use>`, `<symbol>`, CSS, script, font, or remote URL references.

### Known Review Hotspots

Prior reviews already accepted the pack, but this polish pass should re-check these areas first because they are most likely to matter during UI wiring:

- `graph_editor/pins/*` for input/output direction and pin-type distinction.
- `build_plugins/build/*` for build/rebuild/test/settings/pipeline semantic clarity.
- `graph_editor/shader/parameter.svg` versus `build_plugins/build/build-settings.svg`, because both use slider-like settings metaphors.
- `workbench/tabs/*` because tab controls are likely to appear in small UI chrome.
- `scene_viewport/tools/*`, `scene_viewport/snapping/*`, and `scene_viewport/camera/*` because these are likely to be first visible replacements for toolbar icons.
- `asset_browser/navigation/*` and `asset_browser/import_pipeline/*` because they map directly to existing asset toolbar controls.

### Formatting Policy

The existing pack uses compact single-line SVG files. This pass should not reformat all files just to improve diffs. If an edited file becomes hard to inspect, that individual file may be rewritten in a lightly formatted multi-line SVG form, but bulk style-only formatting is out of scope.

## C. UI Mapping Preparation Design

The mapping pass creates `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`. It inventories current Editor template icon roles and recommends future `editor_pages` replacements.

### Mapping Inputs

The first mapping pass should focus on production Editor asset templates under `zircon_editor/assets/ui/editor`, especially:

- `host/workbench_shell.v2.ui.toml`
- `host/scene_viewport_toolbar.v2.ui.toml`
- `host/asset_surface_controls.v2.ui.toml`
- `host/startup_welcome_controls.v2.ui.toml`
- `workbench_activity_rail.v2.ui.toml`
- `workbench_dock_header.v2.ui.toml`
- `host/console_body.v2.ui.toml`
- `host/hierarchy_body.v2.ui.toml`
- `host/animation_graph_body.v2.ui.toml`
- `host/animation_sequence_body.v2.ui.toml`
- `host/performance_timeline_body.v2.ui.toml`
- `host/runtime_diagnostics_body.v2.ui.toml`
- `host/build_export_desktop_body.v2.ui.toml`
- `host/module_plugins_body.v2.ui.toml`

Material component lab `.zui` files can be listed as a secondary surface, but they should not dominate the first mapping because many of their icons are component showcase examples rather than core Editor shell chrome.

### Mapping Table Contract

The mapping document should contain a table with these columns:

- `Template path`
- `Control id or local node`
- `Current icon/value`
- `Current role`
- `Recommended editor_pages path`
- `Confidence` (`direct`, `near`, or `gap`)
- `Notes`

Use `direct` when a generated icon has the same UI role, `near` when the generated icon is acceptable but not exact, and `gap` when wiring should wait for a future coverage pass.

### Initial Mapping Policy

Examples of expected mapping decisions:

- `folder-open-outline` in workbench open-project context maps to `workbench/menu/open-project.svg` with direct confidence.
- `grid-outline` used for Reset Layout maps to `workbench/dock/reset-layout.svg` with direct confidence.
- Scene viewport transform controls map to `scene_viewport/tools/*` where names align.
- `play-outline` and `stop-outline` in viewport preview controls map to `scene_viewport/play/play.svg` and `scene_viewport/play/stop.svg`.
- Asset browser folder/import/link controls map to `asset_browser/navigation/folder.svg`, `asset_browser/import_pipeline/import.svg`, and `asset_browser/references/reference.svg` or `dependency.svg` based on role.
- Generic controls such as terminal, flash, plus, edit, cloud upload, and timeline may be marked `near` or `gap` if no page/function icon precisely covers the role.

The mapping document is advisory. It must not claim that templates already use the new icons.

## Documentation Updates

`docs/zircon_editor/assets/editor-page-function-svg-resources.md` should be updated with a new polish/mapping section after implementation. The update should record:

- the final icon count after polish,
- a summary of edited icons or confirmation that no edits were needed,
- the mapping document path,
- validation commands and results,
- remaining `gap` mappings that require future icon coverage or UI design decisions.

`docs/zircon_editor/assets/icon-resource-audit.md` may remain unchanged unless the implementation adds, removes, or renames SVG files. A polish-only replacement that preserves the same 204 paths does not require refreshing the broader audit inventory.

## Testing And Acceptance

The implementation is accepted when:

- The `editor_pages` tree still contains exactly 204 SVG files unless a reviewed plan explicitly changes the count.
- Edited SVGs retain `viewBox="0 0 24 24"`, `role="img"`, useful `aria-label`, ASCII content, and no forbidden references.
- Full-pack scans find no forbidden `<image>`, `href=`, `<use>`, `<symbol>`, CSS/script/font, or remote URL references.
- A visual/semantic audit records any edited icons and the reason each edit was made.
- `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` exists and maps or explicitly marks deferred gaps for the production Editor template icon usages in scope.
- `docs/zircon_editor/assets/editor-page-function-svg-resources.md` links to the mapping doc and records validation evidence.

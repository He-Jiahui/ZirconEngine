# Editor SVG Polish And UI Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the existing 204 `editor_pages` SVG assets and prepare an advisory UI template mapping without changing Editor UI behavior.

**Architecture:** This is an asset/documentation pass. Existing `editor_pages` icon paths remain stable; weak SVGs may be replaced in place, but no new page families are added and no UI templates are rewired. A separate mapping document records how current Editor template icon roles can later move from Ionicons/generic names to `editor_pages` paths.

**Tech Stack:** SVG, Markdown documentation, PowerShell/Glob/Grep validation.

---

## File Structure

- Modify if needed: `zircon_editor/assets/icons/editor_pages/**/*.svg`
- Create: `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`
- Modify: `docs/zircon_editor/assets/editor-page-function-svg-resources.md`
- Reference only: `docs/superpowers/specs/2026-05-21-editor-svg-polish-ui-mapping-design.md`
- Reference only: `docs/zircon_editor/assets/icon-resource-audit.md`
- Read-only mapping inputs under `zircon_editor/assets/ui/editor/**`:
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

## Shared Rules

- Do not modify `*.v2.ui.toml`, `.zui`, Rust source files, icon registries, atlas logic, or existing icon packs outside `editor_pages`.
- Keep the `editor_pages` inventory at exactly 204 SVG files unless a reviewer explicitly approves a count change. This plan does not approve a count change.
- Every SVG must keep `viewBox="0 0 24 24"`, `role="img"`, a useful `aria-label`, ASCII content, and no forbidden external references.
- Prefer minimal in-place SVG replacements over broad formatting churn.
- Do not update `docs/zircon_editor/assets/icon-resource-audit.md` unless an SVG path is added, removed, or renamed. This plan should preserve paths.

## Milestone 1: Visual And Semantic Polish Audit

### Goal

Review all 204 `editor_pages` SVG files, identify concrete quality defects, and fix only icons whose semantics or compact readability would block future UI adoption.

### In-Scope Behaviors

- All 204 icons are reviewed against semantic fit, pair directionality, 16 px readability, family consistency, palette consistency, accessibility metadata, and SVG contract.
- Known hotspots are reviewed first: `graph_editor/pins`, `build_plugins/build`, `graph_editor/shader/parameter.svg`, `build_plugins/build/build-settings.svg`, `workbench/tabs`, `scene_viewport/tools`, `scene_viewport/snapping`, `scene_viewport/camera`, `asset_browser/navigation`, and `asset_browser/import_pipeline`.
- Any edited icon keeps the same path and conceptual role.

### Dependencies

- Approved design spec: `docs/superpowers/specs/2026-05-21-editor-svg-polish-ui-mapping-design.md`.
- Existing generated asset doc: `docs/zircon_editor/assets/editor-page-function-svg-resources.md`.

### Implementation Slices

- [ ] Inventory all `editor_pages` SVG files and group them by page/function directory.
- [ ] Inspect hotspot groups first and record each defect found with path, issue, and planned replacement intent.
- [ ] Inspect remaining groups and record any additional defects with path, issue, and planned replacement intent.
- [ ] Replace only defective SVGs in place. Preserve filename, directory, `viewBox="0 0 24 24"`, `role="img"`, meaningful `aria-label`, and ASCII-only content.
- [ ] Do not rewrite icons that already meet the criteria; note in the report if no edit is needed for a group.

### Testing Stage: SVG Polish Gate

- [ ] Count all generated icons with a PowerShell or `rg` file inventory and confirm `204` SVG files under `zircon_editor/assets/icons/editor_pages`.
- [ ] Count page groups and confirm the existing distribution remains `61 + 54 + 89` across A/B/C page groups.
- [ ] Scan all `editor_pages` SVG files for missing `viewBox="0 0 24 24"`, `role="img"`, or `aria-label`; fix any failures.
- [ ] Scan all `editor_pages` SVG files for forbidden constructs: `<image`, `href=`, `<use`, `<symbol`, `<script`, `<style`, `style=`, `class=`, `@font`, `font-family`, and remote `url(http`.
- [ ] Scan all `editor_pages` SVG files for non-ASCII bytes.
- [ ] Parse edited SVG files as XML using PowerShell/.NET or equivalent to catch malformed replacements.

### Exit Evidence

- Final inventory remains 204 SVG files.
- Full-pack SVG contract scans pass.
- Report lists edited icons and the reason for each edit, or confirms no edits were necessary.

## Milestone 2: UI Template Icon Mapping Document

### Goal

Create an advisory mapping from current production Editor template icon usage to `editor_pages` paths, without modifying any template.

### In-Scope Behaviors

- Production Editor templates listed in the design spec are inventoried for `Icon`, `IconButton`, `icon = "..."`, and explicit `value = "ionicons/..."` usage.
- Each mapped row records template path, control id or node, current icon/value, current role, recommended `editor_pages` path, confidence, and notes.
- Every production template icon usage in the mapping scope is either `direct`, `near`, or `gap`.
- Material component lab `.zui` examples are summarized as secondary surfaces, not treated as core shell blockers.

### Dependencies

- Milestone 1 is complete, so the icon pack is ready for mapping recommendations.

### Implementation Slices

- [ ] Read the production mapping input templates listed in the design spec.
- [ ] Inventory each relevant `Icon` or `IconButton` row with path, local node/control id, current icon/value, and UI role.
- [ ] Match obvious direct replacements:
  - Workbench open project -> `workbench/menu/open-project.svg`
  - Workbench save -> `workbench/menu/save-all.svg` or mark `near` if the UI role is single-document save
  - Workbench reset layout -> `workbench/dock/reset-layout.svg`
  - Activity rail assets -> `asset_browser/navigation/folder.svg` or `asset_browser/asset_types/*` based on role
  - Activity rail hierarchy -> `hierarchy/entity/scene.svg` or `hierarchy/entity/entity.svg` based on role
  - Activity rail console -> `console_profiler/logs/log-info.svg` or mark `near` if terminal semantics are required
  - Scene viewport transform/display/snap/camera/play controls -> matching `scene_viewport/**` paths
  - Asset surface folder/import/link/locate/view/tool controls -> matching `asset_browser/**`, `workbench/**`, or `gap` rows
  - Animation graph and sequence controls -> matching `graph_editor/**` or `animation_timeline/**` paths
  - Performance/runtime diagnostics/build/plugin focus buttons -> matching `console_profiler/**` or `build_plugins/**` paths
- [ ] Create `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` with a machine-readable header.
- [ ] Include a summary of counts by confidence: `direct`, `near`, and `gap`.
- [ ] Include a clear statement that no UI templates were changed and the mapping is advisory.
- [ ] Include secondary notes for Material component lab `.zui` icon examples and why they are deferred.

### Testing Stage: Mapping Coverage Gate

- [ ] Re-run a content search for `component = "Icon"`, `component = "IconButton"`, `icon = "`, and `value = "ionicons/` in the mapped template scope.
- [ ] Confirm every production template usage in scope appears in `editor-page-function-icon-template-map.md` or is explicitly listed as deferred.
- [ ] Confirm the mapping document has required header fields: `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`.
- [ ] Confirm the document does not claim templates already use `editor_pages` icons.

### Exit Evidence

- Mapping document exists.
- Mapping document covers the production template scope and marks unresolved roles as `gap`.
- No UI template files were modified by this milestone.

## Milestone 3: Documentation Synchronization And Final Asset Gate

### Goal

Update the existing `editor_pages` asset documentation with polish and mapping results, then run final whole-pack validation.

### In-Scope Behaviors

- `docs/zircon_editor/assets/editor-page-function-svg-resources.md` links to the mapping document.
- The doc records the final 204 count, edited icon summary, mapping confidence counts, validation evidence, and remaining gaps.
- Full-pack validation confirms the SVG contract remains intact after polish.

### Dependencies

- Milestone 1 polish is complete.
- Milestone 2 mapping document exists.

### Implementation Slices

- [ ] Update the machine-readable header in `docs/zircon_editor/assets/editor-page-function-svg-resources.md` by adding `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` to both `related_code` and `implementation_files`.
- [ ] Add a `Polish And UI Mapping Follow-Up` section to `docs/zircon_editor/assets/editor-page-function-svg-resources.md`.
- [ ] Record edited icon paths and reasons, or state that no SVG edits were needed after review.
- [ ] Record mapping document path and confidence counts.
- [ ] Record remaining `gap` mappings and whether they require new icon coverage or future UI design decisions.
- [ ] Do not update `docs/zircon_editor/assets/icon-resource-audit.md` unless path count changed; if unchanged, state that it remains inventory-stable.

### Testing Stage: Final SVG Polish And Mapping Gate

- [ ] Count all generated icons and confirm `204` SVG files under `zircon_editor/assets/icons/editor_pages`.
- [ ] Confirm the existing page-group distribution remains A `61`, B `54`, C `89`.
- [ ] Scan all `editor_pages` SVG files for forbidden external/reference constructs.
- [ ] Scan all `editor_pages` SVG files for non-ASCII bytes.
- [ ] Confirm `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` and `docs/zircon_editor/assets/editor-page-function-svg-resources.md` both contain current machine-readable headers.
- [ ] Inspect `git status --short -- zircon_editor/assets/ui/editor docs/zircon_editor/assets/editor-page-function-icon-template-map.md docs/zircon_editor/assets/editor-page-function-svg-resources.md zircon_editor/assets/icons/editor_pages` and confirm no UI templates were modified by this plan.

### Exit Evidence

- Full SVG scans pass.
- Documentation is synchronized with the mapping and polish result.
- No Editor UI templates were modified.

## Self-Review

- Spec coverage: Milestone 1 covers A visual polish, Milestone 2 covers C UI mapping preparation, and Milestone 3 covers docs and final validation.
- Scope check: the plan does not add broad new icon coverage and does not wire templates, Rust code, registries, or atlas generation.
- Placeholder scan: no unresolved plan steps remain.
- Path consistency: the plan uses `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` for the mapping doc and preserves `zircon_editor/assets/icons/editor_pages/**` as the asset root.

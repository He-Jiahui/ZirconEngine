# Editor Page Function SVG Icons Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate 204 self-contained Editor SVG icons organized by page and function under `zircon_editor/assets/icons/editor_pages`.

**Architecture:** The new resource tree is asset-only and does not modify retained-host UI templates. Icons are grouped by owning Editor page first and by page function second, so later template wiring or sprite-atlas generation can map assets by screen context without relying on generic engine concept names. Documentation records the taxonomy, reference routing, style contract, and validation evidence.

**Tech Stack:** SVG, Markdown documentation, PowerShell/ripgrep inventory validation.

---

## File Structure

- Create: `zircon_editor/assets/icons/editor_pages/workbench/menu/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/workbench/dock/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/workbench/tabs/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/workbench/status/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/scene_viewport/tools/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/scene_viewport/snapping/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/scene_viewport/camera/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/scene_viewport/display/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/scene_viewport/play/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/hierarchy/entity/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/hierarchy/component/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/hierarchy/row_state/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/asset_browser/navigation/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/asset_browser/asset_types/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/asset_browser/import_pipeline/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/asset_browser/references/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/inspector/sections/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/inspector/properties/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/inspector/material_channels/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/inspector/validation/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/animation_timeline/transport/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/animation_timeline/tracks/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/animation_timeline/keys/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/animation_timeline/curves/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/graph_editor/nodes/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/graph_editor/pins/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/graph_editor/execution/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/graph_editor/shader/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/ui_layout_editor/layout/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/ui_layout_editor/widgets/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/ui_layout_editor/constraints/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/console_profiler/logs/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/console_profiler/diagnostics/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/console_profiler/profiling/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/build_plugins/build/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/build_plugins/package/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/build_plugins/deploy/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/build_plugins/plugins/*.svg`
- Create: `zircon_editor/assets/icons/editor_pages/build_plugins/source_control/*.svg`
- Create: `docs/zircon_editor/assets/editor-page-function-svg-resources.md`
- Modify: `docs/superpowers/specs/2026-05-20-editor-page-function-svg-icons-design.md` only if implementation changes the icon inventory or taxonomy.

## Shared SVG Contract

- Every SVG uses `viewBox="0 0 24 24"` and fits a 16 to 24 px Editor UI slot.
- Use neutral stroke `#cbd2dc`, muted stroke `#64748b`, panel fill `#172033`, Zircon cyan `#26d8d1`, amber `#f0b545`, red `#e16666`, violet `#a78bfa`, and green `#63d489` as needed.
- Keep every SVG ASCII, inline, and dependency-free.
- Do not use `<image>`, external `href`, web font, CSS file, remote URL, or vendor symbol references.
- Avoid changing existing `ionicons`, `zircon_editor_shell`, or `zircon_engine_style` assets.

## Milestone A: Main Editor Interface

### Goal

Create the immediate Editor surface vocabulary for `workbench`, `scene_viewport`, and `hierarchy`.

### In-Scope Behaviors

- Workbench menu, dock, tab, and status affordances exist as page-owned SVGs.
- Scene viewport tool, snapping, camera, display, and play controls exist as page-owned SVGs.
- Hierarchy entity, component, and row-state icons exist as page-owned SVGs.

### Dependencies

- Existing `zircon_editor/assets/icons` tree remains untouched except for adding `editor_pages`.
- Design spec `docs/superpowers/specs/2026-05-20-editor-page-function-svg-icons-design.md` is the authoritative inventory.

### Implementation Slices

- [ ] Create all `workbench` icons: `command-palette`, `quick-open`, `open-project`, `save-all`, `undo-history`, `dock-left`, `dock-right`, `dock-bottom`, `split-horizontal`, `reset-layout`, `pin-tab`, `close-tab`, `dirty-tab`, `tab-overflow`, `notification`, `background-task`, `sync-state`, `layout-warning`.
- [ ] Create all `scene_viewport/tools` icons: `select`, `translate`, `rotate`, `scale`, `universal-transform`, `measure-tool`.
- [ ] Create all `scene_viewport/snapping` icons: `grid-snap`, `angle-snap`, `scale-snap`, `vertex-snap`, `snap-settings`.
- [ ] Create all `scene_viewport/camera` icons: `camera-orbit`, `camera-pan`, `camera-zoom`, `frame-selection`, `perspective`, `orthographic`.
- [ ] Create all `scene_viewport/display` icons: `lit`, `wireframe`, `bounds`, `grid-overlay`, `gizmo-visibility`.
- [ ] Create all `scene_viewport/play` icons: `play`, `pause`, `stop`, `simulate`, `step-frame`.
- [ ] Create all `hierarchy` icons: `scene`, `entity`, `empty`, `prefab-instance`, `camera-entity`, `transform-component`, `mesh-renderer-component`, `light-component`, `script-component`, `physics-component`, `visible`, `hidden`, `locked`, `unlocked`, `isolate`, `warning-state`.

### Testing Stage: Main Interface Asset Gate

- [ ] Count A icons with `rg --files zircon_editor/assets/icons/editor_pages/workbench zircon_editor/assets/icons/editor_pages/scene_viewport zircon_editor/assets/icons/editor_pages/hierarchy | rg '\.svg$'` and confirm 61 SVG files.
- [ ] Scan A icons with `rg -n "href=|<image|url\(http" zircon_editor/assets/icons/editor_pages/workbench zircon_editor/assets/icons/editor_pages/scene_viewport zircon_editor/assets/icons/editor_pages/hierarchy` and confirm no matches.
- [ ] Scan A icons for non-ASCII bytes with a PowerShell byte check and confirm no files are reported.
- [ ] If validation fails, fix the lowest owned asset file first, then rerun the focused A scans.

### Exit Evidence

- A focused count reports 61 SVG files.
- External-reference scan reports no matches.
- ASCII scan reports no generated files with bytes greater than `0x7f`.

## Milestone B: Asset Browser And Inspector

### Goal

Create the asset navigation, asset type, import/reference, inspector section, property, material channel, and validation vocabulary.

### In-Scope Behaviors

- Asset Browser has navigation, asset type, import pipeline, and reference-state icons.
- Inspector has section, property operation, material channel, and validation icons.

### Dependencies

- Milestone A resource tree exists and its validation passed.

### Implementation Slices

- [ ] Create all `asset_browser/navigation` icons: `folder`, `favorite`, `recent`, `search`, `filter`, `sort-ascending`, `refresh`.
- [ ] Create all `asset_browser/asset_types` icons: `scene-file`, `prefab`, `mesh`, `material`, `texture`, `shader`, `audio-clip`, `animation-clip`, `script-file`, `font`, `particle-system`.
- [ ] Create all `asset_browser/import_pipeline` icons: `import`, `reimport`, `export`, `import-settings`, `import-error`.
- [ ] Create all `asset_browser/references` icons: `dependency`, `reference`, `missing-reference`, `used-by`, `bundle`.
- [ ] Create all `inspector/sections` icons: `transform`, `rendering`, `physics`, `audio`, `script`, `animation`, `ui`, `material`.
- [ ] Create all `inspector/properties` icons: `add-component`, `reset-property`, `override-property`, `revert-override`, `link`, `unlink`, `keyframe-property`.
- [ ] Create all `inspector/material_channels` icons: `albedo`, `normal`, `metallic`, `roughness`, `emissive`, `opacity`.
- [ ] Create all `inspector/validation` icons: `info`, `success`, `warning`, `error`, `missing-script`.

### Testing Stage: Asset And Inspector Asset Gate

- [ ] Count B icons with `rg --files zircon_editor/assets/icons/editor_pages/asset_browser zircon_editor/assets/icons/editor_pages/inspector | rg '\.svg$'` and confirm 54 SVG files.
- [ ] Scan B icons with `rg -n "href=|<image|url\(http" zircon_editor/assets/icons/editor_pages/asset_browser zircon_editor/assets/icons/editor_pages/inspector` and confirm no matches.
- [ ] Scan B icons for non-ASCII bytes with a PowerShell byte check and confirm no files are reported.
- [ ] If validation fails, fix the lowest owned asset file first, then rerun the focused B scans.

### Exit Evidence

- B focused count reports 54 SVG files.
- External-reference scan reports no matches.
- ASCII scan reports no generated files with bytes greater than `0x7f`.

## Milestone C: Secondary Tool Pages

### Goal

Create animation, graph, UI layout, console/profiler, and build/plugin page vocabularies after the main and asset workflows are available.

### In-Scope Behaviors

- Animation timeline has transport, tracks, keys, and curve icons.
- Graph editor has node, pin, execution, and shader graph icons.
- UI layout editor has layout, widget, and constraint icons.
- Console/profiler has log, diagnostics, and profiling icons.
- Build/plugins has build, package, deploy, plugin/module, and source-control icons.

### Dependencies

- Milestones A and B resource trees exist and their focused validation passed.

### Implementation Slices

- [ ] Create all `animation_timeline` icons: `timeline-play`, `timeline-pause`, `timeline-stop`, `step-prev`, `loop`, `transform-track`, `bone-track`, `audio-track`, `event-track`, `keyframe`, `add-keyframe`, `delete-keyframe`, `key-selected`, `curve-linear`, `curve-bezier`, `dope-sheet`.
- [ ] Create all `graph_editor/nodes` icons: `event-node`, `function-node`, `variable-node`, `comment-node`, `state-node`, `transition-node`, `shader-node`.
- [ ] Create all `graph_editor/pins` icons: `input-pin`, `output-pin`, `exec-pin`, `data-pin`, `reroute-pin`.
- [ ] Create all `graph_editor/execution` icons: `sequence`, `branch`, `breakpoint`, `debug-pulse`.
- [ ] Create all `graph_editor/shader` icons: `material-output`, `texture-sample`, `parameter`, `preview-node`.
- [ ] Create all `ui_layout_editor` icons: `canvas`, `artboard`, `stack`, `grid`, `absolute-layout`, `overlay-layout`, `widget`, `text`, `image`, `button`, `slider`, `anchor`, `constraint`, `margin`, `padding`, `align-center`.
- [ ] Create all `console_profiler` icons: `log-info`, `log-warning`, `log-error`, `clear-logs`, `filter-logs`, `breakpoint`, `watch`, `callstack`, `crash-report`, `cpu`, `gpu`, `memory`, `frame-time`, `frame-graph`.
- [ ] Create all `build_plugins` icons: `build`, `rebuild`, `test`, `build-settings`, `pipeline`, `package`, `archive`, `signing`, `artifact`, `deploy`, `device`, `cloud-upload`, `install`, `plugin`, `module`, `enable-plugin`, `disable-plugin`, `marketplace`, `branch`, `commit`, `merge`, `push`, `conflict`.

### Testing Stage: Secondary Tool Asset Gate

- [ ] Count C icons with `rg --files zircon_editor/assets/icons/editor_pages/animation_timeline zircon_editor/assets/icons/editor_pages/graph_editor zircon_editor/assets/icons/editor_pages/ui_layout_editor zircon_editor/assets/icons/editor_pages/console_profiler zircon_editor/assets/icons/editor_pages/build_plugins | rg '\.svg$'` and confirm 89 SVG files.
- [ ] Scan C icons with `rg -n "href=|<image|url\(http" zircon_editor/assets/icons/editor_pages/animation_timeline zircon_editor/assets/icons/editor_pages/graph_editor zircon_editor/assets/icons/editor_pages/ui_layout_editor zircon_editor/assets/icons/editor_pages/console_profiler zircon_editor/assets/icons/editor_pages/build_plugins` and confirm no matches.
- [ ] Scan C icons for non-ASCII bytes with a PowerShell byte check and confirm no files are reported.
- [ ] If validation fails, fix the lowest owned asset file first, then rerun the focused C scans.

### Exit Evidence

- C focused count reports 89 SVG files.
- External-reference scan reports no matches.
- ASCII scan reports no generated files with bytes greater than `0x7f`.

## Milestone D: Documentation And Full Inventory Gate

### Goal

Document the generated page/function taxonomy and run whole-pack asset acceptance scans.

### In-Scope Behaviors

- The module documentation explains ownership, reference routing, generated categories, style rules, integration boundary, and validation evidence.
- The full tree contains exactly 204 SVG icons.
- The full tree remains self-contained and ASCII-only.

### Dependencies

- Milestones A, B, and C are complete.

### Implementation Slices

- [ ] Create `docs/zircon_editor/assets/editor-page-function-svg-resources.md` with a machine-readable header containing representative `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type` fields.
- [ ] Document each page directory and functional subdirectory.
- [ ] Document that this asset pass intentionally does not wire icons into retained-host templates.
- [ ] Update `docs/superpowers/specs/2026-05-20-editor-page-function-svg-icons-design.md` only if final file inventory differs from the approved 204-icon list.

### Testing Stage: Full Editor Page Icon Gate

- [ ] Count all generated icons with `rg --files zircon_editor/assets/icons/editor_pages | rg '\.svg$'` and confirm 204 SVG files.
- [ ] Scan all generated icons with `rg -n "href=|<image|url\(http" zircon_editor/assets/icons/editor_pages` and confirm no matches.
- [ ] Scan all generated icons for non-ASCII bytes with PowerShell and confirm no files are reported.
- [ ] Review `docs/zircon_editor/assets/editor-page-function-svg-resources.md` for current headers, the 204 count, and A/B/C validation evidence.

### Exit Evidence

- Full count reports 204 SVG files.
- External-reference scan reports no matches.
- ASCII scan reports no generated files with bytes greater than `0x7f`.
- Documentation names the page/function taxonomy and records validation evidence.

## Self-Review

- Spec coverage: Milestones A, B, C, and D cover every directory and icon listed in `docs/superpowers/specs/2026-05-20-editor-page-function-svg-icons-design.md`.
- Placeholder scan: no plan step uses unresolved `TBD`, vague future implementation, or unbounded validation instructions.
- Scope check: the plan is asset-only and does not include UI template wiring, atlas generation, or retained-host behavior changes.
- Type/path consistency: directory names match the approved taxonomy: `workbench`, `scene_viewport`, `hierarchy`, `asset_browser`, `inspector`, `animation_timeline`, `graph_editor`, `ui_layout_editor`, `console_profiler`, and `build_plugins`.

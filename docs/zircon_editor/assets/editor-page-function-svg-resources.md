---
related_code:
  - zircon_editor/assets/icons/editor_pages/workbench/menu/command-palette.svg
  - zircon_editor/assets/icons/editor_pages/scene_viewport/tools/translate.svg
  - zircon_editor/assets/icons/editor_pages/hierarchy/entity/scene.svg
  - zircon_editor/assets/icons/editor_pages/asset_browser/navigation/folder.svg
  - zircon_editor/assets/icons/editor_pages/inspector/sections/transform.svg
  - zircon_editor/assets/icons/editor_pages/animation_timeline/keys/keyframe.svg
  - zircon_editor/assets/icons/editor_pages/graph_editor/nodes/event-node.svg
  - zircon_editor/assets/icons/editor_pages/ui_layout_editor/layout/canvas.svg
  - zircon_editor/assets/icons/editor_pages/console_profiler/logs/log-info.svg
  - zircon_editor/assets/icons/editor_pages/build_plugins/build/build.svg
implementation_files:
  - zircon_editor/assets/icons/editor_pages/**
  - docs/zircon_editor/assets/editor-page-function-svg-resources.md
plan_sources:
  - user: 2026-05-20 generate more Editor SVG icons, reference Fyrox, Unity, Unreal main and secondary editor interfaces
  - docs/superpowers/specs/2026-05-20-editor-page-function-svg-icons-design.md
  - docs/superpowers/plans/2026-05-20-editor-page-function-svg-icons.md
tests:
  - asset inventory: 204 SVG files under zircon_editor/assets/icons/editor_pages
  - external-reference scan: no href/image/remote-url references under zircon_editor/assets/icons/editor_pages
  - ASCII scan: no non-ASCII content under zircon_editor/assets/icons/editor_pages
doc_type: module-detail
---

# Editor Page Function SVG Resources

## Scope

`zircon_editor/assets/icons/editor_pages` is a Zircon-owned, page-first SVG resource tree for Editor page and function icons. It contains 204 generated SVG files arranged by owning Editor surface before functional category. The taxonomy exists so future retained-host template wiring, icon registry work, or sprite-atlas generation can resolve icons by screen context instead of relying on older generic icon packs.

This asset pass is intentionally asset-only. It does not modify retained-host templates, runtime icon lookup, atlas generation, Rust code, or existing icon packs such as `ionicons`, `zircon_editor_shell`, or `zircon_engine_style`. Current UI behavior remains unchanged until a later integration plan maps these paths into templates or runtime asset catalogs and validates rendering behavior separately.

## Ownership

The `editor_pages` pack is owned by `zircon_editor` because it names authoring pages and authoring-page operations rather than runtime systems. The first directory segment identifies the Editor page that owns the vocabulary. The second directory segment identifies the functional role inside that page. File stems then identify the specific action, state, asset type, or panel concept.

The owner path is the stable lookup identity. For example, `scene_viewport/tools/translate.svg` is a viewport tool icon, while a future `graph_editor/nodes/translate.svg` would be a graph concept even if the file stem overlapped. Consumers should keep the page and function path when referencing icons; stem-only lookup would reintroduce collisions between common names such as `play`, `warning`, `branch`, `build`, `image`, and `material`.

## Reference Routing

The vocabulary is informed by Fyrox, Unity, and Unreal editor surfaces, but every file is a Zircon-owned SVG drawing under the repository asset tree.

Fyrox leads the page split because it provides a Rust-native editor/runtime reference with scene, asset, inspector, animation, and graph-style tool organization. Unity leads common scene, project, inspector, timeline, material, and package/build vocabulary. Unreal leads scale-oriented secondary tooling, including content browser concepts, graph editing, profiling, plugin/module workflows, source control, packaging, and deployment.

The generated paths translate those references into Zircon terms. They do not preserve vendor filenames, vendor icon silhouettes, external symbols, or copied artwork. Reference engines guide coverage and categorization only.

## Style Rules

Every icon in this pack is an inline SVG intended for compact Editor UI slots:

- The view box is `0 0 24 24`.
- Geometry is self-contained in the SVG file.
- Files use ASCII text only.
- Files avoid `<image>`, external `href`, web font, CSS file, vendor symbol, and remote `url(http...)` references.
- The visual weight is dark-editor friendly and readable from 16 px to 24 px.
- Neutral structure uses cool gray strokes such as `#cbd2dc` and muted strokes such as `#64748b`.
- Active tools and primary affordances use Zircon cyan `#26d8d1`.
- Warnings, lighting, or attention states use amber `#f0b545`.
- Stop, destructive, and error states use red `#e16666`.
- Graph or secondary-tool accents may use violet `#a78bfa`.
- Success or enabled states may use green `#63d489`.

The pack uses direct path, line, circle, rect, polygon, and inline definition geometry where needed. Inline gradients are allowed only when local to the file and not dependent on external resources.

## Generated Taxonomy

### Workbench - 18 Icons

`workbench` owns global Editor shell operations that are not specific to a scene panel, asset browser, or secondary tool page.

`workbench/menu` contains 5 command-surface icons: `command-palette.svg`, `quick-open.svg`, `open-project.svg`, `save-all.svg`, and `undo-history.svg`. These represent global command search, project entry, save, and history operations.

`workbench/dock` contains 5 layout-management icons: `dock-left.svg`, `dock-right.svg`, `dock-bottom.svg`, `split-horizontal.svg`, and `reset-layout.svg`. These describe workbench docking, panel placement, split layout, and layout reset affordances.

`workbench/tabs` contains 4 tab-state icons: `pin-tab.svg`, `close-tab.svg`, `dirty-tab.svg`, and `tab-overflow.svg`. These are scoped to document or panel tab controls.

`workbench/status` contains 4 status-strip icons: `notification.svg`, `background-task.svg`, `sync-state.svg`, and `layout-warning.svg`. These describe global activity, sync, and layout warnings.

### Scene Viewport - 27 Icons

`scene_viewport` owns the main scene editing viewport, its tool mode controls, display overlays, camera navigation, snapping, and preview-play controls.

`scene_viewport/tools` contains 6 transform and measurement tools: `select.svg`, `translate.svg`, `rotate.svg`, `scale.svg`, `universal-transform.svg`, and `measure-tool.svg`.

`scene_viewport/snapping` contains 5 snapping mode icons: `grid-snap.svg`, `angle-snap.svg`, `scale-snap.svg`, `vertex-snap.svg`, and `snap-settings.svg`.

`scene_viewport/camera` contains 6 viewport camera controls: `camera-orbit.svg`, `camera-pan.svg`, `camera-zoom.svg`, `frame-selection.svg`, `perspective.svg`, and `orthographic.svg`.

`scene_viewport/display` contains 5 display and overlay controls: `lit.svg`, `wireframe.svg`, `bounds.svg`, `grid-overlay.svg`, and `gizmo-visibility.svg`.

`scene_viewport/play` contains 5 preview/simulation controls: `play.svg`, `pause.svg`, `stop.svg`, `simulate.svg`, and `step-frame.svg`.

### Hierarchy - 16 Icons

`hierarchy` owns scene tree rows, scene/entity types, attached component types, and per-row affordance states.

`hierarchy/entity` contains 5 entity-row icons: `scene.svg`, `entity.svg`, `empty.svg`, `prefab-instance.svg`, and `camera-entity.svg`.

`hierarchy/component` contains 5 component glyphs for common row or quick-filter context: `transform-component.svg`, `mesh-renderer-component.svg`, `light-component.svg`, `script-component.svg`, and `physics-component.svg`.

`hierarchy/row_state` contains 6 row state controls: `visible.svg`, `hidden.svg`, `locked.svg`, `unlocked.svg`, `isolate.svg`, and `warning-state.svg`.

### Asset Browser - 28 Icons

`asset_browser` owns project/resource browsing, file type identity, import/export states, and dependency/reference states.

`asset_browser/navigation` contains 7 browsing controls: `folder.svg`, `favorite.svg`, `recent.svg`, `search.svg`, `filter.svg`, `sort-ascending.svg`, and `refresh.svg`.

`asset_browser/asset_types` contains 11 resource-type icons: `scene-file.svg`, `prefab.svg`, `mesh.svg`, `material.svg`, `texture.svg`, `shader.svg`, `audio-clip.svg`, `animation-clip.svg`, `script-file.svg`, `font.svg`, and `particle-system.svg`.

`asset_browser/import_pipeline` contains 5 import/export state icons: `import.svg`, `reimport.svg`, `export.svg`, `import-settings.svg`, and `import-error.svg`.

`asset_browser/references` contains 5 dependency and reference icons: `dependency.svg`, `reference.svg`, `missing-reference.svg`, `used-by.svg`, and `bundle.svg`.

### Inspector - 26 Icons

`inspector` owns component section identity, property editing affordances, material channel controls, and validation states.

`inspector/sections` contains 8 section icons: `transform.svg`, `rendering.svg`, `physics.svg`, `audio.svg`, `script.svg`, `animation.svg`, `ui.svg`, and `material.svg`.

`inspector/properties` contains 7 property operation icons: `add-component.svg`, `reset-property.svg`, `override-property.svg`, `revert-override.svg`, `link.svg`, `unlink.svg`, and `keyframe-property.svg`.

`inspector/material_channels` contains 6 material channel icons: `albedo.svg`, `normal.svg`, `metallic.svg`, `roughness.svg`, `emissive.svg`, and `opacity.svg`.

`inspector/validation` contains 5 diagnostics icons: `info.svg`, `success.svg`, `warning.svg`, `error.svg`, and `missing-script.svg`.

### Animation Timeline - 16 Icons

`animation_timeline` owns animation transport, track identity, key editing, and curve/dope-sheet controls.

`animation_timeline/transport` contains 5 timeline controls: `timeline-play.svg`, `timeline-pause.svg`, `timeline-stop.svg`, `step-prev.svg`, and `loop.svg`.

`animation_timeline/tracks` contains 4 track-type icons: `transform-track.svg`, `bone-track.svg`, `audio-track.svg`, and `event-track.svg`.

`animation_timeline/keys` contains 4 key editing icons: `keyframe.svg`, `add-keyframe.svg`, `delete-keyframe.svg`, and `key-selected.svg`.

`animation_timeline/curves` contains 3 curve-view icons: `curve-linear.svg`, `curve-bezier.svg`, and `dope-sheet.svg`.

### Graph Editor - 20 Icons

`graph_editor` owns visual scripting, state/transition graph, pin, execution-flow, and shader graph concepts.

`graph_editor/nodes` contains 7 node identity icons: `event-node.svg`, `function-node.svg`, `variable-node.svg`, `comment-node.svg`, `state-node.svg`, `transition-node.svg`, and `shader-node.svg`.

`graph_editor/pins` contains 5 pin and routing icons: `input-pin.svg`, `output-pin.svg`, `exec-pin.svg`, `data-pin.svg`, and `reroute-pin.svg`.

`graph_editor/execution` contains 4 execution/debug flow icons: `sequence.svg`, `branch.svg`, `breakpoint.svg`, and `debug-pulse.svg`.

`graph_editor/shader` contains 4 shader graph icons: `material-output.svg`, `texture-sample.svg`, `parameter.svg`, and `preview-node.svg`.

### UI Layout Editor - 16 Icons

`ui_layout_editor` owns retained UI layout tooling, widget identity, and constraint controls.

`ui_layout_editor/layout` contains 6 layout-surface icons: `canvas.svg`, `artboard.svg`, `stack.svg`, `grid.svg`, `absolute-layout.svg`, and `overlay-layout.svg`.

`ui_layout_editor/widgets` contains 5 widget icons: `widget.svg`, `text.svg`, `image.svg`, `button.svg`, and `slider.svg`.

`ui_layout_editor/constraints` contains 5 constraint and spacing icons: `anchor.svg`, `constraint.svg`, `margin.svg`, `padding.svg`, and `align-center.svg`.

### Console Profiler - 14 Icons

`console_profiler` owns logs, diagnostics/debug surfaces, and performance profiler panels.

`console_profiler/logs` contains 5 log controls and severities: `log-info.svg`, `log-warning.svg`, `log-error.svg`, `clear-logs.svg`, and `filter-logs.svg`.

`console_profiler/diagnostics` contains 4 debug/diagnostic icons: `breakpoint.svg`, `watch.svg`, `callstack.svg`, and `crash-report.svg`.

`console_profiler/profiling` contains 5 performance counters and views: `cpu.svg`, `gpu.svg`, `memory.svg`, `frame-time.svg`, and `frame-graph.svg`.

### Build Plugins - 23 Icons

`build_plugins` owns build pipeline, package/archive, deploy target, plugin/module, marketplace, and source-control operations.

`build_plugins/build` contains 5 build pipeline icons: `build.svg`, `rebuild.svg`, `test.svg`, `build-settings.svg`, and `pipeline.svg`.

`build_plugins/package` contains 4 packaging icons: `package.svg`, `archive.svg`, `signing.svg`, and `artifact.svg`.

`build_plugins/deploy` contains 4 deployment icons: `deploy.svg`, `device.svg`, `cloud-upload.svg`, and `install.svg`.

`build_plugins/plugins` contains 5 plugin lifecycle icons: `plugin.svg`, `module.svg`, `enable-plugin.svg`, `disable-plugin.svg`, and `marketplace.svg`.

`build_plugins/source_control` contains 5 source-control icons: `branch.svg`, `commit.svg`, `merge.svg`, `push.svg`, and `conflict.svg`.

## Milestone Coverage

Milestone A generated the immediate Editor surfaces: `workbench`, `scene_viewport`, and `hierarchy`. The focused A gate expected 61 SVG files and required no external references and no non-ASCII content.

Milestone B generated asset and inspector workflows: `asset_browser` and `inspector`. The focused B gate expected 54 SVG files and required no external references and no non-ASCII content.

Milestone C generated secondary tool pages: `animation_timeline`, `graph_editor`, `ui_layout_editor`, `console_profiler`, and `build_plugins`. The focused C gate expected 89 SVG files and required no external references and no non-ASCII content.

Milestone D documents the complete taxonomy and reruns the whole-pack acceptance gate. The full inventory is 204 SVG files, equal to A 61 + B 54 + C 89.

## Integration Boundary

This pack does not currently participate in retained-host template rendering. No `*.v2.ui.toml` template, `Icon`, `IconButton`, runtime asset registry, atlas generator, or Rust module is changed by this documentation milestone. Future UI wiring should be handled as a separate behavior change with its own validation for retained-host template loading, fallback behavior, rendering, atlas packing if used, and page-specific icon lookup rules.

Until such wiring exists, these icons are validated as repository assets only. Their contract is path stability, SVG self-containment, ASCII content, and documented ownership.

## Validation Evidence

Milestone D used PowerShell/.NET fallback scans because `rg` was not available in the local PowerShell environment.

| Check | Scope | Result | Evidence |
| --- | --- | --- | --- |
| Full asset inventory | `zircon_editor/assets/icons/editor_pages/**/*.svg` | pass | PowerShell fallback counted 204 SVG files. |
| A focused inventory | `workbench`, `scene_viewport`, `hierarchy` | pass | Current tree contains 61 SVG files across the A directories. |
| B focused inventory | `asset_browser`, `inspector` | pass | Current tree contains 54 SVG files across the B directories. |
| C focused inventory | `animation_timeline`, `graph_editor`, `ui_layout_editor`, `console_profiler`, `build_plugins` | pass | Current tree contains 89 SVG files across the C directories. |
| External-reference scan | `zircon_editor/assets/icons/editor_pages` | pass | No files reported `href=`, `<image`, or `url(http` matches. |
| ASCII scan | `zircon_editor/assets/icons/editor_pages` | pass | No generated SVG file contained bytes greater than `0x7f`. |

The design spec remains unchanged because the final inventory matches the approved 204-icon list and taxonomy.

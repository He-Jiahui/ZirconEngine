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
  - docs/zircon_editor/assets/editor-shell-svg-resources.md
  - docs/zircon_editor/assets/engine-style-svg-resources.md
tests:
  - asset inventory: expected 204 SVG files under zircon_editor/assets/icons/editor_pages
  - external-reference scan: no href, raster image, or remote url references under zircon_editor/assets/icons/editor_pages
  - ASCII scan: no non-ASCII content under zircon_editor/assets/icons/editor_pages
doc_type: milestone-detail
---

# Editor Page Function SVG Icon Design

## Goal

Expand the Editor SVG icon set with page-owned, function-classified icons that cover the main workbench, asset and inspector workflows, and secondary tool pages. The icons are inspired by the vocabulary of Fyrox, Unity, and Unreal editor surfaces, but they are Zircon-owned vector drawings and do not copy vendor artwork.

## Existing Context

The repository already has three relevant icon sources:

- `zircon_editor/assets/icons/ionicons` provides generic Ionicons used by current retained-host templates.
- `zircon_editor/assets/icons/zircon_editor_shell` covers earlier shell chrome such as toolbar, activity rail, scene, inspector, controls, status, and viewport overlays.
- `zircon_editor/assets/icons/zircon_engine_style` covers broader engine concepts such as assets, scene components, graphs, tools, runtime, and build concepts.

This design adds a new page/function tree rather than extending the older generic categories. That keeps the new assets easy to map to Editor screens and later sprite-atlas groups.

## Reference Vocabulary

Fyrox leads the page split because it is a Rust-native editor/runtime reference with scene, asset, inspector, animation, and graph-style tooling. Unity leads the functional vocabulary for Scene view controls, Project/Inspector workflows, timeline/animation, package/build surfaces, and material channel conventions. Unreal leads scale-oriented secondary tools: content browser, blueprints, behavior graphs, profiling, source control, plugin/module, package, and deploy workflows.

The design translates shared concepts from those references into Zircon names. It does not preserve vendor filenames, icon silhouettes, or exact visual treatment.

## Resource Layout

All new icons live under `zircon_editor/assets/icons/editor_pages`, grouped first by Editor page and then by function:

- `workbench/menu`, `workbench/dock`, `workbench/tabs`, and `workbench/status` for global Editor shell actions.
- `scene_viewport/tools`, `scene_viewport/snapping`, `scene_viewport/camera`, `scene_viewport/display`, and `scene_viewport/play` for the main viewport toolbar and overlay controls.
- `hierarchy/entity`, `hierarchy/component`, and `hierarchy/row_state` for scene tree rows and row affordances.
- `asset_browser/navigation`, `asset_browser/asset_types`, `asset_browser/import_pipeline`, and `asset_browser/references` for project/resource browsing.
- `inspector/sections`, `inspector/properties`, `inspector/material_channels`, and `inspector/validation` for component sections, editable properties, and diagnostics.
- `animation_timeline/transport`, `animation_timeline/tracks`, `animation_timeline/keys`, and `animation_timeline/curves` for animation authoring.
- `graph_editor/nodes`, `graph_editor/pins`, `graph_editor/execution`, and `graph_editor/shader` for visual scripting and material/shader graph editing.
- `ui_layout_editor/layout`, `ui_layout_editor/widgets`, and `ui_layout_editor/constraints` for retained UI layout tooling.
- `console_profiler/logs`, `console_profiler/diagnostics`, and `console_profiler/profiling` for console, debug, and performance views.
- `build_plugins/build`, `build_plugins/package`, `build_plugins/deploy`, `build_plugins/plugins`, and `build_plugins/source_control` for production pipeline pages.

## Initial Icon List

The first implementation should generate these 204 icons. The list is intentionally page-owned first, then function-owned, so future UI template wiring can resolve icons by screen context before falling back to generic engine concept packs.

### A. Main Editor Interface - 61 Icons

Workbench:

- `workbench/menu/command-palette.svg`, `quick-open.svg`, `open-project.svg`, `save-all.svg`, `undo-history.svg`
- `workbench/dock/dock-left.svg`, `dock-right.svg`, `dock-bottom.svg`, `split-horizontal.svg`, `reset-layout.svg`
- `workbench/tabs/pin-tab.svg`, `close-tab.svg`, `dirty-tab.svg`, `tab-overflow.svg`
- `workbench/status/notification.svg`, `background-task.svg`, `sync-state.svg`, `layout-warning.svg`

Scene viewport:

- `scene_viewport/tools/select.svg`, `translate.svg`, `rotate.svg`, `scale.svg`, `universal-transform.svg`, `measure-tool.svg`
- `scene_viewport/snapping/grid-snap.svg`, `angle-snap.svg`, `scale-snap.svg`, `vertex-snap.svg`, `snap-settings.svg`
- `scene_viewport/camera/camera-orbit.svg`, `camera-pan.svg`, `camera-zoom.svg`, `frame-selection.svg`, `perspective.svg`, `orthographic.svg`
- `scene_viewport/display/lit.svg`, `wireframe.svg`, `bounds.svg`, `grid-overlay.svg`, `gizmo-visibility.svg`
- `scene_viewport/play/play.svg`, `pause.svg`, `stop.svg`, `simulate.svg`, `step-frame.svg`

Hierarchy:

- `hierarchy/entity/scene.svg`, `entity.svg`, `empty.svg`, `prefab-instance.svg`, `camera-entity.svg`
- `hierarchy/component/transform-component.svg`, `mesh-renderer-component.svg`, `light-component.svg`, `script-component.svg`, `physics-component.svg`
- `hierarchy/row_state/visible.svg`, `hidden.svg`, `locked.svg`, `unlocked.svg`, `isolate.svg`, `warning-state.svg`

### B. Asset Browser And Inspector - 54 Icons

Asset browser:

- `asset_browser/navigation/folder.svg`, `favorite.svg`, `recent.svg`, `search.svg`, `filter.svg`, `sort-ascending.svg`, `refresh.svg`
- `asset_browser/asset_types/scene-file.svg`, `prefab.svg`, `mesh.svg`, `material.svg`, `texture.svg`, `shader.svg`, `audio-clip.svg`, `animation-clip.svg`, `script-file.svg`, `font.svg`, `particle-system.svg`
- `asset_browser/import_pipeline/import.svg`, `reimport.svg`, `export.svg`, `import-settings.svg`, `import-error.svg`
- `asset_browser/references/dependency.svg`, `reference.svg`, `missing-reference.svg`, `used-by.svg`, `bundle.svg`

Inspector:

- `inspector/sections/transform.svg`, `rendering.svg`, `physics.svg`, `audio.svg`, `script.svg`, `animation.svg`, `ui.svg`, `material.svg`
- `inspector/properties/add-component.svg`, `reset-property.svg`, `override-property.svg`, `revert-override.svg`, `link.svg`, `unlink.svg`, `keyframe-property.svg`
- `inspector/material_channels/albedo.svg`, `normal.svg`, `metallic.svg`, `roughness.svg`, `emissive.svg`, `opacity.svg`
- `inspector/validation/info.svg`, `success.svg`, `warning.svg`, `error.svg`, `missing-script.svg`

### C. Secondary Tool Pages - 89 Icons

Animation timeline:

- `animation_timeline/transport/timeline-play.svg`, `timeline-pause.svg`, `timeline-stop.svg`, `step-prev.svg`, `loop.svg`
- `animation_timeline/tracks/transform-track.svg`, `bone-track.svg`, `audio-track.svg`, `event-track.svg`
- `animation_timeline/keys/keyframe.svg`, `add-keyframe.svg`, `delete-keyframe.svg`, `key-selected.svg`
- `animation_timeline/curves/curve-linear.svg`, `curve-bezier.svg`, `dope-sheet.svg`

Graph editor:

- `graph_editor/nodes/event-node.svg`, `function-node.svg`, `variable-node.svg`, `comment-node.svg`, `state-node.svg`, `transition-node.svg`, `shader-node.svg`
- `graph_editor/pins/input-pin.svg`, `output-pin.svg`, `exec-pin.svg`, `data-pin.svg`, `reroute-pin.svg`
- `graph_editor/execution/sequence.svg`, `branch.svg`, `breakpoint.svg`, `debug-pulse.svg`
- `graph_editor/shader/material-output.svg`, `texture-sample.svg`, `parameter.svg`, `preview-node.svg`

UI layout editor:

- `ui_layout_editor/layout/canvas.svg`, `artboard.svg`, `stack.svg`, `grid.svg`, `absolute-layout.svg`, `overlay-layout.svg`
- `ui_layout_editor/widgets/widget.svg`, `text.svg`, `image.svg`, `button.svg`, `slider.svg`
- `ui_layout_editor/constraints/anchor.svg`, `constraint.svg`, `margin.svg`, `padding.svg`, `align-center.svg`

Console and profiler:

- `console_profiler/logs/log-info.svg`, `log-warning.svg`, `log-error.svg`, `clear-logs.svg`, `filter-logs.svg`
- `console_profiler/diagnostics/breakpoint.svg`, `watch.svg`, `callstack.svg`, `crash-report.svg`
- `console_profiler/profiling/cpu.svg`, `gpu.svg`, `memory.svg`, `frame-time.svg`, `frame-graph.svg`

Build and plugins:

- `build_plugins/build/build.svg`, `rebuild.svg`, `test.svg`, `build-settings.svg`, `pipeline.svg`
- `build_plugins/package/package.svg`, `archive.svg`, `signing.svg`, `artifact.svg`
- `build_plugins/deploy/deploy.svg`, `device.svg`, `cloud-upload.svg`, `install.svg`
- `build_plugins/plugins/plugin.svg`, `module.svg`, `enable-plugin.svg`, `disable-plugin.svg`, `marketplace.svg`
- `build_plugins/source_control/branch.svg`, `commit.svg`, `merge.svg`, `push.svg`, `conflict.svg`

## Milestone Order

### A. Main Editor Interface

Generate `workbench`, `scene_viewport`, and `hierarchy` icons first. This pass covers the surfaces users see immediately: command/menu actions, docking and tabs, viewport transform/display/play controls, and scene hierarchy row states.

### B. Asset Browser And Inspector

Generate `asset_browser` and `inspector` icons second. This pass covers resource navigation, import/reimport, dependency/reference states, common asset types, inspector section categories, property operations, material channels, and validation states.

### C. Secondary Tool Pages

Generate `animation_timeline`, `graph_editor`, `ui_layout_editor`, `console_profiler`, and `build_plugins` icons third. This pass covers deeper authoring surfaces and production tooling after the main workflow vocabulary is in place.

## Style Contract

Each icon is a self-contained ASCII SVG with a `24x24` view box. Icons use dark-editor friendly strokes and compact filled accents so they remain readable at 16 to 24 px sizes. Neutral structure uses cool gray strokes, active tool states use Zircon cyan, warning or lighting concepts use amber, destructive/error/stop states use red, and graph or secondary-tool highlights may use violet where it improves category distinction.

SVG files must not reference raster images, external symbols, remote URLs, CSS files, web fonts, or vendor icon packs. Gradients are allowed only when they remain inline and are necessary for category clarity; most icons should use simple paths, lines, circles, rects, and polygons.

## Integration Boundary

This milestone generates and documents assets only. It does not replace current `Icon` or `IconButton` values in retained-host templates. Wiring the new paths into `*.v2.ui.toml` or atlas generation is a follow-up behavior change that should carry separate validation for retained-host rendering and GPU/UI atlas behavior.

## Testing And Acceptance

The implementation is accepted when:

- The `editor_pages` tree contains the requested A, B, and C page/function directories.
- Every listed SVG file exists and is self-contained.
- Inventory counts match the 204-icon implementation plan.
- A scan confirms no `href`, `<image>`, or remote `url(http...)` references exist in the generated icons.
- A scan confirms generated SVG content stays ASCII.
- `docs/zircon_editor/assets/editor-page-function-svg-resources.md` documents the page/function taxonomy, reference routing, style contract, generated categories, and validation evidence.

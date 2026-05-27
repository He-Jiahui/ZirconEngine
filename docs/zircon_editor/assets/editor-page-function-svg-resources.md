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
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
implementation_files:
  - zircon_editor/assets/icons/editor_pages/**
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
  - docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md
  - docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
plan_sources:
  - user: 2026-05-20 generate more Editor SVG icons, reference Fyrox, Unity, Unreal main and secondary editor interfaces
  - docs/superpowers/specs/2026-05-20-editor-page-function-svg-icons-design.md
  - docs/superpowers/plans/2026-05-20-editor-page-function-svg-icons.md
  - user: 2026-05-21 continue improving Editor SVGs; choose A + C: polish existing icons, then prepare UI mapping
  - docs/superpowers/specs/2026-05-21-editor-svg-polish-ui-mapping-design.md
  - docs/superpowers/plans/2026-05-21-editor-svg-polish-ui-mapping.md
  - user: 2026-05-23 wire production Editor templates to accepted editor_pages icon mappings
  - docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md
  - docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md
  - user: 2026-05-25 complete live Editor visual rendering and 16px readability validation for wired editor_pages icons
tests:
  - asset inventory: 204 SVG files under zircon_editor/assets/icons/editor_pages
  - page-group inventory: A 61, B 54, C 89 under zircon_editor/assets/icons/editor_pages
  - external-reference scan: no href/image/remote-url references under zircon_editor/assets/icons/editor_pages
  - ASCII scan: no non-ASCII content under zircon_editor/assets/icons/editor_pages
  - template wiring validation: 39 direct/near rows use expected editor_pages paths, referenced SVGs exist, and 7 gaps remain unchanged
  - machine-readable header check for docs/zircon_editor/assets/editor-page-function-svg-resources.md, docs/zircon_editor/assets/editor-page-function-icon-template-map.md, and docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md
  - scoped git status check for production templates, editor_pages icons, wiring docs, spec, and plan
  - repository asset test: cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture passed with 1 test, 0 failures, and 1510 filtered on the closeout rerun
  - retained-host 16px readability test: cargo test -p zircon_editor editor_pages_template_icons_have_readable_16px_raster_footprints --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture
  - retained-host screenshot gate: cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --ignored --nocapture
  - live editor build: cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation
  - live editor screenshot: target/visual-layout/editor-live-window-900x620.png
doc_type: module-detail
---

# Editor Page Function SVG Resources

## Scope

`zircon_editor/assets/icons/editor_pages` is a Zircon-owned, page-first SVG resource tree for Editor page and function icons. It contains 204 generated SVG files arranged by owning Editor surface before functional category. The taxonomy exists so future retained-host template wiring, icon registry work, or sprite-atlas generation can resolve icons by screen context instead of relying on older generic icon packs.

The original asset pass was asset-only. The 2026-05-23 template wiring follow-up now references accepted `direct` and `near` paths from production Editor templates, while runtime icon lookup, atlas generation, and existing icon packs such as `ionicons`, `zircon_editor_shell`, or `zircon_engine_style` remain unchanged. The later visual validation pass adds retained-host readability coverage and live Editor build evidence without changing the SVG inventory or gap-row policy.

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

## Polish And UI Mapping Follow-Up

The 2026-05-21 polish and mapping pass kept the `editor_pages` inventory stable at 204 SVG files. The page-group distribution remains A 61, B 54, and C 89, so `docs/zircon_editor/assets/icon-resource-audit.md` remains inventory-stable and was not refreshed.

One SVG was edited during the polish pass:

| Path | Reason |
| --- | --- |
| `zircon_editor/assets/icons/editor_pages/build_plugins/build/build-settings.svg` | Replaced the previous slider-like settings duplicate with a build/config panel plus gear-like glyph, making it visually distinct from `zircon_editor/assets/icons/editor_pages/graph_editor/shader/parameter.svg` while preserving the same path and build-settings role. |

The 2026-05-21 advisory UI mapping was recorded at `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`. It covered 46 production Editor template icon usages in scope without changing `*.v2.ui.toml`, `.zui`, Rust source, icon registry, atlas, or runtime lookup behavior; the later 2026-05-23 follow-up below wires only accepted production template rows.

Mapping confidence counts after review:

| Confidence | Count |
| --- | ---: |
| `direct` | 22 |
| `near` | 17 |
| `gap` | 7 |

Remaining `gap` mappings should not be wired to `editor_pages` until a future coverage or UI-design pass resolves the missing role. The current gaps are:

| Template path | Control id or local node | Current icon/value | Gap reason |
| --- | --- | --- | --- |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetTransformSpace` / `set_transform_space` | `resize-outline` | Requires local/world transform-space icon coverage or a more specific transform-space UI decision. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `SetPreviewSkybox` / `set_preview_skybox` | `cloud-outline` | Requires skybox, cloud, or environment icon coverage. |
| `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml` | `AlignView` / `align_view` | `navigate-outline` | Requires view-align or axis-alignment icon coverage. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `SelectItem` / `select_item` | `cube-outline` | Requires a generic asset item or selected-asset role, or a future decision to specialize this control by asset type. |
| `zircon_editor/assets/ui/editor/host/asset_surface_controls.v2.ui.toml` | `SetViewMode` / `view_mode` | `list-outline` | Requires asset browser list/grid view-mode icon coverage. |
| `zircon_editor/assets/ui/editor/host/pane_surface_controls.v2.ui.toml` | `TriggerAction` / `trigger_action` | `flash-outline` | Requires a generic flash/action icon only if this placeholder-like pane action becomes a stable page/function role. |
| `zircon_editor/assets/ui/editor/host/startup_welcome_controls.v2.ui.toml` | `CreateProject` / `create` | `add-circle-outline` | Requires new-project or project-create icon coverage. |

## Template Wiring Follow-Up

The 2026-05-23 template wiring pass converted the prior advisory mapping into production template metadata for accepted rows. It wires all 39 `direct` and `near` rows with repository-icon-root-relative `editor_pages/...svg` values and leaves all 7 `gap` rows unchanged.

The follow-up is driven by `docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md` and `docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md`. It does not add, remove, rename, or rewrite any SVG file, so the inventory remains 204 files with A 61, B 54, and C 89.

Current template wiring status:

| Status | Count | Template behavior |
| --- | ---: | --- |
| Wired `direct` rows | 22 | Production templates use the matching `editor_pages/...svg` path. |
| Wired `near` rows | 17 | Production templates use the accepted close semantic `editor_pages/...svg` path. |
| Unwired `gap` rows | 7 | Production templates keep the existing generic icon names until future coverage exists. |

Validation for this follow-up reruns the template wiring gate, whole-pack inventory, page-group distribution, forbidden-construct scan, ASCII scan, documentation-header check, scoped git status, and the scoped `zircon_editor` repository asset test.

## Live Visual Validation Follow-Up

The live visual validation pass closes the static-wiring blind spot without changing the icon pack. `zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs` now contains `editor_pages_template_icons_have_readable_16px_raster_footprints`, a retained-host painter regression that rasterizes the unique wired `editor_pages` template icon paths at 16 x 16 px through the same template icon path used by production chrome. It rejects missing icons, blank rasters, collapsed footprints smaller than 6 x 6 px, and full-slot silhouettes that would read as solid blocks instead of icons.

The existing ignored screenshot gate `capture_m3_gui_acceptance_visual_artifacts` generated retained-host artifacts under `target/visual-layout`, including `editor-window-m3-svg-icon-scale-small-640x420.png` and `editor-window-m3-svg-icon-scale-large-1260x780.png`. A live Editor smoke capture then launched the built `zircon_editor.exe` and wrote `target/visual-layout/editor-live-window-900x620.png`; the OS window capture reported title `Zircon Editor`, actual PNG size `1296 x 759`, and `86492` bytes.

The live binary build required two build-boundary corrections outside the icon pack itself. `zircon_runtime/src/lib.rs` now re-exports the manifest-specific runtime-profile helper APIs already owned by `zircon_runtime::builtin`, and `zircon_app/src/entry/builtin_modules.rs` clones the optional project manifest before the feature-registration resolver path consumes it. These fixes keep app bootstrap on the existing profile/provider architecture while making the editor-host binary compile for live validation.

## Integration Boundary

This pack now participates in production retained-host template metadata for accepted `direct` and `near` mappings. Runtime asset registries and atlas generation remain unchanged by the wiring pass. The visual validation follow-up adds test/build evidence around the existing retained-host icon resolver, but it does not introduce a new runtime asset registry, atlas path, or page-specific lookup rule.

The icon pack contract remains path stability, SVG self-containment, ASCII content, and documented ownership.

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

Milestone 3 final validation reran the whole-pack gate after the polish and mapping documentation update.

| Check | Scope | Result | Evidence |
| --- | --- | --- | --- |
| Full asset inventory | `zircon_editor/assets/icons/editor_pages/**/*.svg` | pass | Fresh validation counted 204 SVG files. |
| Page-group inventory | A `workbench`, `scene_viewport`, `hierarchy`; B `asset_browser`, `inspector`; C `animation_timeline`, `graph_editor`, `ui_layout_editor`, `console_profiler`, `build_plugins` | pass | Fresh validation counted A 61, B 54, and C 89 SVG files. |
| Forbidden construct scan | `zircon_editor/assets/icons/editor_pages` | pass | Fresh validation reported no `<image`, `href=`, `<use`, `<symbol`, `<script`, `<style`, `style=`, `class=`, `@font`, `font-family`, or remote `url(http` matches. |
| ASCII scan | `zircon_editor/assets/icons/editor_pages` | pass | Fresh validation reported no SVG bytes greater than `0x7f`. |
| Header check | `docs/zircon_editor/assets/editor-page-function-svg-resources.md`; `docs/zircon_editor/assets/editor-page-function-icon-template-map.md` | pass | Both documents start with machine-readable YAML headers containing `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`. |
| UI template status check | `zircon_editor/assets/ui/editor` | pass | Scoped git status reported no modified UI template files for this plan. |

The design spec remains unchanged because the final inventory matches the approved 204-icon list and taxonomy.

Milestone 2 final validation reran the wiring and asset gate after production template wiring.

Commands rerun for this gate:

| Command | Purpose |
| --- | --- |
| PowerShell template-map script over the 46 production rows from `docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md` | Confirm 39 expected wired rows, referenced SVG existence, 22 `direct`, 17 `near`, and 7 unchanged `gap` rows. |
| PowerShell `Get-ChildItem -Recurse -Filter *.svg` under `zircon_editor/assets/icons/editor_pages` | Confirm full SVG inventory and A/B/C group counts. |
| PowerShell forbidden-construct scan for `<image`, `href=`, `<use`, `<symbol`, `<script`, `<style`, `style=`, `class=`, `@font`, `font-family`, and `url(http` | Confirm SVG files remain self-contained and registry/atlas-safe. |
| PowerShell byte scan over `zircon_editor/assets/icons/editor_pages/**/*.svg` | Confirm SVG files remain ASCII-only. |
| PowerShell YAML-header check for the two asset docs and the 2026-05-23 wiring design spec | Confirm machine-readable headers contain `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`. |
| `git status --short -- zircon_editor/assets/ui/editor zircon_editor/assets/icons/editor_pages docs/zircon_editor/assets/editor-page-function-icon-template-map.md docs/zircon_editor/assets/editor-page-function-svg-resources.md docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md` | Confirm intended docs and pre-existing Milestone 1 paths against unrelated dirty workspace state. |
| `cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture` | Attempt scoped repository asset test after static asset/template checks. |

| Check | Scope | Result | Evidence |
| --- | --- | --- | --- |
| Template wiring validation | 46 mapped production rows | pass | Validation confirmed 39 wired `direct`/`near` rows use expected `editor_pages/...svg` values, every referenced SVG exists under `zircon_editor/assets/icons`, and all 7 `gap` rows retain their original generic icon values. |
| Full asset inventory | `zircon_editor/assets/icons/editor_pages/**/*.svg` | pass | Fresh validation counted 204 SVG files. |
| Page-group inventory | A `workbench`, `scene_viewport`, `hierarchy`; B `asset_browser`, `inspector`; C `animation_timeline`, `graph_editor`, `ui_layout_editor`, `console_profiler`, `build_plugins` | pass | Fresh validation counted A 61, B 54, and C 89 SVG files. |
| Forbidden construct scan | `zircon_editor/assets/icons/editor_pages` | pass | Fresh validation reported no `<image`, `href=`, `<use`, `<symbol`, `<script`, `<style`, `style=`, `class=`, `@font`, `font-family`, or remote `url(http` matches. |
| ASCII scan | `zircon_editor/assets/icons/editor_pages` | pass | Fresh validation reported no SVG bytes greater than `0x7f`. |
| Header check | `docs/zircon_editor/assets/editor-page-function-icon-template-map.md`; `docs/zircon_editor/assets/editor-page-function-svg-resources.md`; `docs/superpowers/specs/2026-05-23-editor-pages-template-icon-wiring-design.md` | pass | All three documents start with machine-readable YAML headers containing `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`. |
| Scoped git status check | `zircon_editor/assets/ui/editor`; `zircon_editor/assets/icons/editor_pages`; wiring docs, spec, and plan | reviewed | Broad scoped status reported intended dirty template/docs/icon-polish files for this wiring/doc audit. It also surfaced pre-existing unrelated `zircon_editor/assets/ui/editor/material_components/*.zui` dirty files under the broad UI path; those Material component lab files are out of scope for this pass, were not modified by this wiring pass, and are not intended changes. |
| Repository asset test | `zircon_editor` library `repository_assets` tests | pass | `cargo test -p zircon_editor --lib repository_assets --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture` passed with 1 test, 0 failures, and 1510 filtered on the closeout rerun. |

Visual validation evidence added after the static wiring gate:

| Check | Scope | Result | Evidence |
| --- | --- | --- | --- |
| 16px retained-host readability | 29 unique wired `editor_pages` template icon paths | pass | `cargo test -p zircon_editor editor_pages_template_icons_have_readable_16px_raster_footprints --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture` passed on the closeout rerun with 1 test, 0 failures, and 1510 filtered; it printed per-icon `ICON_16PX_READABILITY` footprints. |
| Retained-host screenshot artifact gate | M3 workbench, asset browser, assets drawer, menu popup SVG icons, and small/large SVG icon scaling screenshots | pass | `cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --ignored --nocapture` passed in the earlier visual pass and wrote PNGs under `target/visual-layout`. |
| Live editor-host binary build | `zircon_app` editor-host executable | pass | `cargo build -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation` passed on the closeout rerun after the app/runtime build-boundary fixes. |
| Live Editor native window capture | `target/visual-layout/editor-live-window-900x620.png` | captured | The live capture reported title `Zircon Editor`, actual PNG dimensions `1296 x 759`, and `86492` bytes. |

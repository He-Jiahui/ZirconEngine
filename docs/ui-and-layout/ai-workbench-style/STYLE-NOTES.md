# AI Workbench Layout Reference Notes

These raster references are layout-first design drafts. Use them to judge the overall interaction structure: main editor tabs, JetBrains-style dock drawers, central workspace ownership, bottom output/timeline placement, and the balance between left and right tool windows.

Do not treat AI-generated text or tiny control details as implementation targets. Final UI chrome, labels, controls, dimensions, and exportable review PNGs remain owned by deterministic HTML/CSS work: the PNG export corpus under `tools/editor-workbench-preview/` and the responsive interactive prototype under `docs/ui-and-layout/ai-workbench-style/prototype/`.

Interactive HTML/CSS prototype:

- `prototype/index.html` - semantic shell for the normalized workbench reference.
- `component-prototype/index.html` - component-stack recreation of `../workbench.png` with extracted atoms, collections, surfaces, responsive rules, and pixel audit tooling.
- `prototype/base.css` - design tokens and global element rules.
- `prototype/shell.css` - top bar, main tabs, rail, dock grid, and status bar.
- `prototype/panels.css` - drawer panels, tables, fields, bottom output, and overlays.
- `prototype/stages.css` - CSS-generated scene, material graph, UI, montage, assets, diagnostics, and project surfaces.
- `prototype/responsive.css` - medium and compact viewport behavior.
- `prototype/page-data.js` - declarative page, drawer, bottom-pane, and tool data.
- `prototype/stage-renderers.js` - CSS-generated center surfaces for each page type.
- `prototype/view-utils.js` - shared HTML escaping helpers.
- `prototype/app.js` - plain JavaScript page routing, drawer switching, tool selection, layout presets, dock-preview state, selected layout-region state, and overlay state.
- `prototype/README.md` - Rust UI migration mapping and style normalization notes.

The prototype deliberately uses CSS-generated surfaces instead of embedding the AI PNGs as UI. It keeps the AI references useful for layout intent while making proportions, panels, controls, active states, responsive behavior, and dock-slot sizing explicit enough to migrate into retained Rust UI windows and a Taffy layout tree.

Use `ai-workbench-web-framework.png` as the current visual authority for the prototype: near-black top chrome, low-contrast dark panels, subtle blue-green vertical gradients, teal selected rows/buttons, wider left scene and right inspector docks, a large central scene surface, bottom console/timeline area, and a thin status bar. Taffy inspection UI should remain available but optional so the design still reads like the reference workbench by default.

Current structure references:

- `ai-workbench-web-framework.png` - shared workbench shell and color direction.
- `ai-scene-editor-layout.png` - scene editor layout with viewport, placement, hierarchy, inspector, and output drawers.
- `ai-material-editor-layout.png` - material editor layout with graph canvas, node palette, properties, and shader output drawers.
- `ai-material-editor-workbench.png` - material editor tab with graph-focused center and property drawers.
- `ai-montage-editor-layout.png` - montage editor layout with preview viewport, animation list, skeleton, and timeline drawers.
- `ai-montage-editor-workbench.png` - animation montage tab with timeline-first bottom drawer.
- `ai-asset-browser-layout.png` - asset browser layout with filters, folder tree, dependency, metadata, and import output drawers.
- `ai-asset-browser-workbench.png` - asset browser tab with dense grid/list organization.
- `ai-ui-asset-editor-layout.png` - UI asset editor layout with widget/tool drawers.
- `ai-console-diagnostics-layout.png` - diagnostics and console layout with log/detail drawers.
- `ai-project-overview-layout.png` - project overview layout with dashboard, project tree, status, and activity drawers.
- `ai-prefab-editor-layout.png` - prefab editor layout with placement tools, nested hierarchy, inspector, and validation drawers.
- `ai-vfx-editor-layout.png` - VFX editor layout with emitter palette, system stack, particle preview, timeline, and compile drawers.
- `ai-shader-editor-layout.png` - shader editor layout with code/preview split, source tree, symbols, and compiler output drawers.
- `ai-terrain-editor-layout.png` - terrain editor layout with sculpt shelf, asset tree, world regions, brush settings, and bake output drawers.
- `ai-behavior-tree-layout.png` - behavior tree layout with node palette, AI assets, runtime outline, node details, and debug drawers.
- `ai-sequencer-layout.png` - sequencer layout with track tools, cinematic viewport, bound actors, key details, and wide timeline drawer.
- `ai-lighting-bake-layout.png` - lighting bake layout with bake presets, light/probe hierarchy, bake settings, and progress output drawers.
- `ai-physics-collision-layout.png` - physics/collision layout with collider tools, body hierarchy, collision properties, and contact/debug drawers.
- `ai-level-streaming-layout.png` - level streaming layout with world cells, level tree, loaded-cell hierarchy, streaming rules, and event output drawers.
- `ai-navmesh-ai-layout.png` - navmesh/AI layout with navigation tools, nav data tree, tile/agent structure, and path query output drawers.
- `ai-render-pipeline-layout.png` - render pipeline layout with pass palette, render graph, pass/resource outline, GPU metrics, and compile output drawers.
- `ai-data-table-layout.png` - data table layout with row/schema tools, data sources, schema outline, selected row details, and validation drawers.
- `ai-performance-layout.png` - performance profiler layout with capture tools, saved sessions, frame hierarchy, sample details, and timeline drawers.
- `ai-runtime-diagnostics-layout.png` - runtime diagnostics layout with watch tools, target sessions, runtime tree, live values, and console/event drawers.
- `ai-plugin-manager-layout.png` - plugin manager layout with category filters, plugin folders, dependency graph, selected plugin details, and install output drawers.
- `ai-build-export-layout.png` - build/export layout with build profiles, package outputs, build steps, profile settings, and cook/package log drawers.
- `ai-source-control-layout.png` - source control layout with changelist actions, depot tree, ownership/review state, diff details, and submit/merge output drawers.
- `ai-automation-report-layout.png` - automation report layout with test filters, suite tree, failure hierarchy, selected test details, and live worker output drawers.
- `ai-hud-editor-layout.png` - HUD editor layout with widget palette, UI asset tree, widget hierarchy, selected widget inspector, and validation drawers.
- `ai-menu-flow-layout.png` - menu flow layout with screen templates, screen tree, route/focus outline, transition details, and simulation output drawers.
- `ai-ui-binding-layout.png` - UI binding layout with binding tools, view-model tree, widget/binding outline, expression details, and validation drawers.
- `ai-accessibility-audit-layout.png` - accessibility audit layout with rule filters, screen tree, issue hierarchy, remediation details, and audit output drawers.
- `ai-font-atlas-layout.png` - font atlas layout with glyph tools, font families, glyph ranges, selected glyph metrics, and bake/coverage output drawers.
- `ai-icon-library-layout.png` - icon library layout with category filters, icon asset tree, usage references, selected icon details, and validation drawers.
- `ai-control-rig-layout.png` - control rig layout with rig tools, skeleton tree, rig hierarchy, selected control details, and solve/timeline drawers.
- `ai-motion-matching-layout.png` - motion matching layout with query tools, animation database tree, pose clusters, selected pose details, and match timeline drawers.
- `ai-blend-space-layout.png` - blend space layout with sample tools, animation clip tree, blend sample list, sample properties, and preview diagnostics drawers.
- `ai-retarget-layout.png` - retarget layout with chain mapping tools, source/target skeleton trees, bone mapping hierarchy, chain settings, and export queue drawers.
- `ai-pose-library-layout.png` - pose library layout with pose capture tools, pose collections, pose set hierarchy, selected pose metadata, and batch output drawers.
- `ai-animation-compression-layout.png` - animation compression layout with compression presets, animation tree, track hierarchy, error/memory details, and batch log drawers.
- `ai-foliage-editor-layout.png` - foliage editor layout with brush tools, foliage asset tree, biome/cluster hierarchy, selected foliage settings, and paint/scatter output drawers.
- `ai-scatter-editor-layout.png` - procedural scatter layout with rule tools, scatter asset sets, spawn rule stack, selected rule constraints, and generation/validation output drawers.
- `ai-volume-editor-layout.png` - volume editor layout with volume creation tools, volume profiles, overlap hierarchy, selected bounds/properties, and overlap/event output drawers.
- `ai-weather-editor-layout.png` - weather editor layout with weather presets, region/profile assets, layer/event stack, selected weather curves, and weather timeline/output drawers.
- `ai-post-process-layout.png` - post process layout with effect tools, LUT/camera profile assets, volume/effect hierarchy, selected effect scopes/settings, and compare/compile output drawers.
- `ai-particle-library-layout.png` - particle library layout with particle filters, VFX asset tree, emitter/module hierarchy, selected particle metadata, and import/compile output drawers.
- `ai-collision-proxy-layout.png` - collision proxy layout with proxy creation tools, physics asset tree, collider hierarchy, selected collider/channel details, and validation/contact output drawers.
- `ai-level-variant-layout.png` - level variant layout with variant actions, level/variant assets, override hierarchy, selected variant details, and diff/apply output drawers.
- `ai-gameplay-ability-layout.png` - gameplay ability layout with task palette, ability asset tree, graph outline, selected task details, and timeline/compile output drawers.
- `ai-gameplay-effect-layout.png` - gameplay effect layout with modifier tools, effect asset tree, modifier/execution hierarchy, selected modifier details, and simulation/validation output drawers.
- `ai-ai-perception-layout.png` - AI perception layout with sense tools, AI asset tree, perceived actor hierarchy, selected sense details, and perception timeline/debug output drawers.
- `ai-spawn-rules-layout.png` - spawn rules layout with rule tools, spawn asset tree, zone/condition hierarchy, selected rule details, and simulation/validation output drawers.
- `ai-gameplay-tags-layout.png` - gameplay tags layout with tag actions, tag source files, hierarchy/reference drawers, selected tag details, and validation/migration output drawers.
- `ai-save-data-layout.png` - save data layout with save slot tools, save file tree, object hierarchy, selected serialized fields, and diff/migration output drawers.
- `ai-world-state-layout.png` - world state layout with state layer tools, scenario assets, region/system hierarchy, selected key/value details, and state timeline/output drawers.
- `ai-telemetry-dashboard-layout.png` - telemetry dashboard layout with query tools, saved query sources, event/segment hierarchy, selected metric details, and query/raw-event output drawers.
- `ai-lobby-editor-layout.png` - lobby editor layout with lobby template tools, online asset tree, lobby/slot hierarchy, selected slot rules, and simulated lobby/network output drawers.
- `ai-matchmaking-editor-layout.png` - matchmaking editor layout with rule tools, playlist assets, queue hierarchy, selected rule details, and matchmaking simulation output drawers.

Style direction for future prompts: flat modern dark UI, near-black surfaces, simple teal active states, subtle iOS-like rounded rectangles, 1px separators, compact 28-32 px controls, minimal gradients, no glossy or 3D button treatment, and no marketing-page composition.

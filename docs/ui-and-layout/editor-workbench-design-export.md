---
related_code:
  - tools/editor-workbench-preview/design.html
  - tools/editor-workbench-preview/design.css
  - tools/editor-workbench-preview/design.js
  - tools/editor-workbench-preview/preview-sheet.js
  - tools/editor-workbench-preview/design-manifest.mjs
  - tools/editor-workbench-preview/export-options.mjs
  - tools/editor-workbench-preview/export-designs.mjs
  - tools/editor-workbench-preview/verify-designs.mjs
  - tools/editor-workbench-preview/verify-reference-negative-guard.mjs
  - tools/editor-workbench-preview/package.json
  - tools/editor-workbench-preview/server.mjs
  - docs/ui-and-layout/workbench.png
  - zircon_editor/assets/ui/editor/reference/workbench.png
implementation_files:
  - tools/editor-workbench-preview/design.html
  - tools/editor-workbench-preview/design.css
  - tools/editor-workbench-preview/design.js
  - tools/editor-workbench-preview/preview-sheet.js
  - tools/editor-workbench-preview/design-manifest.mjs
  - tools/editor-workbench-preview/export-options.mjs
  - tools/editor-workbench-preview/export-designs.mjs
  - tools/editor-workbench-preview/verify-designs.mjs
  - tools/editor-workbench-preview/verify-reference-negative-guard.mjs
  - tools/editor-workbench-preview/package.json
plan_sources:
  - user: 2026-05-29 implement Editor Workbench PNG Design Plan
tests:
  - npm --prefix tools/editor-workbench-preview run design:export
  - npm --prefix tools/editor-workbench-preview run design:verify
  - npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=sheet --no-sheet
  - npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=audio-mixer-workbench --no-sheet
  - npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=runtime-commands-workbench --no-sheet
  - npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=asset-migration-workbench,scene-diff-workbench,prefab-diff-workbench,performance-budget-workbench,memory-budget-workbench,dependency-cleanup-workbench,naming-rules-workbench,release-checklist-workbench --no-sheet
  - npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=gameplay-debugger-workbench,replay-timeline-workbench,network-packet-inspector-workbench,latency-map-workbench,input-trace-workbench,save-state-diff-workbench,repro-recorder-workbench,qa-triage-workbench --no-sheet
  - npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=sheet
  - node --check tools/editor-workbench-preview/preview-sheet.js
  - npm --prefix tools/editor-workbench-preview run design:verify:reference-negative
  - negative guard: confirm a scene-workbench.png runtime-reference override is ignored without the guard flag, then enabled with the guard flag to make direct byte comparison fail without modifying zircon_editor/assets/ui/editor/reference/workbench.png
doc_type: workflow-detail
---

# Editor Workbench Design Export

## Purpose

The editor workbench design export tool produces deterministic PNG reference drafts for the core editor UI. It uses `docs/ui-and-layout/workbench.png` as the density and scale reference, then explores a flatter JetBrains-style main-tab layout with tool drawers. UI chrome, labels, panels, and controls stay in HTML/CSS so the exported images remain readable and reproducible. The byte-exact editor runtime target is `zircon_editor/assets/ui/editor/reference/workbench.png`: it must remain a direct match for `docs/ui-and-layout/workbench.png`; the exported design corpus is a state-coverage and style-translation set, not 270 copies of that same screenshot.

This workflow is separate from the older interactive workbench preview. The original `index.html` / `app.js` path continues to preview draggable workbench layout behavior, while `design.html` / `design.js` renders fixed design compositions for screenshot export.

## Export Surface

`design-manifest.mjs` is the source of truth for deliverable PNG names and the fixed `1672x941` canvas size. It includes the initial core editor workbench batch plus the follow-up tool/window batch. `design.js` reads the `design` query parameter and renders either a full workbench composition, a focused modal-style panel detail, or the `sheet` overview for browser review. `preview-sheet.js` owns that `sheet` overview: it compresses every manifest design id into a six-column coverage grid so `preview-sheet.png` remains complete even as the fixed-size corpus grows.

`design.css` defines the revised editor workbench visual language:

- near-black root chrome and panel fills with flatter modern contrast;
- JetBrains-like top-level editor tabs for Scene Editor, Material Editor, Montage Editor, UI Asset Editor, Asset Browser, Diagnostics, and Project;
- drawer-style tool windows arranged as Left Top, Left Bottom, Right Top, Right Bottom, and Bottom zones;
- rounded rectangle controls, simple fills, 1px separators, and minimal/no shadow;
- teal active, selected, and focus states without heavy glow;
- deterministic editor text and component shapes;
- raster-like content only inside viewport scenery and asset thumbnails.

The second batch keeps the same main-tab and drawer system while covering tool and standalone window surfaces: Material Lab, UI Asset Editor, Animation/Montage, Performance, Runtime Diagnostics, Plugin Manager, Build Export, and Welcome/New Project. Their focused companion states enlarge the primary working area for review without using generated UI text or bitmap-only controls.

The layout-spec batch is intentionally more diagrammatic. It adds first-class `layout-spec` PNGs for the top-level editor tab model, the canonical five drawer zones, and editor-specific allocations for Scene, Material, Montage, and UI Asset authoring. These files are meant to guide implementation decisions when a new editor page needs to choose which tools live in left-top, left-bottom, right-top, right-bottom, or bottom drawers.

The state-spec batch captures interactive layout states that should be preserved when the real editor UI becomes dynamic: collapsed drawers, expanded inspection drawers, split editor panes, split bottom timeline/console panes, floating tool windows, and compact fallbacks. These specs keep the drawer role identity visible even when the geometry changes.

The content-spec batch drills into the interior of each major drawer type. It documents row density, control placement, tab usage, metrics, and action priorities for prefab placement, files, hierarchy, inspector, animation list, console, timeline, and asset grid surfaces.

The overlay-spec batch covers floating/editor windows that appear above the workbench: command palette, right-click context menu, main-tab overflow, asset picker, import wizard, project settings, destructive confirmation, and notification center. These use the same flat rounded rectangle language as drawers so transient windows do not drift into a separate visual system.

The expanded editor-page batch applies the same main-tab and drawer model to more engine tools: Prefab Editor, VFX Editor, Shader Editor, Terrain Editor, Audio Editor, Behavior Tree, Lighting Bake, Physics/Collision, Level Streaming, Sequencer, NavMesh/AI, Render Pipeline, Input Mapping, Data Table, Network Replication, and Localization. These pages are intended to show how new editor tools should reuse the shared shell instead of inventing page-specific chrome.

The production editor-page batch extends the same rule set to more day-to-day authoring and validation tools: Visual Script, State Machine, Skeleton Mesh, Texture Editor, Material Instance, Prefab Variant, Level Audit, and Test Runner. Each page keeps the main editor tab as the owner of the workflow, uses left drawers for palettes or source lists, right drawers for structure and detail, and the bottom drawer for compiler, validation, or test output.

The diagnostics/resource editor-page batch focuses on engineering investigation and release preparation: Frame Debugger, Memory Profiler, Asset Dependency, Reference Finder, Cook Package, Crash Session Replay, Log Analysis, and Automation Report. These pages keep dense tables, graphs, and output logs inside the same tab/drawer grammar so diagnostic tools do not become separate applications visually.

The infrastructure/project editor-page batch adds the shared editor workspaces that support daily team and project operation: Layout Manager, Theme Tokens, Command Center, Module Graph, Asset Validation, Hot Reload, Project History, and Task Board. These reinforce that project infrastructure tools should use the same top-level tab model as creative editor pages instead of opening disconnected admin-style screens.

The collaboration/release editor-page batch covers the team-facing and delivery-facing parts of the workbench: Source Control, Review Comments, Build Farm, Release Notes, Project Settings, Plugin Development, Remote Device, and Session Sync. These drafts keep review, submit, deployment, and shared-session states inside the same main-tab and drawer system instead of treating them as separate utilities.

The cinematic/animation editor-page batch extends the same shell to content production tools: Cutscene Editor, Dialogue Editor, Quest Editor, Camera Rig, Control Rig, Motion Matching, Facial Animation, and Blend Space. These pages make timeline-heavy, graph-heavy, and table-heavy creative tools share the same drawer roles and flat control styling as the core workbench.

The world-building/environment editor-page batch covers environment authoring tools that need both viewport context and dense rule editing: Foliage Editor, Scatter Editor, Volume Editor, Weather Editor, Post Process, Particle Library, Collision Proxy, and Level Variant. These keep brush presets, rule lists, effect stacks, and variant diffs in drawers around the main editor tab rather than opening separate full-screen utilities.

The gameplay/runtime editor-page batch covers gameplay systems that need both authoring and live inspection: Gameplay Ability, Gameplay Effect, AI Perception, Spawn Rules, Gameplay Tags, Save Data, World State, and Telemetry Dashboard. These pages keep runtime traces, validation tables, live counters, and state diffs inside the same workbench grammar as content tools.

The platform/online editor-page batch covers multiplayer and account-facing service tools: Lobby Editor, Matchmaking Editor, Server Browser, Replay Browser, Achievements Editor, Entitlements Editor, User Profile, and Online Diagnostics. These pages use the same main-tab plus drawer structure so session lists, queue simulation, server filters, replay markers, ownership checks, social state, and service traces stay close to their active editor context.

The UI/UX editor-page batch covers interface authoring tools that sit between runtime UI assets and product polish: HUD Editor, Menu Flow, Font Atlas, Icon Library, UI Binding, Accessibility Audit, Input Prompts, and UI Motion. These pages keep widget lists, screen routes, glyph ranges, icon usage, binding contracts, accessibility issues, device prompt glyphs, and motion curves inside flat dense workbench drawers.

The rendering/GPU editor-page batch covers graphics-side authoring and inspection tools: Shader Permutations, Render Target, GPU Profiler, Light Probes, Reflection Capture, Decal Editor, Virtual Texture, and Material Audit. These pages keep keyword matrices, attachment lists, pass timings, probe coverage, cubemap faces, decal projection rules, page residency, and material cost warnings in the same drawer grammar as scene and asset tools.

The audio/voice editor-page batch covers sound and dialogue tooling: Sound Cue, Audio Mixer, Music System, Audio Occlusion, Voice Bank, Subtitle Timing, Lip Sync, and Audio Profiler. These pages keep cue graphs, mixer routing, music transition states, occlusion traces, localization coverage, subtitle timing checks, viseme curves, and live voice counts in compact drawers around the active editor tab.

The physics/simulation editor-page batch covers simulation-heavy scene tools: Rigid Body, Physics Constraints, Destruction, Cloth Simulation, Vehicle Physics, Fluid Simulation, Rope Cable, and Physics Profiler. These pages keep body tables, joint limits, fracture clusters, cloth maps, wheel setup, solver metrics, cable attachments, and contact profiling in the same compact drawer layout.

The AI/navigation editor-page batch covers behavior design and runtime AI debugging: AI Director, Blackboard, EQS Query, Crowd Simulation, Smart Objects, Patrol Routes, Cover System, and AI Profiler. These pages keep threat budgets, runtime keys, query scoring, crowd flow, object reservations, waypoint routes, cover exposure, and behavior timings close to the scene context.

The asset pipeline/DCC editor-page batch covers production import and batch tooling: Mesh Import, LOD Chain, Redirect Map, Texture Compression Queue, Source Asset Trace, DCC Live Link, Metadata Editor, and Batch Process Queue. These pages keep importer settings, reduction levels, redirect references, compression jobs, source lineage, live DCC sync, metadata schemas, and worker queues in dense workbench drawers.

The engineering/production editor-page batch covers developer-facing production tools: Script Editor, API Browser, Plugin Packaging, Module Settings, Automation Suite, Build Config, Cook Rules, and Runtime Commands. These pages keep source files, symbol outlines, package manifests, module feature flags, automation agents, build targets, cook overrides, and command bindings inside the same main-tab workbench frame.

The project governance editor-page batch covers long-running maintenance and quality-control tools: Asset Migration, Scene Diff, Prefab Diff, Performance Budget, Memory Budget, Dependency Cleanup, Naming Rules, and Release Checklist. These pages keep migration batches, ownership diffs, prefab overrides, frame budgets, memory pools, unused assets, naming violations, and release gates in compact table-first drawers.

The runtime QA editor-page batch covers debugging and reproducibility tools: Gameplay Debugger, Replay Timeline, Network Packet Inspector, Latency Map, Input Trace, Save State Diff, Repro Recorder, and QA Triage. These pages keep live watches, replay markers, decoded packets, latency routes, input events, save deltas, repro artifacts, and QA ownership close to the active runtime context.

The graphics deep-dive editor-page batch covers renderer debugging and GPU analysis tools: Render Graph, Shader Debugger, Texture Streaming, Shadow Map, Occlusion Culling, Frame Compare, Material Layers, and GPU Memory. These pages keep pass graphs, shader variables, residency pages, cascades, visibility queries, frame deltas, material stacks, and heap allocations in the same compact drawer grammar.

The animation production editor-page batch covers animation authoring and runtime motion debugging tools: Retarget, IK Solver, Pose Library, Mocap Cleanup, Animation Compression, Root Motion, Event Tracks, and Montage Debugger. These pages keep skeleton chains, IK effectors, pose tags, mocap issues, compression errors, motion trajectories, notify tracks, and montage sections in the same main-tab layout.

The UI diagnostics editor-page batch covers runtime and design-system UI inspection tools: Widget Tree Debugger, Layout Constraint Solver, Theme Variant Preview, Localization Preview, Focus Navigation, Input Glyph Mapper, UI Snapshot Diff, and Widget Performance. These pages keep widget hierarchies, constraint solves, token variants, string expansion, focus graphs, platform glyph maps, visual deltas, and invalidation costs inside the same compact drawer grammar.

The world streaming editor-page batch covers scene scale and placement tools: World Partition, HLOD Builder, Level Instance, Streaming Profiler, Scene Bookmarks, Spawn Point Editor, Collision Matrix, and Environment Probes. These pages keep partition cells, HLOD clusters, instance overrides, streaming events, bookmark cameras, spawn validation, collision response rules, and probe bake queues inside the same workbench drawer layout.

The LiveOps editor-page batch covers release operations and live configuration tools: Feature Flags, Remote Config, Telemetry Query, Patch Planner, DLC Catalog, Crash Symbolication, Player Segment, and Experiment Console. These pages keep rollouts, config diffs, telemetry funnels, patch risks, entitlement packs, symbolication queues, audience cohorts, and experiment metrics in the same dense workbench drawers.

The AI workbench style references under `docs/ui-and-layout/ai-workbench-style/` are intentionally separate from the deterministic manifest output directory. They are generated raster references for color, density, panel rhythm, and page-specific mood; the implementation target remains the HTML/CSS export pipeline where UI text and controls are rendered deterministically. The `*-layout.png` AI references are layout-first review drafts: use them to judge main editor tab ownership, JetBrains-style drawer placement, bottom output/timeline placement, and overall interaction structure before tuning page-specific content.

The workflow-spec batch combines those parts into actual task references. Prefab placement, asset import, shader compile errors, animation event editing, runtime debugging, build export, UI data binding, and lighting bake each show which main tab owns the workflow, which drawer slots should stay visible, where temporary windows appear, and which bottom output panel closes the loop. These PNGs are intentionally annotated so implementation work can translate the design into interaction routes without guessing drawer ownership.

The floating-window batch covers heavier operational windows that appear over a live workbench but are not just transient menus: Preferences, Keyboard Shortcuts, Reimport Conflict, Source Control Submit, Crash Report, Find In Project, Startup Tasks, and Editor Update. These keep the iOS-like flat rounded-rectangle control language while using JetBrains-style dense navigation lists, tables, previews, and footer actions.

## Export And Verification

`export-designs.mjs` starts the local preview server on `ZIRCON_WORKBENCH_PREVIEW_PORT` or port `5187`, captures the preview sheet and all manifest entries with Playwright CLI, and writes the files under `docs/ui-and-layout/editor-workbench-designs/`. It uses the installed Microsoft Edge channel so the workflow does not require vendoring browser binaries into the repository; the preview server serves both `.js` and `.mjs` files as JavaScript so browser modules can import `design-manifest.mjs` directly. `export-options.mjs` normalizes `--ids`, `--design`, and positional ids, splitting comma-separated or whitespace-joined batches so PowerShell/npm forwarding quirks still select the intended outputs; it also reads npm config environment fallbacks when long options such as `--ids=...` or `--no-sheet` are consumed by the npm wrapper before they reach `process.argv`. Passing design ids or output file names to `npm --prefix tools/editor-workbench-preview run design:export:only -- <id-or-file>` captures only those requested PNGs while still refreshing `STYLE-NOTES.md`; adding `--no-sheet` suppresses `preview-sheet.png` for both full and partial exports.

`verify-designs.mjs` first pins both source references: `docs/ui-and-layout/workbench.png` and `zircon_editor/assets/ui/editor/reference/workbench.png` must keep SHA-256 `4AD7706C08138EF422802C0C46B5DE4775237021F474200EFC100DAD577C02D0`, byte length `1526533`, exact `1672x941` dimensions, and decodable non-interlaced 8-bit RGB PNG content. It also performs a direct byte comparison between those two files, making the runtime reference a zero-drift byte match for the source workbench screenshot instead of relying only on parallel hash checks. The runtime-reference override used by the negative guard is guarded: `verify-designs.mjs` ignores `ZIRCON_WORKBENCH_RUNTIME_REFERENCE_OVERRIDE` unless `ZIRCON_WORKBENCH_REFERENCE_NEGATIVE_GUARD=1` is also set, so accidental environment variables cannot redirect normal verification. The verifier then reads each exported PNG header directly and checks that every expected output exists, is non-empty, is a PNG, and has the exact `1672x941` dimensions. It also treats `design-manifest.mjs` as a strict output contract: duplicate design ids, duplicate output names, non-PNG outputs, render ids missing from `design.js`, manifest ids that `design.js` cannot render, extra PNG files in `editor-workbench-designs/`, and stale `STYLE-NOTES.md` entries are verification failures. The verifier also imports `preview-sheet.js` and checks that the preview sheet entry model covers every manifest id exactly once, and it checks that `preview-sheet.png` is newer than `design.html`, `design.css`, `design.js`, `design-manifest.mjs`, and `preview-sheet.js`, so the browser review sheet cannot quietly fall behind the rendering inputs. `STYLE-NOTES.md` must also be newer than `design-manifest.mjs` and `export-designs.mjs`, keeping the generated output list and group notes synchronized with the manifest and note writer. Every manifest single-page PNG must now also be newer than `design.html`, `design.css`, `design.js`, and `design-manifest.mjs`, so partial targeted exports cannot leave accepted screenshots behind newer rendering inputs. The verifier checks package script definitions in `tools/editor-workbench-preview/package.json`, including `design:verify` and `design:verify:reference-negative`, so the documented command entry points continue to invoke the intended verifier scripts. It also samples decoded RGB pixels to catch blank or visually drifted exports: each PNG must remain non-interlaced 8-bit RGB, stay within the dark workbench luminance band, keep a high dark-pixel ratio, avoid too much bright surface area, preserve teal accent pixels, and retain enough sampled color variety to reject flat empty images. The same verifier exercises `export-options.mjs` so `--all --no-sheet`, whitespace-joined ids, comma ids, separate `--ids <value>`, output-file selection, npm config fallbacks, sheet-only selection, sheet-plus-design selection, and unknown-id rejection stay covered by the standard `design:verify` command. It also reads this workflow document, the PNG design plan, the layout index, and `STYLE-NOTES.md` so published counts, the dual-reference baseline, `preview-sheet.png`, and the latest world streaming and LiveOps markers stay synchronized with the manifest. On Windows, it additionally fails if stale `editor-workbench-preview` export, server, or Playwright screenshot processes are still running, excluding the active verifier process itself. The current manifest and renderer registry contain 270 design entries, so verification expects 271 PNG files including `preview-sheet.png`. This verification deliberately checks the pinned source and runtime asset baselines, direct byte comparison, guarded runtime-reference override behavior, exported file structure, manifest/renderer consistency, preview-sheet coverage and freshness, single-page PNG freshness, package script definitions, `STYLE-NOTES.md` coverage and freshness, dimensions, deterministic export-option parsing, documentation markers, stale preview process cleanup, and broad visual profile invariants; subjective visual acceptance still requires comparing the draft corpus against `workbench.png` for density and control treatment, while exact pixel identity belongs to the pinned runtime reference asset.

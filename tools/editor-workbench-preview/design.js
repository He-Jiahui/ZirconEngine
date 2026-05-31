import { renderPreviewSheet } from "./preview-sheet.js";

const FULL_DESIGNS = [
  {
    id: "scene-workbench",
    output: "scene-workbench.png",
    title: "Scene Workbench",
    description: "Default scene authoring surface with hierarchy, viewport, inspector, and component lab.",
    activeRail: "scene",
    leftTab: "Scene",
    center: "scene",
    right: "inspector",
    bottom: "lab",
    status: "Ready",
  },
  {
    id: "hierarchy-workbench",
    output: "hierarchy-workbench.png",
    title: "Hierarchy Workbench",
    description: "Hierarchy-heavy layout with selected prefab branch and object context actions.",
    activeRail: "hierarchy",
    leftTab: "Hierarchy",
    center: "scene",
    right: "inspector-compact",
    bottom: "hierarchy-log",
    context: "hierarchy",
    status: "Selection: Props / Crate_01",
  },
  {
    id: "inspector-workbench",
    output: "inspector-workbench.png",
    title: "Inspector Workbench",
    description: "Inspector-led editing state with transform, renderer, material, and component controls.",
    activeRail: "inspector",
    leftTab: "Scene",
    center: "scene",
    right: "inspector-deep",
    bottom: "lab",
    status: "Inspector: Props / Box_01",
  },
  {
    id: "asset-browser-workbench",
    output: "asset-browser-workbench.png",
    title: "Asset Browser Workbench",
    description: "Asset browser as a first-class editor page with tree, grid, and import details.",
    activeRail: "assets",
    leftTab: "Assets",
    center: "asset-browser",
    right: "asset-detail",
    bottom: "asset-table",
    status: "Assets: crate://project/content",
  },
  {
    id: "console-workbench",
    output: "console-workbench.png",
    title: "Console Workbench",
    description: "Runtime console and diagnostic workflow with filters, warnings, and detail drilldown.",
    activeRail: "console",
    leftTab: "Scene",
    center: "scene-small",
    right: "diagnostics",
    bottom: "console",
    status: "2 warnings, 0 errors",
  },
  {
    id: "project-overview-workbench",
    output: "project-overview-workbench.png",
    title: "Project Overview Workbench",
    description: "Project dashboard with common actions, status, and catalog/workflow surfaces.",
    activeRail: "project",
    leftTab: "Project",
    center: "project",
    right: "project-actions",
    bottom: "project-activity",
    status: "Project: Zircon Labs / Sandbox",
  },
  {
    id: "material-lab-workbench",
    output: "material-lab-workbench.png",
    title: "Material Lab Workbench",
    description: "Material component catalog page with dense component groups and editor-native styling.",
    activeRail: "inspector",
    leftTab: "Catalog",
    center: "material-lab",
    right: "component-state",
    bottom: "material-log",
    status: "Material Lab: 68 components",
  },
  {
    id: "ui-asset-editor-workbench",
    output: "ui-asset-editor-workbench.png",
    title: "UI Asset Editor Workbench",
    description: "Tree-shaped ZUI asset authoring with source, preview, diagnostics, and bindings.",
    activeRail: "assets",
    leftTab: "UI Tree",
    center: "ui-asset-editor",
    right: "ui-asset-inspector",
    bottom: "ui-asset-diagnostics",
    status: "UI Asset: workbench_shell.v2.ui.toml",
  },
  {
    id: "animation-workbench",
    output: "animation-workbench.png",
    title: "Animation Workbench",
    description: "Animation timeline and graph editing with track hierarchy, curves, and keyframes.",
    activeRail: "scene",
    leftTab: "Rig",
    center: "animation",
    right: "animation-properties",
    bottom: "animation-timeline",
    status: "Animation: Idle_Run.blend",
  },
  {
    id: "performance-workbench",
    output: "performance-workbench.png",
    title: "Performance Workbench",
    description: "Frame profiler and UI surface metrics for editor/runtime performance diagnosis.",
    activeRail: "console",
    leftTab: "Captures",
    center: "performance",
    right: "performance-detail",
    bottom: "performance-events",
    status: "Frame: 16.6 ms / UI nodes 318",
  },
  {
    id: "runtime-diagnostics-workbench",
    output: "runtime-diagnostics-workbench.png",
    title: "Runtime Diagnostics Workbench",
    description: "Runtime event, render, task, and asset readiness diagnostics in workbench style.",
    activeRail: "console",
    leftTab: "Channels",
    center: "runtime-diagnostics",
    right: "diagnostics",
    bottom: "console",
    status: "Runtime Diagnostics: live preview",
  },
  {
    id: "plugin-manager-workbench",
    output: "plugin-manager-workbench.png",
    title: "Plugin Manager Workbench",
    description: "Plugin catalog and native runtime plugin health detail page.",
    activeRail: "project",
    leftTab: "Plugins",
    center: "plugin-manager",
    right: "plugin-detail",
    bottom: "plugin-log",
    status: "Plugins: 12 installed / 1 warning",
  },
  {
    id: "build-export-workbench",
    output: "build-export-workbench.png",
    title: "Build Export Workbench",
    description: "Desktop export target setup with package steps, validation, and build queue.",
    activeRail: "project",
    leftTab: "Targets",
    center: "build-export",
    right: "build-detail",
    bottom: "build-log",
    status: "Build Export: Windows desktop profile",
  },
  {
    id: "welcome-workbench",
    output: "welcome-workbench.png",
    title: "Welcome And New Project Workbench",
    description: "Startup project chooser and new project creation surface in the flat main-tab shell.",
    activeRail: "project",
    leftTab: "Recent",
    center: "welcome",
    right: "welcome-detail",
    bottom: "welcome-status",
    status: "Welcome: no project open",
  },
  {
    id: "prefab-editor-workbench",
    output: "prefab-editor-workbench.png",
    title: "Prefab Editor Workbench",
    description: "Prefab authoring tab with nested object graph, preview viewport, variants, and validation output.",
    activeRail: "scene",
    leftTab: "Prefab",
    center: "prefab-editor",
    right: "prefab-detail",
    bottom: "prefab-output",
    status: "Prefab: AudioZone.prefab",
  },
  {
    id: "vfx-editor-workbench",
    output: "vfx-editor-workbench.png",
    title: "VFX Editor Workbench",
    description: "Particle and effect authoring tab with emitter graph, preview, parameters, and simulation output.",
    activeRail: "assets",
    leftTab: "Emitters",
    center: "vfx-editor",
    right: "vfx-detail",
    bottom: "vfx-output",
    status: "VFX: P_Sparks.fx",
  },
  {
    id: "shader-editor-workbench",
    output: "shader-editor-workbench.png",
    title: "Shader Editor Workbench",
    description: "Shader source and preview tab with compile diagnostics, pipeline state, and variant controls.",
    activeRail: "assets",
    leftTab: "Shaders",
    center: "shader-editor",
    right: "shader-detail",
    bottom: "shader-output",
    status: "Shader: unlit.zshader",
  },
  {
    id: "terrain-editor-workbench",
    output: "terrain-editor-workbench.png",
    title: "Terrain Editor Workbench",
    description: "Terrain sculpt/paint tab with brush palette, layer stack, viewport, and bake output.",
    activeRail: "scene",
    leftTab: "Brushes",
    center: "terrain-editor",
    right: "terrain-detail",
    bottom: "terrain-output",
    status: "Terrain: Valley_01",
  },
  {
    id: "audio-editor-workbench",
    output: "audio-editor-workbench.png",
    title: "Audio Editor Workbench",
    description: "Audio graph and preview tab with event routing, mixer lanes, and analysis output.",
    activeRail: "assets",
    leftTab: "Events",
    center: "audio-editor",
    right: "audio-detail",
    bottom: "audio-output",
    status: "Audio: Ambience_Hangar",
  },
  {
    id: "behavior-tree-workbench",
    output: "behavior-tree-workbench.png",
    title: "Behavior Tree Workbench",
    description: "AI behavior tree tab with node palette, blackboard, graph editor, and runtime trace.",
    activeRail: "project",
    leftTab: "Nodes",
    center: "behavior-tree",
    right: "behavior-detail",
    bottom: "behavior-output",
    status: "Behavior Tree: Guard_Patrol.bt",
  },
  {
    id: "lighting-bake-workbench",
    output: "lighting-bake-workbench.png",
    title: "Lighting Bake Workbench",
    description: "Lighting bake tab with light list, bake settings, preview viewport, and job output.",
    activeRail: "scene",
    leftTab: "Lights",
    center: "lighting-bake",
    right: "lighting-detail",
    bottom: "lighting-output",
    status: "Lighting Bake: A1_Hangar",
  },
  {
    id: "physics-collision-workbench",
    output: "physics-collision-workbench.png",
    title: "Physics Collision Workbench",
    description: "Collision editing tab with shape palette, body hierarchy, preview viewport, and solver output.",
    activeRail: "scene",
    leftTab: "Shapes",
    center: "physics-collision",
    right: "physics-detail",
    bottom: "physics-output",
    status: "Physics: Box_01 collision",
  },
  {
    id: "level-streaming-workbench",
    output: "level-streaming-workbench.png",
    title: "Level Streaming Workbench",
    description: "Level streaming tab with streaming cells, world partitions, viewport preview, and load diagnostics.",
    activeRail: "scene",
    leftTab: "Cells",
    center: "level-streaming",
    right: "level-streaming-detail",
    bottom: "level-streaming-output",
    status: "Level Streaming: A1_Hangar partitions",
  },
  {
    id: "sequencer-workbench",
    output: "sequencer-workbench.png",
    title: "Sequencer Workbench",
    description: "Cinematic sequencer tab with shot browser, camera cuts, track list, key detail, and timeline output.",
    activeRail: "scene",
    leftTab: "Shots",
    center: "sequencer",
    right: "sequencer-detail",
    bottom: "sequencer-output",
    status: "Sequencer: Intro_Hangar.sequence",
  },
  {
    id: "navmesh-ai-workbench",
    output: "navmesh-ai-workbench.png",
    title: "NavMesh AI Workbench",
    description: "Navigation authoring tab with agent profiles, area layers, viewport preview, and bake output.",
    activeRail: "scene",
    leftTab: "Agents",
    center: "navmesh-ai",
    right: "navmesh-detail",
    bottom: "navmesh-output",
    status: "NavMesh: Agent_Humanoid",
  },
  {
    id: "render-pipeline-workbench",
    output: "render-pipeline-workbench.png",
    title: "Render Pipeline Workbench",
    description: "Render pipeline tab with pass graph, render targets, frame metrics, and compile output.",
    activeRail: "console",
    leftTab: "Passes",
    center: "render-pipeline",
    right: "render-pipeline-detail",
    bottom: "render-pipeline-output",
    status: "Render Pipeline: Default Forward+",
  },
  {
    id: "input-mapping-workbench",
    output: "input-mapping-workbench.png",
    title: "Input Mapping Workbench",
    description: "Input action map tab with devices, contexts, binding table, conflict detail, and validation output.",
    activeRail: "project",
    leftTab: "Devices",
    center: "input-mapping",
    right: "input-mapping-detail",
    bottom: "input-mapping-output",
    status: "Input Mapping: Editor_Default.input",
  },
  {
    id: "data-table-workbench",
    output: "data-table-workbench.png",
    title: "Data Table Workbench",
    description: "Data table editor tab with schema browser, row grid, validation detail, and import/export output.",
    activeRail: "assets",
    leftTab: "Schemas",
    center: "data-table",
    right: "data-table-detail",
    bottom: "data-table-output",
    status: "Data Table: items.zdata",
  },
  {
    id: "network-replication-workbench",
    output: "network-replication-workbench.png",
    title: "Network Replication Workbench",
    description: "Networking tab with replicated entities, property graph, session stats, and replication log.",
    activeRail: "console",
    leftTab: "Peers",
    center: "network-replication",
    right: "network-replication-detail",
    bottom: "network-replication-output",
    status: "Network Replication: 3 peers",
  },
  {
    id: "localization-workbench",
    output: "localization-workbench.png",
    title: "Localization Workbench",
    description: "Localization tab with locale sets, translation table, missing string detail, and export output.",
    activeRail: "assets",
    leftTab: "Locales",
    center: "localization",
    right: "localization-detail",
    bottom: "localization-output",
    status: "Localization: zh-CN / en-US",
  },
  {
    id: "visual-script-workbench",
    output: "visual-script-workbench.png",
    title: "Visual Script Workbench",
    description: "Visual scripting tab with node palette, graph canvas, variable watch, and compile output.",
    activeRail: "assets",
    leftTab: "Nodes",
    center: "visual-script",
    right: "visual-script-detail",
    bottom: "visual-script-output",
    status: "Visual Script: DoorController.zvs",
  },
  {
    id: "state-machine-workbench",
    output: "state-machine-workbench.png",
    title: "State Machine Workbench",
    description: "State machine tab with transition graph, state list, conditions, preview, and validation output.",
    activeRail: "project",
    leftTab: "States",
    center: "state-machine",
    right: "state-machine-detail",
    bottom: "state-machine-output",
    status: "State Machine: CharacterLocomotion.sm",
  },
  {
    id: "skeleton-mesh-workbench",
    output: "skeleton-mesh-workbench.png",
    title: "Skeleton Mesh Workbench",
    description: "Skeletal mesh tab with bone tree, preview viewport, skin weights, sockets, and import output.",
    activeRail: "scene",
    leftTab: "Bones",
    center: "skeleton-mesh",
    right: "skeleton-mesh-detail",
    bottom: "skeleton-mesh-output",
    status: "Skeleton Mesh: SK_Guard",
  },
  {
    id: "texture-editor-workbench",
    output: "texture-editor-workbench.png",
    title: "Texture Editor Workbench",
    description: "Texture inspection tab with mip chain, channel controls, preview, compression, and analysis output.",
    activeRail: "assets",
    leftTab: "Textures",
    center: "texture-editor",
    right: "texture-detail",
    bottom: "texture-output",
    status: "Texture: T_Grid_01.png",
  },
  {
    id: "material-instance-workbench",
    output: "material-instance-workbench.png",
    title: "Material Instance Workbench",
    description: "Material instance tab with parent material, parameter overrides, preview, and shader output.",
    activeRail: "assets",
    leftTab: "Instances",
    center: "material-instance",
    right: "material-instance-detail",
    bottom: "material-instance-output",
    status: "Material Instance: MI_Metal_Rough",
  },
  {
    id: "prefab-variant-workbench",
    output: "prefab-variant-workbench.png",
    title: "Prefab Variant Workbench",
    description: "Prefab variant tab with override diff, nested prefab tree, viewport preview, and validation output.",
    activeRail: "scene",
    leftTab: "Variants",
    center: "prefab-variant",
    right: "prefab-variant-detail",
    bottom: "prefab-variant-output",
    status: "Prefab Variant: AudioZone / Variant B",
  },
  {
    id: "level-audit-workbench",
    output: "level-audit-workbench.png",
    title: "Level Audit Workbench",
    description: "Level audit tab with rule packs, issue table, scene links, fix actions, and audit output.",
    activeRail: "console",
    leftTab: "Rules",
    center: "level-audit",
    right: "level-audit-detail",
    bottom: "level-audit-output",
    status: "Level Audit: A1_Hangar / 12 issues",
  },
  {
    id: "test-runner-workbench",
    output: "test-runner-workbench.png",
    title: "Test Runner Workbench",
    description: "Editor test runner tab with suites, result table, failure detail, and live task output.",
    activeRail: "console",
    leftTab: "Suites",
    center: "test-runner",
    right: "test-runner-detail",
    bottom: "test-runner-output",
    status: "Test Runner: UI layout smoke / 42 passed",
  },
  {
    id: "frame-debugger-workbench",
    output: "frame-debugger-workbench.png",
    title: "Frame Debugger Workbench",
    description: "Frame capture tab with render pass list, target preview, draw-call detail, and GPU event output.",
    activeRail: "console",
    leftTab: "Captures",
    center: "frame-debugger",
    right: "frame-debugger-detail",
    bottom: "frame-debugger-output",
    status: "Frame Debugger: capture #1842",
  },
  {
    id: "memory-profiler-workbench",
    output: "memory-profiler-workbench.png",
    title: "Memory Profiler Workbench",
    description: "Memory analysis tab with heaps, allocation graph, asset groups, leak suspects, and snapshot output.",
    activeRail: "console",
    leftTab: "Snapshots",
    center: "memory-profiler",
    right: "memory-profiler-detail",
    bottom: "memory-profiler-output",
    status: "Memory Profiler: 1.42 GB captured",
  },
  {
    id: "asset-dependency-workbench",
    output: "asset-dependency-workbench.png",
    title: "Asset Dependency Workbench",
    description: "Asset dependency graph tab with package roots, reference graph, affected assets, and validation output.",
    activeRail: "assets",
    leftTab: "Packages",
    center: "asset-dependency",
    right: "asset-dependency-detail",
    bottom: "asset-dependency-output",
    status: "Asset Dependency: A1_Hangar.scene",
  },
  {
    id: "reference-finder-workbench",
    output: "reference-finder-workbench.png",
    title: "Reference Finder Workbench",
    description: "Reference search tab with scoped queries, result table, owner detail, and replace output.",
    activeRail: "assets",
    leftTab: "Scopes",
    center: "reference-finder",
    right: "reference-finder-detail",
    bottom: "reference-finder-output",
    status: "Reference Finder: Box_01.mesh / 18 refs",
  },
  {
    id: "cook-package-workbench",
    output: "cook-package-workbench.png",
    title: "Cook Package Workbench",
    description: "Cook and package tab with platform profiles, staged asset graph, task queue, and package log.",
    activeRail: "project",
    leftTab: "Profiles",
    center: "cook-package",
    right: "cook-package-detail",
    bottom: "cook-package-output",
    status: "Cook Package: Windows / Development",
  },
  {
    id: "crash-session-replay-workbench",
    output: "crash-session-replay-workbench.png",
    title: "Crash Session Replay Workbench",
    description: "Crash session replay tab with event timeline, captured state, stack detail, and recovery output.",
    activeRail: "console",
    leftTab: "Sessions",
    center: "crash-session-replay",
    right: "crash-session-detail",
    bottom: "crash-session-output",
    status: "Crash Replay: renderer-panic-09-58",
  },
  {
    id: "log-analysis-workbench",
    output: "log-analysis-workbench.png",
    title: "Log Analysis Workbench",
    description: "Log analysis tab with channels, pattern table, selected log detail, and triage output.",
    activeRail: "console",
    leftTab: "Channels",
    center: "log-analysis",
    right: "log-analysis-detail",
    bottom: "log-analysis-output",
    status: "Log Analysis: 2 warnings / 1 pattern",
  },
  {
    id: "automation-report-workbench",
    output: "automation-report-workbench.png",
    title: "Automation Report Workbench",
    description: "Automation report tab with suites, pass/fail trends, artifacts, failure detail, and report output.",
    activeRail: "console",
    leftTab: "Reports",
    center: "automation-report",
    right: "automation-report-detail",
    bottom: "automation-report-output",
    status: "Automation Report: nightly-2026-05-29",
  },
  {
    id: "layout-manager-workbench",
    output: "layout-manager-workbench.png",
    title: "Layout Manager Workbench",
    description: "Workbench layout management tab with saved arrangements, drawer zones, split previews, and apply output.",
    activeRail: "project",
    leftTab: "Layouts",
    center: "layout-manager",
    right: "layout-manager-detail",
    bottom: "layout-manager-output",
    status: "Layout Manager: default-workbench",
  },
  {
    id: "theme-token-workbench",
    output: "theme-token-workbench.png",
    title: "Theme Token Workbench",
    description: "Theme token tab with surface colors, control radius, accent states, preview metrics, and validation output.",
    activeRail: "assets",
    leftTab: "Token Sets",
    center: "theme-token",
    right: "theme-token-detail",
    bottom: "theme-token-output",
    status: "Theme Token: workbench-strict",
  },
  {
    id: "command-center-workbench",
    output: "command-center-workbench.png",
    title: "Command Center Workbench",
    description: "Command catalog tab with commands, shortcuts, contexts, usage detail, and command audit output.",
    activeRail: "project",
    leftTab: "Groups",
    center: "command-center",
    right: "command-center-detail",
    bottom: "command-center-output",
    status: "Command Center: 184 commands",
  },
  {
    id: "module-graph-workbench",
    output: "module-graph-workbench.png",
    title: "Module Graph Workbench",
    description: "Module dependency tab with runtime/editor packages, service graph, module detail, and boundary output.",
    activeRail: "project",
    leftTab: "Packages",
    center: "module-graph",
    right: "module-graph-detail",
    bottom: "module-graph-output",
    status: "Module Graph: zircon_runtime",
  },
  {
    id: "asset-validation-workbench",
    output: "asset-validation-workbench.png",
    title: "Asset Validation Workbench",
    description: "Asset validation tab with rule packs, failed assets, repair detail, and validation queue output.",
    activeRail: "assets",
    leftTab: "Rule Packs",
    center: "asset-validation",
    right: "asset-validation-detail",
    bottom: "asset-validation-output",
    status: "Asset Validation: 7 warnings",
  },
  {
    id: "hot-reload-workbench",
    output: "hot-reload-workbench.png",
    title: "Hot Reload Workbench",
    description: "Hot reload tab with changed modules, reload dependency graph, session state, and live reload output.",
    activeRail: "console",
    leftTab: "Changes",
    center: "hot-reload",
    right: "hot-reload-detail",
    bottom: "hot-reload-output",
    status: "Hot Reload: 3 modules dirty",
  },
  {
    id: "project-history-workbench",
    output: "project-history-workbench.png",
    title: "Project History Workbench",
    description: "Project history tab with recent sessions, changelist table, asset diffs, and history output.",
    activeRail: "project",
    leftTab: "History",
    center: "project-history",
    right: "project-history-detail",
    bottom: "project-history-output",
    status: "Project History: 12 recent changes",
  },
  {
    id: "task-board-workbench",
    output: "task-board-workbench.png",
    title: "Task Board Workbench",
    description: "Production task board tab with team lanes, assigned work, review detail, and activity output.",
    activeRail: "project",
    leftTab: "Boards",
    center: "task-board",
    right: "task-board-detail",
    bottom: "task-board-output",
    status: "Task Board: Editor UI / Sprint 04",
  },
  {
    id: "source-control-workbench",
    output: "source-control-workbench.png",
    title: "Source Control Workbench",
    description: "Source control tab with changelists, file status, diff summary, submit detail, and sync output.",
    activeRail: "project",
    leftTab: "Changelists",
    center: "source-control",
    right: "source-control-detail",
    bottom: "source-control-output",
    status: "Source Control: 14 modified",
  },
  {
    id: "review-comments-workbench",
    output: "review-comments-workbench.png",
    title: "Review Comments Workbench",
    description: "Code and content review tab with threads, affected files, comment detail, and review activity output.",
    activeRail: "project",
    leftTab: "Reviews",
    center: "review-comments",
    right: "review-comments-detail",
    bottom: "review-comments-output",
    status: "Review Comments: 6 unresolved",
  },
  {
    id: "build-farm-workbench",
    output: "build-farm-workbench.png",
    title: "Build Farm Workbench",
    description: "Build farm tab with agents, job graph, queue metrics, worker detail, and farm event output.",
    activeRail: "console",
    leftTab: "Agents",
    center: "build-farm",
    right: "build-farm-detail",
    bottom: "build-farm-output",
    status: "Build Farm: 4 agents / 2 busy",
  },
  {
    id: "release-notes-workbench",
    output: "release-notes-workbench.png",
    title: "Release Notes Workbench",
    description: "Release notes tab with milestone sections, change table, publish checklist, and release output.",
    activeRail: "project",
    leftTab: "Milestones",
    center: "release-notes",
    right: "release-notes-detail",
    bottom: "release-notes-output",
    status: "Release Notes: 0.18.0 draft",
  },
  {
    id: "project-settings-workbench",
    output: "project-settings-workbench.png",
    title: "Project Settings Workbench",
    description: "Project settings tab with categories, setting table, environment detail, and validation output.",
    activeRail: "project",
    leftTab: "Settings",
    center: "project-settings-page",
    right: "project-settings-detail",
    bottom: "project-settings-output",
    status: "Project Settings: Sandbox",
  },
  {
    id: "plugin-development-workbench",
    output: "plugin-development-workbench.png",
    title: "Plugin Development Workbench",
    description: "Plugin development tab with plugin modules, extension graph, manifest detail, and build output.",
    activeRail: "project",
    leftTab: "Plugins",
    center: "plugin-development",
    right: "plugin-development-detail",
    bottom: "plugin-development-output",
    status: "Plugin Development: editor.tools.validation",
  },
  {
    id: "remote-device-workbench",
    output: "remote-device-workbench.png",
    title: "Remote Device Workbench",
    description: "Remote device tab with connected targets, deployment graph, device detail, and deploy output.",
    activeRail: "project",
    leftTab: "Devices",
    center: "remote-device",
    right: "remote-device-detail",
    bottom: "remote-device-output",
    status: "Remote Device: Windows-DevKit-01",
  },
  {
    id: "session-sync-workbench",
    output: "session-sync-workbench.png",
    title: "Session Sync Workbench",
    description: "Multi-user session sync tab with peers, replicated editor state, conflict detail, and sync output.",
    activeRail: "project",
    leftTab: "Peers",
    center: "session-sync",
    right: "session-sync-detail",
    bottom: "session-sync-output",
    status: "Session Sync: 3 peers connected",
  },
  {
    id: "cutscene-editor-workbench",
    output: "cutscene-editor-workbench.png",
    title: "Cutscene Editor Workbench",
    description: "Cutscene tab with shot list, track tree, camera/detail drawers, and render/timeline output.",
    activeRail: "scene",
    leftTab: "Shots",
    center: "cutscene-editor",
    right: "cutscene-detail",
    bottom: "cutscene-output",
    status: "Cutscene: Intro_Hangar / Shot 020",
  },
  {
    id: "dialogue-editor-workbench",
    output: "dialogue-editor-workbench.png",
    title: "Dialogue Editor Workbench",
    description: "Dialogue tab with speaker graph, line table, localization/detail drawers, and validation output.",
    activeRail: "assets",
    leftTab: "Speakers",
    center: "dialogue-editor",
    right: "dialogue-detail",
    bottom: "dialogue-output",
    status: "Dialogue: Hangar_Intro / zh-CN",
  },
  {
    id: "quest-editor-workbench",
    output: "quest-editor-workbench.png",
    title: "Quest Editor Workbench",
    description: "Quest authoring tab with objective graph, task lists, condition detail, and validation output.",
    activeRail: "project",
    leftTab: "Quests",
    center: "quest-editor",
    right: "quest-detail",
    bottom: "quest-output",
    status: "Quest: Restore Power / 6 objectives",
  },
  {
    id: "camera-rig-workbench",
    output: "camera-rig-workbench.png",
    title: "Camera Rig Workbench",
    description: "Camera rig tab with viewport framing, rig controls, camera stack, lens detail, and preview output.",
    activeRail: "scene",
    leftTab: "Rigs",
    center: "camera-rig",
    right: "camera-rig-detail",
    bottom: "camera-rig-output",
    status: "Camera Rig: Crane_A / 35mm",
  },
  {
    id: "control-rig-workbench",
    output: "control-rig-workbench.png",
    title: "Control Rig Workbench",
    description: "Control rig tab with rig graph, controls, solver detail, and rig validation output.",
    activeRail: "scene",
    leftTab: "Controls",
    center: "control-rig",
    right: "control-rig-detail",
    bottom: "control-rig-output",
    status: "Control Rig: SK_Guard / Spine IK",
  },
  {
    id: "motion-matching-workbench",
    output: "motion-matching-workbench.png",
    title: "Motion Matching Workbench",
    description: "Motion matching tab with pose database metrics, query features, clip detail, and analysis output.",
    activeRail: "scene",
    leftTab: "Databases",
    center: "motion-matching",
    right: "motion-matching-detail",
    bottom: "motion-matching-output",
    status: "Motion Matching: Locomotion_DB / 842 poses",
  },
  {
    id: "facial-animation-workbench",
    output: "facial-animation-workbench.png",
    title: "Facial Animation Workbench",
    description: "Facial animation tab with phoneme/curve tables, expression detail, and solve output.",
    activeRail: "scene",
    leftTab: "Expressions",
    center: "facial-animation",
    right: "facial-animation-detail",
    bottom: "facial-animation-output",
    status: "Facial Animation: Captain_Line_03",
  },
  {
    id: "blend-space-workbench",
    output: "blend-space-workbench.png",
    title: "Blend Space Workbench",
    description: "Blend space tab with sample graph, parameter drawers, blend detail, and preview output.",
    activeRail: "scene",
    leftTab: "Samples",
    center: "blend-space",
    right: "blend-space-detail",
    bottom: "blend-space-output",
    status: "Blend Space: BS_Locomotion_2D",
  },
  {
    id: "foliage-editor-workbench",
    output: "foliage-editor-workbench.png",
    title: "Foliage Editor Workbench",
    description: "Foliage tab with brush presets, density layers, instance detail, and paint output.",
    activeRail: "scene",
    leftTab: "Brushes",
    center: "foliage-editor",
    right: "foliage-detail",
    bottom: "foliage-output",
    status: "Foliage: Valley_01 / 18k instances",
  },
  {
    id: "scatter-editor-workbench",
    output: "scatter-editor-workbench.png",
    title: "Scatter Editor Workbench",
    description: "Scatter tab with placement rules, distribution graph, rule detail, and validation output.",
    activeRail: "scene",
    leftTab: "Rules",
    center: "scatter-editor",
    right: "scatter-detail",
    bottom: "scatter-output",
    status: "Scatter: RockField_A / 42 rules",
  },
  {
    id: "volume-editor-workbench",
    output: "volume-editor-workbench.png",
    title: "Volume Editor Workbench",
    description: "Volume tab with trigger/physics/post volumes, bounds detail, and overlap output.",
    activeRail: "scene",
    leftTab: "Volumes",
    center: "volume-editor",
    right: "volume-detail",
    bottom: "volume-output",
    status: "Volume: FogZone_A / 6 overlaps",
  },
  {
    id: "weather-editor-workbench",
    output: "weather-editor-workbench.png",
    title: "Weather Editor Workbench",
    description: "Weather tab with storm graph, sky layers, preset detail, and simulation output.",
    activeRail: "scene",
    leftTab: "Presets",
    center: "weather-editor",
    right: "weather-detail",
    bottom: "weather-output",
    status: "Weather: StormFront_02 / 38% rain",
  },
  {
    id: "post-process-workbench",
    output: "post-process-workbench.png",
    title: "Post Process Workbench",
    description: "Post process tab with effect stack, LUT/volume controls, pass detail, and preview output.",
    activeRail: "scene",
    leftTab: "Looks",
    center: "post-process",
    right: "post-process-detail",
    bottom: "post-process-output",
    status: "Post Process: Hangar_Night / Filmic",
  },
  {
    id: "particle-library-workbench",
    output: "particle-library-workbench.png",
    title: "Particle Library Workbench",
    description: "Particle library tab with reusable emitters, preview metrics, emitter detail, and simulation output.",
    activeRail: "assets",
    leftTab: "Particles",
    center: "particle-library",
    right: "particle-library-detail",
    bottom: "particle-library-output",
    status: "Particle Library: 64 emitters",
  },
  {
    id: "collision-proxy-workbench",
    output: "collision-proxy-workbench.png",
    title: "Collision Proxy Workbench",
    description: "Collision proxy tab with proxy generation, mesh analysis, proxy detail, and bake output.",
    activeRail: "scene",
    leftTab: "Meshes",
    center: "collision-proxy",
    right: "collision-proxy-detail",
    bottom: "collision-proxy-output",
    status: "Collision Proxy: Hangar_Props / 12 proxies",
  },
  {
    id: "level-variant-workbench",
    output: "level-variant-workbench.png",
    title: "Level Variant Workbench",
    description: "Level variant tab with variant sets, changed actors, variant detail, and diff output.",
    activeRail: "scene",
    leftTab: "Variants",
    center: "level-variant",
    right: "level-variant-detail",
    bottom: "level-variant-output",
    status: "Level Variant: Hangar_DayNight / Night",
  },
  {
    id: "gameplay-ability-workbench",
    output: "gameplay-ability-workbench.png",
    title: "Gameplay Ability Workbench",
    description: "Gameplay ability tab with ability graph, activation rules, ability detail, and simulation output.",
    activeRail: "project",
    leftTab: "Abilities",
    center: "gameplay-ability",
    right: "gameplay-ability-detail",
    bottom: "gameplay-ability-output",
    status: "Gameplay Ability: DashStrike / Ready",
  },
  {
    id: "gameplay-effect-workbench",
    output: "gameplay-effect-workbench.png",
    title: "Gameplay Effect Workbench",
    description: "Gameplay effect tab with modifiers, stacking rules, effect detail, and validation output.",
    activeRail: "project",
    leftTab: "Effects",
    center: "gameplay-effect",
    right: "gameplay-effect-detail",
    bottom: "gameplay-effect-output",
    status: "Gameplay Effect: Burning / 8 modifiers",
  },
  {
    id: "ai-perception-workbench",
    output: "ai-perception-workbench.png",
    title: "AI Perception Workbench",
    description: "AI perception tab with sensors, stimuli, agent detail, and perception trace output.",
    activeRail: "scene",
    leftTab: "Sensors",
    center: "ai-perception",
    right: "ai-perception-detail",
    bottom: "ai-perception-output",
    status: "AI Perception: Guard_01 / 12 stimuli",
  },
  {
    id: "spawn-rules-workbench",
    output: "spawn-rules-workbench.png",
    title: "Spawn Rules Workbench",
    description: "Spawn rules tab with spawn graph, wave tables, rule detail, and simulation output.",
    activeRail: "project",
    leftTab: "Rules",
    center: "spawn-rules",
    right: "spawn-rules-detail",
    bottom: "spawn-rules-output",
    status: "Spawn Rules: HangarWave_A / 5 waves",
  },
  {
    id: "gameplay-tags-workbench",
    output: "gameplay-tags-workbench.png",
    title: "Gameplay Tags Workbench",
    description: "Gameplay tags tab with tag taxonomy, usage grid, tag detail, and validation output.",
    activeRail: "project",
    leftTab: "Tags",
    center: "gameplay-tags",
    right: "gameplay-tags-detail",
    bottom: "gameplay-tags-output",
    status: "Gameplay Tags: 286 tags / 0 conflicts",
  },
  {
    id: "save-data-workbench",
    output: "save-data-workbench.png",
    title: "Save Data Workbench",
    description: "Save data tab with save slots, schema migration table, slot detail, and validation output.",
    activeRail: "project",
    leftTab: "Slots",
    center: "save-data",
    right: "save-data-detail",
    bottom: "save-data-output",
    status: "Save Data: Slot_A / schema v12",
  },
  {
    id: "world-state-workbench",
    output: "world-state-workbench.png",
    title: "World State Workbench",
    description: "World state tab with runtime flags, state graph, flag detail, and diff output.",
    activeRail: "project",
    leftTab: "States",
    center: "world-state",
    right: "world-state-detail",
    bottom: "world-state-output",
    status: "World State: Runtime Snapshot / 184 flags",
  },
  {
    id: "telemetry-dashboard-workbench",
    output: "telemetry-dashboard-workbench.png",
    title: "Telemetry Dashboard Workbench",
    description: "Telemetry dashboard tab with live counters, event streams, metric detail, and capture output.",
    activeRail: "console",
    leftTab: "Dashboards",
    center: "telemetry-dashboard",
    right: "telemetry-dashboard-detail",
    bottom: "telemetry-dashboard-output",
    status: "Telemetry Dashboard: Live Session / 1.2k events",
  },
  {
    id: "lobby-editor-workbench",
    output: "lobby-editor-workbench.png",
    title: "Lobby Editor Workbench",
    description: "Lobby tab with room list, members, lobby rules/detail, and session output.",
    activeRail: "project",
    leftTab: "Lobbies",
    center: "lobby-editor",
    right: "lobby-detail",
    bottom: "lobby-output",
    status: "Lobby: Dev Room / 4 members",
  },
  {
    id: "matchmaking-editor-workbench",
    output: "matchmaking-editor-workbench.png",
    title: "Matchmaking Editor Workbench",
    description: "Matchmaking tab with queue rules, pool metrics, ticket detail, and matchmaker output.",
    activeRail: "project",
    leftTab: "Queues",
    center: "matchmaking-editor",
    right: "matchmaking-detail",
    bottom: "matchmaking-output",
    status: "Matchmaking: Ranked_2v2 / 128 tickets",
  },
  {
    id: "server-browser-workbench",
    output: "server-browser-workbench.png",
    title: "Server Browser Workbench",
    description: "Server browser tab with filters, region status, server detail, and ping output.",
    activeRail: "project",
    leftTab: "Regions",
    center: "server-browser",
    right: "server-browser-detail",
    bottom: "server-browser-output",
    status: "Server Browser: 42 servers / Asia",
  },
  {
    id: "replay-browser-workbench",
    output: "replay-browser-workbench.png",
    title: "Replay Browser Workbench",
    description: "Replay browser tab with replay catalog, timeline markers, replay detail, and playback output.",
    activeRail: "console",
    leftTab: "Replays",
    center: "replay-browser",
    right: "replay-detail",
    bottom: "replay-output",
    status: "Replay Browser: Match_042 / 12 markers",
  },
  {
    id: "achievements-editor-workbench",
    output: "achievements-editor-workbench.png",
    title: "Achievements Editor Workbench",
    description: "Achievements tab with achievement definitions, progress stats, achievement detail, and validation output.",
    activeRail: "project",
    leftTab: "Achievements",
    center: "achievements-editor",
    right: "achievements-detail",
    bottom: "achievements-output",
    status: "Achievements: 38 definitions / 2 drafts",
  },
  {
    id: "entitlements-editor-workbench",
    output: "entitlements-editor-workbench.png",
    title: "Entitlements Editor Workbench",
    description: "Entitlements tab with catalog grants, ownership checks, entitlement detail, and validation output.",
    activeRail: "project",
    leftTab: "Catalog",
    center: "entitlements-editor",
    right: "entitlements-detail",
    bottom: "entitlements-output",
    status: "Entitlements: FounderPack / 3 grants",
  },
  {
    id: "user-profile-editor-workbench",
    output: "user-profile-editor-workbench.png",
    title: "User Profile Workbench",
    description: "User profile tab with identity fields, friends/social state, profile detail, and sync output.",
    activeRail: "project",
    leftTab: "Profiles",
    center: "user-profile-editor",
    right: "user-profile-detail",
    bottom: "user-profile-output",
    status: "User Profile: Player_042 / Online",
  },
  {
    id: "online-diagnostics-workbench",
    output: "online-diagnostics-workbench.png",
    title: "Online Diagnostics Workbench",
    description: "Online diagnostics tab with service health, auth/session checks, diagnostic detail, and trace output.",
    activeRail: "console",
    leftTab: "Services",
    center: "online-diagnostics",
    right: "online-diagnostics-detail",
    bottom: "online-diagnostics-output",
    status: "Online Diagnostics: Auth OK / 1 warning",
  },
  {
    id: "hud-editor-workbench",
    output: "hud-editor-workbench.png",
    title: "HUD Editor Workbench",
    description: "HUD editor tab with widget layers, anchor rules, widget detail, and preview output.",
    activeRail: "project",
    leftTab: "Widgets",
    center: "hud-editor",
    right: "hud-detail",
    bottom: "hud-output",
    status: "HUD Editor: Combat_HUD / 18 widgets",
  },
  {
    id: "menu-flow-workbench",
    output: "menu-flow-workbench.png",
    title: "Menu Flow Workbench",
    description: "Menu flow tab with navigation graph, screen stack, transition detail, and validation output.",
    activeRail: "project",
    leftTab: "Screens",
    center: "menu-flow",
    right: "menu-flow-detail",
    bottom: "menu-flow-output",
    status: "Menu Flow: MainMenu / 9 screens",
  },
  {
    id: "font-atlas-workbench",
    output: "font-atlas-workbench.png",
    title: "Font Atlas Workbench",
    description: "Font atlas tab with glyph sets, atlas metrics, glyph detail, and bake output.",
    activeRail: "project",
    leftTab: "Fonts",
    center: "font-atlas",
    right: "font-atlas-detail",
    bottom: "font-atlas-output",
    status: "Font Atlas: Inter_UI / 1248 glyphs",
  },
  {
    id: "icon-library-workbench",
    output: "icon-library-workbench.png",
    title: "Icon Library Workbench",
    description: "Icon library tab with icon catalog, usage stats, icon detail, and export output.",
    activeRail: "project",
    leftTab: "Icons",
    center: "icon-library",
    right: "icon-library-detail",
    bottom: "icon-library-output",
    status: "Icon Library: Editor_Core / 286 icons",
  },
  {
    id: "ui-binding-workbench",
    output: "ui-binding-workbench.png",
    title: "UI Binding Workbench",
    description: "UI binding tab with binding table, data contracts, binding detail, and validation output.",
    activeRail: "project",
    leftTab: "Bindings",
    center: "ui-binding-editor",
    right: "ui-binding-detail",
    bottom: "ui-binding-output",
    status: "UI Binding: inventory_panel / 24 bindings",
  },
  {
    id: "accessibility-audit-workbench",
    output: "accessibility-audit-workbench.png",
    title: "Accessibility Audit Workbench",
    description: "Accessibility audit tab with contrast checks, focus order, issue detail, and audit output.",
    activeRail: "console",
    leftTab: "Audits",
    center: "accessibility-audit",
    right: "accessibility-detail",
    bottom: "accessibility-output",
    status: "Accessibility Audit: 42 checks / 3 issues",
  },
  {
    id: "input-prompts-workbench",
    output: "input-prompts-workbench.png",
    title: "Input Prompts Workbench",
    description: "Input prompts tab with device glyph sets, prompt rules, prompt detail, and localization output.",
    activeRail: "project",
    leftTab: "Prompts",
    center: "input-prompts",
    right: "input-prompts-detail",
    bottom: "input-prompts-output",
    status: "Input Prompts: Gamepad_Xbox / 64 prompts",
  },
  {
    id: "ui-motion-workbench",
    output: "ui-motion-workbench.png",
    title: "UI Motion Workbench",
    description: "UI motion tab with motion clips, curves, motion detail, and timeline output.",
    activeRail: "project",
    leftTab: "Motion",
    center: "ui-motion",
    right: "ui-motion-detail",
    bottom: "ui-motion-output",
    status: "UI Motion: Panel_Open / 18 clips",
  },
  {
    id: "shader-permutations-workbench",
    output: "shader-permutations-workbench.png",
    title: "Shader Permutations Workbench",
    description: "Shader permutations tab with keyword sets, variant matrix, permutation detail, and compiler output.",
    activeRail: "project",
    leftTab: "Shaders",
    center: "shader-permutations",
    right: "shader-permutations-detail",
    bottom: "shader-permutations-output",
    status: "Shader Permutations: M_Metal / 128 variants",
  },
  {
    id: "render-target-workbench",
    output: "render-target-workbench.png",
    title: "Render Target Workbench",
    description: "Render target tab with target catalog, attachment state, target detail, and capture output.",
    activeRail: "project",
    leftTab: "Targets",
    center: "render-targets",
    right: "render-target-detail",
    bottom: "render-target-output",
    status: "Render Target: HDR_Main / 4 attachments",
  },
  {
    id: "gpu-profiler-workbench",
    output: "gpu-profiler-workbench.png",
    title: "GPU Profiler Workbench",
    description: "GPU profiler tab with pass timings, marker graph, pass detail, and profiling output.",
    activeRail: "console",
    leftTab: "Frames",
    center: "gpu-profiler",
    right: "gpu-profiler-detail",
    bottom: "gpu-profiler-output",
    status: "GPU Profiler: Frame 1842 / 12.8 ms",
  },
  {
    id: "light-probes-workbench",
    output: "light-probes-workbench.png",
    title: "Light Probes Workbench",
    description: "Light probes tab with probe placement, bake settings, probe detail, and bake output.",
    activeRail: "scene",
    leftTab: "Probe Sets",
    center: "light-probes",
    right: "light-probes-detail",
    bottom: "light-probes-output",
    status: "Light Probes: Hangar_ProbeGrid / 64 probes",
  },
  {
    id: "reflection-capture-workbench",
    output: "reflection-capture-workbench.png",
    title: "Reflection Capture Workbench",
    description: "Reflection capture tab with capture volumes, cubemap states, capture detail, and bake output.",
    activeRail: "scene",
    leftTab: "Captures",
    center: "reflection-capture",
    right: "reflection-capture-detail",
    bottom: "reflection-capture-output",
    status: "Reflection Capture: Hangar_Cubemap / 6 faces",
  },
  {
    id: "decal-editor-workbench",
    output: "decal-editor-workbench.png",
    title: "Decal Editor Workbench",
    description: "Decal editor tab with decal presets, projection settings, decal detail, and validation output.",
    activeRail: "scene",
    leftTab: "Decals",
    center: "decal-editor",
    right: "decal-detail",
    bottom: "decal-output",
    status: "Decal Editor: WarningStripe / 12 placements",
  },
  {
    id: "virtual-texture-workbench",
    output: "virtual-texture-workbench.png",
    title: "Virtual Texture Workbench",
    description: "Virtual texture tab with page table metrics, residency state, texture detail, and streaming output.",
    activeRail: "project",
    leftTab: "Virtual Textures",
    center: "virtual-texture",
    right: "virtual-texture-detail",
    bottom: "virtual-texture-output",
    status: "Virtual Texture: TerrainMega / 82% residency",
  },
  {
    id: "material-audit-workbench",
    output: "material-audit-workbench.png",
    title: "Material Audit Workbench",
    description: "Material audit tab with material cost table, rule violations, material detail, and audit output.",
    activeRail: "console",
    leftTab: "Audits",
    center: "material-audit",
    right: "material-audit-detail",
    bottom: "material-audit-output",
    status: "Material Audit: 184 materials / 7 warnings",
  },
  {
    id: "sound-cue-workbench",
    output: "sound-cue-workbench.png",
    title: "Sound Cue Workbench",
    description: "Sound cue tab with cue graph, randomization rules, cue detail, and preview output.",
    activeRail: "project",
    leftTab: "Cues",
    center: "sound-cue",
    right: "sound-cue-detail",
    bottom: "sound-cue-output",
    status: "Sound Cue: Weapon_Fire / 6 nodes",
  },
  {
    id: "audio-mixer-workbench",
    output: "audio-mixer-workbench.png",
    title: "Audio Mixer Workbench",
    description: "Audio mixer tab with bus levels, routing table, bus detail, and mix output.",
    activeRail: "project",
    leftTab: "Buses",
    center: "audio-mixer",
    right: "audio-mixer-detail",
    bottom: "audio-mixer-output",
    status: "Audio Mixer: Gameplay Mix / -18 LUFS",
  },
  {
    id: "music-system-workbench",
    output: "music-system-workbench.png",
    title: "Music System Workbench",
    description: "Music system tab with music state graph, transition rules, state detail, and preview output.",
    activeRail: "project",
    leftTab: "States",
    center: "music-system",
    right: "music-system-detail",
    bottom: "music-system-output",
    status: "Music System: Combat_Loop / 4 states",
  },
  {
    id: "audio-occlusion-workbench",
    output: "audio-occlusion-workbench.png",
    title: "Audio Occlusion Workbench",
    description: "Audio occlusion tab with trace rules, obstacle table, occlusion detail, and simulation output.",
    activeRail: "scene",
    leftTab: "Zones",
    center: "audio-occlusion",
    right: "audio-occlusion-detail",
    bottom: "audio-occlusion-output",
    status: "Audio Occlusion: Hangar_A / 12 traces",
  },
  {
    id: "voice-bank-workbench",
    output: "voice-bank-workbench.png",
    title: "Voice Bank Workbench",
    description: "Voice bank tab with line catalog, localization coverage, voice detail, and import output.",
    activeRail: "project",
    leftTab: "Banks",
    center: "voice-bank",
    right: "voice-bank-detail",
    bottom: "voice-bank-output",
    status: "Voice Bank: Captain_EN / 248 lines",
  },
  {
    id: "subtitle-timing-workbench",
    output: "subtitle-timing-workbench.png",
    title: "Subtitle Timing Workbench",
    description: "Subtitle timing tab with subtitle rows, timing checks, subtitle detail, and validation output.",
    activeRail: "project",
    leftTab: "Subtitles",
    center: "subtitle-timing",
    right: "subtitle-timing-detail",
    bottom: "subtitle-timing-output",
    status: "Subtitle Timing: Intro_Hangar / 42 cues",
  },
  {
    id: "lip-sync-workbench",
    output: "lip-sync-workbench.png",
    title: "Lip Sync Workbench",
    description: "Lip sync tab with phoneme curves, viseme tracks, clip detail, and solve output.",
    activeRail: "project",
    leftTab: "Clips",
    center: "lip-sync",
    right: "lip-sync-detail",
    bottom: "lip-sync-output",
    status: "Lip Sync: Captain_Line_03 / 18 phonemes",
  },
  {
    id: "audio-profiler-workbench",
    output: "audio-profiler-workbench.png",
    title: "Audio Profiler Workbench",
    description: "Audio profiler tab with voice counts, bus meters, event detail, and capture output.",
    activeRail: "console",
    leftTab: "Captures",
    center: "audio-profiler",
    right: "audio-profiler-detail",
    bottom: "audio-profiler-output",
    status: "Audio Profiler: Live Capture / 64 voices",
  },
  {
    id: "rigid-body-workbench",
    output: "rigid-body-workbench.png",
    title: "Rigid Body Workbench",
    description: "Rigid body tab with body lists, mass properties, body detail, and simulation output.",
    activeRail: "scene",
    leftTab: "Bodies",
    center: "rigid-body",
    right: "rigid-body-detail",
    bottom: "rigid-body-output",
    status: "Rigid Body: Crate_07 / 12 kg",
  },
  {
    id: "physics-constraints-workbench",
    output: "physics-constraints-workbench.png",
    title: "Physics Constraints Workbench",
    description: "Physics constraints tab with joint graph, limits, constraint detail, and solver output.",
    activeRail: "scene",
    leftTab: "Constraints",
    center: "physics-constraints",
    right: "physics-constraints-detail",
    bottom: "physics-constraints-output",
    status: "Physics Constraints: Door_Hinge / 8 joints",
  },
  {
    id: "destruction-workbench",
    output: "destruction-workbench.png",
    title: "Destruction Workbench",
    description: "Destruction tab with fracture clusters, damage thresholds, cluster detail, and bake output.",
    activeRail: "scene",
    leftTab: "Fractures",
    center: "destruction-editor",
    right: "destruction-detail",
    bottom: "destruction-output",
    status: "Destruction: WallPanel_A / 42 chunks",
  },
  {
    id: "cloth-simulation-workbench",
    output: "cloth-simulation-workbench.png",
    title: "Cloth Simulation Workbench",
    description: "Cloth simulation tab with cloth assets, constraint maps, cloth detail, and solve output.",
    activeRail: "scene",
    leftTab: "Cloth",
    center: "cloth-simulation",
    right: "cloth-detail",
    bottom: "cloth-output",
    status: "Cloth Simulation: Cape_A / 184 verts",
  },
  {
    id: "vehicle-physics-workbench",
    output: "vehicle-physics-workbench.png",
    title: "Vehicle Physics Workbench",
    description: "Vehicle physics tab with wheel setup, drivetrain metrics, vehicle detail, and test output.",
    activeRail: "scene",
    leftTab: "Vehicles",
    center: "vehicle-physics",
    right: "vehicle-physics-detail",
    bottom: "vehicle-physics-output",
    status: "Vehicle Physics: Rover_01 / 4 wheels",
  },
  {
    id: "fluid-simulation-workbench",
    output: "fluid-simulation-workbench.png",
    title: "Fluid Simulation Workbench",
    description: "Fluid simulation tab with emitters, solver metrics, fluid detail, and simulation output.",
    activeRail: "scene",
    leftTab: "Fluids",
    center: "fluid-simulation",
    right: "fluid-detail",
    bottom: "fluid-output",
    status: "Fluid Simulation: SteamVent_A / 18k particles",
  },
  {
    id: "rope-cable-workbench",
    output: "rope-cable-workbench.png",
    title: "Rope Cable Workbench",
    description: "Rope and cable tab with cable segments, attachment points, cable detail, and solve output.",
    activeRail: "scene",
    leftTab: "Cables",
    center: "rope-cable",
    right: "rope-cable-detail",
    bottom: "rope-cable-output",
    status: "Rope Cable: BridgeCable_A / 32 segments",
  },
  {
    id: "physics-profiler-workbench",
    output: "physics-profiler-workbench.png",
    title: "Physics Profiler Workbench",
    description: "Physics profiler tab with solver timings, island counts, profiler detail, and capture output.",
    activeRail: "console",
    leftTab: "Captures",
    center: "physics-profiler",
    right: "physics-profiler-detail",
    bottom: "physics-profiler-output",
    status: "Physics Profiler: Frame 1842 / 3.6 ms",
  },
  {
    id: "ai-director-workbench",
    output: "ai-director-workbench.png",
    title: "AI Director Workbench",
    description: "AI director tab with encounter pacing, threat budget, director detail, and simulation output.",
    activeRail: "project",
    leftTab: "Directors",
    center: "ai-director",
    right: "ai-director-detail",
    bottom: "ai-director-output",
    status: "AI Director: Hangar_Assault / 72 threat",
  },
  {
    id: "blackboard-workbench",
    output: "blackboard-workbench.png",
    title: "Blackboard Workbench",
    description: "Blackboard tab with key tables, runtime values, key detail, and validation output.",
    activeRail: "project",
    leftTab: "Blackboards",
    center: "blackboard-editor",
    right: "blackboard-detail",
    bottom: "blackboard-output",
    status: "Blackboard: Guard_Patrol / 18 keys",
  },
  {
    id: "eqs-query-workbench",
    output: "eqs-query-workbench.png",
    title: "EQS Query Workbench",
    description: "EQS query tab with test graph, score table, query detail, and debug output.",
    activeRail: "scene",
    leftTab: "Queries",
    center: "eqs-query",
    right: "eqs-query-detail",
    bottom: "eqs-query-output",
    status: "EQS Query: FindCover / 128 candidates",
  },
  {
    id: "crowd-simulation-workbench",
    output: "crowd-simulation-workbench.png",
    title: "Crowd Simulation Workbench",
    description: "Crowd simulation tab with agent groups, flow lanes, crowd detail, and simulation output.",
    activeRail: "scene",
    leftTab: "Crowds",
    center: "crowd-simulation",
    right: "crowd-detail",
    bottom: "crowd-output",
    status: "Crowd Simulation: Plaza_Test / 240 agents",
  },
  {
    id: "smart-objects-workbench",
    output: "smart-objects-workbench.png",
    title: "Smart Objects Workbench",
    description: "Smart objects tab with object slots, reservation state, object detail, and validation output.",
    activeRail: "scene",
    leftTab: "Objects",
    center: "smart-objects",
    right: "smart-object-detail",
    bottom: "smart-object-output",
    status: "Smart Objects: CoverWall_A / 12 slots",
  },
  {
    id: "patrol-routes-workbench",
    output: "patrol-routes-workbench.png",
    title: "Patrol Routes Workbench",
    description: "Patrol routes tab with route graph, waypoint table, route detail, and validation output.",
    activeRail: "scene",
    leftTab: "Routes",
    center: "patrol-routes",
    right: "patrol-route-detail",
    bottom: "patrol-route-output",
    status: "Patrol Routes: GuardLoop_A / 8 waypoints",
  },
  {
    id: "cover-system-workbench",
    output: "cover-system-workbench.png",
    title: "Cover System Workbench",
    description: "Cover system tab with cover points, exposure scoring, cover detail, and bake output.",
    activeRail: "scene",
    leftTab: "Cover Sets",
    center: "cover-system",
    right: "cover-detail",
    bottom: "cover-output",
    status: "Cover System: Hangar_Cover / 64 points",
  },
  {
    id: "ai-profiler-workbench",
    output: "ai-profiler-workbench.png",
    title: "AI Profiler Workbench",
    description: "AI profiler tab with behavior timings, perception events, profiler detail, and capture output.",
    activeRail: "console",
    leftTab: "Captures",
    center: "ai-profiler",
    right: "ai-profiler-detail",
    bottom: "ai-profiler-output",
    status: "AI Profiler: Live Capture / 32 agents",
  },
  {
    id: "mesh-import-workbench",
    output: "mesh-import-workbench.png",
    title: "Mesh Import Workbench",
    description: "Mesh import tab with import queue, mesh settings, import detail, and validation output.",
    activeRail: "project",
    leftTab: "Imports",
    center: "mesh-import",
    right: "mesh-import-detail",
    bottom: "mesh-import-output",
    status: "Mesh Import: SK_Crate.fbx / 3 warnings",
  },
  {
    id: "lod-chain-workbench",
    output: "lod-chain-workbench.png",
    title: "LOD Chain Workbench",
    description: "LOD chain tab with LOD levels, reduction metrics, LOD detail, and build output.",
    activeRail: "project",
    leftTab: "Meshes",
    center: "lod-chain",
    right: "lod-chain-detail",
    bottom: "lod-chain-output",
    status: "LOD Chain: SM_Rock_A / 4 levels",
  },
  {
    id: "redirect-map-workbench",
    output: "redirect-map-workbench.png",
    title: "Redirect Map Workbench",
    description: "Redirect map tab with asset redirects, owner usage, redirect detail, and validation output.",
    activeRail: "project",
    leftTab: "Redirects",
    center: "redirect-map",
    right: "redirect-map-detail",
    bottom: "redirect-map-output",
    status: "Redirect Map: 42 redirects / 0 broken",
  },
  {
    id: "texture-compression-queue-workbench",
    output: "texture-compression-queue-workbench.png",
    title: "Texture Compression Queue Workbench",
    description: "Texture compression queue tab with compression jobs, format rules, job detail, and batch output.",
    activeRail: "project",
    leftTab: "Queue",
    center: "texture-compression-queue",
    right: "texture-compression-detail",
    bottom: "texture-compression-output",
    status: "Texture Compression: 18 queued / BC7",
  },
  {
    id: "source-asset-trace-workbench",
    output: "source-asset-trace-workbench.png",
    title: "Source Asset Trace Workbench",
    description: "Source asset trace tab with source lineage, generated assets, source detail, and trace output.",
    activeRail: "project",
    leftTab: "Sources",
    center: "source-asset-trace",
    right: "source-asset-detail",
    bottom: "source-asset-output",
    status: "Source Asset Trace: crate_source.fbx / 6 outputs",
  },
  {
    id: "dcc-live-link-workbench",
    output: "dcc-live-link-workbench.png",
    title: "DCC Live Link Workbench",
    description: "DCC live link tab with connection sessions, sync events, session detail, and live-link output.",
    activeRail: "project",
    leftTab: "Sessions",
    center: "dcc-live-link",
    right: "dcc-live-link-detail",
    bottom: "dcc-live-link-output",
    status: "DCC Live Link: Blender / Connected",
  },
  {
    id: "metadata-editor-workbench",
    output: "metadata-editor-workbench.png",
    title: "Metadata Editor Workbench",
    description: "Metadata editor tab with metadata fields, schema rules, field detail, and validation output.",
    activeRail: "project",
    leftTab: "Schemas",
    center: "metadata-editor",
    right: "metadata-detail",
    bottom: "metadata-output",
    status: "Metadata Editor: AssetSchema.v2 / 24 fields",
  },
  {
    id: "batch-process-queue-workbench",
    output: "batch-process-queue-workbench.png",
    title: "Batch Process Queue Workbench",
    description: "Batch process queue tab with jobs, workers, job detail, and process output.",
    activeRail: "console",
    leftTab: "Batches",
    center: "batch-process-queue",
    right: "batch-process-detail",
    bottom: "batch-process-output",
    status: "Batch Process: Reimport Materials / 64 jobs",
  },
  {
    id: "script-editor-workbench",
    output: "script-editor-workbench.png",
    title: "Script Editor Workbench",
    description: "Script editor tab with source preview, symbol drawers, function detail, and compiler output.",
    activeRail: "code",
    leftTab: "Scripts",
    center: "script-editor",
    right: "script-detail",
    bottom: "script-output",
    status: "Script Editor: player_controller.zs / 0 errors",
  },
  {
    id: "api-browser-workbench",
    output: "api-browser-workbench.png",
    title: "API Browser Workbench",
    description: "API browser tab with namespace index, signatures, API detail, and docs output.",
    activeRail: "code",
    leftTab: "Namespaces",
    center: "api-browser",
    right: "api-detail",
    bottom: "api-output",
    status: "API Browser: zircon_runtime::ui / 184 symbols",
  },
  {
    id: "plugin-packaging-workbench",
    output: "plugin-packaging-workbench.png",
    title: "Plugin Packaging Workbench",
    description: "Plugin packaging tab with package targets, manifest checks, package detail, and publish output.",
    activeRail: "project",
    leftTab: "Plugins",
    center: "plugin-packaging",
    right: "plugin-packaging-detail",
    bottom: "plugin-packaging-output",
    status: "Plugin Packaging: editor.tools.validation / Ready",
  },
  {
    id: "module-settings-workbench",
    output: "module-settings-workbench.png",
    title: "Module Settings Workbench",
    description: "Module settings tab with module flags, dependency rules, setting detail, and validation output.",
    activeRail: "project",
    leftTab: "Modules",
    center: "module-settings",
    right: "module-settings-detail",
    bottom: "module-settings-output",
    status: "Module Settings: zircon_runtime / 2 warnings",
  },
  {
    id: "automation-suite-workbench",
    output: "automation-suite-workbench.png",
    title: "Automation Suite Workbench",
    description: "Automation suite tab with test suites, agents, suite detail, and run output.",
    activeRail: "console",
    leftTab: "Suites",
    center: "automation-suite",
    right: "automation-suite-detail",
    bottom: "automation-suite-output",
    status: "Automation Suite: Editor Smoke / 306 passed",
  },
  {
    id: "build-config-workbench",
    output: "build-config-workbench.png",
    title: "Build Config Workbench",
    description: "Build config tab with targets, feature flags, config detail, and build output.",
    activeRail: "project",
    leftTab: "Targets",
    center: "build-config",
    right: "build-config-detail",
    bottom: "build-config-output",
    status: "Build Config: Windows Editor / Development",
  },
  {
    id: "cook-rules-workbench",
    output: "cook-rules-workbench.png",
    title: "Cook Rules Workbench",
    description: "Cook rules tab with rule tables, platform overrides, rule detail, and cook output.",
    activeRail: "project",
    leftTab: "Rules",
    center: "cook-rules",
    right: "cook-rules-detail",
    bottom: "cook-rules-output",
    status: "Cook Rules: Desktop / 84 rules",
  },
  {
    id: "runtime-commands-workbench",
    output: "runtime-commands-workbench.png",
    title: "Runtime Commands Workbench",
    description: "Runtime commands tab with command catalog, bindings, command detail, and execution output.",
    activeRail: "console",
    leftTab: "Commands",
    center: "runtime-commands",
    right: "runtime-command-detail",
    bottom: "runtime-command-output",
    status: "Runtime Commands: 184 commands / 0 conflicts",
  },
  {
    id: "asset-migration-workbench",
    output: "asset-migration-workbench.png",
    title: "Asset Migration Workbench",
    description: "Asset migration tab with migration batches, schema checks, migration detail, and migration output.",
    activeRail: "project",
    leftTab: "Migrations",
    center: "asset-migration",
    right: "asset-migration-detail",
    bottom: "asset-migration-output",
    status: "Asset Migration: v17 -> v18 / 128 assets",
  },
  {
    id: "scene-diff-workbench",
    output: "scene-diff-workbench.png",
    title: "Scene Diff Workbench",
    description: "Scene diff tab with changed entities, ownership rules, diff detail, and review output.",
    activeRail: "scene",
    leftTab: "Diffs",
    center: "scene-diff",
    right: "scene-diff-detail",
    bottom: "scene-diff-output",
    status: "Scene Diff: A1_Hangar / 18 changes",
  },
  {
    id: "prefab-diff-workbench",
    output: "prefab-diff-workbench.png",
    title: "Prefab Diff Workbench",
    description: "Prefab diff tab with variant deltas, override rules, diff detail, and apply output.",
    activeRail: "project",
    leftTab: "Prefabs",
    center: "prefab-diff",
    right: "prefab-diff-detail",
    bottom: "prefab-diff-output",
    status: "Prefab Diff: Door_A Variant B / 7 changes",
  },
  {
    id: "performance-budget-workbench",
    output: "performance-budget-workbench.png",
    title: "Performance Budget Workbench",
    description: "Performance budget tab with frame budgets, subsystem costs, budget detail, and budget output.",
    activeRail: "console",
    leftTab: "Budgets",
    center: "performance-budget",
    right: "performance-budget-detail",
    bottom: "performance-budget-output",
    status: "Performance Budget: Editor Frame / 12.8 ms",
  },
  {
    id: "memory-budget-workbench",
    output: "memory-budget-workbench.png",
    title: "Memory Budget Workbench",
    description: "Memory budget tab with pools, asset groups, budget detail, and memory output.",
    activeRail: "console",
    leftTab: "Pools",
    center: "memory-budget",
    right: "memory-budget-detail",
    bottom: "memory-budget-output",
    status: "Memory Budget: Texture Pool / 1.2 GB",
  },
  {
    id: "dependency-cleanup-workbench",
    output: "dependency-cleanup-workbench.png",
    title: "Dependency Cleanup Workbench",
    description: "Dependency cleanup tab with unused assets, owner traces, cleanup detail, and cleanup output.",
    activeRail: "project",
    leftTab: "Cleanup",
    center: "dependency-cleanup",
    right: "dependency-cleanup-detail",
    bottom: "dependency-cleanup-output",
    status: "Dependency Cleanup: 42 unused / 0 broken",
  },
  {
    id: "naming-rules-workbench",
    output: "naming-rules-workbench.png",
    title: "Naming Rules Workbench",
    description: "Naming rules tab with naming violations, rule scopes, rule detail, and naming output.",
    activeRail: "project",
    leftTab: "Rules",
    center: "naming-rules",
    right: "naming-rule-detail",
    bottom: "naming-rule-output",
    status: "Naming Rules: 12 warnings / 5 autofix",
  },
  {
    id: "release-checklist-workbench",
    output: "release-checklist-workbench.png",
    title: "Release Checklist Workbench",
    description: "Release checklist tab with gate status, approval owners, gate detail, and release output.",
    activeRail: "project",
    leftTab: "Gates",
    center: "release-checklist",
    right: "release-checklist-detail",
    bottom: "release-checklist-output",
    status: "Release Checklist: 0.18.0 / 6 of 8 ready",
  },
  {
    id: "gameplay-debugger-workbench",
    output: "gameplay-debugger-workbench.png",
    title: "Gameplay Debugger Workbench",
    description: "Gameplay debugger tab with live actors, watches, actor detail, and debug output.",
    activeRail: "scene",
    leftTab: "Actors",
    center: "gameplay-debugger",
    right: "gameplay-debugger-detail",
    bottom: "gameplay-debugger-output",
    status: "Gameplay Debugger: Player_01 / Live",
  },
  {
    id: "replay-timeline-workbench",
    output: "replay-timeline-workbench.png",
    title: "Replay Timeline Workbench",
    description: "Replay timeline tab with event tracks, markers, replay detail, and playback output.",
    activeRail: "console",
    leftTab: "Replays",
    center: "replay-timeline",
    right: "replay-timeline-detail",
    bottom: "replay-timeline-output",
    status: "Replay Timeline: Match_042 / 12:42",
  },
  {
    id: "network-packet-inspector-workbench",
    output: "network-packet-inspector-workbench.png",
    title: "Network Packet Inspector Workbench",
    description: "Network packet inspector tab with packet streams, channel stats, packet detail, and capture output.",
    activeRail: "console",
    leftTab: "Streams",
    center: "network-packet-inspector",
    right: "packet-detail",
    bottom: "packet-output",
    status: "Packet Inspector: Client A / 1.2 KB frame",
  },
  {
    id: "latency-map-workbench",
    output: "latency-map-workbench.png",
    title: "Latency Map Workbench",
    description: "Latency map tab with region paths, latency metrics, route detail, and probe output.",
    activeRail: "console",
    leftTab: "Regions",
    center: "latency-map",
    right: "latency-detail",
    bottom: "latency-output",
    status: "Latency Map: Asia / 42 ms p50",
  },
  {
    id: "input-trace-workbench",
    output: "input-trace-workbench.png",
    title: "Input Trace Workbench",
    description: "Input trace tab with device events, action mapping watches, trace detail, and input output.",
    activeRail: "console",
    leftTab: "Devices",
    center: "input-trace",
    right: "input-trace-detail",
    bottom: "input-trace-output",
    status: "Input Trace: Keyboard+Mouse / 184 events",
  },
  {
    id: "save-state-diff-workbench",
    output: "save-state-diff-workbench.png",
    title: "Save State Diff Workbench",
    description: "Save state diff tab with slot deltas, schema guards, state detail, and diff output.",
    activeRail: "project",
    leftTab: "Slots",
    center: "save-state-diff",
    right: "save-state-detail",
    bottom: "save-state-output",
    status: "Save State Diff: Slot_A -> Slot_B / 12 flags",
  },
  {
    id: "repro-recorder-workbench",
    output: "repro-recorder-workbench.png",
    title: "Repro Recorder Workbench",
    description: "Repro recorder tab with capture steps, environment facts, repro detail, and recording output.",
    activeRail: "console",
    leftTab: "Recordings",
    center: "repro-recorder",
    right: "repro-detail",
    bottom: "repro-output",
    status: "Repro Recorder: Crash_1842 / Recording",
  },
  {
    id: "qa-triage-workbench",
    output: "qa-triage-workbench.png",
    title: "QA Triage Workbench",
    description: "QA triage tab with bug queue, owners, issue detail, and triage output.",
    activeRail: "project",
    leftTab: "Issues",
    center: "qa-triage",
    right: "qa-triage-detail",
    bottom: "qa-triage-output",
    status: "QA Triage: 18 open / 3 blocking",
  },
  {
    id: "render-graph-workbench",
    output: "render-graph-workbench.png",
    title: "Render Graph Workbench",
    description: "Render graph tab with pass graph, resource barriers, pass detail, and compile output.",
    activeRail: "scene",
    leftTab: "Passes",
    center: "render-graph",
    right: "render-graph-detail",
    bottom: "render-graph-output",
    status: "Render Graph: Forward+ / 7 passes",
  },
  {
    id: "shader-debugger-workbench",
    output: "shader-debugger-workbench.png",
    title: "Shader Debugger Workbench",
    description: "Shader debugger tab with source preview, variables, shader detail, and debug output.",
    activeRail: "code",
    leftTab: "Shaders",
    center: "shader-debugger",
    right: "shader-debugger-detail",
    bottom: "shader-debugger-output",
    status: "Shader Debugger: M_Metal.fragment / breakpoint",
  },
  {
    id: "texture-streaming-workbench",
    output: "texture-streaming-workbench.png",
    title: "Texture Streaming Workbench",
    description: "Texture streaming tab with residency tables, page requests, texture detail, and streaming output.",
    activeRail: "project",
    leftTab: "Textures",
    center: "texture-streaming",
    right: "texture-streaming-detail",
    bottom: "texture-streaming-output",
    status: "Texture Streaming: 82% residency / 18 misses",
  },
  {
    id: "shadow-map-workbench",
    output: "shadow-map-workbench.png",
    title: "Shadow Map Workbench",
    description: "Shadow map tab with cascade metrics, light lists, cascade detail, and shadow output.",
    activeRail: "scene",
    leftTab: "Lights",
    center: "shadow-map",
    right: "shadow-map-detail",
    bottom: "shadow-map-output",
    status: "Shadow Map: KeyLight / 4 cascades",
  },
  {
    id: "occlusion-culling-workbench",
    output: "occlusion-culling-workbench.png",
    title: "Occlusion Culling Workbench",
    description: "Occlusion culling tab with visibility sets, query batches, object detail, and culling output.",
    activeRail: "scene",
    leftTab: "Cells",
    center: "occlusion-culling",
    right: "occlusion-detail",
    bottom: "occlusion-output",
    status: "Occlusion Culling: A1_Hangar / 184 visible",
  },
  {
    id: "frame-compare-workbench",
    output: "frame-compare-workbench.png",
    title: "Frame Compare Workbench",
    description: "Frame compare tab with frame deltas, pass comparisons, frame detail, and compare output.",
    activeRail: "console",
    leftTab: "Captures",
    center: "frame-compare",
    right: "frame-compare-detail",
    bottom: "frame-compare-output",
    status: "Frame Compare: 1841 -> 1842 / +0.8 ms",
  },
  {
    id: "material-layers-workbench",
    output: "material-layers-workbench.png",
    title: "Material Layers Workbench",
    description: "Material layers tab with layer stacks, parameter overrides, layer detail, and compile output.",
    activeRail: "project",
    leftTab: "Layers",
    center: "material-layers",
    right: "material-layer-detail",
    bottom: "material-layer-output",
    status: "Material Layers: M_Armor / 5 layers",
  },
  {
    id: "gpu-memory-workbench",
    output: "gpu-memory-workbench.png",
    title: "GPU Memory Workbench",
    description: "GPU memory tab with heap usage, allocation groups, allocation detail, and memory output.",
    activeRail: "console",
    leftTab: "Heaps",
    center: "gpu-memory",
    right: "gpu-memory-detail",
    bottom: "gpu-memory-output",
    status: "GPU Memory: 2.4 GB used / 512 MB delta",
  },
  {
    id: "retarget-workbench",
    output: "retarget-workbench.png",
    title: "Retarget Workbench",
    description: "Retarget tab with skeleton mapping, chain rules, retarget detail, and solve output.",
    activeRail: "project",
    leftTab: "Skeletons",
    center: "retarget",
    right: "retarget-detail",
    bottom: "retarget-output",
    status: "Retarget: Humanoid_A -> Guard_B / 68 bones",
  },
  {
    id: "ik-solver-workbench",
    output: "ik-solver-workbench.png",
    title: "IK Solver Workbench",
    description: "IK solver tab with solver chains, effector watches, solver detail, and solve output.",
    activeRail: "scene",
    leftTab: "Solvers",
    center: "ik-solver",
    right: "ik-solver-detail",
    bottom: "ik-solver-output",
    status: "IK Solver: FullBodyIK / 4 effectors",
  },
  {
    id: "pose-library-workbench",
    output: "pose-library-workbench.png",
    title: "Pose Library Workbench",
    description: "Pose library tab with pose table, tags, pose detail, and library output.",
    activeRail: "project",
    leftTab: "Libraries",
    center: "pose-library",
    right: "pose-detail",
    bottom: "pose-library-output",
    status: "Pose Library: Combat / 842 poses",
  },
  {
    id: "mocap-cleanup-workbench",
    output: "mocap-cleanup-workbench.png",
    title: "Mocap Cleanup Workbench",
    description: "Mocap cleanup tab with take issues, cleanup filters, take detail, and cleanup output.",
    activeRail: "project",
    leftTab: "Takes",
    center: "mocap-cleanup",
    right: "mocap-detail",
    bottom: "mocap-output",
    status: "Mocap Cleanup: Take_042 / 18 issues",
  },
  {
    id: "animation-compression-workbench",
    output: "animation-compression-workbench.png",
    title: "Animation Compression Workbench",
    description: "Animation compression tab with clip compression, error metrics, clip detail, and compression output.",
    activeRail: "project",
    leftTab: "Clips",
    center: "animation-compression",
    right: "animation-compression-detail",
    bottom: "animation-compression-output",
    status: "Animation Compression: 42 clips / 68% saved",
  },
  {
    id: "root-motion-workbench",
    output: "root-motion-workbench.png",
    title: "Root Motion Workbench",
    description: "Root motion tab with motion curves, trajectory checks, motion detail, and extraction output.",
    activeRail: "scene",
    leftTab: "Clips",
    center: "root-motion",
    right: "root-motion-detail",
    bottom: "root-motion-output",
    status: "Root Motion: DashStrike / 4.2 m",
  },
  {
    id: "event-tracks-workbench",
    output: "event-tracks-workbench.png",
    title: "Event Tracks Workbench",
    description: "Event tracks tab with timeline events, notify rules, event detail, and validation output.",
    activeRail: "project",
    leftTab: "Tracks",
    center: "event-tracks",
    right: "event-track-detail",
    bottom: "event-track-output",
    status: "Event Tracks: Attack_Montage / 14 notifies",
  },
  {
    id: "montage-debugger-workbench",
    output: "montage-debugger-workbench.png",
    title: "Montage Debugger Workbench",
    description: "Montage debugger tab with runtime sections, blend watches, montage detail, and trace output.",
    activeRail: "console",
    leftTab: "Montages",
    center: "montage-debugger",
    right: "montage-debugger-detail",
    bottom: "montage-debugger-output",
    status: "Montage Debugger: Attack_Montage / Section B",
  },
  {
    id: "widget-tree-debugger-workbench",
    output: "widget-tree-debugger-workbench.png",
    title: "Widget Tree Debugger Workbench",
    description: "Widget tree debugger tab with widget nodes, layout watches, widget detail, and debug output.",
    activeRail: "ui",
    leftTab: "Widgets",
    center: "widget-tree-debugger",
    right: "widget-tree-detail",
    bottom: "widget-tree-output",
    status: "Widget Tree Debugger: Combat_HUD / 128 widgets",
  },
  {
    id: "layout-constraint-solver-workbench",
    output: "layout-constraint-solver-workbench.png",
    title: "Layout Constraint Solver Workbench",
    description: "Layout constraint solver tab with constraint rows, solve watches, constraint detail, and solve output.",
    activeRail: "ui",
    leftTab: "Layouts",
    center: "layout-constraint-solver",
    right: "layout-constraint-detail",
    bottom: "layout-constraint-output",
    status: "Layout Solver: InventoryPanel / 0 conflicts",
  },
  {
    id: "theme-variant-preview-workbench",
    output: "theme-variant-preview-workbench.png",
    title: "Theme Variant Preview Workbench",
    description: "Theme variant preview tab with token variants, contrast checks, token detail, and preview output.",
    activeRail: "ui",
    leftTab: "Themes",
    center: "theme-variant-preview",
    right: "theme-variant-detail",
    bottom: "theme-variant-output",
    status: "Theme Preview: Workbench Dark / AA",
  },
  {
    id: "localization-preview-workbench",
    output: "localization-preview-workbench.png",
    title: "Localization Preview Workbench",
    description: "Localization preview tab with string expansion, locale coverage, string detail, and localization output.",
    activeRail: "ui",
    leftTab: "Locales",
    center: "localization-preview",
    right: "localization-preview-detail",
    bottom: "localization-preview-output",
    status: "Localization Preview: zh-CN / 97%",
  },
  {
    id: "focus-navigation-workbench",
    output: "focus-navigation-workbench.png",
    title: "Focus Navigation Workbench",
    description: "Focus navigation tab with focus graph, route checks, focus detail, and navigation output.",
    activeRail: "ui",
    leftTab: "Screens",
    center: "focus-navigation",
    right: "focus-navigation-detail",
    bottom: "focus-navigation-output",
    status: "Focus Navigation: Inventory / 0 dead ends",
  },
  {
    id: "input-glyph-mapper-workbench",
    output: "input-glyph-mapper-workbench.png",
    title: "Input Glyph Mapper Workbench",
    description: "Input glyph mapper tab with device glyphs, platform mapping, glyph detail, and mapping output.",
    activeRail: "ui",
    leftTab: "Devices",
    center: "input-glyph-mapper",
    right: "input-glyph-detail",
    bottom: "input-glyph-output",
    status: "Input Glyph Mapper: Xbox / 64 prompts",
  },
  {
    id: "ui-snapshot-diff-workbench",
    output: "ui-snapshot-diff-workbench.png",
    title: "UI Snapshot Diff Workbench",
    description: "UI snapshot diff tab with snapshot deltas, visual checks, diff detail, and snapshot output.",
    activeRail: "ui",
    leftTab: "Snapshots",
    center: "ui-snapshot-diff",
    right: "ui-snapshot-detail",
    bottom: "ui-snapshot-output",
    status: "UI Snapshot Diff: 1841 -> 1842 / 4 diffs",
  },
  {
    id: "widget-performance-workbench",
    output: "widget-performance-workbench.png",
    title: "Widget Performance Workbench",
    description: "Widget performance tab with widget costs, invalidation groups, widget perf detail, and performance output.",
    activeRail: "console",
    leftTab: "Widgets",
    center: "widget-performance",
    right: "widget-performance-detail",
    bottom: "widget-performance-output",
    status: "Widget Performance: Combat_HUD / 1.2 ms",
  },
  {
    id: "world-partition-workbench",
    output: "world-partition-workbench.png",
    title: "World Partition Workbench",
    description: "World partition tab with cell states, streaming layers, cell detail, and partition output.",
    activeRail: "scene",
    leftTab: "Cells",
    center: "world-partition",
    right: "world-partition-detail",
    bottom: "world-partition-output",
    status: "World Partition: A1_Hangar / 42 cells",
  },
  {
    id: "hlod-builder-workbench",
    output: "hlod-builder-workbench.png",
    title: "HLOD Builder Workbench",
    description: "HLOD builder tab with cluster tables, merge rules, cluster detail, and build output.",
    activeRail: "scene",
    leftTab: "Clusters",
    center: "hlod-builder",
    right: "hlod-detail",
    bottom: "hlod-output",
    status: "HLOD Builder: Hangar_Exterior / 12 clusters",
  },
  {
    id: "level-instance-workbench",
    output: "level-instance-workbench.png",
    title: "Level Instance Workbench",
    description: "Level instance tab with instance lists, overrides, instance detail, and validation output.",
    activeRail: "scene",
    leftTab: "Instances",
    center: "level-instance",
    right: "level-instance-detail",
    bottom: "level-instance-output",
    status: "Level Instance: DockModule_A / 6 instances",
  },
  {
    id: "streaming-profiler-workbench",
    output: "streaming-profiler-workbench.png",
    title: "Streaming Profiler Workbench",
    description: "Streaming profiler tab with load events, memory bands, stream detail, and profiler output.",
    activeRail: "console",
    leftTab: "Captures",
    center: "streaming-profiler",
    right: "streaming-profiler-detail",
    bottom: "streaming-profiler-output",
    status: "Streaming Profiler: Live / 184 MB peak",
  },
  {
    id: "scene-bookmarks-workbench",
    output: "scene-bookmarks-workbench.png",
    title: "Scene Bookmarks Workbench",
    description: "Scene bookmarks tab with bookmark lists, camera targets, bookmark detail, and navigation output.",
    activeRail: "scene",
    leftTab: "Bookmarks",
    center: "scene-bookmarks",
    right: "scene-bookmark-detail",
    bottom: "scene-bookmark-output",
    status: "Scene Bookmarks: A1_Hangar / 18 bookmarks",
  },
  {
    id: "spawn-point-editor-workbench",
    output: "spawn-point-editor-workbench.png",
    title: "Spawn Point Editor Workbench",
    description: "Spawn point editor tab with spawn points, validation checks, spawn detail, and simulation output.",
    activeRail: "scene",
    leftTab: "Spawn Sets",
    center: "spawn-point-editor",
    right: "spawn-point-detail",
    bottom: "spawn-point-output",
    status: "Spawn Points: HangarWave_A / 32 points",
  },
  {
    id: "collision-matrix-workbench",
    output: "collision-matrix-workbench.png",
    title: "Collision Matrix Workbench",
    description: "Collision matrix tab with collision channels, response rules, channel detail, and validation output.",
    activeRail: "project",
    leftTab: "Channels",
    center: "collision-matrix",
    right: "collision-matrix-detail",
    bottom: "collision-matrix-output",
    status: "Collision Matrix: 18 channels / 0 conflicts",
  },
  {
    id: "environment-probes-workbench",
    output: "environment-probes-workbench.png",
    title: "Environment Probes Workbench",
    description: "Environment probes tab with probe coverage, bake queues, probe detail, and bake output.",
    activeRail: "scene",
    leftTab: "Probes",
    center: "environment-probes",
    right: "environment-probe-detail",
    bottom: "environment-probe-output",
    status: "Environment Probes: Hangar / 64 probes",
  },
  {
    id: "feature-flags-workbench",
    output: "feature-flags-workbench.png",
    title: "Feature Flags Workbench",
    description: "Feature flags tab with rollout flags, audience rules, flag detail, and rollout output.",
    activeRail: "project",
    leftTab: "Flags",
    center: "feature-flags",
    right: "feature-flag-detail",
    bottom: "feature-flag-output",
    status: "Feature Flags: 12 live / 2 staged",
  },
  {
    id: "remote-config-workbench",
    output: "remote-config-workbench.png",
    title: "Remote Config Workbench",
    description: "Remote config tab with live keys, diff preview, key detail, and publish output.",
    activeRail: "project",
    leftTab: "Configs",
    center: "remote-config",
    right: "remote-config-detail",
    bottom: "remote-config-output",
    status: "Remote Config: Live v42 / 3 drafts",
  },
  {
    id: "telemetry-query-workbench",
    output: "telemetry-query-workbench.png",
    title: "Telemetry Query Workbench",
    description: "Telemetry query tab with funnel metrics, dimensions, query detail, and run output.",
    activeRail: "console",
    leftTab: "Queries",
    center: "telemetry-query",
    right: "telemetry-query-detail",
    bottom: "telemetry-query-output",
    status: "Telemetry Query: Session Funnel / 1.2k events",
  },
  {
    id: "patch-planner-workbench",
    output: "patch-planner-workbench.png",
    title: "Patch Planner Workbench",
    description: "Patch planner tab with patch changes, release gates, patch detail, and validation output.",
    activeRail: "project",
    leftTab: "Patches",
    center: "patch-planner",
    right: "patch-planner-detail",
    bottom: "patch-planner-output",
    status: "Patch Planner: 0.18.1 / 24 changes",
  },
  {
    id: "dlc-catalog-workbench",
    output: "dlc-catalog-workbench.png",
    title: "DLC Catalog Workbench",
    description: "DLC catalog tab with pack SKUs, entitlement checks, DLC detail, and packaging output.",
    activeRail: "project",
    leftTab: "DLC",
    center: "dlc-catalog",
    right: "dlc-detail",
    bottom: "dlc-output",
    status: "DLC Catalog: FounderPack / 8 packs",
  },
  {
    id: "crash-symbolication-workbench",
    output: "crash-symbolication-workbench.png",
    title: "Crash Symbolication Workbench",
    description: "Crash symbolication tab with crash queue, symbol status, stack detail, and symbol output.",
    activeRail: "console",
    leftTab: "Crashes",
    center: "crash-symbolication",
    right: "symbolication-detail",
    bottom: "symbolication-output",
    status: "Crash Symbolication: 184 crashes / 92% resolved",
  },
  {
    id: "player-segment-workbench",
    output: "player-segment-workbench.png",
    title: "Player Segment Workbench",
    description: "Player segment tab with cohort metrics, audience rules, segment detail, and refresh output.",
    activeRail: "project",
    leftTab: "Segments",
    center: "player-segment",
    right: "segment-detail",
    bottom: "segment-output",
    status: "Player Segment: Returning Players / 42k users",
  },
  {
    id: "experiment-console-workbench",
    output: "experiment-console-workbench.png",
    title: "Experiment Console Workbench",
    description: "Experiment console tab with live experiments, metric cards, experiment detail, and analysis output.",
    activeRail: "project",
    leftTab: "Experiments",
    center: "experiment-console",
    right: "experiment-detail",
    bottom: "experiment-output",
    status: "Experiment Console: Store Banner / 3 live",
  },
];

const FOCUS_DESIGNS = [
  {
    id: "scene-toolbar-focus",
    output: "scene-toolbar-focus.png",
    title: "Scene Viewport Toolbar",
    base: "scene-workbench",
    focus: "scene-toolbar",
  },
  {
    id: "scene-gizmo-focus",
    output: "scene-gizmo-focus.png",
    title: "Scene Gizmo And Selection",
    base: "scene-workbench",
    focus: "scene-gizmo",
  },
  {
    id: "hierarchy-selection-focus",
    output: "hierarchy-selection-focus.png",
    title: "Hierarchy Selection",
    base: "hierarchy-workbench",
    focus: "hierarchy-selection",
  },
  {
    id: "hierarchy-context-menu-focus",
    output: "hierarchy-context-menu-focus.png",
    title: "Hierarchy Context Menu",
    base: "hierarchy-workbench",
    focus: "hierarchy-context-menu",
  },
  {
    id: "inspector-transform-focus",
    output: "inspector-transform-focus.png",
    title: "Inspector Transform",
    base: "inspector-workbench",
    focus: "inspector-transform",
  },
  {
    id: "inspector-material-focus",
    output: "inspector-material-focus.png",
    title: "Inspector Material",
    base: "inspector-workbench",
    focus: "inspector-material",
  },
  {
    id: "asset-browser-grid-focus",
    output: "asset-browser-grid-focus.png",
    title: "Asset Browser Grid",
    base: "asset-browser-workbench",
    focus: "asset-grid",
  },
  {
    id: "asset-browser-import-focus",
    output: "asset-browser-import-focus.png",
    title: "Asset Import Details",
    base: "asset-browser-workbench",
    focus: "asset-import",
  },
  {
    id: "console-log-filter-focus",
    output: "console-log-filter-focus.png",
    title: "Console Log Filters",
    base: "console-workbench",
    focus: "console-filter",
  },
  {
    id: "console-detail-focus",
    output: "console-detail-focus.png",
    title: "Console Detail",
    base: "console-workbench",
    focus: "console-detail",
  },
  {
    id: "project-overview-dashboard-focus",
    output: "project-overview-dashboard-focus.png",
    title: "Project Dashboard",
    base: "project-overview-workbench",
    focus: "project-dashboard",
  },
  {
    id: "project-overview-actions-focus",
    output: "project-overview-actions-focus.png",
    title: "Project Actions",
    base: "project-overview-workbench",
    focus: "project-actions",
  },
  {
    id: "material-lab-components-focus",
    output: "material-lab-components-focus.png",
    title: "Material Lab Components",
    base: "material-lab-workbench",
    focus: "material-components",
  },
  {
    id: "ui-asset-tree-focus",
    output: "ui-asset-tree-focus.png",
    title: "UI Asset Tree",
    base: "ui-asset-editor-workbench",
    focus: "ui-asset-tree",
  },
  {
    id: "animation-timeline-focus",
    output: "animation-timeline-focus.png",
    title: "Animation Timeline",
    base: "animation-workbench",
    focus: "animation-timeline",
  },
  {
    id: "performance-frame-focus",
    output: "performance-frame-focus.png",
    title: "Performance Frame Detail",
    base: "performance-workbench",
    focus: "performance-frame",
  },
  {
    id: "runtime-diagnostics-events-focus",
    output: "runtime-diagnostics-events-focus.png",
    title: "Runtime Diagnostic Events",
    base: "runtime-diagnostics-workbench",
    focus: "runtime-events",
  },
  {
    id: "plugin-manager-detail-focus",
    output: "plugin-manager-detail-focus.png",
    title: "Plugin Manager Detail",
    base: "plugin-manager-workbench",
    focus: "plugin-detail",
  },
  {
    id: "build-export-targets-focus",
    output: "build-export-targets-focus.png",
    title: "Build Export Targets",
    base: "build-export-workbench",
    focus: "build-targets",
  },
  {
    id: "welcome-new-project-focus",
    output: "welcome-new-project-focus.png",
    title: "Welcome New Project",
    base: "welcome-workbench",
    focus: "welcome-new-project",
  },
];

const LAYOUT_SPEC_DESIGNS = [
  {
    id: "main-tabs-layout-spec",
    output: "main-tabs-layout-spec.png",
    title: "Main Tabs Layout Spec",
    description: "Top-level editor windows as document tabs for scene, material, montage, UI, assets, diagnostics, and project.",
    spec: "main-tabs",
  },
  {
    id: "tool-drawers-layout-spec",
    output: "tool-drawers-layout-spec.png",
    title: "Tool Drawers Layout Spec",
    description: "Canonical left-top, left-bottom, right-top, right-bottom, and bottom drawer placement.",
    spec: "tool-drawers",
  },
  {
    id: "scene-drawer-layout-spec",
    output: "scene-drawer-layout-spec.png",
    title: "Scene Drawer Layout Spec",
    description: "Scene Editor tab with prefab placement, files, hierarchy, inspector, and output drawers.",
    spec: "scene",
  },
  {
    id: "material-drawer-layout-spec",
    output: "material-drawer-layout-spec.png",
    title: "Material Drawer Layout Spec",
    description: "Material Editor tab with node palette, file tree, parameters, preview details, and shader output.",
    spec: "material",
  },
  {
    id: "montage-drawer-layout-spec",
    output: "montage-drawer-layout-spec.png",
    title: "Montage Drawer Layout Spec",
    description: "Montage Editor tab with clip library, skeleton hierarchy, animation list, and bottom timeline.",
    spec: "montage",
  },
  {
    id: "ui-asset-drawer-layout-spec",
    output: "ui-asset-drawer-layout-spec.png",
    title: "UI Asset Drawer Layout Spec",
    description: "UI Asset Editor tab with widget palette, file tree, UI hierarchy, property inspector, and diagnostics.",
    spec: "ui-asset",
  },
];

const STATE_SPEC_DESIGNS = [
  {
    id: "drawer-collapsed-state-spec",
    output: "drawer-collapsed-state-spec.png",
    title: "Drawer Collapsed State Spec",
    description: "Collapsed drawer rails preserve slot identity while giving width back to the active editor.",
    state: "collapsed",
  },
  {
    id: "drawer-expanded-state-spec",
    output: "drawer-expanded-state-spec.png",
    title: "Drawer Expanded State Spec",
    description: "Expanded drawer state for inspection-heavy workflows with pinned right-side tools.",
    state: "expanded",
  },
  {
    id: "split-editor-state-spec",
    output: "split-editor-state-spec.png",
    title: "Split Editor State Spec",
    description: "Two main editor panes under the same top-level tab for graph/preview or source/preview work.",
    state: "split-editor",
  },
  {
    id: "bottom-timeline-console-state-spec",
    output: "bottom-timeline-console-state-spec.png",
    title: "Bottom Timeline Console State Spec",
    description: "Bottom drawer split between a wide timeline and a compact output console.",
    state: "bottom-split",
  },
  {
    id: "floating-tool-window-state-spec",
    output: "floating-tool-window-state-spec.png",
    title: "Floating Tool Window State Spec",
    description: "Undocked tool window keeps the same drawer role while floating above the active editor tab.",
    state: "floating",
  },
  {
    id: "compact-editor-state-spec",
    output: "compact-editor-state-spec.png",
    title: "Compact Editor State Spec",
    description: "Compact workspace keeps center editing usable by collapsing secondary drawers first.",
    state: "compact",
  },
];

const CONTENT_SPEC_DESIGNS = [
  {
    id: "prefab-drawer-content-spec",
    output: "prefab-drawer-content-spec.png",
    title: "Prefab Drawer Content Spec",
    description: "Left-top placement drawer content: prefab shelf, filters, quick actions, and compact item density.",
    content: "prefab",
  },
  {
    id: "files-drawer-content-spec",
    output: "files-drawer-content-spec.png",
    title: "Files Drawer Content Spec",
    description: "Left-bottom file/project drawer content: project tree, asset folders, path chips, and file actions.",
    content: "files",
  },
  {
    id: "hierarchy-drawer-content-spec",
    output: "hierarchy-drawer-content-spec.png",
    title: "Hierarchy Drawer Content Spec",
    description: "Right-top structure drawer content: scene hierarchy, outline controls, selection and visibility states.",
    content: "hierarchy",
  },
  {
    id: "inspector-drawer-content-spec",
    output: "inspector-drawer-content-spec.png",
    title: "Inspector Drawer Content Spec",
    description: "Right-bottom detail drawer content: object header, transform, renderer, material, and component sections.",
    content: "inspector",
  },
  {
    id: "animation-list-drawer-content-spec",
    output: "animation-list-drawer-content-spec.png",
    title: "Animation List Drawer Content Spec",
    description: "Right-bottom animation drawer content: clips, tracks, events, and key selection state.",
    content: "animation-list",
  },
  {
    id: "console-drawer-content-spec",
    output: "console-drawer-content-spec.png",
    title: "Console Drawer Content Spec",
    description: "Bottom output drawer content: filters, warning rows, detail drilldown, and status grouping.",
    content: "console",
  },
  {
    id: "timeline-drawer-content-spec",
    output: "timeline-drawer-content-spec.png",
    title: "Timeline Drawer Content Spec",
    description: "Bottom timeline drawer content: tracks, keyframes, events, and curve tabs.",
    content: "timeline",
  },
  {
    id: "asset-grid-drawer-content-spec",
    output: "asset-grid-drawer-content-spec.png",
    title: "Asset Grid Drawer Content Spec",
    description: "Asset browser content: folder toolbar, grid/list toggle, tiles, import detail, and metadata rows.",
    content: "asset-grid",
  },
];

const OVERLAY_SPEC_DESIGNS = [
  {
    id: "command-palette-window-spec",
    output: "command-palette-window-spec.png",
    title: "Command Palette Window Spec",
    description: "Search-first command launcher overlay with grouped commands and keyboard hints.",
    overlay: "command-palette",
  },
  {
    id: "context-menu-window-spec",
    output: "context-menu-window-spec.png",
    title: "Context Menu Window Spec",
    description: "Right-click menu, nested menu, disabled state, destructive action, and shortcut text.",
    overlay: "context-menu",
  },
  {
    id: "tab-overflow-window-spec",
    output: "tab-overflow-window-spec.png",
    title: "Tab Overflow Window Spec",
    description: "Main editor tab overflow dropdown with search, pinned tabs, and recent editor windows.",
    overlay: "tab-overflow",
  },
  {
    id: "asset-picker-window-spec",
    output: "asset-picker-window-spec.png",
    title: "Asset Picker Window Spec",
    description: "Modal asset picker with search, filter chips, grid/list results, and selected asset details.",
    overlay: "asset-picker",
  },
  {
    id: "import-wizard-window-spec",
    output: "import-wizard-window-spec.png",
    title: "Import Wizard Window Spec",
    description: "Step-based import window for meshes, textures, and UI assets.",
    overlay: "import-wizard",
  },
  {
    id: "project-settings-window-spec",
    output: "project-settings-window-spec.png",
    title: "Project Settings Window Spec",
    description: "Settings window with navigation list, grouped preferences, switches, and footer actions.",
    overlay: "project-settings",
  },
  {
    id: "confirm-dialog-window-spec",
    output: "confirm-dialog-window-spec.png",
    title: "Confirm Dialog Window Spec",
    description: "Small destructive confirmation dialog with concise copy and flat action buttons.",
    overlay: "confirm-dialog",
  },
  {
    id: "notification-center-window-spec",
    output: "notification-center-window-spec.png",
    title: "Notification Center Window Spec",
    description: "Non-blocking notification tray with grouped build, import, and runtime messages.",
    overlay: "notification-center",
  },
];

const WORKFLOW_SPEC_DESIGNS = [
  {
    id: "prefab-placement-workflow-spec",
    output: "prefab-placement-workflow-spec.png",
    title: "Prefab Placement Workflow Spec",
    description: "Scene placement flow using left-top prefabs, center viewport, right hierarchy, right detail, and bottom output.",
    workflow: "prefab-placement",
  },
  {
    id: "asset-import-workflow-spec",
    output: "asset-import-workflow-spec.png",
    title: "Asset Import Workflow Spec",
    description: "Asset import flow from file drawer and asset grid into a compact import review window.",
    workflow: "asset-import",
  },
  {
    id: "shader-error-workflow-spec",
    output: "shader-error-workflow-spec.png",
    title: "Shader Error Workflow Spec",
    description: "Shader edit and compile failure flow with source, preview, pipeline settings, and bottom diagnostics.",
    workflow: "shader-error",
  },
  {
    id: "animation-event-workflow-spec",
    output: "animation-event-workflow-spec.png",
    title: "Animation Event Workflow Spec",
    description: "Montage event authoring flow with rig tree, graph, animation list, property detail, and timeline output.",
    workflow: "animation-event",
  },
  {
    id: "runtime-debug-workflow-spec",
    output: "runtime-debug-workflow-spec.png",
    title: "Runtime Debug Workflow Spec",
    description: "Runtime diagnostics drilldown with live event channels, detail inspector, and filtered console output.",
    workflow: "runtime-debug",
  },
  {
    id: "build-export-workflow-spec",
    output: "build-export-workflow-spec.png",
    title: "Build Export Workflow Spec",
    description: "Desktop build export flow with target selection, validation settings, queue, and package log.",
    workflow: "build-export",
  },
  {
    id: "ui-binding-workflow-spec",
    output: "ui-binding-workflow-spec.png",
    title: "UI Binding Workflow Spec",
    description: "UI asset binding flow across widget tree, source/preview editor, property detail, and diagnostics.",
    workflow: "ui-binding",
  },
  {
    id: "lighting-bake-workflow-spec",
    output: "lighting-bake-workflow-spec.png",
    title: "Lighting Bake Workflow Spec",
    description: "Lighting bake setup flow with lights drawer, viewport, bake settings, probe detail, and job output.",
    workflow: "lighting-bake",
  },
];

const FLOATING_WINDOW_DESIGNS = [
  {
    id: "preferences-window-workbench",
    output: "preferences-window-workbench.png",
    title: "Preferences Window Workbench",
    description: "Editor preferences window with category navigation, compact fields, and flat apply actions.",
    window: "preferences",
  },
  {
    id: "keyboard-shortcuts-window-workbench",
    output: "keyboard-shortcuts-window-workbench.png",
    title: "Keyboard Shortcuts Window Workbench",
    description: "Shortcut search and command binding window with JetBrains-like keymap editing density.",
    window: "keyboard-shortcuts",
  },
  {
    id: "reimport-conflict-window-workbench",
    output: "reimport-conflict-window-workbench.png",
    title: "Reimport Conflict Window Workbench",
    description: "Asset reimport conflict window with source/destination comparison and resolution actions.",
    window: "reimport-conflict",
  },
  {
    id: "source-control-submit-window-workbench",
    output: "source-control-submit-window-workbench.png",
    title: "Source Control Submit Window Workbench",
    description: "Submit changelist window with file selection, diff preview, validation, and commit message.",
    window: "source-control-submit",
  },
  {
    id: "crash-report-window-workbench",
    output: "crash-report-window-workbench.png",
    title: "Crash Report Window Workbench",
    description: "Crash report window with stack summary, attachments, recovery options, and send controls.",
    window: "crash-report",
  },
  {
    id: "find-in-project-window-workbench",
    output: "find-in-project-window-workbench.png",
    title: "Find In Project Window Workbench",
    description: "Find window with filters, scoped results, preview, and replace-ready controls.",
    window: "find-in-project",
  },
  {
    id: "startup-tasks-window-workbench",
    output: "startup-tasks-window-workbench.png",
    title: "Startup Tasks Window Workbench",
    description: "Startup task window for project open, asset indexing, shader warmup, and plugin loading.",
    window: "startup-tasks",
  },
  {
    id: "editor-update-window-workbench",
    output: "editor-update-window-workbench.png",
    title: "Editor Update Window Workbench",
    description: "Editor update window with release notes, component selection, and restart/install actions.",
    window: "editor-update",
  },
];

const ALL_DESIGNS = [
  ...FULL_DESIGNS,
  ...FOCUS_DESIGNS,
  ...LAYOUT_SPEC_DESIGNS,
  ...STATE_SPEC_DESIGNS,
  ...CONTENT_SPEC_DESIGNS,
  ...OVERLAY_SPEC_DESIGNS,
  ...WORKFLOW_SPEC_DESIGNS,
  ...FLOATING_WINDOW_DESIGNS,
];

const treeRows = [
  ["Root", 0, false],
  ["Environment", 1, false],
  ["Lighting", 2, false],
  ["Sky", 2, false],
  ["Level", 1, false],
  ["Geometry", 2, false],
  ["Props", 2, true],
  ["Crate_01", 3, false],
  ["PlayerStart", 1, false],
  ["AudioZone", 1, false],
];

const assetItems = [
  ["A1_Hangar.scene", "scene"],
  ["Box_01.mesh", "mesh"],
  ["M_Metal.zmat", "material"],
  ["T_Grid_01.png", "texture"],
  ["SM_Railing.mesh", "mesh"],
  ["AudioZone.prefab", "prefab"],
  ["P_Sparks.fx", "effect"],
  ["UI_Workbench.zui", "ui"],
  ["unlit.zshader", "shader"],
  ["PlayerStart.prefab", "prefab"],
];

const logRows = [
  ["12:04:18", "Info", "Renderer data compiled for A1_Hangar.scene"],
  ["12:04:22", "Info", "Material M_Metal bound to Box_01"],
  ["12:04:25", "Warn", "Texture T_Grid_01 uses fallback sampler"],
  ["12:04:29", "Info", "Viewport camera projection updated"],
  ["12:04:31", "Warn", "AudioZone has no listener route in editor preview"],
  ["12:04:37", "Info", "Asset package cache hit: 42 entries"],
  ["12:04:41", "Info", "UI surface frame rebuilt: 318 nodes"],
  ["12:04:46", "Info", "Scene picking target: Props / Crate_01"],
];

function main() {
  const params = new URLSearchParams(window.location.search);
  const id = params.get("design") ?? "scene-workbench";
  const root = document.querySelector("#design-root");
  root.innerHTML = "";

  if (id === "sheet") {
    root.append(renderPreviewSheet(ALL_DESIGNS));
    return;
  }

  const design = ALL_DESIGNS.find((entry) => entry.id === id) ?? FULL_DESIGNS[0];
  if (LAYOUT_SPEC_DESIGNS.includes(design)) {
    root.append(renderLayoutSpec(design));
    return;
  }

  if (STATE_SPEC_DESIGNS.includes(design)) {
    root.append(renderStateSpec(design));
    return;
  }

  if (CONTENT_SPEC_DESIGNS.includes(design)) {
    root.append(renderContentSpec(design));
    return;
  }

  if (OVERLAY_SPEC_DESIGNS.includes(design)) {
    root.append(renderOverlaySpec(design));
    return;
  }

  if (WORKFLOW_SPEC_DESIGNS.includes(design)) {
    root.append(renderWorkflowSpec(design));
    return;
  }

  if (FLOATING_WINDOW_DESIGNS.includes(design)) {
    root.append(renderFloatingWindowSpec(design));
    return;
  }

  if (FOCUS_DESIGNS.includes(design)) {
    const base = FULL_DESIGNS.find((entry) => entry.id === design.base) ?? FULL_DESIGNS[0];
    const frame = renderShell(base);
    frame.append(renderFocusLayer(design));
    root.append(frame);
    return;
  }

  root.append(renderShell(design));
}

function renderLayoutSpec(design) {
  const spec = el("div", "layout-spec-frame");
  const header = el("div", "layout-spec-header");
  header.innerHTML = `<div><strong>${design.title}</strong><span>${design.description}</span></div><div class="pill">1672 x 941 layout reference</div>`;

  const tabs = renderSpecMainTabs(design.spec);
  const body = el("div", "layout-spec-body");
  if (design.spec === "main-tabs") {
    body.append(renderMainTabsMatrix(), renderSpecNotes("Main editor tab contract", [
      "Each main tab owns a complete editor-window layout, not just a document body.",
      "Tabs mirror Unreal-style editor windows while the shell keeps JetBrains-style tab switching.",
      "Scene, Material, Montage, UI Asset, Asset Browser, Diagnostics, and Project all share the same drawer grammar.",
      "The active tab can swap the center editor without moving persistent project/file drawers.",
    ]));
  } else {
    body.append(renderDrawerBlueprint(design.spec), renderDrawerRoleTable(design.spec));
  }

  spec.append(header, tabs, body);
  return spec;
}

function renderSpecMainTabs(activeSpec) {
  const tabs = el("div", "spec-main-tabs");
  [
    ["scene", "Scene Editor"],
    ["material", "Material Editor"],
    ["montage", "Montage Editor"],
    ["ui-asset", "UI Asset Editor"],
    ["assets", "Asset Browser"],
    ["diagnostics", "Diagnostics"],
    ["project", "Project"],
  ].forEach(([id, label]) => {
    const tab = el("div", `spec-tab ${activeSpec === id || (activeSpec === "main-tabs" && id === "scene") ? "active" : ""}`);
    tab.textContent = label;
    tabs.append(tab);
  });
  return tabs;
}

function renderMainTabsMatrix() {
  const matrix = el("div", "main-tabs-matrix");
  [
    ["Scene Editor", "Viewport, transforms, prefabs, hierarchy, inspector, output"],
    ["Material Editor", "Node palette, material canvas, parameters, preview asset, shader output"],
    ["Montage Editor", "Clip library, graph/curves, skeleton, animation list, timeline"],
    ["UI Asset Editor", "Widget palette, source/preview, UI tree, bindings, diagnostics"],
    ["Asset Browser", "Folder tree, grid/list browser, import details, import queue"],
    ["Diagnostics", "Capture tools, runtime tree, frame/detail panes, console"],
    ["Project", "Actions, modules/plugins, build profiles, activity output"],
  ].forEach(([title, desc], index) => {
    const item = el("div", `matrix-item ${index === 0 ? "active" : ""}`);
    item.innerHTML = `<strong>${title}</strong><span>${desc}</span>`;
    matrix.append(item);
  });
  return matrix;
}

function renderDrawerBlueprint(specId) {
  const cfg = drawerSpecConfig(specId);
  const blueprint = el("div", "drawer-blueprint");
  const center = el("div", "blueprint-center");
  center.innerHTML = `<strong>${cfg.editor}</strong><span>${cfg.center}</span>`;

  [
    ["left-top", cfg.leftTop],
    ["left-bottom", cfg.leftBottom],
    ["right-top", cfg.rightTop],
    ["right-bottom", cfg.rightBottom],
    ["bottom", cfg.bottom],
  ].forEach(([slot, data]) => {
    const zone = el("div", `blueprint-zone ${slot}`);
    zone.innerHTML = `<strong>${data.title}</strong><span>${data.role}</span><small>${data.examples}</small>`;
    blueprint.append(zone);
  });
  blueprint.append(center);
  return blueprint;
}

function renderDrawerRoleTable(specId) {
  const cfg = drawerSpecConfig(specId);
  const table = el("div", "drawer-role-table");
  table.append(renderSpecNotes(`${cfg.editor} drawer contract`, cfg.notes));
  const rows = [
    ["Left Top", cfg.leftTop],
    ["Left Bottom", cfg.leftBottom],
    ["Right Top", cfg.rightTop],
    ["Right Bottom", cfg.rightBottom],
    ["Bottom", cfg.bottom],
  ];
  rows.forEach(([slot, data]) => {
    const row = el("div", "drawer-role-row");
    row.innerHTML = `<strong>${slot}</strong><span>${data.title}</span><em>${data.role}</em><small>${data.examples}</small>`;
    table.append(row);
  });
  return table;
}

function renderSpecNotes(title, rows) {
  const notes = el("div", "spec-notes");
  notes.innerHTML = `<h2>${title}</h2>`;
  rows.forEach((row) => {
    const item = el("div", "spec-note");
    item.textContent = row;
    notes.append(item);
  });
  return notes;
}

function drawerSpecConfig(specId) {
  const base = {
    editor: "Tool Drawer Layout",
    center: "Main editor document canvas",
    leftTop: { title: "Tool Shelf", role: "Fast placement and authoring tools", examples: "Prefabs, node palette, clip tools" },
    leftBottom: { title: "Files", role: "Project and asset navigation", examples: "Project tree, content folders" },
    rightTop: { title: "Structure", role: "Object hierarchy or document outline", examples: "Hierarchy, UI tree, skeleton" },
    rightBottom: { title: "Details", role: "Properties and contextual lists", examples: "Inspector, animation list, import details" },
    bottom: { title: "Output", role: "Long horizontal working surfaces", examples: "Console, timeline, diagnostics, import queue" },
    notes: [
      "Drawers are persistent tool windows around the active main editor tab.",
      "Each drawer can collapse into its header but keeps the same slot identity.",
      "Bottom drawers are reserved for wide data: output, timelines, queues, and diagnostics.",
      "The center editor is never a marketing card; it is the active tool surface.",
    ],
  };

  const configs = {
    scene: {
      editor: "Scene Editor",
      center: "3D viewport with toolbar, transform gizmo, selection and camera controls",
      leftTop: { title: "Prefabs", role: "Placement shelf", examples: "Crate, light, railing, player start" },
      leftBottom: { title: "Files", role: "Project navigation", examples: "Scenes, materials, meshes, textures" },
      rightTop: { title: "Hierarchy", role: "Scene object structure", examples: "Root, Environment, Lighting, Props" },
      rightBottom: { title: "Inspector", role: "Selected object properties", examples: "Transform, mesh renderer, material" },
      bottom: { title: "Output", role: "Editor feedback", examples: "UI components, console, warnings" },
      notes: [
        "Scene editing keeps placement tools near the upper-left for repeated drag/drop use.",
        "Hierarchy stays right-top so object structure is close to inspector details.",
        "The bottom drawer can switch between console, component lab, timeline, or diagnostics.",
      ],
    },
    material: {
      editor: "Material Editor",
      center: "Material canvas or component state matrix",
      leftTop: { title: "Material Nodes", role: "Node palette", examples: "Inputs, surfaces, navigation, data display" },
      leftBottom: { title: "Files", role: "Material asset navigation", examples: "Materials, textures, shaders" },
      rightTop: { title: "Parameters", role: "Selected node state", examples: "Tokens, variants, selected state" },
      rightBottom: { title: "Preview Asset", role: "Preview mesh and import data", examples: "Box mesh, material slots, readiness" },
      bottom: { title: "Material Output", role: "Compile and shader feedback", examples: "Preview log, shader, visual diff" },
      notes: [
        "Material authoring uses left-top for palette actions and right-top for selected parameter state.",
        "Preview details sit right-bottom so shader feedback stays separate in the bottom drawer.",
        "The center can switch between node graph, compact matrix, or preview surface.",
      ],
    },
    montage: {
      editor: "Montage Editor",
      center: "Animation graph, blend tree, curves or montage canvas",
      leftTop: { title: "Clip Library", role: "Animation source picker", examples: "Montages, clips, blend spaces" },
      leftBottom: { title: "Files", role: "Animation asset navigation", examples: "Characters, rigs, animation sets" },
      rightTop: { title: "Skeleton", role: "Rig hierarchy", examples: "Character root, hips, spine, arms" },
      rightBottom: { title: "Animation List", role: "Clip and event details", examples: "Tracks, events, frame properties" },
      bottom: { title: "Timeline", role: "Wide keyframe editing", examples: "Dope sheet, curves, events" },
      notes: [
        "Montage editing reserves the bottom drawer for timeline-first work.",
        "Skeleton hierarchy belongs right-top; selected track/event detail belongs right-bottom.",
        "Clip library remains left-top for fast source switching.",
      ],
    },
    "ui-asset": {
      editor: "UI Asset Editor",
      center: "Source, retained preview, compiler diagnostics, and bindings",
      leftTop: { title: "Widget Palette", role: "Widget and prefab insertion", examples: "Controls, slots, layout prefabs" },
      leftBottom: { title: "Files", role: "UI asset navigation", examples: "UI docs, icons, style files" },
      rightTop: { title: "UI Tree", role: "Document structure", examples: "WorkbenchShell, TopBar, DocumentHost" },
      rightBottom: { title: "Properties", role: "Selected node settings", examples: "Component, binding, clip, style token" },
      bottom: { title: "Diagnostics", role: "Compiler and binding output", examples: "Compiler, resources, missing refs" },
      notes: [
        "UI editing mirrors IDE layout: source/preview center, tree right-top, properties right-bottom.",
        "Widget palette stays left-top for authoring speed, while files stay left-bottom.",
        "Compiler diagnostics must live in the bottom drawer to avoid stealing editor width.",
      ],
    },
  };

  return configs[specId] ?? base;
}

function renderStateSpec(design) {
  const spec = el("div", "layout-spec-frame state-spec-frame");
  const header = el("div", "layout-spec-header");
  header.innerHTML = `<div><strong>${design.title}</strong><span>${design.description}</span></div><div class="pill">1672 x 941 state reference</div>`;

  const tabs = renderSpecMainTabs(stateActiveTab(design.state));
  const body = el("div", "state-spec-body");
  body.append(renderStateBlueprint(design.state), renderStateRules(design.state));
  spec.append(header, tabs, body);
  return spec;
}

function stateActiveTab(state) {
  if (state === "split-editor" || state === "floating") return "material";
  if (state === "bottom-split") return "montage";
  if (state === "compact") return "scene";
  return "scene";
}

function renderStateBlueprint(state) {
  const cfg = stateSpecConfig(state);
  const root = el("div", `state-blueprint ${state}`);
  root.innerHTML = `
    <div class="state-zone state-left-top">${cfg.leftTop}</div>
    <div class="state-zone state-left-bottom">${cfg.leftBottom}</div>
    <div class="state-zone state-right-top">${cfg.rightTop}</div>
    <div class="state-zone state-right-bottom">${cfg.rightBottom}</div>
    <div class="state-center">${cfg.center}</div>
    <div class="state-bottom">${cfg.bottom}</div>
  `;
  if (state === "split-editor") {
    root.querySelector(".state-center").innerHTML = `<div class="split-pane">Graph</div><div class="split-pane">Preview</div>`;
  }
  if (state === "bottom-split") {
    root.querySelector(".state-bottom").innerHTML = `<div class="bottom-major">Timeline</div><div class="bottom-minor">Console</div>`;
  }
  if (state === "floating") {
    const floating = el("div", "floating-window-spec");
    floating.innerHTML = `<strong>Floating Parameters</strong><span>Undocked from Right Top</span><small>Same drawer role, temporary screen position</small>`;
    root.append(floating);
  }
  if (state === "collapsed" || state === "compact") {
    ["left-top", "left-bottom", "right-top", "right-bottom"].forEach((slot) => {
      const node = root.querySelector(`.state-${slot}`);
      node.classList.add("collapsed");
      node.textContent = slot.split("-").map((part) => part[0].toUpperCase() + part.slice(1)).join(" ");
    });
  }
  return root;
}

function renderStateRules(state) {
  const cfg = stateSpecConfig(state);
  const side = el("div", "state-rules");
  side.innerHTML = `<h2>${cfg.title}</h2><p>${cfg.summary}</p>`;
  cfg.rules.forEach((rule, index) => {
    const row = el("div", "state-rule");
    row.innerHTML = `<strong>${String(index + 1).padStart(2, "0")}</strong><span>${rule}</span>`;
    side.append(row);
  });
  return side;
}

function stateSpecConfig(state) {
  const configs = {
    collapsed: {
      title: "Collapsed Drawer Behavior",
      summary: "Collapsed state keeps every drawer discoverable through compact slot headers.",
      leftTop: "Prefabs",
      leftBottom: "Files",
      rightTop: "Hierarchy",
      rightBottom: "Inspector",
      center: "Scene Editor gets maximum width",
      bottom: "Output remains visible",
      rules: [
        "Collapse side drawers before shrinking the active center editor below usable size.",
        "Collapsed headers keep slot labels and active-state color, not just icons.",
        "Bottom drawer can stay open for warnings, logs, and short timeline feedback.",
        "Restoring a drawer returns it to the same slot and tab selection.",
      ],
    },
    expanded: {
      title: "Expanded Inspection Behavior",
      summary: "Expanded state favors detail-heavy workflows while preserving the center editor.",
      leftTop: "Prefab Shelf",
      leftBottom: "Project Files",
      rightTop: "Hierarchy Expanded",
      rightBottom: "Inspector Expanded",
      center: "Active editor keeps primary interaction area",
      bottom: "Console / Problems",
      rules: [
        "Right-side drawers may widen together for object inspection and batch editing.",
        "Expanded drawers use the same tab/header language as the default shell.",
        "The active editor keeps a clear toolbar and content region.",
        "Pinned expanded drawers should not cover the top-level main tabs.",
      ],
    },
    "split-editor": {
      title: "Split Editor Behavior",
      summary: "Split panes live inside the active main editor tab, not as separate top-level pages.",
      leftTop: "Node Palette",
      leftBottom: "Files",
      rightTop: "Parameters",
      rightBottom: "Preview Asset",
      center: "Graph + Preview",
      bottom: "Shader Output",
      rules: [
        "Use split editor for source/preview, graph/preview, or material graph/property editing.",
        "Both panes share one top-level main tab and one bottom output drawer.",
        "The split divider should be simple, flat, and aligned with panel separators.",
        "Tool drawers remain attached to the active main tab context.",
      ],
    },
    "bottom-split": {
      title: "Bottom Timeline And Console Behavior",
      summary: "Wide bottom tools can split when timeline and output both need persistent visibility.",
      leftTop: "Clip Library",
      leftBottom: "Files",
      rightTop: "Skeleton",
      rightBottom: "Animation List",
      center: "Montage Graph",
      bottom: "Timeline + Console",
      rules: [
        "Timeline owns the larger bottom region because it is the primary horizontal editor.",
        "Console remains compact and scan-oriented with filters and warning count visible.",
        "Bottom split should not introduce nested cards or elevated surfaces.",
        "The split is per editor tab; Scene and UI Asset can choose different bottom tools.",
      ],
    },
    floating: {
      title: "Floating Tool Window Behavior",
      summary: "Floating windows are temporary undocked drawers with the same role and contents.",
      leftTop: "Node Palette",
      leftBottom: "Files",
      rightTop: "Dock Slot Empty",
      rightBottom: "Preview Asset",
      center: "Material Editor",
      bottom: "Shader Output",
      rules: [
        "Floating windows keep the drawer title, tabs, and role identity.",
        "The original dock slot can show a lightweight placeholder or collapsed target.",
        "Use floating windows for temporary comparison or multi-monitor workflows.",
        "Floating chrome stays flat and compact, without heavy elevation.",
      ],
    },
    compact: {
      title: "Compact Workspace Behavior",
      summary: "Compact layout prioritizes the active editor and collapses lower-priority drawers first.",
      leftTop: "Tools",
      leftBottom: "Files",
      rightTop: "Tree",
      rightBottom: "Details",
      center: "Scene Editor compact viewport",
      bottom: "Output compact",
      rules: [
        "Collapse side drawers before reducing center editor height.",
        "Keep main tabs readable; truncate metadata before editor names.",
        "Bottom drawer becomes short and status-oriented in compact mode.",
        "Controls retain tap/click targets with rounded rectangle shapes.",
      ],
    },
  };

  return configs[state] ?? configs.collapsed;
}

function renderContentSpec(design) {
  const spec = el("div", "layout-spec-frame content-spec-frame");
  const header = el("div", "layout-spec-header");
  const cfg = contentSpecConfig(design.content);
  header.innerHTML = `<div><strong>${design.title}</strong><span>${design.description}</span></div><div class="pill">${cfg.slot} drawer content</div>`;

  const body = el("div", "content-spec-body");
  body.append(renderContentSpecGuide(cfg), renderContentSpecSurface(cfg));
  spec.append(header, body);
  return spec;
}

function renderContentSpecGuide(cfg) {
  const guide = el("div", "content-spec-guide");
  guide.innerHTML = `<h2>${cfg.title}</h2><p>${cfg.summary}</p>`;
  cfg.rules.forEach((rule, index) => {
    const row = el("div", "content-rule");
    row.innerHTML = `<strong>${String(index + 1).padStart(2, "0")}</strong><span>${rule}</span>`;
    guide.append(row);
  });
  return guide;
}

function renderContentSpecSurface(cfg) {
  const surface = el("div", `content-spec-surface ${cfg.orientation}`);
  const drawer = el("div", `content-demo-drawer ${cfg.orientation}`);
  const header = el("div", "drawer-header");
  header.innerHTML = `<div><strong>${cfg.title}</strong><span>${cfg.slot}</span></div>`;
  const actions = el("div", "drawer-actions");
  actions.append(iconButton("−", "ghost"), iconButton("↗", "ghost"));
  header.append(actions);

  const tabs = el("div", "drawer-tabs");
  cfg.tabs.forEach((tab, index) => {
    const item = el("div", `drawer-tab ${index === 0 ? "active" : ""}`);
    item.textContent = tab;
    tabs.append(item);
  });

  const body = el("div", "drawer-body content-demo-body");
  body.append(cfg.body());
  drawer.append(header, tabs, body);

  const metrics = el("div", "content-demo-metrics");
  cfg.metrics.forEach(([label, value]) => {
    const row = el("div", "drawer-role-row");
    row.innerHTML = `<strong>${label}</strong><span>${value}</span><em>${cfg.slot}</em><small>${cfg.metricNote}</small>`;
    metrics.append(row);
  });

  surface.append(drawer, metrics);
  return surface;
}

function contentSpecConfig(content) {
  const configs = {
    prefab: {
      title: "Prefabs",
      slot: "Left Top",
      orientation: "side",
      tabs: ["Place", "Recent", "Pinned"],
      summary: "Placement drawers are small, repeat-use tool shelves. They should be dense, scannable, and friendly to drag/drop.",
      rules: [
        "Use a two-column item shelf for common prefabs when width allows.",
        "Keep item labels short; secondary text names the concrete asset.",
        "Expose search/filter only when content exceeds the first visible page.",
        "Primary action color marks selected placement mode, not every item.",
      ],
      metrics: [["Width", "260-320 px"], ["Item", "64-76 px"], ["Tabs", "24 px"]],
      metricNote: "Sized for repeated tool picking without stealing center editor space.",
      body: () => renderPrefabShelf(),
    },
    files: {
      title: "Files",
      slot: "Left Bottom",
      orientation: "side",
      tabs: ["Project", "Assets", "Search"],
      summary: "File drawers keep project navigation persistent. They should favor path clarity and stable tree indentation.",
      rules: [
        "Keep left-bottom persistent across main editor tabs.",
        "Tree rows need visible selection, hover, and disclosure states.",
        "Avoid thumbnail grids in the narrow drawer; use asset browser for larger browsing.",
        "Path chips and compact actions belong above the tree.",
      ],
      metrics: [["Width", "260-320 px"], ["Row", "28-32 px"], ["Indent", "18 px"]],
      metricNote: "Stable file tree density keeps the editor predictable.",
      body: () => renderAssetTree(),
    },
    hierarchy: {
      title: "Hierarchy",
      slot: "Right Top",
      orientation: "side",
      tabs: ["Scene", "Layers", "Search"],
      summary: "Structure drawers describe the active document object model. They should stay close to detail drawers.",
      rules: [
        "Right-top is reserved for object/document hierarchy, skeleton, or UI tree.",
        "Rows expose visibility/lock/active state without large controls.",
        "Selection uses the shared teal row state.",
        "Search remains compact and does not resize the row list.",
      ],
      metrics: [["Width", "300-380 px"], ["Row", "28-32 px"], ["Actions", "Icon-only"]],
      metricNote: "Hierarchy should pair visually with right-bottom details.",
      body: () => renderHierarchy(),
    },
    inspector: {
      title: "Inspector",
      slot: "Right Bottom",
      orientation: "side-tall",
      tabs: ["Properties", "History", "Components"],
      summary: "Detail drawers edit the selected object. They need dense fields but clearer grouping than the tree drawers.",
      rules: [
        "Group related controls into collapsible sections with 1px separators.",
        "Use rounded rectangle fields with simple fill, not bevels or glow.",
        "Object header stays visible at top of the drawer body.",
        "Add-component actions stay full-width and visually quiet.",
      ],
      metrics: [["Width", "320-420 px"], ["Field", "30-34 px"], ["Section", "Header + rows"]],
      metricNote: "Inspector density should not obscure field labels or values.",
      body: () => renderInspector("inspector-deep"),
    },
    "animation-list": {
      title: "Animation List",
      slot: "Right Bottom",
      orientation: "side-tall",
      tabs: ["Tracks", "Events", "Keys"],
      summary: "Animation detail drawers list selected clips, tracks, events, and keyframe properties beside the montage editor.",
      rules: [
        "Track and event lists belong right-bottom; the timeline remains bottom.",
        "Key selection fields should mirror inspector density.",
        "Event rows need compact time labels and status color.",
        "Avoid moving timeline editing controls into the side drawer.",
      ],
      metrics: [["Width", "320-420 px"], ["Track row", "28-32 px"], ["Fields", "30 px"]],
      metricNote: "Side animation details support, but do not replace, the timeline.",
      body: () => renderAnimationProperties(),
    },
    console: {
      title: "Console",
      slot: "Bottom",
      orientation: "bottom",
      tabs: ["Console", "Problems", "Tasks"],
      summary: "Output drawers need horizontal scan density: filters, severity, timestamp, source, and detail actions.",
      rules: [
        "Bottom drawer width is used for long log text, not tall controls.",
        "Filters stay in the drawer toolbar and preserve warning/error counts.",
        "Rows should be readable at compact height.",
        "Detail drilldown can open as focus panel or right-bottom drawer.",
      ],
      metrics: [["Height", "180-280 px"], ["Row", "28-32 px"], ["Toolbar", "36-42 px"]],
      metricNote: "Console is the default bottom fallback for most editor tabs.",
      body: () => renderConsole(),
    },
    timeline: {
      title: "Timeline",
      slot: "Bottom",
      orientation: "bottom",
      tabs: ["Dope Sheet", "Curves", "Events"],
      summary: "Timeline drawers own wide horizontal editing. They should remain bottom-first and avoid side panel compression.",
      rules: [
        "Reserve bottom width for track names, rows, keyframes, and event markers.",
        "Tabs switch timeline modes without replacing the active montage editor.",
        "Keyframes use simple flat markers and stable row heights.",
        "Console can split to the right or collapse below depending on height.",
      ],
      metrics: [["Height", "220-340 px"], ["Track row", "32-42 px"], ["Keys", "Flat markers"]],
      metricNote: "Timeline is the primary bottom tool for animation and montage tabs.",
      body: () => renderAnimationTimeline(),
    },
    "asset-grid": {
      title: "Asset Grid",
      slot: "Main / Drawer",
      orientation: "wide",
      tabs: ["Grid", "List", "Import"],
      summary: "Asset browsing can live as a main tab or expanded bottom/center tool. It needs tile density without marketing cards.",
      rules: [
        "Tiles use flat thumbnails and compact metadata.",
        "Grid/list/import controls stay in one toolbar.",
        "Import details should dock right or open as a focused panel.",
        "Avoid oversized cards; asset browsing is operational work.",
      ],
      metrics: [["Tile", "110-150 px"], ["Toolbar", "36-42 px"], ["Metadata", "2 lines"]],
      metricNote: "Asset grid scales between main tab and larger drawer surfaces.",
      body: () => renderAssetBrowser(),
    },
  };

  return configs[content] ?? configs.prefab;
}

function renderOverlaySpec(design) {
  const base = renderShell(FULL_DESIGNS[0]);
  base.classList.add("overlay-spec-base");
  const layer = el("div", "overlay-spec-layer");
  const cfg = overlaySpecConfig(design.overlay);
  const note = el("div", "overlay-spec-note");
  note.innerHTML = `<strong>${design.title}</strong><span>${design.description}</span>`;
  layer.append(note, cfg.body());
  base.append(layer);
  return base;
}

function overlaySpecConfig(overlay) {
  const configs = {
    "command-palette": {
      body: () => {
        const modal = overlayWindow("Command Palette", "Search commands, tools, settings, and recent editor actions", "command-palette");
        modal.append(field("Search actions or files..."));
        modal.append(commandGroup("Scene", [["Open Scene Editor", "Ctrl+1"], ["Focus Hierarchy", "Alt+H"], ["Place Prefab", "P"]]));
        modal.append(commandGroup("Tools", [["Open Material Editor", "Ctrl+M"], ["Run Asset Import", "Ctrl+I"], ["Toggle Bottom Console", "Alt+4"]]));
        return modal;
      },
    },
    "context-menu": {
      body: () => {
        const wrap = el("div", "overlay-menu-stage");
        const menu = el("div", "context-menu spec-menu");
        ["Open", "Rename", "Duplicate", "Create Prefab", "Show in Files", "Delete"].forEach((label, index) => {
          const row = el("div", `menu-row ${index === 3 ? "active" : ""} ${index === 5 ? "danger-text" : ""}`);
          row.innerHTML = `<span>${label}</span><span>${index === 0 ? "Enter" : index === 5 ? "Del" : index === 3 ? ">" : ""}</span>`;
          menu.append(row);
        });
        const sub = el("div", "context-menu spec-submenu");
        ["Scene Prefab", "UI Prefab", "Animation Clip"].forEach((label, index) => {
          const row = el("div", `menu-row ${index === 1 ? "disabled" : ""}`);
          row.innerHTML = `<span>${label}</span><span>${index === 0 ? "Ctrl+P" : ""}</span>`;
          sub.append(row);
        });
        wrap.append(menu, sub);
        return wrap;
      },
    },
    "tab-overflow": {
      body: () => {
        const pop = overlayWindow("Open Editor Tabs", "Switch to hidden, pinned, or recent editor windows", "tab-overflow");
        pop.append(field("Filter tabs..."));
        [["Scene Editor", "A1_Hangar.scene", true], ["Material Editor", "M_Metal.zmat", true], ["Montage Editor", "Idle_Run.blend", false], ["UI Asset Editor", "workbench_shell.v2.ui", false], ["Diagnostics", "Live Session", false]].forEach(([title, meta, active]) => {
          const row = el("div", `overlay-list-row ${active ? "selected" : ""}`);
          row.innerHTML = `<strong>${title}</strong><span>${meta}</span><small>${active ? "Pinned" : "Recent"}</small>`;
          pop.append(row);
        });
        return pop;
      },
    },
    "asset-picker": {
      body: () => {
        const modal = overlayWindow("Pick Asset", "Choose a mesh, material, texture, prefab, or UI document", "asset-picker wide");
        const toolbar = el("div", "overlay-toolbar");
        toolbar.append(field("Search assets..."), selectBtn("Type: All"), selectBtn("Sort: Recent"));
        const content = el("div", "asset-picker-grid");
        content.append(renderAssetGrid(), renderAssetDetail());
        modal.append(toolbar, content);
        return modal;
      },
    },
    "import-wizard": {
      body: () => {
        const modal = overlayWindow("Import Wizard", "Review source, target path, conversion settings, and validation", "import-wizard wide");
        const steps = el("div", "wizard-steps");
        ["Source", "Options", "Validation", "Import"].forEach((label, index) => {
          const step = el("div", `wizard-step ${index === 1 ? "active" : ""}`);
          step.textContent = label;
          steps.append(step);
        });
        const body = el("div", "wizard-body");
        body.append(renderSpecNotes("Mesh Import Options", ["Target folder: crate://content/meshes", "Generate collision: enabled", "Material policy: preserve source slots", "Scale: 1.0, axis: Y-up to Z-up"]));
        modal.append(steps, body, overlayFooter("Back", "Import"));
        return modal;
      },
    },
    "project-settings": {
      body: () => {
        const modal = overlayWindow("Project Settings", "Configure editor, runtime, asset, and build preferences", "project-settings wide");
        const grid = el("div", "settings-grid");
        grid.append(navList(["General", "Editor", "Runtime", "Assets", "Build", "Plugins"], 1), settingsForm());
        modal.append(grid, overlayFooter("Cancel", "Apply"));
        return modal;
      },
    },
    "confirm-dialog": {
      body: () => {
        const modal = overlayWindow("Delete Selected Asset?", "This removes the asset from the project content folder.", "confirm-dialog small");
        const body = el("div", "confirm-body");
        body.innerHTML = `<strong>Box_01.mesh</strong><span>References in open scenes will be marked missing until reassigned.</span>`;
        modal.append(body, overlayFooter("Cancel", "Delete", true));
        return modal;
      },
    },
    "notification-center": {
      body: () => {
        const tray = overlayWindow("Notifications", "Recent imports, builds, runtime messages, and editor tasks", "notification-center tray");
        [["Build Export", "Windows desktop profile completed", "Success"], ["Asset Import", "Texture T_Grid_01 uses fallback sampler", "Warning"], ["Runtime", "Preview surface rebuilt: 318 nodes", "Info"], ["Plugin Manager", "Physics plugin update available", "Info"]].forEach(([title, desc, level]) => {
          const row = el("div", `notification-row ${level.toLowerCase()}`);
          row.innerHTML = `<strong>${title}</strong><span>${desc}</span><small>${level}</small>`;
          tray.append(row);
        });
        return tray;
      },
    },
  };
  return configs[overlay] ?? configs["command-palette"];
}

function overlayWindow(title, subtitle, variant = "") {
  const node = el("div", `overlay-window ${variant}`);
  const header = el("div", "overlay-window-header");
  header.innerHTML = `<div><strong>${title}</strong><span>${subtitle}</span></div><button class="icon-btn ghost">×</button>`;
  node.append(header);
  return node;
}

function commandGroup(title, rows) {
  const group = el("div", "command-group");
  group.innerHTML = `<h3>${title}</h3>`;
  rows.forEach(([label, shortcut], index) => {
    const row = el("div", `command-row ${index === 0 ? "selected" : ""}`);
    row.innerHTML = `<span>${label}</span><small>${shortcut}</small>`;
    group.append(row);
  });
  return group;
}

function overlayFooter(left, right, danger = false) {
  const footer = el("div", "overlay-footer");
  footer.append(button(left, "secondary-btn"), button(right, danger ? "danger-btn" : "primary-btn"));
  return footer;
}

function settingsForm() {
  const form = el("div", "settings-form");
  form.append(
    fieldRow("Theme", "Workbench Dark"),
    fieldRow("Accent", "Teal"),
    fieldRow("Panel Radius", "12 px"),
    fieldRow("Open last project", "On"),
    fieldRow("Compact bottom drawer", "Auto"),
    fieldRow("Viewport toolbar", "Always visible")
  );
  return form;
}

function renderFloatingWindowSpec(design) {
  const cfg = floatingWindowConfig(design.window);
  const baseDesign = FULL_DESIGNS.find((entry) => entry.id === cfg.base) ?? FULL_DESIGNS[0];
  const frame = renderShell(baseDesign);
  frame.classList.add("floating-window-base", `floating-${design.window}`);

  const layer = el("div", "floating-window-layer");
  const note = el("div", "floating-window-note");
  note.innerHTML = `<strong>${design.title}</strong><span>${design.description}</span><small>${cfg.intent}</small>`;
  layer.append(note, cfg.body());
  frame.append(layer);
  return frame;
}

function floatingWindowConfig(name) {
  const configs = {
    preferences: {
      base: "project-overview-workbench",
      intent: "Persistent editor settings should stay wide, dense, and clearly tied to the active workbench shell.",
      body: () => {
        const win = workbenchWindow("Preferences", "Editor behavior, appearance, workspace, viewport, and tool drawers", "preferences large");
        const grid = el("div", "floating-settings-grid");
        const nav = navList(["Appearance", "Workbench", "Viewport", "Assets", "Keyboard", "Plugins", "Advanced"], 1);
        const form = el("div", "floating-form");
        form.append(
          floatingSection("Workbench Layout", [
            ["Main tab placement", "Top, pinned"],
            ["Open documents", "Restore last session"],
            ["Left drawer default", "Collapsed until focused"],
            ["Right drawer default", "Hierarchy + Inspector"],
            ["Bottom drawer", "Console / Timeline split"],
          ]),
          floatingSection("Modern Dark Theme", [
            ["Surface tone", "Near black"],
            ["Accent color", "Teal"],
            ["Button style", "Flat rounded"],
            ["Panel radius", "10 px"],
            ["Reduce gradients", "On"],
          ])
        );
        grid.append(nav, form);
        win.append(grid, overlayFooter("Reset", "Apply"));
        return win;
      },
    },
    "keyboard-shortcuts": {
      base: "scene-workbench",
      intent: "A keymap editor should be searchable, table-first, and close to JetBrains shortcut editing density.",
      body: () => {
        const win = workbenchWindow("Keyboard Shortcuts", "Search commands and edit active key bindings", "keymap large");
        const toolbar = el("div", "floating-toolbar");
        toolbar.append(field("Search action or shortcut..."), selectBtn("Keymap: Zircon Default"), selectBtn("Scope: All"));
        const layout = el("div", "keymap-layout");
        layout.append(
          commandTable([
            ["Action", "Shortcut", "Context", "Source"],
            ["Focus Scene Viewport", "Ctrl+1", "Global", "Default"],
            ["Toggle Left Top Drawer", "Alt+1", "Workbench", "Default"],
            ["Place Selected Prefab", "P", "Scene", "Default"],
            ["Open Material Editor", "Ctrl+M", "Global", "Custom"],
            ["Run Asset Import", "Ctrl+I", "Assets", "Default"],
            ["Toggle Console", "Alt+4", "Workbench", "Default"],
          ]),
          sideDetail("Selected Action", [["Action", "Place Selected Prefab"], ["Current", "P"], ["Conflict", "None"], ["Scope", "Scene Editor"]], "Add Shortcut")
        );
        win.append(toolbar, layout, overlayFooter("Cancel", "Save Keymap"));
        return win;
      },
    },
    "reimport-conflict": {
      base: "asset-browser-workbench",
      intent: "Conflict resolution stays asset-focused with source, existing asset, references, and compact resolution actions.",
      body: () => {
        const win = workbenchWindow("Reimport Conflict", "Imported file differs from the existing project asset", "reimport medium");
        const body = el("div", "conflict-layout");
        body.append(
          compareColumn("Incoming Source", [["File", "SM_Railing.fbx"], ["Modified", "09:42"], ["Vertices", "1,904"], ["Materials", "3 slots"]]),
          compareColumn("Existing Asset", [["Asset", "SM_Railing.mesh"], ["Modified", "Yesterday"], ["Vertices", "1,876"], ["Materials", "2 slots"]])
        );
        const refs = renderMiniDataGrid([
          ["Reference", "Owner", "Use", "State", ""],
          ["A1_Hangar.scene", "Level", "Mesh", "Will update", ""],
          ["BP_Railing.prefab", "Prefab", "Variant", "Needs review", ""],
          ["M_Rail.metal", "Material", "Slot", "Unchanged", ""],
        ]);
        const options = el("div", "resolution-row");
        options.append(pill("Keep existing collision"), pill("Preserve material slots"), pill("Mark references dirty"));
        win.append(body, refs, options, overlayFooter("Keep Existing", "Replace Asset"));
        return win;
      },
    },
    "source-control-submit": {
      base: "project-overview-workbench",
      intent: "Submit windows need changelist summary, selectable files, diff preview, validation, and commit text in one place.",
      body: () => {
        const win = workbenchWindow("Submit Changelist", "Review modified files and submit to source control", "source-control xlarge");
        const layout = el("div", "submit-layout");
        const files = renderMiniDataGrid([
          ["File", "Type", "State", "Size", ""],
          ["preferences-window-workbench.png", "PNG", "Added", "182 KB", ""],
          ["design.js", "Script", "Modified", "122 KB", ""],
          ["design.css", "Style", "Modified", "48 KB", ""],
          ["editor-workbench-design-export.md", "Doc", "Modified", "9 KB", ""],
        ]);
        const diff = codePreview("Diff Preview", "+ const FLOATING_WINDOW_DESIGNS = [...]\n+ renderFloatingWindowSpec(design)\n+ .floating-window-layer { background: rgba(...) }");
        const side = el("div", "submit-side");
        side.append(sideDetail("Validation", [["PNG size", "1672 x 941"], ["UI text", "HTML/CSS"], ["Style note", "Updated"], ["Risk", "Visual review"]], "Run Checks"));
        side.append(field("Describe this changelist..."));
        layout.append(files, diff, side);
        win.append(layout, overlayFooter("Cancel", "Submit"));
        return win;
      },
    },
    "crash-report": {
      base: "runtime-diagnostics-workbench",
      intent: "Crash reporting should be calm and operational: stack, attachments, recovery, and send controls without visual noise.",
      body: () => {
        const win = workbenchWindow("Editor Crash Report", "The editor recovered from a renderer panic during viewport preview", "crash medium");
        const summary = el("div", "crash-summary");
        summary.innerHTML = `<strong>Renderer panic: invalid surface frame</strong><span>Session recovered from autosave checkpoint at 09:58:14.</span>`;
        const stack = codePreview("Stack Summary", "zircon_editor::viewport::present_frame\nzircon_runtime::render::surface::acquire\nwgpu::Surface::get_current_texture");
        const attach = el("div", "attachment-list");
        [["Autosave scene", "Included"], ["Crash log", "Included"], ["GPU report", "Optional"], ["Screenshot", "Optional"]].forEach(([label, state], index) => {
          const row = el("div", `attachment-row ${index < 2 ? "selected" : ""}`);
          row.innerHTML = `<span>${label}</span><small>${state}</small>`;
          attach.append(row);
        });
        win.append(summary, stack, attach, overlayFooter("Close", "Send And Restart"));
        return win;
      },
    },
    "find-in-project": {
      base: "ui-asset-editor-workbench",
      intent: "Project-wide search should be a large floating work window with query, scope, result table, and selected preview.",
      body: () => {
        const win = workbenchWindow("Find In Project", "Search assets, UI documents, scripts, and editor settings", "find large");
        const toolbar = el("div", "floating-toolbar");
        toolbar.append(field("Find: WorkbenchShell"), selectBtn("Scope: Project"), selectBtn("File mask: *.ui.toml"), button("Search", "primary-btn"));
        const layout = el("div", "find-layout");
        const results = renderMiniDataGrid([
          ["Result", "File", "Line", "Match", ""],
          ["WorkbenchShellReferenceImage", "workbench_shell.v2.ui.toml", "42", "Image brush", ""],
          ["WorkbenchShellRoot", "workbench_window.v2.ui.toml", "18", "Root node", ""],
          ["WorkbenchShellPreview", "design.js", "212", "Render shell", ""],
          ["WorkbenchShellStyle", "design.css", "86", "Top chrome", ""],
        ]);
        const preview = codePreview("Preview", "42  node = WorkbenchShellReferenceImage\n    input_policy = Ignore\n    brush = ui/editor/reference/workbench.png");
        layout.append(results, preview);
        win.append(toolbar, layout, overlayFooter("Replace...", "Open"));
        return win;
      },
    },
    "startup-tasks": {
      base: "welcome-workbench",
      intent: "Startup progress is a compact workbench window that keeps task states visible without covering the entire shell.",
      body: () => {
        const win = workbenchWindow("Opening Project", "Zircon Labs / Sandbox is preparing the editor workspace", "startup small-window");
        const tasks = el("div", "startup-task-list");
        [
          ["Load project manifest", "Complete", 100],
          ["Index content folder", "Running", 72],
          ["Warm shader cache", "Running", 48],
          ["Load editor plugins", "Queued", 0],
          ["Restore main tabs", "Queued", 0],
        ].forEach(([label, state, progress]) => tasks.append(taskRow(label, state, progress)));
        win.append(tasks, overlayFooter("Open Minimal", "Continue"));
        return win;
      },
    },
    "editor-update": {
      base: "project-overview-workbench",
      intent: "Update windows should feel like editor infrastructure: compact release notes, components, and restart actions.",
      body: () => {
        const win = workbenchWindow("Editor Update Available", "Zircon Editor 0.18.0 can be installed after current tasks finish", "update medium");
        const notes = el("div", "release-notes");
        notes.innerHTML = `<strong>0.18.0 Workbench UI Update</strong><span>Improved main tabs, drawer persistence, asset import review, and retained UI diagnostics.</span>`;
        const components = el("div", "component-list");
        [["Editor shell", "Required"], ["Runtime UI tools", "Required"], ["Asset importers", "Selected"], ["Example content", "Optional"]].forEach(([label, state], index) => {
          const row = el("div", `component-row ${index < 3 ? "selected" : ""}`);
          row.innerHTML = `<span>${label}</span><small>${state}</small>`;
          components.append(row);
        });
        win.append(notes, components, overlayFooter("Later", "Install And Restart"));
        return win;
      },
    },
  };
  return configs[name] ?? configs.preferences;
}

function workbenchWindow(title, subtitle, variant = "") {
  return overlayWindow(title, subtitle, `floating-workbench-window ${variant}`);
}

function floatingSection(title, rows) {
  const section = el("div", "floating-section");
  section.innerHTML = `<div class="floating-section-title">${title}</div>`;
  rows.forEach(([label, value], index) => {
    section.append(fieldRow(label, value, index === 1 && title.includes("Theme")));
  });
  return section;
}

function commandTable(rows) {
  const table = el("div", "shortcut-table");
  rows.forEach((cols, index) => {
    const row = el("div", `shortcut-row ${index === 0 ? "head" : index === 3 ? "selected" : ""}`);
    row.innerHTML = cols.map((col) => `<span>${col}</span>`).join("");
    table.append(row);
  });
  return table;
}

function sideDetail(title, rows, action) {
  const detail = el("div", "floating-side-detail");
  detail.innerHTML = `<div class="section-title">${title}</div>`;
  rows.forEach(([label, value]) => detail.append(fieldRow(label, value)));
  detail.append(button(action, "primary-btn"));
  return detail;
}

function compareColumn(title, rows) {
  const column = el("div", "compare-column");
  column.innerHTML = `<div class="section-title">${title}</div><div class="compare-thumb"></div>`;
  rows.forEach(([label, value]) => column.append(fieldRow(label, value)));
  return column;
}

function codePreview(title, code) {
  const panel = el("div", "diff-panel");
  const head = el("div", "diff-title");
  const pre = el("pre");
  head.textContent = title;
  pre.textContent = code;
  panel.append(head, pre);
  return panel;
}

function taskRow(label, state, progress) {
  const row = el("div", `startup-task-row ${progress === 100 ? "done" : progress > 0 ? "running" : ""}`);
  row.innerHTML = `<div><strong>${label}</strong><span>${state}</span></div><div class="task-progress"><i style="width:${progress}%"></i></div>`;
  return row;
}

function renderWorkflowSpec(design) {
  const cfg = workflowSpecConfig(design.workflow);
  const baseDesign = FULL_DESIGNS.find((entry) => entry.id === cfg.base) ?? FULL_DESIGNS[0];
  const frame = renderShell(baseDesign);
  frame.classList.add("workflow-spec-base", `workflow-${design.workflow}`);

  const layer = el("div", "workflow-spec-layer");
  const note = el("div", "workflow-note");
  note.innerHTML = `<strong>${design.title}</strong><span>${design.description}</span><small>${cfg.intent}</small>`;

  const steps = el("div", "workflow-steps");
  steps.innerHTML = `<div class="workflow-steps-title">Task Flow</div>`;
  cfg.steps.forEach((step, index) => {
    const row = el("div", `workflow-step ${index === cfg.activeStep ? "active" : ""}`);
    row.innerHTML = `<strong>${String(index + 1).padStart(2, "0")}</strong><span>${step}</span>`;
    steps.append(row);
  });

  layer.append(note, steps);
  cfg.callouts.forEach((callout) => layer.append(workflowCallout(callout)));
  if (cfg.window) {
    layer.append(workflowMiniWindow(cfg.window));
  }
  frame.append(layer);
  return frame;
}

function workflowSpecConfig(workflow) {
  const zones = {
    mainTabs: { left: 10, top: 58, width: 1210, height: 38 },
    leftTop: { left: 14, top: 112, width: 262, height: 218 },
    leftBottom: { left: 14, top: 342, width: 262, height: 306 },
    center: { left: 304, top: 112, width: 1008, height: 536 },
    rightTop: { left: 1335, top: 112, width: 320, height: 246 },
    rightBottom: { left: 1335, top: 366, width: 320, height: 282 },
    bottom: { left: 14, top: 668, width: 1642, height: 220 },
  };

  const configs = {
    "prefab-placement": {
      base: "scene-workbench",
      intent: "Use the main Scene Editor tab as the task container while drawers hold placement, structure, detail, and output tools.",
      activeStep: 1,
      steps: ["Pick a prefab from Left Top", "Place it in the viewport", "Confirm hierarchy selection", "Tune detail and verify output"],
      callouts: [
        { ...zones.leftTop, label: "Left Top", role: "Prefab shelf and placement modes" },
        { ...zones.center, label: "Scene Editor", role: "Drop target and transform gizmo" },
        { ...zones.rightTop, label: "Right Top", role: "Hierarchy confirms inserted node" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Transform and component detail" },
        { ...zones.bottom, label: "Bottom", role: "Placement log and validation output" },
      ],
    },
    "asset-import": {
      base: "asset-browser-workbench",
      intent: "Keep asset browsing operational: file tree, result grid, metadata, and import review stay in one workbench tab.",
      activeStep: 2,
      steps: ["Choose source files", "Preview destination grid", "Review import settings", "Run import and read output"],
      callouts: [
        { ...zones.leftBottom, label: "Left Bottom", role: "Project files and target folders" },
        { ...zones.center, label: "Asset Browser", role: "Grid/list result surface" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Selected asset import detail" },
        { ...zones.bottom, label: "Bottom", role: "Import queue and table output" },
      ],
      window: {
        title: "Import Mesh",
        subtitle: "Review target path, collision, materials, and scale before import.",
        left: 560,
        top: 188,
        width: 500,
        rows: [["Source", "SM_Railing.fbx"], ["Target", "content/meshes"], ["Collision", "Generate simple"], ["Materials", "Preserve slots"]],
        action: "Import",
      },
    },
    "shader-error": {
      base: "shader-editor-workbench",
      intent: "Shader authoring keeps source/preview in the center and treats compile diagnostics as a bottom drawer workflow.",
      activeStep: 2,
      steps: ["Edit shader source", "Run compile", "Open error row in bottom output", "Adjust pipeline state"],
      callouts: [
        { ...zones.center, label: "Shader Editor", role: "Source, preview, and diagnostics split" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Pipeline and variant settings" },
        { ...zones.bottom, label: "Bottom", role: "Compiler output and selected error" },
      ],
      window: {
        title: "Compile Error",
        subtitle: "Line 42: unresolved binding 'material.color'.",
        left: 920,
        top: 482,
        width: 390,
        rows: [["Stage", "Fragment"], ["Variant", "Default Lit"], ["Severity", "Error"], ["Fix", "Bind material parameter"]],
        action: "Jump to Line",
      },
    },
    "animation-event": {
      base: "animation-workbench",
      intent: "Montage editing uses the same drawer grammar: rig/clip sources left, event details right, timeline below.",
      activeStep: 3,
      steps: ["Choose clip and rig", "Add event marker", "Edit event payload", "Scrub timeline and verify"],
      callouts: [
        { ...zones.leftTop, label: "Left Top", role: "Rig and clip source drawer" },
        { ...zones.center, label: "Montage Editor", role: "Blend graph and curve preview" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Animation event detail" },
        { ...zones.bottom, label: "Bottom", role: "Timeline, tracks, keys, and events" },
      ],
      window: {
        title: "Event Marker",
        subtitle: "Footstep.Left at 1.42s on the Events track.",
        left: 1020,
        top: 604,
        width: 350,
        rows: [["Notify", "Footstep.Left"], ["Time", "1.42 s"], ["Payload", "Surface: Metal"], ["Preview", "Enabled"]],
        action: "Apply",
      },
    },
    "runtime-debug": {
      base: "runtime-diagnostics-workbench",
      intent: "Live debugging keeps channel filters and event drilldown close to the bottom console without changing tabs.",
      activeStep: 1,
      steps: ["Filter runtime channel", "Select live event", "Inspect detail payload", "Correlate console rows"],
      callouts: [
        { ...zones.leftTop, label: "Left Top", role: "Diagnostic channels and filters" },
        { ...zones.center, label: "Diagnostics", role: "Runtime event stream and frame state" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Selected event payload" },
        { ...zones.bottom, label: "Bottom", role: "Filtered console and warnings" },
      ],
      window: {
        title: "Runtime Event Detail",
        subtitle: "UI surface rebuilt after asset hot reload.",
        left: 760,
        top: 230,
        width: 420,
        rows: [["Channel", "Runtime UI"], ["Frame", "1842"], ["Surface", "editor.ui_asset"], ["Nodes", "318 arranged"]],
        action: "Pin Event",
      },
    },
    "build-export": {
      base: "build-export-workbench",
      intent: "Build/export reads as an editor task page, with targets left, pipeline center, settings right, and log below.",
      activeStep: 2,
      steps: ["Select export target", "Validate build settings", "Run package pipeline", "Review output log"],
      callouts: [
        { ...zones.leftTop, label: "Left Top", role: "Target presets and profiles" },
        { ...zones.center, label: "Build Export", role: "Validation pipeline and queue" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Package settings and actions" },
        { ...zones.bottom, label: "Bottom", role: "Build log and warnings" },
      ],
      window: {
        title: "Package Queue",
        subtitle: "Windows desktop profile is running validation.",
        left: 574,
        top: 292,
        width: 440,
        rows: [["Target", "Windows Desktop"], ["Profile", "Editor Debug"], ["Step", "Cook Assets"], ["Status", "Running"]],
        action: "Open Log",
      },
    },
    "ui-binding": {
      base: "ui-asset-editor-workbench",
      intent: "UI asset authoring keeps widget structure visible while source, preview, bindings, and diagnostics stay docked.",
      activeStep: 2,
      steps: ["Select widget node", "Bind data source", "Preview retained UI", "Resolve diagnostics"],
      callouts: [
        { ...zones.leftTop, label: "Left Top", role: "Widget palette and UI tree" },
        { ...zones.center, label: "UI Asset Editor", role: "Source, preview, and retained surface" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Binding and style properties" },
        { ...zones.bottom, label: "Bottom", role: "Template diagnostics and warnings" },
      ],
      window: {
        title: "Create Binding",
        subtitle: "Bind selected TextLabel.content to project status.",
        left: 682,
        top: 210,
        width: 430,
        rows: [["Node", "StatusLabel"], ["Property", "content"], ["Source", "project.status"], ["Fallback", "Ready"]],
        action: "Bind",
      },
    },
    "lighting-bake": {
      base: "lighting-bake-workbench",
      intent: "Lighting bake setup remains a scene editor task with light sources left, settings right, and job output below.",
      activeStep: 1,
      steps: ["Select lights and probes", "Tune bake settings", "Start bake job", "Check warnings and preview"],
      callouts: [
        { ...zones.leftTop, label: "Left Top", role: "Light list and probe groups" },
        { ...zones.center, label: "Lighting Viewport", role: "Scene preview and bake overlays" },
        { ...zones.rightBottom, label: "Right Bottom", role: "Bake settings and selected light detail" },
        { ...zones.bottom, label: "Bottom", role: "Bake job output and warnings" },
      ],
      window: {
        title: "Bake Progress",
        subtitle: "A1_Hangar lightmap job is computing indirect samples.",
        left: 566,
        top: 232,
        width: 460,
        rows: [["Quality", "Preview"], ["Samples", "128"], ["Atlas", "2048"], ["Progress", "62%"]],
        action: "View Jobs",
      },
    },
  };

  return configs[workflow] ?? configs["prefab-placement"];
}

function workflowCallout(callout) {
  const node = el("div", "workflow-callout");
  node.style.left = `${callout.left}px`;
  node.style.top = `${callout.top}px`;
  node.style.width = `${callout.width}px`;
  node.style.height = `${callout.height}px`;
  node.innerHTML = `<strong>${callout.label}</strong><span>${callout.role}</span>`;
  return node;
}

function workflowMiniWindow(cfg) {
  const win = el("div", "workflow-mini-window");
  win.style.left = `${cfg.left}px`;
  win.style.top = `${cfg.top}px`;
  win.style.width = `${cfg.width}px`;
  win.innerHTML = `<div class="workflow-mini-header"><div><strong>${cfg.title}</strong><span>${cfg.subtitle}</span></div><button class="icon-btn ghost">x</button></div>`;
  const body = el("div", "workflow-mini-body");
  cfg.rows.forEach(([label, value]) => {
    const row = el("div", "workflow-mini-row");
    row.innerHTML = `<span>${label}</span><strong>${value}</strong>`;
    body.append(row);
  });
  const footer = el("div", "workflow-mini-footer");
  footer.append(button("Cancel", "secondary-btn"), button(cfg.action, "primary-btn"));
  win.append(body, footer);
  return win;
}

function isAdditionalEditor(center) {
  return [
    "prefab-editor",
    "vfx-editor",
    "shader-editor",
    "terrain-editor",
    "audio-editor",
    "behavior-tree",
    "lighting-bake",
    "physics-collision",
    "level-streaming",
    "sequencer",
    "navmesh-ai",
    "render-pipeline",
    "input-mapping",
    "data-table",
    "network-replication",
    "localization",
    "visual-script",
    "state-machine",
    "skeleton-mesh",
    "texture-editor",
    "material-instance",
    "prefab-variant",
    "level-audit",
    "test-runner",
    "frame-debugger",
    "memory-profiler",
    "asset-dependency",
    "reference-finder",
    "cook-package",
    "crash-session-replay",
    "log-analysis",
    "automation-report",
    "layout-manager",
    "theme-token",
    "command-center",
    "module-graph",
    "asset-validation",
    "hot-reload",
    "project-history",
    "task-board",
    "source-control",
    "review-comments",
    "build-farm",
    "release-notes",
    "project-settings-page",
    "plugin-development",
    "remote-device",
    "session-sync",
    "cutscene-editor",
    "dialogue-editor",
    "quest-editor",
    "camera-rig",
    "control-rig",
    "motion-matching",
    "facial-animation",
    "blend-space",
    "foliage-editor",
    "scatter-editor",
    "volume-editor",
    "weather-editor",
    "post-process",
    "particle-library",
    "collision-proxy",
    "level-variant",
    "gameplay-ability",
    "gameplay-effect",
    "ai-perception",
    "spawn-rules",
    "gameplay-tags",
    "save-data",
    "world-state",
    "telemetry-dashboard",
    "lobby-editor",
    "matchmaking-editor",
    "server-browser",
    "replay-browser",
    "achievements-editor",
    "entitlements-editor",
    "user-profile-editor",
    "online-diagnostics",
    "hud-editor",
    "menu-flow",
    "font-atlas",
    "icon-library",
    "ui-binding-editor",
    "accessibility-audit",
    "input-prompts",
    "ui-motion",
    "shader-permutations",
    "render-targets",
    "gpu-profiler",
    "light-probes",
    "reflection-capture",
    "decal-editor",
    "virtual-texture",
    "material-audit",
    "sound-cue",
    "audio-mixer",
    "music-system",
    "audio-occlusion",
    "voice-bank",
    "subtitle-timing",
    "lip-sync",
    "audio-profiler",
    "rigid-body",
    "physics-constraints",
    "destruction-editor",
    "cloth-simulation",
    "vehicle-physics",
    "fluid-simulation",
    "rope-cable",
    "physics-profiler",
    "ai-director",
    "blackboard-editor",
    "eqs-query",
    "crowd-simulation",
    "smart-objects",
    "patrol-routes",
    "cover-system",
    "ai-profiler",
    "mesh-import",
    "lod-chain",
    "redirect-map",
    "texture-compression-queue",
    "source-asset-trace",
    "dcc-live-link",
    "metadata-editor",
    "batch-process-queue",
    "script-editor",
    "api-browser",
    "plugin-packaging",
    "module-settings",
    "automation-suite",
    "build-config",
    "cook-rules",
    "runtime-commands",
    "asset-migration",
    "scene-diff",
    "prefab-diff",
    "performance-budget",
    "memory-budget",
    "dependency-cleanup",
    "naming-rules",
    "release-checklist",
    "gameplay-debugger",
    "replay-timeline",
    "network-packet-inspector",
    "latency-map",
    "input-trace",
    "save-state-diff",
    "repro-recorder",
    "qa-triage",
    "render-graph",
    "shader-debugger",
    "texture-streaming",
    "shadow-map",
    "occlusion-culling",
    "frame-compare",
    "material-layers",
    "gpu-memory",
    "retarget",
    "ik-solver",
    "pose-library",
    "mocap-cleanup",
    "animation-compression",
    "root-motion",
    "event-tracks",
    "montage-debugger",
    "widget-tree-debugger",
    "layout-constraint-solver",
    "theme-variant-preview",
    "localization-preview",
    "focus-navigation",
    "input-glyph-mapper",
    "ui-snapshot-diff",
    "widget-performance",
    "world-partition",
    "hlod-builder",
    "level-instance",
    "streaming-profiler",
    "scene-bookmarks",
    "spawn-point-editor",
    "collision-matrix",
    "environment-probes",
    "feature-flags",
    "remote-config",
    "telemetry-query",
    "patch-planner",
    "dlc-catalog",
    "crash-symbolication",
    "player-segment",
    "experiment-console",
  ].includes(center);
}

function isAdditionalDetail(right) {
  return [
    "prefab-detail",
    "vfx-detail",
    "shader-detail",
    "terrain-detail",
    "audio-detail",
    "behavior-detail",
    "lighting-detail",
    "physics-detail",
    "level-streaming-detail",
    "sequencer-detail",
    "navmesh-detail",
    "render-pipeline-detail",
    "input-mapping-detail",
    "data-table-detail",
    "network-replication-detail",
    "localization-detail",
    "visual-script-detail",
    "state-machine-detail",
    "skeleton-mesh-detail",
    "texture-detail",
    "material-instance-detail",
    "prefab-variant-detail",
    "level-audit-detail",
    "test-runner-detail",
    "frame-debugger-detail",
    "memory-profiler-detail",
    "asset-dependency-detail",
    "reference-finder-detail",
    "cook-package-detail",
    "crash-session-detail",
    "log-analysis-detail",
    "automation-report-detail",
    "layout-manager-detail",
    "theme-token-detail",
    "command-center-detail",
    "module-graph-detail",
    "asset-validation-detail",
    "hot-reload-detail",
    "project-history-detail",
    "task-board-detail",
    "source-control-detail",
    "review-comments-detail",
    "build-farm-detail",
    "release-notes-detail",
    "project-settings-detail",
    "plugin-development-detail",
    "remote-device-detail",
    "session-sync-detail",
    "cutscene-detail",
    "dialogue-detail",
    "quest-detail",
    "camera-rig-detail",
    "control-rig-detail",
    "motion-matching-detail",
    "facial-animation-detail",
    "blend-space-detail",
    "foliage-detail",
    "scatter-detail",
    "volume-detail",
    "weather-detail",
    "post-process-detail",
    "particle-library-detail",
    "collision-proxy-detail",
    "level-variant-detail",
    "gameplay-ability-detail",
    "gameplay-effect-detail",
    "ai-perception-detail",
    "spawn-rules-detail",
    "gameplay-tags-detail",
    "save-data-detail",
    "world-state-detail",
    "telemetry-dashboard-detail",
    "lobby-detail",
    "matchmaking-detail",
    "server-browser-detail",
    "replay-detail",
    "achievements-detail",
    "entitlements-detail",
    "user-profile-detail",
    "online-diagnostics-detail",
    "hud-detail",
    "menu-flow-detail",
    "font-atlas-detail",
    "icon-library-detail",
    "ui-binding-detail",
    "accessibility-detail",
    "input-prompts-detail",
    "ui-motion-detail",
    "shader-permutations-detail",
    "render-target-detail",
    "gpu-profiler-detail",
    "light-probes-detail",
    "reflection-capture-detail",
    "decal-detail",
    "virtual-texture-detail",
    "material-audit-detail",
    "sound-cue-detail",
    "audio-mixer-detail",
    "music-system-detail",
    "audio-occlusion-detail",
    "voice-bank-detail",
    "subtitle-timing-detail",
    "lip-sync-detail",
    "audio-profiler-detail",
    "rigid-body-detail",
    "physics-constraints-detail",
    "destruction-detail",
    "cloth-detail",
    "vehicle-physics-detail",
    "fluid-detail",
    "rope-cable-detail",
    "physics-profiler-detail",
    "ai-director-detail",
    "blackboard-detail",
    "eqs-query-detail",
    "crowd-detail",
    "smart-object-detail",
    "patrol-route-detail",
    "cover-detail",
    "ai-profiler-detail",
    "mesh-import-detail",
    "lod-chain-detail",
    "redirect-map-detail",
    "texture-compression-detail",
    "source-asset-detail",
    "dcc-live-link-detail",
    "metadata-detail",
    "batch-process-detail",
    "script-detail",
    "api-detail",
    "plugin-packaging-detail",
    "module-settings-detail",
    "automation-suite-detail",
    "build-config-detail",
    "cook-rules-detail",
    "runtime-command-detail",
    "asset-migration-detail",
    "scene-diff-detail",
    "prefab-diff-detail",
    "performance-budget-detail",
    "memory-budget-detail",
    "dependency-cleanup-detail",
    "naming-rule-detail",
    "release-checklist-detail",
    "gameplay-debugger-detail",
    "replay-timeline-detail",
    "packet-detail",
    "latency-detail",
    "input-trace-detail",
    "save-state-detail",
    "repro-detail",
    "qa-triage-detail",
    "render-graph-detail",
    "shader-debugger-detail",
    "texture-streaming-detail",
    "shadow-map-detail",
    "occlusion-detail",
    "frame-compare-detail",
    "material-layer-detail",
    "gpu-memory-detail",
    "retarget-detail",
    "ik-solver-detail",
    "pose-detail",
    "mocap-detail",
    "animation-compression-detail",
    "root-motion-detail",
    "event-track-detail",
    "montage-debugger-detail",
    "widget-tree-detail",
    "layout-constraint-detail",
    "theme-variant-detail",
    "localization-preview-detail",
    "focus-navigation-detail",
    "input-glyph-detail",
    "ui-snapshot-detail",
    "widget-performance-detail",
    "world-partition-detail",
    "hlod-detail",
    "level-instance-detail",
    "streaming-profiler-detail",
    "scene-bookmark-detail",
    "spawn-point-detail",
    "collision-matrix-detail",
    "environment-probe-detail",
    "feature-flag-detail",
    "remote-config-detail",
    "telemetry-query-detail",
    "patch-planner-detail",
    "dlc-detail",
    "symbolication-detail",
    "segment-detail",
    "experiment-detail",
  ].includes(right);
}

function isAdditionalOutput(bottom) {
  return [
    "prefab-output",
    "vfx-output",
    "shader-output",
    "terrain-output",
    "audio-output",
    "behavior-output",
    "lighting-output",
    "physics-output",
    "level-streaming-output",
    "sequencer-output",
    "navmesh-output",
    "render-pipeline-output",
    "input-mapping-output",
    "data-table-output",
    "network-replication-output",
    "localization-output",
    "visual-script-output",
    "state-machine-output",
    "skeleton-mesh-output",
    "texture-output",
    "material-instance-output",
    "prefab-variant-output",
    "level-audit-output",
    "test-runner-output",
    "frame-debugger-output",
    "memory-profiler-output",
    "asset-dependency-output",
    "reference-finder-output",
    "cook-package-output",
    "crash-session-output",
    "log-analysis-output",
    "automation-report-output",
    "layout-manager-output",
    "theme-token-output",
    "command-center-output",
    "module-graph-output",
    "asset-validation-output",
    "hot-reload-output",
    "project-history-output",
    "task-board-output",
    "source-control-output",
    "review-comments-output",
    "build-farm-output",
    "release-notes-output",
    "project-settings-output",
    "plugin-development-output",
    "remote-device-output",
    "session-sync-output",
    "cutscene-output",
    "dialogue-output",
    "quest-output",
    "camera-rig-output",
    "control-rig-output",
    "motion-matching-output",
    "facial-animation-output",
    "blend-space-output",
    "foliage-output",
    "scatter-output",
    "volume-output",
    "weather-output",
    "post-process-output",
    "particle-library-output",
    "collision-proxy-output",
    "level-variant-output",
    "gameplay-ability-output",
    "gameplay-effect-output",
    "ai-perception-output",
    "spawn-rules-output",
    "gameplay-tags-output",
    "save-data-output",
    "world-state-output",
    "telemetry-dashboard-output",
    "lobby-output",
    "matchmaking-output",
    "server-browser-output",
    "replay-output",
    "achievements-output",
    "entitlements-output",
    "user-profile-output",
    "online-diagnostics-output",
    "hud-output",
    "menu-flow-output",
    "font-atlas-output",
    "icon-library-output",
    "ui-binding-output",
    "accessibility-output",
    "input-prompts-output",
    "ui-motion-output",
    "shader-permutations-output",
    "render-target-output",
    "gpu-profiler-output",
    "light-probes-output",
    "reflection-capture-output",
    "decal-output",
    "virtual-texture-output",
    "material-audit-output",
    "sound-cue-output",
    "audio-mixer-output",
    "music-system-output",
    "audio-occlusion-output",
    "voice-bank-output",
    "subtitle-timing-output",
    "lip-sync-output",
    "audio-profiler-output",
    "rigid-body-output",
    "physics-constraints-output",
    "destruction-output",
    "cloth-output",
    "vehicle-physics-output",
    "fluid-output",
    "rope-cable-output",
    "physics-profiler-output",
    "ai-director-output",
    "blackboard-output",
    "eqs-query-output",
    "crowd-output",
    "smart-object-output",
    "patrol-route-output",
    "cover-output",
    "ai-profiler-output",
    "mesh-import-output",
    "lod-chain-output",
    "redirect-map-output",
    "texture-compression-output",
    "source-asset-output",
    "dcc-live-link-output",
    "metadata-output",
    "batch-process-output",
    "script-output",
    "api-output",
    "plugin-packaging-output",
    "module-settings-output",
    "automation-suite-output",
    "build-config-output",
    "cook-rules-output",
    "runtime-command-output",
    "asset-migration-output",
    "scene-diff-output",
    "prefab-diff-output",
    "performance-budget-output",
    "memory-budget-output",
    "dependency-cleanup-output",
    "naming-rule-output",
    "release-checklist-output",
    "gameplay-debugger-output",
    "replay-timeline-output",
    "packet-output",
    "latency-output",
    "input-trace-output",
    "save-state-output",
    "repro-output",
    "qa-triage-output",
    "render-graph-output",
    "shader-debugger-output",
    "texture-streaming-output",
    "shadow-map-output",
    "occlusion-output",
    "frame-compare-output",
    "material-layer-output",
    "gpu-memory-output",
    "retarget-output",
    "ik-solver-output",
    "pose-library-output",
    "mocap-output",
    "animation-compression-output",
    "root-motion-output",
    "event-track-output",
    "montage-debugger-output",
    "widget-tree-output",
    "layout-constraint-output",
    "theme-variant-output",
    "localization-preview-output",
    "focus-navigation-output",
    "input-glyph-output",
    "ui-snapshot-output",
    "widget-performance-output",
    "world-partition-output",
    "hlod-output",
    "level-instance-output",
    "streaming-profiler-output",
    "scene-bookmark-output",
    "spawn-point-output",
    "collision-matrix-output",
    "environment-probe-output",
    "feature-flag-output",
    "remote-config-output",
    "telemetry-query-output",
    "patch-planner-output",
    "dlc-output",
    "symbolication-output",
    "segment-output",
    "experiment-output",
  ].includes(bottom);
}

function renderAdditionalEditorPage(kind) {
  const page = el("div", "tool-page additional-editor");
  const cfg = additionalEditorConfig(kind);
  page.append(toolToolbar(cfg.toolbar));
  const body = el("div", `additional-editor-body ${cfg.layout}`);
  if (cfg.layout === "source-preview") {
    body.append(codePanel(), previewPanel(), diagnosticsPanel(cfg.diagnosticTitle, cfg.diagnostics));
  } else if (cfg.layout === "viewport-tools") {
    body.append(renderViewport(), renderToolPropertyPanel(cfg.panelTitle, cfg.panelRows));
  } else if (cfg.layout === "waveform") {
    body.append(renderAudioGraph(), renderToolPropertyPanel(cfg.panelTitle, cfg.panelRows));
  } else if (cfg.layout === "table-editor") {
    body.append(renderDataEditorGrid(cfg.tableRows), renderToolPropertyPanel(cfg.panelTitle, cfg.panelRows));
  } else if (cfg.layout === "metrics-graph") {
    body.append(renderMetricsGraph(cfg.nodes, cfg.metrics), renderToolPropertyPanel(cfg.panelTitle, cfg.panelRows));
  } else {
    body.append(renderGraphCanvas(cfg.nodes), renderToolPropertyPanel(cfg.panelTitle, cfg.panelRows));
  }
  page.append(body);
  return page;
}

function renderAdditionalDetail(kind) {
  const cfg = additionalDetailConfig(kind);
  const wrap = el("div", "asset-detail");
  wrap.innerHTML = `<div class="section-title">${cfg.title}</div><div class="muted" style="margin:4px 0 12px">${cfg.subtitle}</div>`;
  cfg.rows.forEach(([label, value]) => wrap.append(fieldRow(label, value)));
  wrap.append(button(cfg.action, "primary-btn"));
  return wrap;
}

function renderAdditionalOutput(kind) {
  const cfg = additionalOutputConfig(kind);
  return tabbedLog(cfg.tabs, cfg.rows);
}

function additionalDrawerLayoutFor(design, fileTree) {
  const cfg = additionalEditorConfig(design.center);
  return {
    leftTop: { title: cfg.leftTop.title, zone: "Left Top", tabs: cfg.leftTop.tabs, body: cfg.leftTop.body() },
    leftBottom: fileTree,
    rightTop: { title: cfg.rightTop.title, zone: "Right Top", tabs: cfg.rightTop.tabs, body: cfg.rightTop.body() },
    rightBottom: { title: cfg.rightBottom.title, zone: "Right Bottom", tabs: cfg.rightBottom.tabs, body: cfg.rightBottom.body() },
    bottom: { title: cfg.bottomTitle, zone: "Bottom", tabs: cfg.bottomTabs, body: renderAdditionalOutput(design.bottom) },
  };
}

function additionalEditorConfig(kind) {
  const graphNodes = [
    ["Input", 8, 16],
    ["Process", 36, 32],
    ["Output", 66, 58],
    ["Preview", 38, 70],
  ];
  const configs = {
    "prefab-editor": {
      toolbar: ["Prefab", "Viewport", "Variants", "Validate", "Save"],
      layout: "viewport-tools",
      panelTitle: "Prefab Variants",
      panelRows: [["Default", "Enabled"], ["LOD", "Auto"], ["Validation", "Clean"]],
      leftTop: { title: "Prefab Parts", tabs: ["Parts", "Variants"], body: () => navList(["Root", "Geometry", "AudioZone", "Trigger", "Spawn Points"], 2) },
      rightTop: { title: "Prefab Tree", tabs: ["Hierarchy", "Overrides"], body: () => renderHierarchy() },
      rightBottom: { title: "Overrides", tabs: ["Properties", "Diff"], body: () => renderAdditionalDetail("prefab-detail") },
      bottomTitle: "Prefab Output",
      bottomTabs: ["Validation", "References", "Console"],
    },
    "vfx-editor": {
      toolbar: ["VFX", "Simulate", "Emitters", "Curves", "Save"],
      layout: "graph",
      nodes: [["Spawn", 10, 20], ["Velocity", 36, 22], ["Color", 38, 56], ["Renderer", 68, 40]],
      panelTitle: "Emitter Preview",
      panelRows: [["Particles", "1,024"], ["Lifetime", "2.5 s"], ["Bounds", "Auto"]],
      leftTop: { title: "Emitters", tabs: ["Emitters", "Modules"], body: () => navList(["Spark Burst", "Smoke Trail", "Glow Sprite", "Collision"], 0) },
      rightTop: { title: "Effect Graph", tabs: ["Graph", "Events"], body: () => treeList([["P_Sparks", 0, true], ["Spawn", 1, false], ["Update", 1, false], ["Render", 1, false]]) },
      rightBottom: { title: "Emitter State", tabs: ["Params", "Curves"], body: () => renderAdditionalDetail("vfx-detail") },
      bottomTitle: "Simulation Output",
      bottomTabs: ["Simulation", "Warnings", "Console"],
    },
    "shader-editor": {
      toolbar: ["Shader", "Source", "Preview", "Compile", "Variants"],
      layout: "source-preview",
      diagnosticTitle: "Shader compile ready",
      diagnostics: ["3 variants", "0 errors", "2 warnings"],
      leftTop: { title: "Shader Files", tabs: ["Files", "Includes"], body: () => navList(["unlit.zshader", "common_lighting", "surface_inputs", "debug_view"], 0) },
      rightTop: { title: "Pipeline", tabs: ["States", "Bindings"], body: () => renderBuildPipeline() },
      rightBottom: { title: "Variants", tabs: ["Keywords", "Preview"], body: () => renderAdditionalDetail("shader-detail") },
      bottomTitle: "Shader Output",
      bottomTabs: ["Compiler", "SPIR-V", "Console"],
    },
    "terrain-editor": {
      toolbar: ["Terrain", "Sculpt", "Paint", "Erode", "Bake"],
      layout: "viewport-tools",
      panelTitle: "Brush",
      panelRows: [["Size", "18 m"], ["Strength", "0.42"], ["Falloff", "Smooth"]],
      leftTop: { title: "Brushes", tabs: ["Sculpt", "Paint"], body: () => navList(["Raise", "Smooth", "Flatten", "Noise", "Layer Paint"], 1) },
      rightTop: { title: "Layers", tabs: ["Terrain", "Masks"], body: () => navList(["Base Rock", "Dust", "Metal Floor", "Decals"], 1) },
      rightBottom: { title: "Brush Settings", tabs: ["Brush", "Bake"], body: () => renderAdditionalDetail("terrain-detail") },
      bottomTitle: "Terrain Output",
      bottomTabs: ["Bake", "Paint", "Console"],
    },
    "audio-editor": {
      toolbar: ["Audio", "Graph", "Mixer", "Preview", "Analyze"],
      layout: "waveform",
      panelTitle: "Audio Event",
      panelRows: [["Bus", "Ambience"], ["Loop", "On"], ["Loudness", "-18 LUFS"]],
      leftTop: { title: "Events", tabs: ["Events", "Clips"], body: () => navList(["Ambience_Hangar", "Door_Open", "Alarm_Loop", "Footstep_Metal"], 0) },
      rightTop: { title: "Routing", tabs: ["Graph", "Buses"], body: () => renderGraphCanvas(graphNodes) },
      rightBottom: { title: "Event Detail", tabs: ["Event", "Mixer"], body: () => renderAdditionalDetail("audio-detail") },
      bottomTitle: "Audio Output",
      bottomTabs: ["Analysis", "Mixer", "Console"],
    },
    "behavior-tree": {
      toolbar: ["Behavior", "Run", "Blackboard", "Trace", "Save"],
      layout: "graph",
      nodes: [["Root", 12, 16], ["Selector", 38, 30], ["Patrol", 22, 58], ["Chase", 58, 58]],
      panelTitle: "Runtime Trace",
      panelRows: [["State", "Running"], ["Tick", "0.16 ms"], ["Blackboard", "12 keys"]],
      leftTop: { title: "Node Palette", tabs: ["Tasks", "Decorators"], body: () => navList(["Selector", "Sequence", "Move To", "Wait", "Set Blackboard"], 2) },
      rightTop: { title: "Blackboard", tabs: ["Keys", "Watch"], body: () => renderMiniDataGrid([["Key", "Type", "Value"], ["Target", "Entity", "None"], ["Alert", "Bool", "False"], ["PatrolIndex", "Int", "2"]]) },
      rightBottom: { title: "Node Detail", tabs: ["Node", "Trace"], body: () => renderAdditionalDetail("behavior-detail") },
      bottomTitle: "Behavior Output",
      bottomTabs: ["Trace", "Blackboard", "Console"],
    },
    "lighting-bake": {
      toolbar: ["Lighting", "Preview", "Bake", "Denoise", "Apply"],
      layout: "viewport-tools",
      panelTitle: "Bake Job",
      panelRows: [["Quality", "Preview"], ["Samples", "128"], ["Denoise", "On"]],
      leftTop: { title: "Lights", tabs: ["Lights", "Volumes"], body: () => navList(["Key Area", "Door Strip", "Console Glow", "Volume A1"], 0) },
      rightTop: { title: "Bake Targets", tabs: ["Scenes", "Maps"], body: () => navList(["A1_Hangar", "Reflection Probes", "Lightmaps", "Irradiance"], 0) },
      rightBottom: { title: "Bake Settings", tabs: ["Quality", "Output"], body: () => renderAdditionalDetail("lighting-detail") },
      bottomTitle: "Bake Output",
      bottomTabs: ["Jobs", "Warnings", "Console"],
    },
    "physics-collision": {
      toolbar: ["Physics", "Shapes", "Simulate", "Contacts", "Apply"],
      layout: "viewport-tools",
      panelTitle: "Collision Shape",
      panelRows: [["Shape", "Box"], ["Mass", "12.0"], ["Friction", "0.62"]],
      leftTop: { title: "Shapes", tabs: ["Shapes", "Presets"], body: () => navList(["Box", "Sphere", "Capsule", "Convex Hull", "Mesh Proxy"], 0) },
      rightTop: { title: "Bodies", tabs: ["Hierarchy", "Contacts"], body: () => renderHierarchy() },
      rightBottom: { title: "Body Detail", tabs: ["Collision", "Solver"], body: () => renderAdditionalDetail("physics-detail") },
      bottomTitle: "Physics Output",
      bottomTabs: ["Simulation", "Contacts", "Console"],
    },
    "level-streaming": {
      toolbar: ["Level", "Cells", "Preview", "Load", "Save"],
      layout: "viewport-tools",
      panelTitle: "Streaming Cell",
      panelRows: [["Cell", "A1_North"], ["State", "Loaded"], ["Memory", "184 MB"], ["Distance", "126 m"]],
      leftTop: { title: "Streaming Cells", tabs: ["Cells", "Volumes"], body: () => navList(["A1_North", "A1_South", "Hangar_Interior", "Dock_Exterior", "Lighting Cells"], 0) },
      rightTop: { title: "World Partition", tabs: ["Grid", "Layers"], body: () => renderMiniDataGrid([["Cell", "State", "Memory", "Layer", ""], ["A1_North", "Loaded", "184 MB", "Gameplay", ""], ["A1_South", "Visible", "92 MB", "Art", ""], ["Dock", "Unloaded", "0 MB", "World", ""]]) },
      rightBottom: { title: "Cell Detail", tabs: ["Cell", "Rules"], body: () => renderAdditionalDetail("level-streaming-detail") },
      bottomTitle: "Streaming Output",
      bottomTabs: ["Loads", "Memory", "Console"],
    },
    sequencer: {
      toolbar: ["Sequencer", "Shots", "Keys", "Preview", "Render"],
      layout: "graph",
      nodes: [["Shot 010", 10, 18], ["Camera Cut", 34, 30], ["Light Cue", 58, 22], ["Audio Cue", 54, 58]],
      panelTitle: "Shot Track",
      panelRows: [["Shot", "010"], ["Camera", "Cam_A"], ["Duration", "4.2 s"], ["Keys", "18"]],
      leftTop: { title: "Shots", tabs: ["Shots", "Takes"], body: () => navList(["Shot 010", "Shot 020", "Shot 030", "Take 02", "Render Queue"], 0) },
      rightTop: { title: "Track Tree", tabs: ["Tracks", "Bindings"], body: () => treeList([["Intro_Hangar", 0, true], ["Camera Cuts", 1, false], ["Lights", 1, false], ["Audio", 1, false], ["Events", 1, false]]) },
      rightBottom: { title: "Key Detail", tabs: ["Key", "Curve"], body: () => renderAdditionalDetail("sequencer-detail") },
      bottomTitle: "Sequencer Timeline",
      bottomTabs: ["Timeline", "Curves", "Render"],
    },
    "navmesh-ai": {
      toolbar: ["NavMesh", "Agents", "Areas", "Bake", "Debug"],
      layout: "viewport-tools",
      panelTitle: "Nav Area",
      panelRows: [["Agent", "Humanoid"], ["Slope", "45 deg"], ["Step", "0.45 m"], ["Tiles", "128"]],
      leftTop: { title: "Agents", tabs: ["Agents", "Areas"], body: () => navList(["Humanoid", "Drone", "Heavy", "NoWalk", "JumpLink"], 0) },
      rightTop: { title: "Navigation Layers", tabs: ["Areas", "Links"], body: () => navList(["Walkable", "Blocked", "Jump Link", "Door Cost", "Cover Points"], 0) },
      rightBottom: { title: "Agent Detail", tabs: ["Agent", "Bake"], body: () => renderAdditionalDetail("navmesh-detail") },
      bottomTitle: "NavMesh Output",
      bottomTabs: ["Bake", "Warnings", "Console"],
    },
    "render-pipeline": {
      toolbar: ["Pipeline", "Pass Graph", "Compile", "Capture", "Save"],
      layout: "metrics-graph",
      nodes: [["GBuffer", 10, 18], ["Lighting", 35, 28], ["UI Pass", 62, 22], ["Post", 58, 60]],
      metrics: [["8.4", "Scene ms"], ["2.2", "UI ms"], ["18", "Batches"], ["7", "Passes"]],
      panelTitle: "Render Target",
      panelRows: [["Target", "HDR_Main"], ["Format", "RGBA16F"], ["Scale", "1.0"], ["Samples", "1"]],
      leftTop: { title: "Pass Library", tabs: ["Passes", "Targets"], body: () => navList(["GBuffer", "Lighting", "Forward+", "UI Pass", "Post Process"], 1) },
      rightTop: { title: "Render Targets", tabs: ["Targets", "Resources"], body: () => renderMiniDataGrid([["Target", "Format", "Size", "State", ""], ["HDR_Main", "RGBA16F", "1672x941", "Ready", ""], ["Depth", "D32", "1672x941", "Ready", ""], ["UI", "RGBA8", "1672x941", "Ready", ""]]) },
      rightBottom: { title: "Pass Detail", tabs: ["Pass", "Bindings"], body: () => renderAdditionalDetail("render-pipeline-detail") },
      bottomTitle: "Pipeline Output",
      bottomTabs: ["Compile", "Capture", "Console"],
    },
    "input-mapping": {
      toolbar: ["Input", "Contexts", "Actions", "Validate", "Save"],
      layout: "table-editor",
      tableRows: [["Action", "Device", "Binding", "Context", ""], ["Move", "Keyboard", "WASD", "Editor", ""], ["Orbit", "Mouse", "Alt+LMB", "Viewport", ""], ["Play", "Keyboard", "Ctrl+P", "Global", ""], ["Duplicate", "Keyboard", "Ctrl+D", "Scene", ""]],
      panelTitle: "Binding Conflict",
      panelRows: [["Action", "Play"], ["Binding", "Ctrl+P"], ["Scope", "Global"], ["Conflicts", "0"]],
      leftTop: { title: "Devices", tabs: ["Devices", "Contexts"], body: () => navList(["Keyboard", "Mouse", "Gamepad", "Pen Tablet", "Touch"], 0) },
      rightTop: { title: "Action Maps", tabs: ["Maps", "Contexts"], body: () => treeList([["Editor_Default", 0, true], ["Global", 1, false], ["Viewport", 1, false], ["Scene", 1, false]]) },
      rightBottom: { title: "Binding Detail", tabs: ["Binding", "Conflicts"], body: () => renderAdditionalDetail("input-mapping-detail") },
      bottomTitle: "Input Output",
      bottomTabs: ["Validation", "Conflicts", "Console"],
    },
    "data-table": {
      toolbar: ["Data Table", "Schema", "Rows", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Id", "Name", "Type", "Value", ""], ["item.crate", "Crate", "Mesh", "Box_01", ""], ["mat.metal", "Metal", "Material", "M_Metal", ""], ["ui.ready", "Ready Text", "String", "Ready", ""], ["fx.spark", "Sparks", "Effect", "P_Sparks", ""]],
      panelTitle: "Row Validation",
      panelRows: [["Row", "mat.metal"], ["Schema", "AssetRef"], ["Status", "Valid"], ["Refs", "3"]],
      leftTop: { title: "Schemas", tabs: ["Schemas", "Imports"], body: () => navList(["AssetRef", "GameplayItem", "LocalizedText", "SpawnProfile", "BuildTarget"], 0) },
      rightTop: { title: "Columns", tabs: ["Columns", "Types"], body: () => renderMiniDataGrid([["Column", "Type", "Required", "Index", ""], ["Id", "String", "Yes", "0", ""], ["Name", "String", "Yes", "1", ""], ["Value", "AssetRef", "No", "3", ""]]) },
      rightBottom: { title: "Row Detail", tabs: ["Row", "Validation"], body: () => renderAdditionalDetail("data-table-detail") },
      bottomTitle: "Data Output",
      bottomTabs: ["Validation", "Import", "Console"],
    },
    "network-replication": {
      toolbar: ["Network", "Peers", "Replicate", "Capture", "Inspect"],
      layout: "metrics-graph",
      nodes: [["Server", 12, 22], ["Client A", 42, 18], ["Client B", 42, 56], ["Ghosts", 68, 38]],
      metrics: [["3", "Peers"], ["42", "Entities"], ["1.2", "KB/frame"], ["18", "RPC/min"]],
      panelTitle: "Replication Channel",
      panelRows: [["Peer", "Client A"], ["Ping", "42 ms"], ["Loss", "0.2%"], ["Priority", "High"]],
      leftTop: { title: "Peers", tabs: ["Peers", "Channels"], body: () => navList(["Server", "Client A", "Client B", "Spectator", "Replay"], 1) },
      rightTop: { title: "Replicated Entities", tabs: ["Entities", "Properties"], body: () => renderMiniDataGrid([["Entity", "Owner", "Dirty", "Rate", ""], ["Player_01", "Client A", "12", "30 Hz", ""], ["Crate_01", "Server", "2", "10 Hz", ""], ["Door_A", "Server", "0", "5 Hz", ""]]) },
      rightBottom: { title: "Peer Detail", tabs: ["Peer", "Traffic"], body: () => renderAdditionalDetail("network-replication-detail") },
      bottomTitle: "Network Output",
      bottomTabs: ["Replication", "RPC", "Console"],
    },
    localization: {
      toolbar: ["Localization", "Locales", "Review", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Key", "en-US", "zh-CN", "Status", ""], ["ui.ready", "Ready", "就绪", "Done", ""], ["ui.warning", "Warning", "警告", "Done", ""], ["menu.build", "Build", "构建", "Review", ""], ["asset.missing", "Missing Asset", "", "Missing", ""]],
      panelTitle: "Translation Detail",
      panelRows: [["Locale", "zh-CN"], ["Missing", "1"], ["Review", "2"], ["Coverage", "97%"]],
      leftTop: { title: "Locales", tabs: ["Locales", "Namespaces"], body: () => navList(["en-US", "zh-CN", "ja-JP", "de-DE", "editor.shell"], 1) },
      rightTop: { title: "Namespaces", tabs: ["Keys", "Coverage"], body: () => renderMiniDataGrid([["Namespace", "Keys", "Missing", "Coverage", ""], ["editor.shell", "184", "1", "99%", ""], ["assets", "92", "0", "100%", ""], ["build", "42", "2", "95%", ""]]) },
      rightBottom: { title: "String Detail", tabs: ["String", "Review"], body: () => renderAdditionalDetail("localization-detail") },
      bottomTitle: "Localization Output",
      bottomTabs: ["Validation", "Export", "Console"],
    },
    "visual-script": {
      toolbar: ["Visual Script", "Run", "Compile", "Debug", "Save"],
      layout: "graph",
      nodes: [["Event", 8, 18], ["Branch", 34, 28], ["Open Door", 58, 18], ["Play Sound", 58, 58], ["Return", 78, 40]],
      panelTitle: "Graph Watch",
      panelRows: [["Graph", "DoorController"], ["Variables", "6"], ["Breakpoints", "2"], ["Compile", "Clean"]],
      leftTop: { title: "Node Palette", tabs: ["Nodes", "Macros"], body: () => navList(["Event BeginPlay", "Branch", "Set Variable", "Play Sound", "Spawn Prefab"], 1) },
      rightTop: { title: "Variables", tabs: ["Variables", "Watch"], body: () => renderMiniDataGrid([["Name", "Type", "Value", "Scope", ""], ["DoorOpen", "Bool", "False", "Graph", ""], ["Speed", "Float", "1.25", "Local", ""], ["Sound", "Asset", "Door_Open", "Graph", ""]]) },
      rightBottom: { title: "Node Detail", tabs: ["Node", "Pins"], body: () => renderAdditionalDetail("visual-script-detail") },
      bottomTitle: "Script Output",
      bottomTabs: ["Compiler", "Trace", "Console"],
    },
    "state-machine": {
      toolbar: ["State Machine", "Preview", "Transitions", "Validate", "Save"],
      layout: "graph",
      nodes: [["Idle", 12, 28], ["Walk", 38, 24], ["Run", 62, 26], ["Jump", 38, 62], ["Land", 64, 62]],
      panelTitle: "Transition Preview",
      panelRows: [["Active", "Walk"], ["Next", "Run"], ["Blend", "0.18 s"], ["Conditions", "3"]],
      leftTop: { title: "States", tabs: ["States", "Clips"], body: () => navList(["Idle", "Walk", "Run", "Jump", "Land", "Aim Offset"], 1) },
      rightTop: { title: "Conditions", tabs: ["Conditions", "Params"], body: () => renderMiniDataGrid([["Param", "Type", "Value", "Use", ""], ["Speed", "Float", "3.8", "Run", ""], ["Grounded", "Bool", "True", "Jump", ""], ["Aim", "Float", "0.2", "Blend", ""]]) },
      rightBottom: { title: "State Detail", tabs: ["State", "Transition"], body: () => renderAdditionalDetail("state-machine-detail") },
      bottomTitle: "State Output",
      bottomTabs: ["Validation", "Preview", "Console"],
    },
    "skeleton-mesh": {
      toolbar: ["Skeleton", "Preview", "Weights", "Sockets", "Reimport"],
      layout: "viewport-tools",
      panelTitle: "Skin Weight",
      panelRows: [["Bone", "spine_02"], ["Influences", "4"], ["Max Weight", "0.72"], ["LOD", "0"]],
      leftTop: { title: "Bone Tree", tabs: ["Bones", "Sockets"], body: () => treeList([["root", 0, false], ["pelvis", 1, false], ["spine_01", 2, false], ["spine_02", 3, true], ["head", 3, false], ["arm_l", 2, false]]) },
      rightTop: { title: "Mesh Sections", tabs: ["Sections", "LODs"], body: () => renderMiniDataGrid([["Section", "Material", "Tris", "LOD", ""], ["Body", "M_Guard", "8.4k", "0", ""], ["Armor", "M_Metal", "2.1k", "0", ""], ["Head", "M_Skin", "1.8k", "0", ""]]) },
      rightBottom: { title: "Bone Detail", tabs: ["Bone", "Weights"], body: () => renderAdditionalDetail("skeleton-mesh-detail") },
      bottomTitle: "Skeleton Output",
      bottomTabs: ["Import", "Weights", "Console"],
    },
    "texture-editor": {
      toolbar: ["Texture", "Channels", "Mips", "Compress", "Apply"],
      layout: "metrics-graph",
      nodes: [["Source", 12, 18], ["Mip 0", 38, 20], ["Mip 1", 56, 36], ["BC7", 74, 56]],
      metrics: [["2048", "Size"], ["8", "Mips"], ["BC7", "Format"], ["5.2", "MB"]],
      panelTitle: "Texture Preview",
      panelRows: [["Channel", "RGBA"], ["Mip", "0"], ["sRGB", "On"], ["Compression", "BC7"]],
      leftTop: { title: "Textures", tabs: ["Textures", "Channels"], body: () => navList(["T_Grid_01", "T_Roughness", "T_Normal", "T_Mask", "T_UIAtlas"], 0) },
      rightTop: { title: "Mip Chain", tabs: ["Mips", "Stats"], body: () => renderMiniDataGrid([["Mip", "Size", "Memory", "State", ""], ["0", "2048", "4.0 MB", "Ready", ""], ["1", "1024", "1.0 MB", "Ready", ""], ["2", "512", "256 KB", "Ready", ""]]) },
      rightBottom: { title: "Texture Detail", tabs: ["Import", "Compression"], body: () => renderAdditionalDetail("texture-detail") },
      bottomTitle: "Texture Output",
      bottomTabs: ["Analysis", "Compression", "Console"],
    },
    "material-instance": {
      toolbar: ["Material Instance", "Preview", "Overrides", "Compile", "Save"],
      layout: "viewport-tools",
      panelTitle: "Parameter Overrides",
      panelRows: [["Base Color", "#8aa0a6"], ["Roughness", "0.72"], ["Metallic", "1.00"], ["Normal", "T_Normal"]],
      leftTop: { title: "Instances", tabs: ["Instances", "Parents"], body: () => navList(["MI_Metal_Rough", "MI_Metal_Clean", "MI_Plastic", "M_Metal parent", "M_Master"], 0) },
      rightTop: { title: "Parameters", tabs: ["Overrides", "Groups"], body: () => renderMiniDataGrid([["Param", "Type", "Value", "Override", ""], ["Base Color", "Color", "#8aa0a6", "Yes", ""], ["Roughness", "Float", "0.72", "Yes", ""], ["Normal", "Texture", "T_Normal", "No", ""]]) },
      rightBottom: { title: "Instance Detail", tabs: ["Params", "Usage"], body: () => renderAdditionalDetail("material-instance-detail") },
      bottomTitle: "Material Instance Output",
      bottomTabs: ["Shader", "Usage", "Console"],
    },
    "prefab-variant": {
      toolbar: ["Prefab Variant", "Diff", "Apply", "Validate", "Save"],
      layout: "viewport-tools",
      panelTitle: "Variant Diff",
      panelRows: [["Variant", "B"], ["Overrides", "7"], ["Removed", "1"], ["Added", "2"]],
      leftTop: { title: "Variants", tabs: ["Variants", "Base"], body: () => navList(["Base", "Variant A", "Variant B", "Combat", "Cinematic"], 2) },
      rightTop: { title: "Nested Prefabs", tabs: ["Tree", "Diff"], body: () => treeList([["AudioZone", 0, true], ["Trigger", 1, false], ["Speaker", 1, false], ["Light Cue", 1, false], ["Debug Marker", 1, false]]) },
      rightBottom: { title: "Override Detail", tabs: ["Override", "References"], body: () => renderAdditionalDetail("prefab-variant-detail") },
      bottomTitle: "Variant Output",
      bottomTabs: ["Validation", "Diff", "Console"],
    },
    "level-audit": {
      toolbar: ["Level Audit", "Run", "Fix", "Filter", "Export"],
      layout: "table-editor",
      tableRows: [["Issue", "Rule", "Object", "Severity", ""], ["Missing lightmap", "Lighting", "Wall_A3", "Warning", ""], ["Unused material", "Assets", "M_Debug", "Info", ""], ["No collision", "Physics", "Crate_07", "Error", ""], ["Oversized texture", "Memory", "T_Wall_4K", "Warning", ""]],
      panelTitle: "Audit Summary",
      panelRows: [["Scene", "A1_Hangar"], ["Issues", "12"], ["Errors", "1"], ["Fixable", "8"]],
      leftTop: { title: "Rule Packs", tabs: ["Rules", "Profiles"], body: () => navList(["Shipping", "Editor Preview", "Lighting", "Physics", "Memory"], 0) },
      rightTop: { title: "Scene Links", tabs: ["Objects", "Assets"], body: () => renderMiniDataGrid([["Object", "Type", "Issue", "Fix", ""], ["Crate_07", "Mesh", "Collision", "Auto", ""], ["Wall_A3", "Mesh", "Lightmap", "Manual", ""], ["M_Debug", "Material", "Unused", "Delete", ""]]) },
      rightBottom: { title: "Issue Detail", tabs: ["Issue", "Fix"], body: () => renderAdditionalDetail("level-audit-detail") },
      bottomTitle: "Audit Output",
      bottomTabs: ["Run", "Fixes", "Console"],
    },
    "test-runner": {
      toolbar: ["Test Runner", "Run", "Debug", "Filter", "Export"],
      layout: "table-editor",
      tableRows: [["Test", "Suite", "Result", "Time", ""], ["layout_smoke", "UI", "Pass", "18 ms", ""], ["asset_import_mesh", "Assets", "Pass", "42 ms", ""], ["viewport_resize", "Editor", "Fail", "31 ms", ""], ["console_filter", "Diagnostics", "Pass", "9 ms", ""]],
      panelTitle: "Run Summary",
      panelRows: [["Suites", "6"], ["Passed", "42"], ["Failed", "1"], ["Duration", "2.4 s"]],
      leftTop: { title: "Suites", tabs: ["Suites", "Targets"], body: () => navList(["UI Layout", "Asset Import", "Runtime", "Editor Host", "Smoke"], 0) },
      rightTop: { title: "Failures", tabs: ["Failures", "History"], body: () => renderMiniDataGrid([["Test", "File", "Line", "State", ""], ["viewport_resize", "host_window.rs", "184", "Failed", ""], ["layout_smoke", "layout.rs", "91", "Pass", ""], ["console_filter", "console.rs", "42", "Pass", ""]]) },
      rightBottom: { title: "Failure Detail", tabs: ["Failure", "Output"], body: () => renderAdditionalDetail("test-runner-detail") },
      bottomTitle: "Test Output",
      bottomTabs: ["Run", "Failures", "Console"],
    },
    "frame-debugger": {
      toolbar: ["Frame Debugger", "Capture", "Step", "Targets", "Export"],
      layout: "metrics-graph",
      nodes: [["Depth", 8, 20], ["GBuffer", 28, 30], ["Lighting", 48, 22], ["UI", 66, 42], ["Post", 80, 58]],
      metrics: [["1842", "Frame"], ["128", "Draws"], ["7", "Passes"], ["16.6", "ms"]],
      panelTitle: "Selected Draw",
      panelRows: [["Pass", "Lighting"], ["Draw", "#064"], ["Pipeline", "Forward+"], ["Cost", "0.42 ms"]],
      leftTop: { title: "Captures", tabs: ["Captures", "Frames"], body: () => navList(["Frame 1842", "Frame 1841", "Viewport Resize", "UI Pass", "Post Stack"], 0) },
      rightTop: { title: "Render Passes", tabs: ["Passes", "Targets"], body: () => renderMiniDataGrid([["Pass", "Draws", "Target", "Cost", ""], ["Depth", "18", "Depth", "0.7 ms", ""], ["Lighting", "64", "HDR_Main", "3.1 ms", ""], ["UI", "12", "UI", "0.8 ms", ""]]) },
      rightBottom: { title: "Draw Detail", tabs: ["Draw", "State"], body: () => renderAdditionalDetail("frame-debugger-detail") },
      bottomTitle: "GPU Event Output",
      bottomTabs: ["Events", "Markers", "Console"],
    },
    "memory-profiler": {
      toolbar: ["Memory", "Snapshot", "Compare", "Leaks", "Export"],
      layout: "metrics-graph",
      nodes: [["Assets", 12, 28], ["Renderer", 36, 18], ["UI", 52, 48], ["Scripts", 70, 32]],
      metrics: [["1.42", "GB"], ["428", "Assets"], ["18", "Leaks"], ["92", "MB delta"]],
      panelTitle: "Heap Summary",
      panelRows: [["Heap", "Renderer"], ["Size", "412 MB"], ["Delta", "+18 MB"], ["Suspects", "3"]],
      leftTop: { title: "Snapshots", tabs: ["Snapshots", "Compare"], body: () => navList(["Startup", "After Import", "Frame 1842", "Before Close", "Leak Pass"], 2) },
      rightTop: { title: "Alloc Groups", tabs: ["Groups", "Assets"], body: () => renderMiniDataGrid([["Group", "Size", "Delta", "Count", ""], ["Textures", "512 MB", "+24 MB", "184", ""], ["Renderer", "412 MB", "+18 MB", "64", ""], ["UI", "86 MB", "+4 MB", "318", ""]]) },
      rightBottom: { title: "Allocation Detail", tabs: ["Alloc", "Stack"], body: () => renderAdditionalDetail("memory-profiler-detail") },
      bottomTitle: "Memory Output",
      bottomTabs: ["Snapshots", "Leaks", "Console"],
    },
    "asset-dependency": {
      toolbar: ["Dependencies", "Trace", "Prune", "Validate", "Export"],
      layout: "graph",
      nodes: [["A1_Hangar", 8, 34], ["Box_01", 34, 18], ["M_Metal", 58, 24], ["T_Grid", 76, 44], ["AudioZone", 38, 62]],
      panelTitle: "Dependency Focus",
      panelRows: [["Root", "A1_Hangar"], ["Direct", "18"], ["Transitive", "146"], ["Missing", "0"]],
      leftTop: { title: "Packages", tabs: ["Packages", "Roots"], body: () => navList(["A1_Hangar.scene", "AudioZone.prefab", "M_Metal.zmat", "UI Shell", "Build Profile"], 0) },
      rightTop: { title: "Affected Assets", tabs: ["Assets", "Cycles"], body: () => renderMiniDataGrid([["Asset", "Type", "Refs", "State", ""], ["Box_01.mesh", "Mesh", "12", "Used", ""], ["M_Metal", "Material", "18", "Used", ""], ["T_Grid_01", "Texture", "7", "Used", ""]]) },
      rightBottom: { title: "Reference Detail", tabs: ["Asset", "Path"], body: () => renderAdditionalDetail("asset-dependency-detail") },
      bottomTitle: "Dependency Output",
      bottomTabs: ["Trace", "Cycles", "Console"],
    },
    "reference-finder": {
      toolbar: ["Reference Finder", "Search", "Replace", "Scope", "Export"],
      layout: "table-editor",
      tableRows: [["Owner", "Path", "Use", "Status", ""], ["A1_Hangar.scene", "Level/Props", "Mesh", "Live", ""], ["AudioZone.prefab", "Root/Trigger", "Mesh", "Live", ""], ["MI_Metal_Rough", "Params/Base", "Texture", "Live", ""], ["BuildProfile", "Cook/Include", "Asset", "Cooked", ""]],
      panelTitle: "Search Target",
      panelRows: [["Asset", "Box_01.mesh"], ["Refs", "18"], ["Writable", "14"], ["Replaceable", "12"]],
      leftTop: { title: "Scopes", tabs: ["Scopes", "Recent"], body: () => navList(["Project", "Open Scenes", "Content", "Prefabs", "Build Profiles"], 0) },
      rightTop: { title: "Owners", tabs: ["Owners", "Types"], body: () => renderMiniDataGrid([["Owner", "Type", "Count", "State", ""], ["Scenes", "Scene", "8", "Live", ""], ["Prefabs", "Prefab", "6", "Live", ""], ["Materials", "Material", "4", "Live", ""]]) },
      rightBottom: { title: "Owner Detail", tabs: ["Owner", "Replace"], body: () => renderAdditionalDetail("reference-finder-detail") },
      bottomTitle: "Reference Output",
      bottomTabs: ["Search", "Replace", "Console"],
    },
    "cook-package": {
      toolbar: ["Cook Package", "Validate", "Cook", "Stage", "Package"],
      layout: "metrics-graph",
      nodes: [["Validate", 10, 26], ["Cook Assets", 34, 28], ["Stage", 58, 36], ["Package", 78, 48]],
      metrics: [["Windows", "Target"], ["428", "Assets"], ["92", "MB"], ["4", "Steps"]],
      panelTitle: "Package Queue",
      panelRows: [["Target", "Windows"], ["Profile", "Development"], ["Step", "Cook Assets"], ["Progress", "62%"]],
      leftTop: { title: "Profiles", tabs: ["Profiles", "Targets"], body: () => navList(["Windows Development", "Windows Shipping", "Linux Server", "Editor Tools", "CI Smoke"], 0) },
      rightTop: { title: "Cook Tasks", tabs: ["Tasks", "Assets"], body: () => renderBuildPipeline() },
      rightBottom: { title: "Package Detail", tabs: ["Target", "Artifacts"], body: () => renderAdditionalDetail("cook-package-detail") },
      bottomTitle: "Cook Output",
      bottomTabs: ["Cook", "Warnings", "Console"],
    },
    "crash-session-replay": {
      toolbar: ["Crash Replay", "Load", "Step", "Bookmark", "Export"],
      layout: "table-editor",
      tableRows: [["Time", "Channel", "Event", "State", ""], ["09:58:10", "Renderer", "Surface resize", "Info", ""], ["09:58:12", "Assets", "Texture fallback", "Warn", ""], ["09:58:14", "Renderer", "Surface acquire panic", "Crash", ""], ["09:58:15", "Editor", "Autosave checkpoint", "Recovered", ""]],
      panelTitle: "Replay State",
      panelRows: [["Session", "renderer-panic"], ["Frame", "1842"], ["Bookmark", "Panic"], ["Recovered", "Yes"]],
      leftTop: { title: "Crash Sessions", tabs: ["Sessions", "Recent"], body: () => navList(["renderer-panic-09-58", "asset-import-warn", "ui-resize-smoke", "startup-cache", "old-session"], 0) },
      rightTop: { title: "Captured State", tabs: ["State", "Files"], body: () => renderMiniDataGrid([["Item", "Type", "Size", "State", ""], ["Autosave", "Scene", "1.2 MB", "Loaded", ""], ["Crash Log", "Text", "42 KB", "Loaded", ""], ["GPU Report", "JSON", "18 KB", "Loaded", ""]]) },
      rightBottom: { title: "Stack Detail", tabs: ["Stack", "Recovery"], body: () => renderAdditionalDetail("crash-session-detail") },
      bottomTitle: "Replay Output",
      bottomTabs: ["Timeline", "Recovery", "Console"],
    },
    "log-analysis": {
      toolbar: ["Log Analysis", "Parse", "Patterns", "Triage", "Export"],
      layout: "table-editor",
      tableRows: [["Time", "Level", "Channel", "Message", ""], ["12:04:18", "Info", "Renderer", "Frame compiled", ""], ["12:04:25", "Warn", "Assets", "Texture fallback", ""], ["12:04:31", "Warn", "Audio", "Listener route missing", ""], ["12:04:46", "Info", "Viewport", "Camera updated", ""]],
      panelTitle: "Pattern Match",
      panelRows: [["Pattern", "Fallback"], ["Hits", "2"], ["Severity", "Warn"], ["Owner", "Assets"]],
      leftTop: { title: "Channels", tabs: ["Channels", "Patterns"], body: () => navList(["Renderer", "Assets", "Audio", "Viewport", "Runtime UI"], 1) },
      rightTop: { title: "Patterns", tabs: ["Patterns", "Owners"], body: () => renderMiniDataGrid([["Pattern", "Hits", "Level", "Owner", ""], ["Fallback", "2", "Warn", "Assets", ""], ["Resize", "1", "Info", "Viewport", ""], ["Listener", "1", "Warn", "Audio", ""]]) },
      rightBottom: { title: "Log Detail", tabs: ["Detail", "Triage"], body: () => renderAdditionalDetail("log-analysis-detail") },
      bottomTitle: "Analysis Output",
      bottomTabs: ["Triage", "Patterns", "Console"],
    },
    "automation-report": {
      toolbar: ["Automation Report", "Refresh", "Compare", "Artifacts", "Export"],
      layout: "metrics-graph",
      nodes: [["UI", 12, 24], ["Assets", 34, 42], ["Runtime", 56, 24], ["Editor", 76, 48]],
      metrics: [["312", "Tests"], ["306", "Pass"], ["6", "Fail"], ["97.8", "%"]],
      panelTitle: "Report Summary",
      panelRows: [["Run", "nightly"], ["Passed", "306"], ["Failed", "6"], ["Artifacts", "18"]],
      leftTop: { title: "Reports", tabs: ["Reports", "Suites"], body: () => navList(["nightly-2026-05-29", "smoke-main", "ui-layout", "asset-import", "runtime"], 0) },
      rightTop: { title: "Artifacts", tabs: ["Artifacts", "Failures"], body: () => renderMiniDataGrid([["Artifact", "Suite", "Type", "State", ""], ["layout.png", "UI", "Image", "Ready", ""], ["failure.log", "Editor", "Log", "Ready", ""], ["trace.json", "Runtime", "Trace", "Ready", ""]]) },
      rightBottom: { title: "Report Detail", tabs: ["Failure", "Artifacts"], body: () => renderAdditionalDetail("automation-report-detail") },
      bottomTitle: "Automation Output",
      bottomTabs: ["Summary", "Failures", "Console"],
    },
    "layout-manager": {
      toolbar: ["Layout Manager", "Preview", "Apply", "Duplicate", "Export"],
      layout: "graph",
      nodes: [["Main Tabs", 10, 18], ["Left Top", 28, 46], ["Center", 52, 28], ["Right Drawers", 74, 46], ["Bottom", 52, 70]],
      panelTitle: "Layout Preview",
      panelRows: [["Layout", "default-workbench"], ["Drawers", "5 zones"], ["Splits", "2"], ["Status", "Valid"]],
      leftTop: { title: "Saved Layouts", tabs: ["Layouts", "Recent"], body: () => navList(["default-workbench", "animation-focus", "asset-review", "diagnostics-wide", "compact"], 0) },
      rightTop: { title: "Drawer Zones", tabs: ["Zones", "Rules"], body: () => renderMiniDataGrid([["Zone", "Tool", "State", "Size", ""], ["Left Top", "Palette", "Open", "282 px", ""], ["Right Top", "Hierarchy", "Open", "336 px", ""], ["Bottom", "Console", "Open", "238 px", ""]]) },
      rightBottom: { title: "Layout Detail", tabs: ["Layout", "Rules"], body: () => renderAdditionalDetail("layout-manager-detail") },
      bottomTitle: "Layout Output",
      bottomTabs: ["Apply", "Diff", "Console"],
    },
    "theme-token": {
      toolbar: ["Theme Tokens", "Preview", "Contrast", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Token", "Value", "Role", "State", ""], ["surface.root", "#111416", "Surface", "Active", ""], ["surface.panel", "#1b1f23", "Panel", "Active", ""], ["accent.teal", "#3cc7d6", "Accent", "Active", ""], ["radius.control", "10px", "Shape", "Active", ""]],
      panelTitle: "Token Preview",
      panelRows: [["Set", "workbench-strict"], ["Contrast", "AA"], ["Accent", "Teal"], ["Gradients", "Reduced"]],
      leftTop: { title: "Token Sets", tabs: ["Sets", "Scopes"], body: () => navList(["workbench-strict", "ios-flat-dark", "diagnostics", "asset-review", "high-contrast"], 0) },
      rightTop: { title: "Components", tabs: ["Controls", "Surfaces"], body: () => renderMiniDataGrid([["Component", "Token", "State", "Check", ""], ["Button", "accent", "Active", "Pass", ""], ["Field", "panel-2", "Focus", "Pass", ""], ["Drawer", "panel", "Open", "Pass", ""]]) },
      rightBottom: { title: "Token Detail", tabs: ["Token", "Usage"], body: () => renderAdditionalDetail("theme-token-detail") },
      bottomTitle: "Theme Output",
      bottomTabs: ["Validation", "Contrast", "Console"],
    },
    "command-center": {
      toolbar: ["Command Center", "Search", "Bind", "Audit", "Export"],
      layout: "table-editor",
      tableRows: [["Command", "Context", "Shortcut", "State", ""], ["Open Scene Editor", "Global", "Ctrl+1", "Bound", ""], ["Toggle Console", "Workbench", "Alt+4", "Bound", ""], ["Place Prefab", "Scene", "P", "Bound", ""], ["Run Cook", "Project", "Ctrl+Shift+C", "Unbound", ""]],
      panelTitle: "Command Audit",
      panelRows: [["Commands", "184"], ["Bound", "172"], ["Conflicts", "0"], ["Unbound", "12"]],
      leftTop: { title: "Command Groups", tabs: ["Groups", "Contexts"], body: () => navList(["Global", "Workbench", "Scene", "Assets", "Project"], 1) },
      rightTop: { title: "Contexts", tabs: ["Contexts", "Keymap"], body: () => renderMiniDataGrid([["Context", "Commands", "Bound", "Conflict", ""], ["Global", "42", "40", "0", ""], ["Scene", "38", "36", "0", ""], ["Project", "24", "21", "0", ""]]) },
      rightBottom: { title: "Command Detail", tabs: ["Command", "Shortcut"], body: () => renderAdditionalDetail("command-center-detail") },
      bottomTitle: "Command Output",
      bottomTabs: ["Audit", "Conflicts", "Console"],
    },
    "module-graph": {
      toolbar: ["Module Graph", "Analyze", "Boundaries", "Services", "Export"],
      layout: "graph",
      nodes: [["zircon_app", 10, 22], ["zircon_runtime", 36, 28], ["zircon_editor", 62, 20], ["resources", 42, 62], ["graphics", 76, 58]],
      panelTitle: "Boundary Summary",
      panelRows: [["Root", "zircon_runtime"], ["Modules", "18"], ["Warnings", "2"], ["Cycles", "0"]],
      leftTop: { title: "Packages", tabs: ["Packages", "Crates"], body: () => navList(["zircon_app", "zircon_runtime", "zircon_editor", "runtime_interface", "tools"], 1) },
      rightTop: { title: "Services", tabs: ["Services", "Edges"], body: () => renderMiniDataGrid([["Service", "Owner", "Deps", "State", ""], ["UiRuntime", "runtime", "4", "Ready", ""], ["AssetDb", "resource", "6", "Ready", ""], ["EditorHost", "editor", "5", "Ready", ""]]) },
      rightBottom: { title: "Module Detail", tabs: ["Module", "Boundary"], body: () => renderAdditionalDetail("module-graph-detail") },
      bottomTitle: "Module Output",
      bottomTabs: ["Analysis", "Boundaries", "Console"],
    },
    "asset-validation": {
      toolbar: ["Asset Validation", "Run", "Repair", "Filter", "Export"],
      layout: "table-editor",
      tableRows: [["Asset", "Rule", "Severity", "Fix", ""], ["T_Grid_01.png", "Compression", "Warning", "Auto", ""], ["Box_01.mesh", "Collision", "Warning", "Manual", ""], ["M_Debug.zmat", "Unused", "Info", "Delete", ""], ["UI Shell", "Reference", "Pass", "-", ""]],
      panelTitle: "Validation Summary",
      panelRows: [["Assets", "428"], ["Warnings", "7"], ["Errors", "0"], ["Fixable", "5"]],
      leftTop: { title: "Rule Packs", tabs: ["Rules", "Profiles"], body: () => navList(["Shipping", "Editor", "Texture", "Mesh", "UI Assets"], 2) },
      rightTop: { title: "Failed Assets", tabs: ["Assets", "Rules"], body: () => renderMiniDataGrid([["Asset", "Type", "Issue", "Fix", ""], ["T_Grid_01", "Texture", "Compression", "Auto", ""], ["Box_01", "Mesh", "Collision", "Manual", ""], ["M_Debug", "Material", "Unused", "Delete", ""]]) },
      rightBottom: { title: "Repair Detail", tabs: ["Issue", "Repair"], body: () => renderAdditionalDetail("asset-validation-detail") },
      bottomTitle: "Validation Output",
      bottomTabs: ["Run", "Repairs", "Console"],
    },
    "hot-reload": {
      toolbar: ["Hot Reload", "Scan", "Reload", "Rollback", "Pin"],
      layout: "graph",
      nodes: [["Changed Files", 12, 28], ["Compile", 36, 24], ["Patch Runtime", 58, 42], ["Refresh UI", 78, 58]],
      panelTitle: "Reload Session",
      panelRows: [["Dirty", "3 modules"], ["Compile", "Ready"], ["Patch", "Pending"], ["Rollback", "Available"]],
      leftTop: { title: "Changes", tabs: ["Changes", "Modules"], body: () => navList(["design.js", "design.css", "ui_template.rs", "asset_loader.rs", "host_window.rs"], 0) },
      rightTop: { title: "Reload Plan", tabs: ["Plan", "Risks"], body: () => renderMiniDataGrid([["Step", "Owner", "State", "Risk", ""], ["Compile", "Rust", "Ready", "Low", ""], ["Patch UI", "Editor", "Queued", "Medium", ""], ["Refresh", "Host", "Queued", "Low", ""]]) },
      rightBottom: { title: "Reload Detail", tabs: ["Module", "Patch"], body: () => renderAdditionalDetail("hot-reload-detail") },
      bottomTitle: "Hot Reload Output",
      bottomTabs: ["Reload", "Warnings", "Console"],
    },
    "project-history": {
      toolbar: ["Project History", "Open", "Diff", "Restore", "Export"],
      layout: "table-editor",
      tableRows: [["Time", "Author", "Change", "State", ""], ["09:12", "Editor", "Added frame debugger page", "Current", ""], ["08:45", "Assets", "Imported Box_01.mesh", "Saved", ""], ["Yesterday", "Runtime", "Updated UI host", "Saved", ""], ["Monday", "Project", "Created Sandbox", "Archived", ""]],
      panelTitle: "History Snapshot",
      panelRows: [["Changes", "12"], ["Assets", "8"], ["Docs", "3"], ["Recover", "Available"]],
      leftTop: { title: "History", tabs: ["Recent", "Bookmarks"], body: () => navList(["Today", "Yesterday", "This Week", "Milestone M3", "Archived"], 0) },
      rightTop: { title: "Changed Files", tabs: ["Files", "Assets"], body: () => renderMiniDataGrid([["File", "Type", "State", "Owner", ""], ["design.js", "Script", "Modified", "Editor", ""], ["workbench.png", "Image", "Added", "Docs", ""], ["index.md", "Doc", "Modified", "Docs", ""]]) },
      rightBottom: { title: "Change Detail", tabs: ["Change", "Diff"], body: () => renderAdditionalDetail("project-history-detail") },
      bottomTitle: "History Output",
      bottomTabs: ["Activity", "Diff", "Console"],
    },
    "task-board": {
      toolbar: ["Task Board", "Sync", "Assign", "Review", "Export"],
      layout: "table-editor",
      tableRows: [["Task", "Owner", "State", "Priority", ""], ["Workbench PNG batch", "UI", "In Review", "High", ""], ["Drawer implementation", "Editor", "Ready", "High", ""], ["Asset validation", "Tools", "In Progress", "Medium", ""], ["Docs pass", "Docs", "Ready", "Medium", ""]],
      panelTitle: "Sprint Summary",
      panelRows: [["Board", "Editor UI"], ["Open", "14"], ["Review", "3"], ["Blocked", "1"]],
      leftTop: { title: "Boards", tabs: ["Boards", "Teams"], body: () => navList(["Editor UI", "Runtime", "Assets", "Rendering", "Docs"], 0) },
      rightTop: { title: "Lanes", tabs: ["Lanes", "People"], body: () => renderMiniDataGrid([["Lane", "Count", "WIP", "State", ""], ["Ready", "6", "2", "OK", ""], ["In Progress", "5", "3", "OK", ""], ["Review", "3", "2", "Watch", ""]]) },
      rightBottom: { title: "Task Detail", tabs: ["Task", "Review"], body: () => renderAdditionalDetail("task-board-detail") },
      bottomTitle: "Task Output",
      bottomTabs: ["Activity", "Reviews", "Console"],
    },
    "source-control": {
      toolbar: ["Source Control", "Sync", "Diff", "Submit", "Revert"],
      layout: "table-editor",
      tableRows: [["File", "State", "Owner", "Changelist", ""], ["design.js", "Modified", "UI", "Workbench PNGs", ""], ["design.css", "Modified", "UI", "Workbench PNGs", ""], ["STYLE-NOTES.md", "Generated", "Docs", "Workbench PNGs", ""], ["workbench.png", "Added", "Docs", "References", ""]],
      panelTitle: "Workspace Status",
      panelRows: [["Modified", "14"], ["Added", "8"], ["Conflicts", "0"], ["Shelved", "2"]],
      leftTop: { title: "Changelists", tabs: ["Changelists", "Shelves"], body: () => navList(["Workbench PNGs", "Runtime UI", "Asset Import", "Docs Update", "Scratch"], 0) },
      rightTop: { title: "File Status", tabs: ["Files", "Locks"], body: () => renderMiniDataGrid([["Path", "State", "Owner", "Lock", ""], ["design.js", "Modified", "UI", "No", ""], ["design.css", "Modified", "UI", "No", ""], ["index.md", "Modified", "Docs", "No", ""]]) },
      rightBottom: { title: "Submit Detail", tabs: ["Submit", "Diff"], body: () => renderAdditionalDetail("source-control-detail") },
      bottomTitle: "Source Output",
      bottomTabs: ["Sync", "Submit", "Console"],
    },
    "review-comments": {
      toolbar: ["Review Comments", "Refresh", "Resolve", "Reply", "Export"],
      layout: "table-editor",
      tableRows: [["Thread", "File", "State", "Owner", ""], ["Right drawer density", "design.css", "Open", "UI", ""], ["Missing doc line", "editor-workbench-design-export.md", "Open", "Docs", ""], ["Button clipping", "test-runner-workbench.png", "Resolved", "UI", ""], ["Token note", "STYLE-NOTES.md", "Open", "Design", ""]],
      panelTitle: "Review Summary",
      panelRows: [["Threads", "12"], ["Open", "6"], ["Resolved", "6"], ["Blocking", "1"]],
      leftTop: { title: "Reviews", tabs: ["Reviews", "Authors"], body: () => navList(["Workbench Design", "Runtime UI", "Asset Pipeline", "Docs", "Release"], 0) },
      rightTop: { title: "Affected Files", tabs: ["Files", "Authors"], body: () => renderMiniDataGrid([["File", "Threads", "Open", "Owner", ""], ["design.css", "4", "2", "UI", ""], ["design.js", "3", "1", "UI", ""], ["docs.md", "2", "2", "Docs", ""]]) },
      rightBottom: { title: "Comment Detail", tabs: ["Comment", "Reply"], body: () => renderAdditionalDetail("review-comments-detail") },
      bottomTitle: "Review Output",
      bottomTabs: ["Activity", "Resolved", "Console"],
    },
    "build-farm": {
      toolbar: ["Build Farm", "Queue", "Dispatch", "Cancel", "Export"],
      layout: "metrics-graph",
      nodes: [["Queue", 12, 24], ["Agent A", 36, 18], ["Agent B", 56, 42], ["Artifacts", 78, 36]],
      metrics: [["4", "Agents"], ["2", "Busy"], ["7", "Jobs"], ["92", "% health"]],
      panelTitle: "Farm Summary",
      panelRows: [["Agents", "4"], ["Busy", "2"], ["Queued", "7"], ["Failures", "1"]],
      leftTop: { title: "Agents", tabs: ["Agents", "Pools"], body: () => navList(["win-builder-01", "win-builder-02", "linux-runner-01", "package-node", "offline-mac"], 0) },
      rightTop: { title: "Job Queue", tabs: ["Jobs", "Artifacts"], body: () => renderMiniDataGrid([["Job", "Target", "State", "Agent", ""], ["editor-ui", "Win", "Running", "win-01", ""], ["asset-cook", "Win", "Queued", "-", ""], ["docs-export", "Any", "Done", "linux", ""]]) },
      rightBottom: { title: "Worker Detail", tabs: ["Worker", "Job"], body: () => renderAdditionalDetail("build-farm-detail") },
      bottomTitle: "Farm Output",
      bottomTabs: ["Queue", "Workers", "Console"],
    },
    "release-notes": {
      toolbar: ["Release Notes", "Collect", "Edit", "Publish", "Export"],
      layout: "table-editor",
      tableRows: [["Section", "Change", "Owner", "State", ""], ["Editor UI", "Workbench PNG export", "UI", "Ready", ""], ["Runtime", "UI host cleanup", "Runtime", "Draft", ""], ["Assets", "Import validation", "Tools", "Ready", ""], ["Docs", "Style notes", "Docs", "Ready", ""]],
      panelTitle: "Release Draft",
      panelRows: [["Version", "0.18.0"], ["Sections", "6"], ["Drafts", "2"], ["Ready", "4"]],
      leftTop: { title: "Milestones", tabs: ["Milestones", "Labels"], body: () => navList(["0.18.0", "Editor UI", "Runtime UI", "Assets", "Hotfix"], 0) },
      rightTop: { title: "Checklist", tabs: ["Checklist", "Publish"], body: () => renderMiniDataGrid([["Item", "Owner", "State", "Gate", ""], ["Screenshots", "UI", "Ready", "Pass", ""], ["Docs", "Docs", "Ready", "Pass", ""], ["Tests", "Runtime", "Draft", "Open", ""]]) },
      rightBottom: { title: "Release Detail", tabs: ["Change", "Publish"], body: () => renderAdditionalDetail("release-notes-detail") },
      bottomTitle: "Release Output",
      bottomTabs: ["Draft", "Publish", "Console"],
    },
    "project-settings-page": {
      toolbar: ["Project Settings", "Validate", "Import", "Export", "Apply"],
      layout: "table-editor",
      tableRows: [["Setting", "Value", "Scope", "State", ""], ["Project Name", "Sandbox", "Project", "Active", ""], ["Default Map", "A1_Hangar", "Runtime", "Active", ""], ["Renderer", "Forward+", "Render", "Active", ""], ["Cook Profile", "Windows Dev", "Build", "Active", ""]],
      panelTitle: "Environment",
      panelRows: [["Project", "Sandbox"], ["Root", "E:/Git/ZirconEngine"], ["Profile", "Development"], ["Dirty", "No"]],
      leftTop: { title: "Settings", tabs: ["Categories", "Recent"], body: () => navList(["General", "Runtime", "Rendering", "Assets", "Build"], 0) },
      rightTop: { title: "Validation", tabs: ["Checks", "Profiles"], body: () => renderMiniDataGrid([["Check", "Scope", "State", "Issue", ""], ["Paths", "Project", "Pass", "0", ""], ["Assets", "Content", "Pass", "0", ""], ["Build", "Profile", "Warn", "1", ""]]) },
      rightBottom: { title: "Setting Detail", tabs: ["Setting", "History"], body: () => renderAdditionalDetail("project-settings-detail") },
      bottomTitle: "Settings Output",
      bottomTabs: ["Validation", "Changes", "Console"],
    },
    "plugin-development": {
      toolbar: ["Plugin Dev", "Build", "Reload", "Validate", "Package"],
      layout: "graph",
      nodes: [["Manifest", 12, 26], ["Runtime Hook", 38, 28], ["Editor UI", 60, 42], ["Package", 80, 58]],
      panelTitle: "Plugin Session",
      panelRows: [["Plugin", "editor.tools.validation"], ["Modules", "3"], ["Hooks", "6"], ["Warnings", "1"]],
      leftTop: { title: "Plugins", tabs: ["Plugins", "Templates"], body: () => navList(["editor.tools.validation", "asset.importer", "runtime.overlay", "sample.plugin", "disabled.plugin"], 0) },
      rightTop: { title: "Extension Points", tabs: ["Hooks", "Modules"], body: () => renderMiniDataGrid([["Hook", "Owner", "State", "Type", ""], ["Menu", "Editor", "Ready", "UI", ""], ["Asset Import", "Tools", "Ready", "Runtime", ""], ["Panel", "Editor", "Warn", "UI", ""]]) },
      rightBottom: { title: "Plugin Detail", tabs: ["Manifest", "Build"], body: () => renderAdditionalDetail("plugin-development-detail") },
      bottomTitle: "Plugin Output",
      bottomTabs: ["Build", "Reload", "Console"],
    },
    "remote-device": {
      toolbar: ["Remote Device", "Connect", "Deploy", "Profile", "Capture"],
      layout: "metrics-graph",
      nodes: [["Editor", 12, 34], ["Package", 36, 28], ["Device", 62, 32], ["Telemetry", 80, 54]],
      metrics: [["3", "Devices"], ["1", "Active"], ["42", "ms ping"], ["92", "% battery"]],
      panelTitle: "Device Session",
      panelRows: [["Device", "Windows-DevKit-01"], ["Status", "Connected"], ["Ping", "42 ms"], ["Build", "Current"]],
      leftTop: { title: "Devices", tabs: ["Devices", "Groups"], body: () => navList(["Windows-DevKit-01", "Linux-Lab-02", "SteamDeck-Test", "Remote VM", "Offline"], 0) },
      rightTop: { title: "Deploy Queue", tabs: ["Queue", "Telemetry"], body: () => renderMiniDataGrid([["Step", "Device", "State", "Time", ""], ["Upload", "DevKit", "Done", "12s", ""], ["Install", "DevKit", "Running", "8s", ""], ["Launch", "DevKit", "Queued", "-", ""]]) },
      rightBottom: { title: "Device Detail", tabs: ["Device", "Deploy"], body: () => renderAdditionalDetail("remote-device-detail") },
      bottomTitle: "Device Output",
      bottomTabs: ["Deploy", "Telemetry", "Console"],
    },
    "session-sync": {
      toolbar: ["Session Sync", "Connect", "Resolve", "Broadcast", "Export"],
      layout: "graph",
      nodes: [["Host", 12, 32], ["Peer A", 38, 18], ["Peer B", 58, 42], ["Conflict", 78, 58]],
      panelTitle: "Sync State",
      panelRows: [["Peers", "3"], ["Conflicts", "1"], ["Latency", "38 ms"], ["Mode", "Live"]],
      leftTop: { title: "Peers", tabs: ["Peers", "Rooms"], body: () => navList(["Host", "JH-Editor", "BuildBot", "Viewer", "Disconnected"], 1) },
      rightTop: { title: "Replicated State", tabs: ["State", "Conflicts"], body: () => renderMiniDataGrid([["Item", "Owner", "State", "Conflict", ""], ["Scene Selection", "Host", "Synced", "No", ""], ["Layout", "JH", "Synced", "No", ""], ["Console Filter", "BuildBot", "Dirty", "Yes", ""]]) },
      rightBottom: { title: "Sync Detail", tabs: ["Conflict", "Peers"], body: () => renderAdditionalDetail("session-sync-detail") },
      bottomTitle: "Sync Output",
      bottomTabs: ["Events", "Conflicts", "Console"],
    },
    "cutscene-editor": {
      toolbar: ["Cutscene", "Preview", "Keys", "Render", "Export"],
      layout: "graph",
      nodes: [["Shot 010", 10, 18], ["Shot 020", 34, 28], ["Camera Cut", 58, 22], ["Event Cue", 58, 58], ["Render", 78, 40]],
      panelTitle: "Shot Summary",
      panelRows: [["Sequence", "Intro_Hangar"], ["Shot", "020"], ["Duration", "6.4 s"], ["Tracks", "9"]],
      leftTop: { title: "Shots", tabs: ["Shots", "Takes"], body: () => navList(["Shot 010", "Shot 020", "Shot 030", "Alt Take 02", "Render Queue"], 1) },
      rightTop: { title: "Track Tree", tabs: ["Tracks", "Bindings"], body: () => treeList([["Intro_Hangar", 0, true], ["Camera Cuts", 1, false], ["Characters", 1, false], ["Audio", 1, false], ["Events", 1, false]]) },
      rightBottom: { title: "Shot Detail", tabs: ["Shot", "Camera"], body: () => renderAdditionalDetail("cutscene-detail") },
      bottomTitle: "Cutscene Timeline",
      bottomTabs: ["Timeline", "Curves", "Render"],
    },
    "dialogue-editor": {
      toolbar: ["Dialogue", "Preview", "Branch", "Localize", "Export"],
      layout: "table-editor",
      tableRows: [["Line", "Speaker", "Text", "State", ""], ["L_001", "Captain", "Hold position.", "Done", ""], ["L_002", "Engineer", "Power is unstable.", "Review", ""], ["L_003", "Captain", "Route it manually.", "Done", ""], ["L_004", "System", "Access denied.", "Missing", ""]],
      panelTitle: "Line Summary",
      panelRows: [["Lines", "42"], ["Localized", "39"], ["Branches", "6"], ["Missing", "1"]],
      leftTop: { title: "Speakers", tabs: ["Speakers", "Scenes"], body: () => navList(["Captain", "Engineer", "System", "Pilot", "Hangar_Intro"], 0) },
      rightTop: { title: "Dialogue Flow", tabs: ["Flow", "Locales"], body: () => renderGraphCanvas([["Start", 12, 26], ["Captain", 36, 20], ["Engineer", 56, 42], ["Choice", 76, 58]]) },
      rightBottom: { title: "Line Detail", tabs: ["Line", "Locale"], body: () => renderAdditionalDetail("dialogue-detail") },
      bottomTitle: "Dialogue Output",
      bottomTabs: ["Validation", "Localization", "Console"],
    },
    "quest-editor": {
      toolbar: ["Quest", "Objectives", "Conditions", "Validate", "Save"],
      layout: "graph",
      nodes: [["Start", 10, 22], ["Find Fuse", 34, 18], ["Restore Power", 58, 38], ["Open Door", 78, 58], ["Fail State", 36, 66]],
      panelTitle: "Quest State",
      panelRows: [["Quest", "Restore Power"], ["Objectives", "6"], ["Branches", "2"], ["Errors", "0"]],
      leftTop: { title: "Quests", tabs: ["Quests", "Chains"], body: () => navList(["Restore Power", "Escape Dock", "Find Captain", "Optional Cache", "Tutorial"], 0) },
      rightTop: { title: "Objectives", tabs: ["Objectives", "Rewards"], body: () => renderMiniDataGrid([["Objective", "State", "Owner", "Gate", ""], ["Find Fuse", "Ready", "Player", "Open", ""], ["Restore Power", "Active", "World", "Pass", ""], ["Open Door", "Locked", "World", "Wait", ""]]) },
      rightBottom: { title: "Condition Detail", tabs: ["Condition", "Reward"], body: () => renderAdditionalDetail("quest-detail") },
      bottomTitle: "Quest Output",
      bottomTabs: ["Validation", "Simulation", "Console"],
    },
    "camera-rig": {
      toolbar: ["Camera Rig", "Frame", "Dolly", "Lens", "Preview"],
      layout: "viewport-tools",
      panelTitle: "Camera Lens",
      panelRows: [["Rig", "Crane_A"], ["Lens", "35 mm"], ["Focus", "4.2 m"], ["Stabilize", "On"]],
      leftTop: { title: "Rigs", tabs: ["Rigs", "Presets"], body: () => navList(["Crane_A", "Handheld", "Rail_Dolly", "Shoulder", "Static_Cam"], 0) },
      rightTop: { title: "Camera Stack", tabs: ["Stack", "Constraints"], body: () => renderMiniDataGrid([["Layer", "Mode", "Weight", "State", ""], ["Base", "Rig", "1.00", "Active", ""], ["Noise", "Handheld", "0.18", "Active", ""], ["LookAt", "Target", "0.72", "Active", ""]]) },
      rightBottom: { title: "Lens Detail", tabs: ["Lens", "Rig"], body: () => renderAdditionalDetail("camera-rig-detail") },
      bottomTitle: "Camera Output",
      bottomTabs: ["Preview", "Keys", "Console"],
    },
    "control-rig": {
      toolbar: ["Control Rig", "Solve", "Controls", "Bake", "Save"],
      layout: "graph",
      nodes: [["Root Ctrl", 12, 22], ["Spine IK", 36, 26], ["Arm FK", 60, 18], ["Foot IK", 58, 60], ["Output Pose", 78, 42]],
      panelTitle: "Rig Solve",
      panelRows: [["Rig", "SK_Guard"], ["Controls", "48"], ["Solve", "0.38 ms"], ["Warnings", "1"]],
      leftTop: { title: "Controls", tabs: ["Controls", "Spaces"], body: () => navList(["Root", "Spine IK", "Arm FK", "Foot IK", "Look At"], 1) },
      rightTop: { title: "Rig Hierarchy", tabs: ["Hierarchy", "Channels"], body: () => treeList([["ControlRig", 0, true], ["Root", 1, false], ["Spine", 1, false], ["Left Arm", 1, false], ["Leg IK", 1, false]]) },
      rightBottom: { title: "Solver Detail", tabs: ["Solver", "Channels"], body: () => renderAdditionalDetail("control-rig-detail") },
      bottomTitle: "Rig Output",
      bottomTabs: ["Solve", "Bake", "Console"],
    },
    "motion-matching": {
      toolbar: ["Motion Match", "Query", "Database", "Analyze", "Bake"],
      layout: "metrics-graph",
      nodes: [["Query", 12, 22], ["Pose 142", 36, 18], ["Pose 318", 58, 36], ["Blend", 78, 58]],
      metrics: [["842", "Poses"], ["18", "Features"], ["0.42", "Best cost"], ["5.6", "MB db"]],
      panelTitle: "Query Result",
      panelRows: [["Database", "Locomotion_DB"], ["Best Pose", "142"], ["Cost", "0.42"], ["Clip", "Run_Start"]],
      leftTop: { title: "Databases", tabs: ["Databases", "Clips"], body: () => navList(["Locomotion_DB", "Combat_DB", "Turn_In_Place", "JumpSet", "Debug Sample"], 0) },
      rightTop: { title: "Feature Channels", tabs: ["Features", "Costs"], body: () => renderMiniDataGrid([["Feature", "Weight", "Value", "Cost", ""], ["Trajectory", "0.60", "Match", "0.18", ""], ["Velocity", "0.25", "Match", "0.12", ""], ["Pose", "0.15", "Near", "0.42", ""]]) },
      rightBottom: { title: "Pose Detail", tabs: ["Pose", "Clip"], body: () => renderAdditionalDetail("motion-matching-detail") },
      bottomTitle: "Motion Output",
      bottomTabs: ["Analysis", "Queries", "Console"],
    },
    "facial-animation": {
      toolbar: ["Facial", "Solve", "Curves", "Phonemes", "Export"],
      layout: "table-editor",
      tableRows: [["Time", "Phoneme", "Curve", "Value", ""], ["00:00.12", "AA", "JawOpen", "0.62", ""], ["00:00.24", "M", "LipPress", "0.48", ""], ["00:00.36", "EH", "MouthWide", "0.31", ""], ["00:00.48", "Blink", "EyeBlink_L", "0.74", ""]],
      panelTitle: "Solve Summary",
      panelRows: [["Line", "Captain_03"], ["Curves", "52"], ["Phonemes", "18"], ["Quality", "92%"]],
      leftTop: { title: "Expressions", tabs: ["Expressions", "Clips"], body: () => navList(["Neutral", "Talk", "Alert", "Blink", "Smile"], 1) },
      rightTop: { title: "Curve Groups", tabs: ["Curves", "Weights"], body: () => renderMiniDataGrid([["Curve", "Group", "Value", "State", ""], ["JawOpen", "Mouth", "0.62", "Active", ""], ["BrowUp", "Brow", "0.18", "Idle", ""], ["EyeBlink_L", "Eye", "0.74", "Active", ""]]) },
      rightBottom: { title: "Expression Detail", tabs: ["Expression", "Solve"], body: () => renderAdditionalDetail("facial-animation-detail") },
      bottomTitle: "Facial Output",
      bottomTabs: ["Solve", "Curves", "Console"],
    },
    "blend-space": {
      toolbar: ["Blend Space", "Preview", "Samples", "Axes", "Save"],
      layout: "graph",
      nodes: [["Idle", 12, 58], ["Walk", 38, 42], ["Run", 66, 24], ["Strafe L", 34, 70], ["Strafe R", 70, 70]],
      panelTitle: "Blend Preview",
      panelRows: [["Asset", "BS_Locomotion_2D"], ["Speed", "3.2"], ["Direction", "-12 deg"], ["Samples", "9"]],
      leftTop: { title: "Samples", tabs: ["Samples", "Animations"], body: () => navList(["Idle", "Walk_Fwd", "Run_Fwd", "Strafe_L", "Strafe_R"], 2) },
      rightTop: { title: "Axes", tabs: ["Axes", "Grid"], body: () => renderMiniDataGrid([["Axis", "Min", "Max", "Value", ""], ["Speed", "0", "6", "3.2", ""], ["Direction", "-180", "180", "-12", ""], ["Weight", "0", "1", "0.84", ""]]) },
      rightBottom: { title: "Sample Detail", tabs: ["Sample", "Blend"], body: () => renderAdditionalDetail("blend-space-detail") },
      bottomTitle: "Blend Output",
      bottomTabs: ["Preview", "Samples", "Console"],
    },
    "foliage-editor": {
      toolbar: ["Foliage", "Paint", "Erase", "Density", "Bake"],
      layout: "viewport-tools",
      panelTitle: "Brush Density",
      panelRows: [["Brush", "Meadow Fill"], ["Radius", "12 m"], ["Density", "0.68"], ["Instances", "18k"]],
      leftTop: { title: "Brushes", tabs: ["Brushes", "Meshes"], body: () => navList(["Meadow Fill", "Rock Edge", "Dry Grass", "Pine Cluster", "Erase Soft"], 0) },
      rightTop: { title: "Foliage Layers", tabs: ["Layers", "Stats"], body: () => renderMiniDataGrid([["Layer", "Mesh", "Count", "State", ""], ["Grass_A", "Blade", "12k", "Painted", ""], ["Shrub_B", "Bush", "2.4k", "Painted", ""], ["RockMoss", "Decal", "412", "Sparse", ""]]) },
      rightBottom: { title: "Instance Detail", tabs: ["Brush", "Instance"], body: () => renderAdditionalDetail("foliage-detail") },
      bottomTitle: "Foliage Output",
      bottomTabs: ["Paint", "Density", "Console"],
    },
    "scatter-editor": {
      toolbar: ["Scatter", "Preview", "Rules", "Randomize", "Apply"],
      layout: "graph",
      nodes: [["Surface", 10, 24], ["Slope Rule", 34, 18], ["Density", 54, 40], ["Avoid Path", 74, 26], ["Output", 76, 62]],
      panelTitle: "Scatter Result",
      panelRows: [["Rule Set", "RockField_A"], ["Rules", "42"], ["Candidates", "8.4k"], ["Accepted", "3.2k"]],
      leftTop: { title: "Rules", tabs: ["Rules", "Presets"], body: () => navList(["Slope Limit", "Height Band", "Avoid Path", "Cluster Noise", "Biome Mask"], 1) },
      rightTop: { title: "Placement Stats", tabs: ["Stats", "Masks"], body: () => renderMiniDataGrid([["Rule", "Pass", "Reject", "Weight", ""], ["Slope", "6.1k", "2.3k", "0.70", ""], ["Height", "5.8k", "1.8k", "0.42", ""], ["Mask", "3.2k", "2.6k", "0.88", ""]]) },
      rightBottom: { title: "Rule Detail", tabs: ["Rule", "Preview"], body: () => renderAdditionalDetail("scatter-detail") },
      bottomTitle: "Scatter Output",
      bottomTabs: ["Validation", "Preview", "Console"],
    },
    "volume-editor": {
      toolbar: ["Volume", "Bounds", "Overlap", "Preview", "Apply"],
      layout: "viewport-tools",
      panelTitle: "Volume Bounds",
      panelRows: [["Volume", "FogZone_A"], ["Type", "Fog"], ["Shape", "Box"], ["Overlaps", "6"]],
      leftTop: { title: "Volumes", tabs: ["Volumes", "Types"], body: () => navList(["FogZone_A", "Audio_Reverb", "KillBox", "Light Probe", "Trigger_Door"], 0) },
      rightTop: { title: "Overlap List", tabs: ["Overlaps", "Rules"], body: () => renderMiniDataGrid([["Actor", "Type", "State", "Priority", ""], ["PlayerStart", "Spawn", "Inside", "High", ""], ["Crate_07", "Prop", "Inside", "Low", ""], ["AudioZone", "Sound", "Touch", "Med", ""]]) },
      rightBottom: { title: "Volume Detail", tabs: ["Bounds", "Rules"], body: () => renderAdditionalDetail("volume-detail") },
      bottomTitle: "Volume Output",
      bottomTabs: ["Overlap", "Warnings", "Console"],
    },
    "weather-editor": {
      toolbar: ["Weather", "Simulate", "Sky", "Wind", "Bake"],
      layout: "metrics-graph",
      nodes: [["Sky", 12, 22], ["Clouds", 34, 18], ["Rain", 56, 38], ["Wind", 76, 58]],
      metrics: [["38", "% rain"], ["12", "m/s wind"], ["0.72", "clouds"], ["4.1", "km view"]],
      panelTitle: "Weather Preset",
      panelRows: [["Preset", "StormFront_02"], ["Rain", "38%"], ["Wind", "12 m/s"], ["Clouds", "0.72"]],
      leftTop: { title: "Presets", tabs: ["Presets", "Regions"], body: () => navList(["Clear Noon", "StormFront_02", "Dust Wind", "Night Fog", "Interior Damp"], 1) },
      rightTop: { title: "Atmosphere Layers", tabs: ["Layers", "Curves"], body: () => renderMiniDataGrid([["Layer", "Value", "Blend", "State", ""], ["Clouds", "0.72", "0.4", "Active", ""], ["Rain", "0.38", "0.2", "Active", ""], ["Fog", "0.16", "0.1", "Idle", ""]]) },
      rightBottom: { title: "Preset Detail", tabs: ["Preset", "Curves"], body: () => renderAdditionalDetail("weather-detail") },
      bottomTitle: "Weather Output",
      bottomTabs: ["Simulation", "Curves", "Console"],
    },
    "post-process": {
      toolbar: ["Post Process", "Preview", "Stack", "LUT", "Apply"],
      layout: "metrics-graph",
      nodes: [["Scene Color", 12, 24], ["Exposure", 34, 18], ["Bloom", 56, 36], ["Color Grade", 76, 58]],
      metrics: [["Filmic", "Tone"], ["1.2", "Exposure"], ["0.18", "Bloom"], ["LUT", "Grade"]],
      panelTitle: "Look Summary",
      panelRows: [["Look", "Hangar_Night"], ["Tone", "Filmic"], ["LUT", "Cool_01"], ["Cost", "0.24 ms"]],
      leftTop: { title: "Looks", tabs: ["Looks", "Volumes"], body: () => navList(["Hangar_Night", "Hangar_Day", "Menu_Preview", "Cinematic", "High Contrast"], 0) },
      rightTop: { title: "Effect Stack", tabs: ["Stack", "Volumes"], body: () => renderMiniDataGrid([["Effect", "Value", "Enabled", "Cost", ""], ["Exposure", "1.2", "Yes", "0.03", ""], ["Bloom", "0.18", "Yes", "0.06", ""], ["Color Grade", "LUT", "Yes", "0.12", ""]]) },
      rightBottom: { title: "Pass Detail", tabs: ["Pass", "LUT"], body: () => renderAdditionalDetail("post-process-detail") },
      bottomTitle: "Post Output",
      bottomTabs: ["Preview", "Validation", "Console"],
    },
    "particle-library": {
      toolbar: ["Particle Library", "Preview", "Tag", "Simulate", "Export"],
      layout: "metrics-graph",
      nodes: [["Emitter", 12, 24], ["Module", 36, 18], ["Material", 58, 38], ["Library", 78, 56]],
      metrics: [["64", "Emitters"], ["18", "Tags"], ["1.8k", "Particles"], ["0", "Errors"]],
      panelTitle: "Emitter Preview",
      panelRows: [["Emitter", "P_Sparks_Library"], ["Particles", "1.8k"], ["Tags", "Metal, Impact"], ["Status", "Ready"]],
      leftTop: { title: "Particles", tabs: ["Emitters", "Tags"], body: () => navList(["P_Sparks_Library", "P_Dust_Puff", "P_Steam_Loop", "P_Energy_Hit", "P_Rain_Splash"], 0) },
      rightTop: { title: "Emitter Modules", tabs: ["Modules", "Usage"], body: () => renderMiniDataGrid([["Module", "Type", "State", "Cost", ""], ["Spawn Burst", "Spawn", "On", "Low", ""], ["Velocity", "Update", "On", "Low", ""], ["Sprite Render", "Render", "On", "Med", ""]]) },
      rightBottom: { title: "Emitter Detail", tabs: ["Emitter", "Usage"], body: () => renderAdditionalDetail("particle-library-detail") },
      bottomTitle: "Particle Output",
      bottomTabs: ["Simulation", "Usage", "Console"],
    },
    "collision-proxy": {
      toolbar: ["Collision Proxy", "Analyze", "Generate", "Bake", "Apply"],
      layout: "metrics-graph",
      nodes: [["Mesh", 12, 22], ["Hull", 34, 18], ["Proxy", 58, 38], ["Bake", 78, 58]],
      metrics: [["12", "Proxies"], ["184", "Hulls"], ["0.42", "ms"], ["2", "Warnings"]],
      panelTitle: "Proxy Build",
      panelRows: [["Set", "Hangar_Props"], ["Mode", "Convex"], ["Proxies", "12"], ["Warnings", "2"]],
      leftTop: { title: "Meshes", tabs: ["Meshes", "Profiles"], body: () => navList(["Crate_Set", "PipeWall", "DoorFrame", "Railings", "Hangar_Props"], 4) },
      rightTop: { title: "Proxy Hulls", tabs: ["Hulls", "Errors"], body: () => renderMiniDataGrid([["Mesh", "Hulls", "Verts", "State", ""], ["Crate_Set", "8", "96", "Ready", ""], ["PipeWall", "12", "184", "Warn", ""], ["Railings", "16", "220", "Ready", ""]]) },
      rightBottom: { title: "Proxy Detail", tabs: ["Proxy", "Bake"], body: () => renderAdditionalDetail("collision-proxy-detail") },
      bottomTitle: "Collision Output",
      bottomTabs: ["Analyze", "Bake", "Console"],
    },
    "level-variant": {
      toolbar: ["Level Variant", "Preview", "Diff", "Apply", "Export"],
      layout: "table-editor",
      tableRows: [["Actor", "Property", "Base", "Variant", ""], ["SkyLight", "Intensity", "1.0", "0.35", ""], ["FogZone_A", "Density", "0.12", "0.28", ""], ["Door_A", "State", "Open", "Closed", ""], ["Music", "Cue", "Day", "Night", ""]],
      panelTitle: "Variant Set",
      panelRows: [["Set", "Hangar_DayNight"], ["Active", "Night"], ["Changes", "18"], ["Conflicts", "0"]],
      leftTop: { title: "Variants", tabs: ["Variants", "Sets"], body: () => navList(["Day", "Night", "Alarm", "Destroyed", "Cinematic"], 1) },
      rightTop: { title: "Changed Actors", tabs: ["Actors", "Conflicts"], body: () => renderMiniDataGrid([["Actor", "Props", "State", "Owner", ""], ["SkyLight", "2", "Changed", "Lighting", ""], ["FogZone_A", "3", "Changed", "World", ""], ["Music", "1", "Changed", "Audio", ""]]) },
      rightBottom: { title: "Variant Detail", tabs: ["Variant", "Diff"], body: () => renderAdditionalDetail("level-variant-detail") },
      bottomTitle: "Variant Output",
      bottomTabs: ["Diff", "Preview", "Console"],
    },
    "gameplay-ability": {
      toolbar: ["Ability", "Activate", "Cooldown", "Trace", "Save"],
      layout: "graph",
      nodes: [["Input", 10, 24], ["Cost Check", 34, 18], ["Dash", 56, 38], ["Hit Window", 76, 26], ["Cooldown", 76, 62]],
      panelTitle: "Ability Runtime",
      panelRows: [["Ability", "DashStrike"], ["Cost", "25 stamina"], ["Cooldown", "4.5 s"], ["State", "Ready"]],
      leftTop: { title: "Abilities", tabs: ["Abilities", "Groups"], body: () => navList(["DashStrike", "Block", "Overload", "Heal Pulse", "Interact"], 0) },
      rightTop: { title: "Activation Rules", tabs: ["Rules", "Costs"], body: () => renderMiniDataGrid([["Rule", "Type", "State", "Value", ""], ["Stamina", "Cost", "Pass", "25", ""], ["Cooldown", "Time", "Ready", "0", ""], ["Tag Block", "Tag", "Pass", "None", ""]]) },
      rightBottom: { title: "Ability Detail", tabs: ["Ability", "Trace"], body: () => renderAdditionalDetail("gameplay-ability-detail") },
      bottomTitle: "Ability Output",
      bottomTabs: ["Simulation", "Trace", "Console"],
    },
    "gameplay-effect": {
      toolbar: ["Effect", "Apply", "Stacking", "Duration", "Validate"],
      layout: "table-editor",
      tableRows: [["Modifier", "Attribute", "Op", "Value", ""], ["Burn_DOT", "Health", "Add", "-8/s", ""], ["Heat", "Temperature", "Add", "12", ""], ["Panic", "Morale", "Mult", "0.9", ""], ["VFX", "Tag", "Grant", "State.Burning", ""]],
      panelTitle: "Effect Summary",
      panelRows: [["Effect", "Burning"], ["Duration", "6 s"], ["Stacks", "3"], ["Modifiers", "8"]],
      leftTop: { title: "Effects", tabs: ["Effects", "Attributes"], body: () => navList(["Burning", "Shielded", "Slowed", "Regeneration", "Overheated"], 0) },
      rightTop: { title: "Stacking Rules", tabs: ["Rules", "Tags"], body: () => renderMiniDataGrid([["Rule", "Policy", "Limit", "State", ""], ["Stack", "Aggregate", "3", "Active", ""], ["Refresh", "Duration", "6 s", "Active", ""], ["Overflow", "Ignore", "-", "Ready", ""]]) },
      rightBottom: { title: "Effect Detail", tabs: ["Effect", "Modifiers"], body: () => renderAdditionalDetail("gameplay-effect-detail") },
      bottomTitle: "Effect Output",
      bottomTabs: ["Validation", "Apply", "Console"],
    },
    "ai-perception": {
      toolbar: ["Perception", "Trace", "Sensors", "Stimuli", "Debug"],
      layout: "metrics-graph",
      nodes: [["Sight", 12, 18], ["Sound", 34, 42], ["Agent", 58, 28], ["Memory", 78, 58]],
      metrics: [["12", "Stimuli"], ["42", "m sight"], ["0.18", "ms"], ["3", "Targets"]],
      panelTitle: "Agent Perception",
      panelRows: [["Agent", "Guard_01"], ["Stimuli", "12"], ["Target", "Player"], ["State", "Alert"]],
      leftTop: { title: "Sensors", tabs: ["Sensors", "Agents"], body: () => navList(["Sight", "Hearing", "Damage", "Team", "Memory"], 0) },
      rightTop: { title: "Stimuli", tabs: ["Stimuli", "Targets"], body: () => renderMiniDataGrid([["Stimulus", "Source", "Age", "Strength", ""], ["Footstep", "Player", "0.4s", "0.82", ""], ["Door", "World", "1.2s", "0.42", ""], ["Damage", "Player", "0.1s", "1.00", ""]]) },
      rightBottom: { title: "Agent Detail", tabs: ["Agent", "Memory"], body: () => renderAdditionalDetail("ai-perception-detail") },
      bottomTitle: "Perception Output",
      bottomTabs: ["Trace", "Stimuli", "Console"],
    },
    "spawn-rules": {
      toolbar: ["Spawn Rules", "Preview", "Wave", "Validate", "Apply"],
      layout: "graph",
      nodes: [["Wave 01", 12, 22], ["Budget", 34, 18], ["Spawn Points", 56, 38], ["Director", 78, 58]],
      panelTitle: "Spawn Simulation",
      panelRows: [["Rule Set", "HangarWave_A"], ["Waves", "5"], ["Budget", "120"], ["Valid", "Yes"]],
      leftTop: { title: "Rules", tabs: ["Rules", "Waves"], body: () => navList(["HangarWave_A", "PatrolSpawn", "AmbientBots", "BossIntro", "Fallback"], 0) },
      rightTop: { title: "Wave Table", tabs: ["Waves", "Points"], body: () => renderMiniDataGrid([["Wave", "Actor", "Count", "Budget", ""], ["01", "Drone", "4", "20", ""], ["02", "Guard", "3", "45", ""], ["03", "Heavy", "1", "55", ""]]) },
      rightBottom: { title: "Rule Detail", tabs: ["Rule", "Budget"], body: () => renderAdditionalDetail("spawn-rules-detail") },
      bottomTitle: "Spawn Output",
      bottomTabs: ["Simulation", "Validation", "Console"],
    },
    "gameplay-tags": {
      toolbar: ["Gameplay Tags", "Search", "Validate", "Rename", "Export"],
      layout: "table-editor",
      tableRows: [["Tag", "Owner", "Uses", "State", ""], ["Ability.Dash", "Gameplay", "12", "Active", ""], ["State.Burning", "Effects", "18", "Active", ""], ["AI.Alert", "AI", "9", "Active", ""], ["UI.Hidden", "UI", "4", "Active", ""]],
      panelTitle: "Tag Summary",
      panelRows: [["Tags", "286"], ["Roots", "18"], ["Conflicts", "0"], ["Unused", "14"]],
      leftTop: { title: "Tags", tabs: ["Tree", "Recent"], body: () => treeList([["Gameplay", 0, true], ["Ability", 1, false], ["State", 1, false], ["AI", 0, false], ["UI", 0, false]]) },
      rightTop: { title: "Usage", tabs: ["Usage", "Owners"], body: () => renderMiniDataGrid([["Owner", "Tags", "Missing", "Conflict", ""], ["Gameplay", "74", "0", "0", ""], ["AI", "36", "0", "0", ""], ["UI", "28", "0", "0", ""]]) },
      rightBottom: { title: "Tag Detail", tabs: ["Tag", "Usage"], body: () => renderAdditionalDetail("gameplay-tags-detail") },
      bottomTitle: "Tags Output",
      bottomTabs: ["Validation", "Rename", "Console"],
    },
    "save-data": {
      toolbar: ["Save Data", "Load", "Migrate", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Slot", "Schema", "Size", "State", ""], ["Slot_A", "v12", "1.8 MB", "Current", ""], ["Slot_B", "v11", "1.6 MB", "Needs Migration", ""], ["Autosave", "v12", "1.9 MB", "Valid", ""], ["Cloud", "v12", "2.0 MB", "Synced", ""]],
      panelTitle: "Save Slot",
      panelRows: [["Slot", "Slot_A"], ["Schema", "v12"], ["Size", "1.8 MB"], ["Dirty", "No"]],
      leftTop: { title: "Slots", tabs: ["Slots", "Schemas"], body: () => navList(["Slot_A", "Slot_B", "Autosave", "Cloud", "Broken Fixture"], 0) },
      rightTop: { title: "Schema Fields", tabs: ["Fields", "Migrations"], body: () => renderMiniDataGrid([["Field", "Type", "Version", "State", ""], ["Player", "Struct", "v12", "OK", ""], ["Inventory", "Array", "v11", "Migrated", ""], ["QuestFlags", "Map", "v12", "OK", ""]]) },
      rightBottom: { title: "Slot Detail", tabs: ["Slot", "Schema"], body: () => renderAdditionalDetail("save-data-detail") },
      bottomTitle: "Save Output",
      bottomTabs: ["Validation", "Migration", "Console"],
    },
    "world-state": {
      toolbar: ["World State", "Snapshot", "Diff", "Patch", "Export"],
      layout: "metrics-graph",
      nodes: [["Snapshot A", 12, 24], ["Flags", 34, 18], ["Runtime", 58, 38], ["Diff", 78, 58]],
      metrics: [["184", "Flags"], ["12", "Dirty"], ["0", "Conflicts"], ["3", "Scopes"]],
      panelTitle: "State Snapshot",
      panelRows: [["Snapshot", "Runtime"], ["Flags", "184"], ["Dirty", "12"], ["Conflicts", "0"]],
      leftTop: { title: "States", tabs: ["Scopes", "Snapshots"], body: () => navList(["Runtime", "Quest", "AI", "World", "Session"], 0) },
      rightTop: { title: "Dirty Flags", tabs: ["Flags", "Diff"], body: () => renderMiniDataGrid([["Flag", "Scope", "Value", "Dirty", ""], ["Power.On", "Quest", "True", "Yes", ""], ["Door.A.Open", "World", "False", "Yes", ""], ["Alarm.Level", "AI", "2", "Yes", ""]]) },
      rightBottom: { title: "Flag Detail", tabs: ["Flag", "Diff"], body: () => renderAdditionalDetail("world-state-detail") },
      bottomTitle: "World State Output",
      bottomTabs: ["Diff", "Patch", "Console"],
    },
    "telemetry-dashboard": {
      toolbar: ["Telemetry", "Capture", "Filter", "Markers", "Export"],
      layout: "metrics-graph",
      nodes: [["Events", 12, 22], ["Session", 34, 18], ["Counters", 58, 38], ["Export", 78, 58]],
      metrics: [["1.2k", "Events"], ["42", "Counters"], ["8", "Markers"], ["0.8", "MB/s"]],
      panelTitle: "Live Metrics",
      panelRows: [["Session", "Live"], ["Events", "1.2k"], ["Counters", "42"], ["Rate", "0.8 MB/s"]],
      leftTop: { title: "Dashboards", tabs: ["Dashboards", "Streams"], body: () => navList(["Live Session", "Gameplay", "AI", "Rendering", "Network"], 0) },
      rightTop: { title: "Event Stream", tabs: ["Events", "Counters"], body: () => renderMiniDataGrid([["Time", "Channel", "Event", "Rate", ""], ["12:00", "Gameplay", "Ability", "42/s", ""], ["12:01", "AI", "Stimulus", "18/s", ""], ["12:02", "World", "State", "9/s", ""]]) },
      rightBottom: { title: "Metric Detail", tabs: ["Metric", "Trace"], body: () => renderAdditionalDetail("telemetry-dashboard-detail") },
      bottomTitle: "Telemetry Output",
      bottomTabs: ["Capture", "Markers", "Console"],
    },
    "lobby-editor": {
      toolbar: ["Lobby", "Create", "Invite", "Rules", "Start"],
      layout: "table-editor",
      tableRows: [["Member", "Role", "Ready", "Ping", ""], ["Player_042", "Host", "Yes", "42 ms", ""], ["Designer", "Guest", "Yes", "38 ms", ""], ["QA_Bot", "Bot", "No", "12 ms", ""], ["Spectator", "Viewer", "Yes", "64 ms", ""]],
      panelTitle: "Lobby State",
      panelRows: [["Room", "Dev Room"], ["Members", "4"], ["Ready", "3"], ["Privacy", "Friends"]],
      leftTop: { title: "Lobbies", tabs: ["Lobbies", "Templates"], body: () => navList(["Dev Room", "QA Session", "Ranked Test", "Private Invite", "Offline"], 0) },
      rightTop: { title: "Members", tabs: ["Members", "Permissions"], body: () => renderMiniDataGrid([["Member", "Role", "State", "Ping", ""], ["Player_042", "Host", "Ready", "42", ""], ["Designer", "Guest", "Ready", "38", ""], ["QA_Bot", "Bot", "Waiting", "12", ""]]) },
      rightBottom: { title: "Lobby Detail", tabs: ["Lobby", "Rules"], body: () => renderAdditionalDetail("lobby-detail") },
      bottomTitle: "Lobby Output",
      bottomTabs: ["Session", "Invites", "Console"],
    },
    "matchmaking-editor": {
      toolbar: ["Matchmaking", "Queue", "Simulate", "Balance", "Export"],
      layout: "metrics-graph",
      nodes: [["Tickets", 12, 22], ["Rules", 34, 18], ["Match", 58, 38], ["Backfill", 78, 58]],
      metrics: [["128", "Tickets"], ["42", "sec wait"], ["0.82", "Quality"], ["4", "Regions"]],
      panelTitle: "Queue Summary",
      panelRows: [["Queue", "Ranked_2v2"], ["Tickets", "128"], ["Avg Wait", "42 s"], ["Quality", "0.82"]],
      leftTop: { title: "Queues", tabs: ["Queues", "Pools"], body: () => navList(["Ranked_2v2", "Casual", "Coop", "Private", "Backfill"], 0) },
      rightTop: { title: "Rule Weights", tabs: ["Rules", "Pools"], body: () => renderMiniDataGrid([["Rule", "Weight", "State", "Range", ""], ["Skill", "0.60", "Active", "+/-120", ""], ["Latency", "0.25", "Active", "<80 ms", ""], ["Region", "0.15", "Active", "Asia", ""]]) },
      rightBottom: { title: "Ticket Detail", tabs: ["Ticket", "Rules"], body: () => renderAdditionalDetail("matchmaking-detail") },
      bottomTitle: "Matchmaker Output",
      bottomTabs: ["Simulation", "Tickets", "Console"],
    },
    "server-browser": {
      toolbar: ["Server Browser", "Refresh", "Filter", "Ping", "Join"],
      layout: "table-editor",
      tableRows: [["Server", "Region", "Players", "Ping", ""], ["asia-dev-01", "Asia", "12/32", "42 ms", ""], ["asia-qa-02", "Asia", "8/16", "58 ms", ""], ["us-west-01", "USW", "24/32", "132 ms", ""], ["eu-lab-04", "EU", "4/16", "168 ms", ""]],
      panelTitle: "Server Filter",
      panelRows: [["Region", "Asia"], ["Servers", "42"], ["Avg Ping", "64 ms"], ["Full", "8"]],
      leftTop: { title: "Regions", tabs: ["Regions", "Filters"], body: () => navList(["Asia", "US West", "US East", "Europe", "Local"], 0) },
      rightTop: { title: "Region Health", tabs: ["Health", "Capacity"], body: () => renderMiniDataGrid([["Region", "Online", "Capacity", "Health", ""], ["Asia", "42", "68%", "OK", ""], ["USW", "18", "72%", "OK", ""], ["EU", "21", "54%", "Warn", ""]]) },
      rightBottom: { title: "Server Detail", tabs: ["Server", "Rules"], body: () => renderAdditionalDetail("server-browser-detail") },
      bottomTitle: "Server Output",
      bottomTabs: ["Ping", "Refresh", "Console"],
    },
    "replay-browser": {
      toolbar: ["Replay", "Open", "Scrub", "Markers", "Export"],
      layout: "table-editor",
      tableRows: [["Replay", "Map", "Duration", "Markers", ""], ["Match_042", "A1_Hangar", "12:42", "12", ""], ["Match_041", "Valley_01", "18:05", "8", ""], ["Crash_Repro", "A1_Hangar", "03:18", "4", ""], ["Tutorial", "Dock", "06:22", "5", ""]],
      panelTitle: "Replay Summary",
      panelRows: [["Replay", "Match_042"], ["Duration", "12:42"], ["Markers", "12"], ["Size", "84 MB"]],
      leftTop: { title: "Replays", tabs: ["Replays", "Folders"], body: () => navList(["Match_042", "Match_041", "Crash_Repro", "Tutorial", "Autosaves"], 0) },
      rightTop: { title: "Timeline Markers", tabs: ["Markers", "Tracks"], body: () => renderMiniDataGrid([["Time", "Type", "Label", "State", ""], ["00:42", "Event", "First Contact", "Pinned", ""], ["04:18", "Kill", "Drone Down", "Pinned", ""], ["09:12", "Bug", "Desync", "Review", ""]]) },
      rightBottom: { title: "Replay Detail", tabs: ["Replay", "Markers"], body: () => renderAdditionalDetail("replay-detail") },
      bottomTitle: "Replay Output",
      bottomTabs: ["Playback", "Markers", "Console"],
    },
    "achievements-editor": {
      toolbar: ["Achievements", "Validate", "Publish", "Progress", "Export"],
      layout: "table-editor",
      tableRows: [["Achievement", "Points", "State", "Progress", ""], ["First Launch", "5", "Live", "100%", ""], ["No Damage Run", "30", "Draft", "12%", ""], ["Explorer", "15", "Live", "68%", ""], ["Speed Clear", "25", "Draft", "4%", ""]],
      panelTitle: "Achievement Set",
      panelRows: [["Definitions", "38"], ["Live", "36"], ["Drafts", "2"], ["Conflicts", "0"]],
      leftTop: { title: "Achievements", tabs: ["Sets", "Drafts"], body: () => navList(["Core", "Combat", "Exploration", "Season 01", "Drafts"], 0) },
      rightTop: { title: "Progress Stats", tabs: ["Stats", "Rewards"], body: () => renderMiniDataGrid([["Metric", "Value", "Trend", "State", ""], ["Unlock Rate", "62%", "+3%", "OK", ""], ["Drafts", "2", "0", "Open", ""], ["Conflicts", "0", "0", "OK", ""]]) },
      rightBottom: { title: "Achievement Detail", tabs: ["Definition", "Rewards"], body: () => renderAdditionalDetail("achievements-detail") },
      bottomTitle: "Achievements Output",
      bottomTabs: ["Validation", "Publish", "Console"],
    },
    "entitlements-editor": {
      toolbar: ["Entitlements", "Grant", "Revoke", "Audit", "Export"],
      layout: "table-editor",
      tableRows: [["Item", "Type", "Grant", "State", ""], ["FounderPack", "Bundle", "Owned", "Active", ""], ["Skin_Red", "Cosmetic", "Owned", "Active", ""], ["DLC_01", "DLC", "Missing", "Locked", ""], ["Currency_100", "Currency", "Granted", "Pending", ""]],
      panelTitle: "Catalog Summary",
      panelRows: [["Catalog", "Live"], ["Grants", "3"], ["Missing", "1"], ["Pending", "1"]],
      leftTop: { title: "Catalog", tabs: ["Catalog", "Users"], body: () => navList(["FounderPack", "Skins", "DLC", "Currency", "Test User"], 0) },
      rightTop: { title: "Ownership Checks", tabs: ["Ownership", "Audit"], body: () => renderMiniDataGrid([["Check", "User", "Result", "State", ""], ["FounderPack", "Player_042", "Owned", "Pass", ""], ["DLC_01", "Player_042", "Missing", "Lock", ""], ["Currency", "Player_042", "Pending", "Review", ""]]) },
      rightBottom: { title: "Entitlement Detail", tabs: ["Item", "Audit"], body: () => renderAdditionalDetail("entitlements-detail") },
      bottomTitle: "Entitlements Output",
      bottomTabs: ["Audit", "Grant", "Console"],
    },
    "user-profile-editor": {
      toolbar: ["User Profile", "Sync", "Friends", "Privacy", "Export"],
      layout: "table-editor",
      tableRows: [["Field", "Value", "Scope", "State", ""], ["Display Name", "Player_042", "Public", "Synced", ""], ["Presence", "Online", "Friends", "Synced", ""], ["Party", "Dev Room", "Friends", "Synced", ""], ["Region", "Asia", "Private", "Synced", ""]],
      panelTitle: "Profile Summary",
      panelRows: [["User", "Player_042"], ["Presence", "Online"], ["Friends", "18"], ["Privacy", "Friends"]],
      leftTop: { title: "Profiles", tabs: ["Profiles", "Friends"], body: () => navList(["Player_042", "Designer", "QA_Bot", "Spectator", "OfflineUser"], 0) },
      rightTop: { title: "Social State", tabs: ["Friends", "Privacy"], body: () => renderMiniDataGrid([["Name", "State", "Party", "Ping", ""], ["Designer", "Online", "Dev Room", "38", ""], ["QA_Bot", "Ready", "Dev Room", "12", ""], ["OfflineUser", "Offline", "-", "-", ""]]) },
      rightBottom: { title: "Profile Detail", tabs: ["Profile", "Privacy"], body: () => renderAdditionalDetail("user-profile-detail") },
      bottomTitle: "Profile Output",
      bottomTabs: ["Sync", "Friends", "Console"],
    },
    "online-diagnostics": {
      toolbar: ["Online Diagnostics", "Run", "Auth", "Sessions", "Export"],
      layout: "metrics-graph",
      nodes: [["Auth", 12, 22], ["Presence", 34, 18], ["Sessions", 58, 38], ["Backend", 78, 58]],
      metrics: [["OK", "Auth"], ["1", "Warn"], ["42", "ms"], ["99.9", "% uptime"]],
      panelTitle: "Service Health",
      panelRows: [["Auth", "OK"], ["Sessions", "OK"], ["Presence", "Warn"], ["Latency", "42 ms"]],
      leftTop: { title: "Services", tabs: ["Services", "Regions"], body: () => navList(["Auth", "Presence", "Sessions", "Inventory", "Telemetry"], 1) },
      rightTop: { title: "Checks", tabs: ["Checks", "Logs"], body: () => renderMiniDataGrid([["Service", "Check", "State", "Latency", ""], ["Auth", "Token", "Pass", "42", ""], ["Presence", "Heartbeat", "Warn", "88", ""], ["Sessions", "Join", "Pass", "56", ""]]) },
      rightBottom: { title: "Diagnostic Detail", tabs: ["Service", "Trace"], body: () => renderAdditionalDetail("online-diagnostics-detail") },
      bottomTitle: "Online Output",
      bottomTabs: ["Checks", "Trace", "Console"],
    },
    "hud-editor": {
      toolbar: ["HUD", "Preview", "Anchor", "States", "Export"],
      layout: "table-editor",
      tableRows: [["Widget", "Layer", "Anchor", "State", ""], ["HealthBar", "Gameplay", "TopLeft", "Bound", ""], ["AmmoCounter", "Gameplay", "BottomRight", "Bound", ""], ["DamagePulse", "Overlay", "Center", "Animated", ""], ["QuestToast", "Notifications", "TopRight", "Hidden", ""]],
      panelTitle: "HUD Summary",
      panelRows: [["Asset", "Combat_HUD"], ["Widgets", "18"], ["Breakpoints", "3"], ["Invalid", "0"]],
      leftTop: { title: "Widgets", tabs: ["Widgets", "Presets"], body: () => navList(["HealthBar", "AmmoCounter", "DamagePulse", "QuestToast", "Crosshair"], 0) },
      rightTop: { title: "Layers", tabs: ["Layers", "Anchors"], body: () => renderMiniDataGrid([["Layer", "Widgets", "Visible", "Input", ""], ["Gameplay", "8", "Yes", "None", ""], ["Overlay", "6", "Yes", "Pass", ""], ["Notifications", "4", "No", "None", ""]]) },
      rightBottom: { title: "Widget Detail", tabs: ["Widget", "State"], body: () => renderAdditionalDetail("hud-detail") },
      bottomTitle: "HUD Output",
      bottomTabs: ["Preview", "Bindings", "Console"],
    },
    "menu-flow": {
      toolbar: ["Menu Flow", "Preview", "Transition", "Validate", "Export"],
      layout: "graph",
      nodes: [["Boot", 10, 18], ["Main", 32, 22], ["Options", 54, 18], ["Save Slots", 42, 50], ["Credits", 74, 54]],
      panelTitle: "Flow Summary",
      panelRows: [["Screens", "9"], ["Transitions", "18"], ["Loops", "2"], ["Errors", "0"]],
      leftTop: { title: "Screens", tabs: ["Screens", "States"], body: () => navList(["Boot", "Main", "Options", "Save Slots", "Credits"], 1) },
      rightTop: { title: "Transitions", tabs: ["Routes", "Guards"], body: () => renderMiniDataGrid([["From", "To", "Input", "State", ""], ["Main", "Options", "Click", "OK", ""], ["Main", "Save Slots", "Click", "OK", ""], ["Options", "Main", "Back", "OK", ""]]) },
      rightBottom: { title: "Transition Detail", tabs: ["Route", "Motion"], body: () => renderAdditionalDetail("menu-flow-detail") },
      bottomTitle: "Menu Flow Output",
      bottomTabs: ["Validation", "Preview", "Console"],
    },
    "font-atlas": {
      toolbar: ["Font Atlas", "Bake", "Ranges", "Fallbacks", "Export"],
      layout: "metrics-graph",
      nodes: [["Latin", 12, 22], ["CJK", 34, 18], ["Icons", 58, 38], ["Fallback", 78, 58]],
      metrics: [["1248", "Glyphs"], ["4", "Ranges"], ["2", "Fallbacks"], ["0", "Missing"]],
      panelTitle: "Atlas Metrics",
      panelRows: [["Font", "Inter_UI"], ["Size", "2048"], ["Glyphs", "1248"], ["Coverage", "99.2%"]],
      leftTop: { title: "Fonts", tabs: ["Fonts", "Ranges"], body: () => navList(["Inter_UI", "NotoSans_CJK", "Mono_Debug", "Icons", "Fallback"], 0) },
      rightTop: { title: "Glyph Ranges", tabs: ["Ranges", "Packing"], body: () => renderMiniDataGrid([["Range", "Glyphs", "Used", "State", ""], ["Latin", "384", "Yes", "Packed", ""], ["CJK", "812", "Yes", "Packed", ""], ["Icons", "52", "Yes", "Packed", ""]]) },
      rightBottom: { title: "Glyph Detail", tabs: ["Glyph", "Fallback"], body: () => renderAdditionalDetail("font-atlas-detail") },
      bottomTitle: "Font Output",
      bottomTabs: ["Bake", "Coverage", "Console"],
    },
    "icon-library": {
      toolbar: ["Icon Library", "Import", "Tags", "Audit", "Export"],
      layout: "table-editor",
      tableRows: [["Icon", "Set", "Size", "Usage", ""], ["save", "Editor_Core", "24", "18", ""], ["folder-open", "Editor_Core", "24", "12", ""], ["play", "Runtime", "24", "22", ""], ["warning", "Status", "20", "34", ""]],
      panelTitle: "Icon Set",
      panelRows: [["Set", "Editor_Core"], ["Icons", "286"], ["Unused", "14"], ["Missing", "0"]],
      leftTop: { title: "Icons", tabs: ["Sets", "Tags"], body: () => navList(["Editor_Core", "Runtime", "Status", "Navigation", "Input"], 0) },
      rightTop: { title: "Usage", tabs: ["Usage", "Tags"], body: () => renderMiniDataGrid([["Icon", "Refs", "Context", "State", ""], ["save", "18", "Toolbar", "OK", ""], ["play", "22", "Runtime", "OK", ""], ["warning", "34", "Status", "OK", ""]]) },
      rightBottom: { title: "Icon Detail", tabs: ["Icon", "Export"], body: () => renderAdditionalDetail("icon-library-detail") },
      bottomTitle: "Icon Output",
      bottomTabs: ["Audit", "Export", "Console"],
    },
    "ui-binding-editor": {
      toolbar: ["UI Binding", "Validate", "Preview", "Trace", "Export"],
      layout: "table-editor",
      tableRows: [["Widget", "Property", "Source", "State", ""], ["ItemName", "text", "inventory.name", "Valid", ""], ["ItemIcon", "image", "inventory.icon", "Valid", ""], ["StackCount", "text", "inventory.count", "Valid", ""], ["EquipButton", "enabled", "inventory.canEquip", "Valid", ""]],
      panelTitle: "Binding Contract",
      panelRows: [["Asset", "inventory_panel"], ["Bindings", "24"], ["Sources", "7"], ["Errors", "0"]],
      leftTop: { title: "Bindings", tabs: ["Bindings", "Models"], body: () => navList(["inventory_panel", "hud_status", "quest_toast", "settings_menu", "debug_overlay"], 0) },
      rightTop: { title: "Data Sources", tabs: ["Sources", "Trace"], body: () => renderMiniDataGrid([["Source", "Type", "Users", "State", ""], ["inventory", "Struct", "12", "Valid", ""], ["player", "Struct", "8", "Valid", ""], ["settings", "Map", "4", "Valid", ""]]) },
      rightBottom: { title: "Binding Detail", tabs: ["Binding", "Trace"], body: () => renderAdditionalDetail("ui-binding-detail") },
      bottomTitle: "Binding Output",
      bottomTabs: ["Validation", "Trace", "Console"],
    },
    "accessibility-audit": {
      toolbar: ["Accessibility", "Run", "Contrast", "Focus", "Export"],
      layout: "table-editor",
      tableRows: [["Issue", "Element", "Rule", "Severity", ""], ["Low contrast", "SecondaryText", "AA", "Warn", ""], ["Missing label", "IconButton_12", "Name", "Error", ""], ["Focus skip", "SettingsTab", "Order", "Warn", ""], ["Tiny target", "CloseButton", "44px", "Warn", ""]],
      panelTitle: "Audit Summary",
      panelRows: [["Checks", "42"], ["Errors", "1"], ["Warnings", "3"], ["Passed", "38"]],
      leftTop: { title: "Audits", tabs: ["Audits", "Rules"], body: () => navList(["Main Menu", "Combat HUD", "Inventory", "Settings", "Pause Menu"], 2) },
      rightTop: { title: "Focus Order", tabs: ["Focus", "Contrast"], body: () => renderMiniDataGrid([["Step", "Element", "Rule", "State", ""], ["01", "StartButton", "Visible", "OK", ""], ["02", "Options", "Visible", "OK", ""], ["03", "IconButton_12", "Label", "Error", ""]]) },
      rightBottom: { title: "Issue Detail", tabs: ["Issue", "Fix"], body: () => renderAdditionalDetail("accessibility-detail") },
      bottomTitle: "Accessibility Output",
      bottomTabs: ["Audit", "Fixes", "Console"],
    },
    "input-prompts": {
      toolbar: ["Input Prompts", "Preview", "Device", "Locale", "Export"],
      layout: "table-editor",
      tableRows: [["Prompt", "Action", "Device", "Locale", ""], ["Press A", "Confirm", "Xbox", "en-US", ""], ["Hold X", "Reload", "Xbox", "en-US", ""], ["Tap E", "Interact", "Keyboard", "en-US", ""], ["R2", "Fire", "DualSense", "en-US", ""]],
      panelTitle: "Prompt Set",
      panelRows: [["Set", "Gamepad_Xbox"], ["Prompts", "64"], ["Devices", "4"], ["Locales", "8"]],
      leftTop: { title: "Prompts", tabs: ["Prompts", "Devices"], body: () => navList(["Gamepad_Xbox", "DualSense", "Keyboard", "SteamDeck", "Touch"], 0) },
      rightTop: { title: "Device Glyphs", tabs: ["Glyphs", "Fallbacks"], body: () => renderMiniDataGrid([["Glyph", "Action", "Device", "State", ""], ["A", "Confirm", "Xbox", "OK", ""], ["X", "Reload", "Xbox", "OK", ""], ["E", "Interact", "Keyboard", "OK", ""]]) },
      rightBottom: { title: "Prompt Detail", tabs: ["Prompt", "Locale"], body: () => renderAdditionalDetail("input-prompts-detail") },
      bottomTitle: "Prompts Output",
      bottomTabs: ["Localization", "Preview", "Console"],
    },
    "ui-motion": {
      toolbar: ["UI Motion", "Play", "Curves", "States", "Export"],
      layout: "graph",
      nodes: [["Hidden", 12, 28], ["Opening", 34, 18], ["Visible", 58, 38], ["Closing", 78, 58]],
      panelTitle: "Motion Clip",
      panelRows: [["Clip", "Panel_Open"], ["Duration", "180 ms"], ["Curves", "4"], ["Warnings", "0"]],
      leftTop: { title: "Motion", tabs: ["Clips", "States"], body: () => navList(["Panel_Open", "Panel_Close", "Toast_In", "Toast_Out", "Button_Press"], 0) },
      rightTop: { title: "Curves", tabs: ["Curves", "Events"], body: () => renderMiniDataGrid([["Curve", "Target", "Ease", "State", ""], ["Opacity", "alpha", "OutCubic", "OK", ""], ["Scale", "transform", "Spring", "OK", ""], ["Y", "offset", "OutQuad", "OK", ""]]) },
      rightBottom: { title: "Motion Detail", tabs: ["Clip", "Curve"], body: () => renderAdditionalDetail("ui-motion-detail") },
      bottomTitle: "Motion Output",
      bottomTabs: ["Timeline", "Events", "Console"],
    },
    "shader-permutations": {
      toolbar: ["Shader Permutations", "Compile", "Strip", "Compare", "Export"],
      layout: "table-editor",
      tableRows: [["Variant", "Keywords", "Stage", "State", ""], ["Metal_Lit_001", "NORMAL+SHADOW", "Fragment", "Compiled", ""], ["Metal_Lit_002", "NORMAL+FOG", "Fragment", "Compiled", ""], ["Metal_Debug", "DEBUG_VIEW", "Fragment", "Warning", ""], ["Metal_Depth", "DEPTH_ONLY", "Vertex", "Compiled", ""]],
      panelTitle: "Variant Summary",
      panelRows: [["Shader", "M_Metal"], ["Variants", "128"], ["Stripped", "42"], ["Warnings", "1"]],
      leftTop: { title: "Shaders", tabs: ["Shaders", "Keywords"], body: () => navList(["M_Metal", "M_Glass", "M_Concrete", "M_UI", "Debug_View"], 0) },
      rightTop: { title: "Keyword Matrix", tabs: ["Keywords", "Stages"], body: () => renderMiniDataGrid([["Keyword", "Variants", "Used", "State", ""], ["NORMAL", "64", "Yes", "OK", ""], ["SHADOW", "48", "Yes", "OK", ""], ["DEBUG_VIEW", "4", "Dev", "Warn", ""]]) },
      rightBottom: { title: "Permutation Detail", tabs: ["Variant", "Compiler"], body: () => renderAdditionalDetail("shader-permutations-detail") },
      bottomTitle: "Shader Output",
      bottomTabs: ["Compiler", "Variants", "Console"],
    },
    "render-targets": {
      toolbar: ["Render Target", "Inspect", "Clear", "Capture", "Export"],
      layout: "table-editor",
      tableRows: [["Target", "Format", "Size", "State", ""], ["HDR_Main", "RGBA16F", "1920x1080", "Bound", ""], ["Depth_Main", "D32", "1920x1080", "Bound", ""], ["Bloom_Chain", "RGBA16F", "960x540", "Ready", ""], ["UI_Final", "RGBA8", "1920x1080", "Ready", ""]],
      panelTitle: "Target Summary",
      panelRows: [["Target", "HDR_Main"], ["Attachments", "4"], ["Memory", "42 MB"], ["Samples", "1x"]],
      leftTop: { title: "Targets", tabs: ["Targets", "History"], body: () => navList(["HDR_Main", "Depth_Main", "Bloom_Chain", "UI_Final", "ShadowAtlas"], 0) },
      rightTop: { title: "Attachments", tabs: ["Attachments", "Barriers"], body: () => renderMiniDataGrid([["Slot", "Format", "Usage", "State", ""], ["Color0", "RGBA16F", "Render", "Bound", ""], ["Depth", "D32", "Depth", "Bound", ""], ["Resolve", "RGBA8", "Sample", "Ready", ""]]) },
      rightBottom: { title: "Target Detail", tabs: ["Target", "Memory"], body: () => renderAdditionalDetail("render-target-detail") },
      bottomTitle: "Target Output",
      bottomTabs: ["Capture", "Memory", "Console"],
    },
    "gpu-profiler": {
      toolbar: ["GPU Profiler", "Capture", "Markers", "Compare", "Export"],
      layout: "metrics-graph",
      nodes: [["Depth", 12, 26], ["GBuffer", 34, 18], ["Lighting", 58, 38], ["Post", 78, 58]],
      metrics: [["12.8", "ms"], ["128", "draws"], ["42", "passes"], ["2.1", "GB/s"]],
      panelTitle: "Frame Summary",
      panelRows: [["Frame", "1842"], ["GPU", "12.8 ms"], ["Draws", "128"], ["Bubbles", "2"]],
      leftTop: { title: "Frames", tabs: ["Frames", "Captures"], body: () => navList(["Frame 1842", "Frame 1841", "Frame 1840", "Spike 1838", "Baseline"], 0) },
      rightTop: { title: "Pass Timings", tabs: ["Passes", "Markers"], body: () => renderMiniDataGrid([["Pass", "Time", "Draws", "State", ""], ["Depth", "1.2 ms", "18", "OK", ""], ["Lighting", "4.8 ms", "42", "Hot", ""], ["Post", "2.1 ms", "16", "OK", ""]]) },
      rightBottom: { title: "Pass Detail", tabs: ["Pass", "Resources"], body: () => renderAdditionalDetail("gpu-profiler-detail") },
      bottomTitle: "GPU Output",
      bottomTabs: ["Capture", "Markers", "Console"],
    },
    "light-probes": {
      toolbar: ["Light Probes", "Place", "Bake", "Validate", "Export"],
      layout: "metrics-graph",
      nodes: [["Probe A", 12, 24], ["Probe B", 34, 18], ["Probe C", 58, 38], ["Probe D", 78, 58]],
      metrics: [["64", "Probes"], ["4", "Volumes"], ["92%", "Coverage"], ["0", "Leaks"]],
      panelTitle: "Probe Grid",
      panelRows: [["Set", "Hangar_ProbeGrid"], ["Probes", "64"], ["Spacing", "4 m"], ["Coverage", "92%"]],
      leftTop: { title: "Probe Sets", tabs: ["Sets", "Volumes"], body: () => navList(["Hangar_ProbeGrid", "Dock_Interior", "Valley_North", "Cave_Entry", "TestGrid"], 0) },
      rightTop: { title: "Coverage", tabs: ["Coverage", "Bake"], body: () => renderMiniDataGrid([["Volume", "Probes", "Coverage", "State", ""], ["Hangar", "64", "92%", "OK", ""], ["Dock", "32", "84%", "Warn", ""], ["Cave", "28", "78%", "Warn", ""]]) },
      rightBottom: { title: "Probe Detail", tabs: ["Probe", "Bake"], body: () => renderAdditionalDetail("light-probes-detail") },
      bottomTitle: "Probe Output",
      bottomTabs: ["Bake", "Validation", "Console"],
    },
    "reflection-capture": {
      toolbar: ["Reflection Capture", "Capture", "Blend", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Capture", "Shape", "Resolution", "State", ""], ["Hangar_Cubemap", "Box", "512", "Captured", ""], ["Dock_Probe", "Sphere", "256", "Ready", ""], ["Corridor_A", "Box", "256", "Stale", ""], ["ExteriorSky", "Sphere", "1024", "Captured", ""]],
      panelTitle: "Capture Summary",
      panelRows: [["Capture", "Hangar_Cubemap"], ["Faces", "6"], ["Resolution", "512"], ["Stale", "1"]],
      leftTop: { title: "Captures", tabs: ["Captures", "Volumes"], body: () => navList(["Hangar_Cubemap", "Dock_Probe", "Corridor_A", "ExteriorSky", "DebugRoom"], 0) },
      rightTop: { title: "Cubemap Faces", tabs: ["Faces", "Blend"], body: () => renderMiniDataGrid([["Face", "Exposure", "State", "Cost", ""], ["+X", "0.0", "Ready", "1.2 MB", ""], ["-X", "0.0", "Ready", "1.2 MB", ""], ["+Y", "0.1", "Ready", "1.2 MB", ""]]) },
      rightBottom: { title: "Capture Detail", tabs: ["Capture", "Blend"], body: () => renderAdditionalDetail("reflection-capture-detail") },
      bottomTitle: "Reflection Output",
      bottomTabs: ["Capture", "Validation", "Console"],
    },
    "decal-editor": {
      toolbar: ["Decal", "Place", "Project", "Sort", "Export"],
      layout: "table-editor",
      tableRows: [["Decal", "Material", "Sort", "State", ""], ["WarningStripe", "M_Decal_Warn", "12", "Ready", ""], ["LeakMark", "M_Decal_Oil", "8", "Ready", ""], ["Scratch_A", "M_Decal_Scratch", "4", "Ready", ""], ["Signage_01", "M_Decal_Text", "16", "Warning", ""]],
      panelTitle: "Decal Preset",
      panelRows: [["Preset", "WarningStripe"], ["Placements", "12"], ["Sort", "12"], ["Warnings", "1"]],
      leftTop: { title: "Decals", tabs: ["Decals", "Presets"], body: () => navList(["WarningStripe", "LeakMark", "Scratch_A", "Signage_01", "PaintMark"], 0) },
      rightTop: { title: "Projection", tabs: ["Projection", "Usage"], body: () => renderMiniDataGrid([["Property", "Value", "Scope", "State", ""], ["Size", "2.4 m", "Local", "OK", ""], ["Fade", "8 m", "Local", "OK", ""], ["Sort", "12", "Material", "OK", ""]]) },
      rightBottom: { title: "Decal Detail", tabs: ["Decal", "Material"], body: () => renderAdditionalDetail("decal-detail") },
      bottomTitle: "Decal Output",
      bottomTabs: ["Validation", "Placement", "Console"],
    },
    "virtual-texture": {
      toolbar: ["Virtual Texture", "Inspect", "Stream", "Evict", "Export"],
      layout: "metrics-graph",
      nodes: [["Page Table", 12, 22], ["Cache", 34, 18], ["Feedback", 58, 38], ["Residency", 78, 58]],
      metrics: [["82%", "Resident"], ["512", "Pages"], ["18", "Misses"], ["64", "MB"]],
      panelTitle: "Streaming State",
      panelRows: [["Texture", "TerrainMega"], ["Residency", "82%"], ["Pages", "512"], ["Misses", "18"]],
      leftTop: { title: "Virtual Textures", tabs: ["Textures", "Pools"], body: () => navList(["TerrainMega", "LandscapeMask", "MegaAlbedo", "MegaNormal", "DebugPages"], 0) },
      rightTop: { title: "Page Residency", tabs: ["Pages", "Pools"], body: () => renderMiniDataGrid([["Pool", "Resident", "Misses", "State", ""], ["Albedo", "84%", "8", "OK", ""], ["Normal", "80%", "6", "OK", ""], ["Masks", "72%", "4", "Warn", ""]]) },
      rightBottom: { title: "Texture Detail", tabs: ["Texture", "Pages"], body: () => renderAdditionalDetail("virtual-texture-detail") },
      bottomTitle: "Virtual Texture Output",
      bottomTabs: ["Streaming", "Feedback", "Console"],
    },
    "material-audit": {
      toolbar: ["Material Audit", "Scan", "Rules", "Fix", "Export"],
      layout: "table-editor",
      tableRows: [["Material", "Cost", "Rule", "State", ""], ["M_Metal", "2.8 ms", "OK", "Pass", ""], ["M_Glass", "4.2 ms", "Overdraw", "Warn", ""], ["M_Foliage", "5.1 ms", "Texture Count", "Warn", ""], ["M_UI_Debug", "0.8 ms", "Dev Only", "Warn", ""]],
      panelTitle: "Audit Summary",
      panelRows: [["Materials", "184"], ["Warnings", "7"], ["Errors", "0"], ["Fixable", "5"]],
      leftTop: { title: "Audits", tabs: ["Audits", "Rules"], body: () => navList(["Scene Materials", "Transparent", "Foliage", "UI Materials", "Debug Only"], 0) },
      rightTop: { title: "Rule Violations", tabs: ["Rules", "Owners"], body: () => renderMiniDataGrid([["Rule", "Hits", "Severity", "Fix", ""], ["Overdraw", "3", "Warn", "Manual", ""], ["Texture Count", "2", "Warn", "Auto", ""], ["Dev Only", "2", "Warn", "Auto", ""]]) },
      rightBottom: { title: "Material Detail", tabs: ["Material", "Fix"], body: () => renderAdditionalDetail("material-audit-detail") },
      bottomTitle: "Material Audit Output",
      bottomTabs: ["Audit", "Fixes", "Console"],
    },
    "sound-cue": {
      toolbar: ["Sound Cue", "Preview", "Randomize", "Attenuation", "Export"],
      layout: "graph",
      nodes: [["Input", 10, 26], ["Random", 32, 18], ["Pitch", 54, 38], ["Output", 78, 54]],
      panelTitle: "Cue Summary",
      panelRows: [["Cue", "Weapon_Fire"], ["Nodes", "6"], ["Variations", "4"], ["Peak", "-3.2 dB"]],
      leftTop: { title: "Cues", tabs: ["Cues", "Folders"], body: () => navList(["Weapon_Fire", "Footstep_Metal", "Door_Open", "Ambience_Hangar", "UI_Click"], 0) },
      rightTop: { title: "Random Rules", tabs: ["Rules", "Waves"], body: () => renderMiniDataGrid([["Wave", "Weight", "Pitch", "State", ""], ["fire_01", "0.35", "1.0", "Ready", ""], ["fire_02", "0.35", "0.98", "Ready", ""], ["fire_tail", "0.30", "1.02", "Ready", ""]]) },
      rightBottom: { title: "Cue Detail", tabs: ["Cue", "Attenuation"], body: () => renderAdditionalDetail("sound-cue-detail") },
      bottomTitle: "Sound Cue Output",
      bottomTabs: ["Preview", "Analysis", "Console"],
    },
    "audio-mixer": {
      toolbar: ["Audio Mixer", "Solo", "Mute", "Snapshot", "Export"],
      layout: "table-editor",
      tableRows: [["Bus", "Level", "Peak", "State", ""], ["Master", "-18 LUFS", "-3.0 dB", "Active", ""], ["SFX", "-16 LUFS", "-2.4 dB", "Active", ""], ["Music", "-22 LUFS", "-8.0 dB", "Ducked", ""], ["Voice", "-19 LUFS", "-5.2 dB", "Active", ""]],
      panelTitle: "Mix Summary",
      panelRows: [["Snapshot", "Gameplay Mix"], ["Loudness", "-18 LUFS"], ["Peaks", "-2.4 dB"], ["Ducks", "1"]],
      leftTop: { title: "Buses", tabs: ["Buses", "Snapshots"], body: () => navList(["Master", "SFX", "Music", "Voice", "UI"], 0) },
      rightTop: { title: "Routing", tabs: ["Routing", "Effects"], body: () => renderMiniDataGrid([["Source", "Bus", "Effect", "State", ""], ["Weapon", "SFX", "Limiter", "On", ""], ["Music", "Music", "Ducker", "On", ""], ["Dialog", "Voice", "EQ", "On", ""]]) },
      rightBottom: { title: "Bus Detail", tabs: ["Bus", "Effects"], body: () => renderAdditionalDetail("audio-mixer-detail") },
      bottomTitle: "Mixer Output",
      bottomTabs: ["Meters", "Snapshots", "Console"],
    },
    "music-system": {
      toolbar: ["Music System", "Play", "Transition", "Stingers", "Export"],
      layout: "graph",
      nodes: [["Explore", 12, 28], ["Tension", 34, 18], ["Combat", 58, 38], ["Victory", 78, 58]],
      panelTitle: "Music State",
      panelRows: [["State", "Combat_Loop"], ["Bars", "8"], ["BPM", "128"], ["Stingers", "3"]],
      leftTop: { title: "States", tabs: ["States", "Segments"], body: () => navList(["Explore", "Tension", "Combat", "Victory", "Menu"], 2) },
      rightTop: { title: "Transitions", tabs: ["Transitions", "Stingers"], body: () => renderMiniDataGrid([["From", "To", "Rule", "State", ""], ["Explore", "Tension", "EnemyNear", "OK", ""], ["Tension", "Combat", "Attack", "OK", ""], ["Combat", "Victory", "Win", "OK", ""]]) },
      rightBottom: { title: "State Detail", tabs: ["State", "Mix"], body: () => renderAdditionalDetail("music-system-detail") },
      bottomTitle: "Music Output",
      bottomTabs: ["Preview", "Transitions", "Console"],
    },
    "audio-occlusion": {
      toolbar: ["Audio Occlusion", "Trace", "Simulate", "Bake", "Export"],
      layout: "table-editor",
      tableRows: [["Trace", "Source", "Obstacle", "State", ""], ["Trace_01", "Emitter_A", "Door_A", "Blocked", ""], ["Trace_02", "Emitter_B", "Wall_02", "Partial", ""], ["Trace_03", "Ambience", "Crate_07", "Clear", ""], ["Trace_04", "Voice", "GlassPanel", "Partial", ""]],
      panelTitle: "Occlusion State",
      panelRows: [["Zone", "Hangar_A"], ["Traces", "12"], ["Blocked", "4"], ["Avg LPF", "0.42"]],
      leftTop: { title: "Zones", tabs: ["Zones", "Rules"], body: () => navList(["Hangar_A", "Dock_Interior", "Corridor_B", "Cave_Entry", "Outdoor"], 0) },
      rightTop: { title: "Materials", tabs: ["Materials", "Filters"], body: () => renderMiniDataGrid([["Material", "LPF", "Volume", "State", ""], ["Metal", "0.52", "-6 dB", "OK", ""], ["Concrete", "0.38", "-9 dB", "OK", ""], ["Glass", "0.74", "-3 dB", "OK", ""]]) },
      rightBottom: { title: "Occlusion Detail", tabs: ["Trace", "Filter"], body: () => renderAdditionalDetail("audio-occlusion-detail") },
      bottomTitle: "Occlusion Output",
      bottomTabs: ["Simulation", "Trace", "Console"],
    },
    "voice-bank": {
      toolbar: ["Voice Bank", "Import", "Validate", "Localize", "Export"],
      layout: "table-editor",
      tableRows: [["Line", "Speaker", "Locale", "State", ""], ["CAP_001", "Captain", "en-US", "Ready", ""], ["CAP_002", "Captain", "zh-CN", "Missing", ""], ["ENG_014", "Engineer", "en-US", "Ready", ""], ["AI_020", "AI Core", "en-US", "Ready", ""]],
      panelTitle: "Bank Summary",
      panelRows: [["Bank", "Captain_EN"], ["Lines", "248"], ["Missing", "1"], ["Duration", "18 min"]],
      leftTop: { title: "Banks", tabs: ["Banks", "Speakers"], body: () => navList(["Captain_EN", "Captain_ZH", "Engineer_EN", "AI_Core", "Narrator"], 0) },
      rightTop: { title: "Coverage", tabs: ["Locales", "Speakers"], body: () => renderMiniDataGrid([["Locale", "Lines", "Missing", "State", ""], ["en-US", "248", "0", "OK", ""], ["zh-CN", "247", "1", "Warn", ""], ["ja-JP", "244", "4", "Warn", ""]]) },
      rightBottom: { title: "Voice Detail", tabs: ["Line", "Localization"], body: () => renderAdditionalDetail("voice-bank-detail") },
      bottomTitle: "Voice Output",
      bottomTabs: ["Import", "Validation", "Console"],
    },
    "subtitle-timing": {
      toolbar: ["Subtitle Timing", "Play", "Align", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Cue", "Start", "End", "State", ""], ["SUB_001", "00:01.20", "00:03.80", "OK", ""], ["SUB_002", "00:04.10", "00:06.20", "OK", ""], ["SUB_003", "00:06.00", "00:08.10", "Overlap", ""], ["SUB_004", "00:09.20", "00:11.40", "OK", ""]],
      panelTitle: "Timing Summary",
      panelRows: [["Sequence", "Intro_Hangar"], ["Cues", "42"], ["Overlaps", "1"], ["Locale", "zh-CN"]],
      leftTop: { title: "Subtitles", tabs: ["Sequences", "Locales"], body: () => navList(["Intro_Hangar", "Tutorial", "Combat_Start", "Ending_A", "Debug"], 0) },
      rightTop: { title: "Timing Checks", tabs: ["Checks", "Reading"], body: () => renderMiniDataGrid([["Rule", "Hits", "Severity", "State", ""], ["Overlap", "1", "Warn", "Open", ""], ["Too Fast", "2", "Warn", "Open", ""], ["Missing Audio", "0", "Error", "OK", ""]]) },
      rightBottom: { title: "Subtitle Detail", tabs: ["Cue", "Locale"], body: () => renderAdditionalDetail("subtitle-timing-detail") },
      bottomTitle: "Subtitle Output",
      bottomTabs: ["Validation", "Preview", "Console"],
    },
    "lip-sync": {
      toolbar: ["Lip Sync", "Solve", "Preview", "Curves", "Export"],
      layout: "metrics-graph",
      nodes: [["Audio", 12, 22], ["Phonemes", 34, 18], ["Visemes", 58, 38], ["Curves", 78, 58]],
      metrics: [["18", "Phonemes"], ["12", "Visemes"], ["92%", "Quality"], ["0", "Gaps"]],
      panelTitle: "Solve Summary",
      panelRows: [["Clip", "Captain_Line_03"], ["Phonemes", "18"], ["Visemes", "12"], ["Quality", "92%"]],
      leftTop: { title: "Clips", tabs: ["Clips", "Characters"], body: () => navList(["Captain_Line_03", "Captain_Line_04", "Engineer_014", "AI_Core_020", "Narrator_001"], 0) },
      rightTop: { title: "Viseme Tracks", tabs: ["Visemes", "Curves"], body: () => renderMiniDataGrid([["Viseme", "Weight", "Frame", "State", ""], ["AA", "0.82", "18", "OK", ""], ["FV", "0.64", "32", "OK", ""], ["MBP", "0.91", "44", "OK", ""]]) },
      rightBottom: { title: "Lip Sync Detail", tabs: ["Clip", "Curves"], body: () => renderAdditionalDetail("lip-sync-detail") },
      bottomTitle: "Lip Sync Output",
      bottomTabs: ["Solve", "Preview", "Console"],
    },
    "audio-profiler": {
      toolbar: ["Audio Profiler", "Capture", "Voices", "Buses", "Export"],
      layout: "metrics-graph",
      nodes: [["Events", 12, 26], ["Voices", 34, 18], ["Mixer", 58, 38], ["Output", 78, 58]],
      metrics: [["64", "Voices"], ["18", "Events"], ["-18", "LUFS"], ["3.2", "ms"]],
      panelTitle: "Capture Summary",
      panelRows: [["Capture", "Live"], ["Voices", "64"], ["CPU", "3.2 ms"], ["Peaks", "-2.4 dB"]],
      leftTop: { title: "Captures", tabs: ["Captures", "Markers"], body: () => navList(["Live Capture", "Combat Spike", "Menu Idle", "Voice Scene", "Baseline"], 0) },
      rightTop: { title: "Voice Counts", tabs: ["Voices", "Buses"], body: () => renderMiniDataGrid([["Group", "Voices", "CPU", "State", ""], ["SFX", "32", "1.4 ms", "OK", ""], ["Voice", "18", "0.9 ms", "OK", ""], ["Music", "4", "0.4 ms", "OK", ""]]) },
      rightBottom: { title: "Event Detail", tabs: ["Event", "Bus"], body: () => renderAdditionalDetail("audio-profiler-detail") },
      bottomTitle: "Audio Profiler Output",
      bottomTabs: ["Capture", "Events", "Console"],
    },
    "rigid-body": {
      toolbar: ["Rigid Body", "Simulate", "Mass", "Collision", "Export"],
      layout: "table-editor",
      tableRows: [["Body", "Shape", "Mass", "State", ""], ["Crate_07", "Box", "12 kg", "Active", ""], ["Door_A", "Convex", "42 kg", "Sleeping", ""], ["Barrel_02", "Cylinder", "18 kg", "Active", ""], ["Pipe_03", "Capsule", "8 kg", "Static", ""]],
      panelTitle: "Body Summary",
      panelRows: [["Body", "Crate_07"], ["Mass", "12 kg"], ["Friction", "0.62"], ["Layer", "World"]],
      leftTop: { title: "Bodies", tabs: ["Bodies", "Shapes"], body: () => navList(["Crate_07", "Door_A", "Barrel_02", "Pipe_03", "Lift_Platform"], 0) },
      rightTop: { title: "Mass Properties", tabs: ["Mass", "Collision"], body: () => renderMiniDataGrid([["Property", "Value", "Unit", "State", ""], ["Mass", "12", "kg", "OK", ""], ["Friction", "0.62", "-", "OK", ""], ["Restitution", "0.12", "-", "OK", ""]]) },
      rightBottom: { title: "Body Detail", tabs: ["Body", "Collision"], body: () => renderAdditionalDetail("rigid-body-detail") },
      bottomTitle: "Rigid Body Output",
      bottomTabs: ["Simulation", "Contacts", "Console"],
    },
    "physics-constraints": {
      toolbar: ["Constraints", "Simulate", "Limits", "Breakage", "Export"],
      layout: "graph",
      nodes: [["Frame", 12, 28], ["Hinge", 34, 18], ["Door", 58, 38], ["Motor", 78, 58]],
      panelTitle: "Constraint Summary",
      panelRows: [["Asset", "Door_Hinge"], ["Joints", "8"], ["Broken", "0"], ["Solver", "Stable"]],
      leftTop: { title: "Constraints", tabs: ["Joints", "Presets"], body: () => navList(["Door_Hinge", "Lift_Piston", "Bridge_Pin", "Crane_Rope", "DebugJoint"], 0) },
      rightTop: { title: "Limits", tabs: ["Limits", "Forces"], body: () => renderMiniDataGrid([["Axis", "Min", "Max", "State", ""], ["Swing", "-45", "90", "OK", ""], ["Twist", "0", "0", "Locked", ""], ["Motor", "0", "120", "OK", ""]]) },
      rightBottom: { title: "Constraint Detail", tabs: ["Joint", "Limits"], body: () => renderAdditionalDetail("physics-constraints-detail") },
      bottomTitle: "Constraint Output",
      bottomTabs: ["Solver", "Breakage", "Console"],
    },
    "destruction-editor": {
      toolbar: ["Destruction", "Fracture", "Cluster", "Damage", "Export"],
      layout: "metrics-graph",
      nodes: [["Root", 12, 22], ["Cluster A", 34, 18], ["Cluster B", 58, 38], ["Debris", 78, 58]],
      metrics: [["42", "Chunks"], ["6", "Clusters"], ["120", "Damage"], ["0", "Errors"]],
      panelTitle: "Fracture Summary",
      panelRows: [["Asset", "WallPanel_A"], ["Chunks", "42"], ["Clusters", "6"], ["Threshold", "120"]],
      leftTop: { title: "Fractures", tabs: ["Assets", "Patterns"], body: () => navList(["WallPanel_A", "GlassPane_B", "Crate_Break", "Pillar_Stone", "DebugWall"], 0) },
      rightTop: { title: "Damage Rules", tabs: ["Rules", "Clusters"], body: () => renderMiniDataGrid([["Cluster", "Chunks", "Threshold", "State", ""], ["Root", "12", "180", "OK", ""], ["A", "18", "120", "OK", ""], ["B", "12", "90", "OK", ""]]) },
      rightBottom: { title: "Cluster Detail", tabs: ["Cluster", "Damage"], body: () => renderAdditionalDetail("destruction-detail") },
      bottomTitle: "Destruction Output",
      bottomTabs: ["Fracture", "Simulation", "Console"],
    },
    "cloth-simulation": {
      toolbar: ["Cloth", "Paint", "Simulate", "Constraints", "Export"],
      layout: "table-editor",
      tableRows: [["Region", "Verts", "Weight", "State", ""], ["Cape_Top", "42", "Pinned", "OK", ""], ["Cape_Mid", "86", "0.55", "OK", ""], ["Cape_Edge", "56", "0.72", "OK", ""], ["Collar", "18", "Pinned", "OK", ""]],
      panelTitle: "Cloth Summary",
      panelRows: [["Asset", "Cape_A"], ["Verts", "184"], ["Pinned", "60"], ["Wind", "0.35"]],
      leftTop: { title: "Cloth", tabs: ["Assets", "Paint"], body: () => navList(["Cape_A", "Banner_Long", "Robe_Sleeve", "Flag_Hangar", "TestCloth"], 0) },
      rightTop: { title: "Constraint Maps", tabs: ["Maps", "Collision"], body: () => renderMiniDataGrid([["Map", "Weight", "Verts", "State", ""], ["MaxDistance", "0.55", "184", "OK", ""], ["Backstop", "0.24", "132", "OK", ""], ["AnimDrive", "0.42", "86", "OK", ""]]) },
      rightBottom: { title: "Cloth Detail", tabs: ["Cloth", "Paint"], body: () => renderAdditionalDetail("cloth-detail") },
      bottomTitle: "Cloth Output",
      bottomTabs: ["Solve", "Paint", "Console"],
    },
    "vehicle-physics": {
      toolbar: ["Vehicle", "Drive", "Suspension", "Tires", "Export"],
      layout: "table-editor",
      tableRows: [["Wheel", "Radius", "Grip", "State", ""], ["FL", "0.42 m", "0.88", "Grounded", ""], ["FR", "0.42 m", "0.88", "Grounded", ""], ["RL", "0.44 m", "0.92", "Grounded", ""], ["RR", "0.44 m", "0.92", "Grounded", ""]],
      panelTitle: "Vehicle Summary",
      panelRows: [["Vehicle", "Rover_01"], ["Wheels", "4"], ["Mass", "1240 kg"], ["Top Speed", "82 km/h"]],
      leftTop: { title: "Vehicles", tabs: ["Vehicles", "Presets"], body: () => navList(["Rover_01", "Truck_A", "Buggy_Test", "DroneWheel", "Forklift"], 0) },
      rightTop: { title: "Drivetrain", tabs: ["Drive", "Tires"], body: () => renderMiniDataGrid([["Part", "Value", "Unit", "State", ""], ["Torque", "420", "Nm", "OK", ""], ["Gear", "3", "-", "OK", ""], ["Grip", "0.90", "-", "OK", ""]]) },
      rightBottom: { title: "Vehicle Detail", tabs: ["Vehicle", "Wheels"], body: () => renderAdditionalDetail("vehicle-physics-detail") },
      bottomTitle: "Vehicle Output",
      bottomTabs: ["Test", "Telemetry", "Console"],
    },
    "fluid-simulation": {
      toolbar: ["Fluid", "Emit", "Simulate", "Cache", "Export"],
      layout: "metrics-graph",
      nodes: [["Emitter", 12, 22], ["Solver", 34, 18], ["Collide", 58, 38], ["Cache", 78, 58]],
      metrics: [["18k", "Particles"], ["4", "Emitters"], ["2.8", "ms"], ["0", "Leaks"]],
      panelTitle: "Fluid Summary",
      panelRows: [["Emitter", "SteamVent_A"], ["Particles", "18k"], ["Step", "2.8 ms"], ["Cache", "Ready"]],
      leftTop: { title: "Fluids", tabs: ["Emitters", "Caches"], body: () => navList(["SteamVent_A", "WaterLeak", "DustPuff", "OilSplash", "DebugFluid"], 0) },
      rightTop: { title: "Solver Metrics", tabs: ["Solver", "Collision"], body: () => renderMiniDataGrid([["Metric", "Value", "Budget", "State", ""], ["Particles", "18k", "25k", "OK", ""], ["Step", "2.8 ms", "4 ms", "OK", ""], ["Colliders", "12", "32", "OK", ""]]) },
      rightBottom: { title: "Fluid Detail", tabs: ["Emitter", "Cache"], body: () => renderAdditionalDetail("fluid-detail") },
      bottomTitle: "Fluid Output",
      bottomTabs: ["Simulation", "Cache", "Console"],
    },
    "rope-cable": {
      toolbar: ["Rope Cable", "Attach", "Simulate", "Tension", "Export"],
      layout: "graph",
      nodes: [["Anchor A", 12, 28], ["Segment", 34, 18], ["Sag", 58, 38], ["Anchor B", 78, 58]],
      panelTitle: "Cable Summary",
      panelRows: [["Cable", "BridgeCable_A"], ["Segments", "32"], ["Tension", "0.72"], ["Length", "18 m"]],
      leftTop: { title: "Cables", tabs: ["Cables", "Anchors"], body: () => navList(["BridgeCable_A", "CraneLine", "Hose_01", "PowerCable", "DebugRope"], 0) },
      rightTop: { title: "Attachments", tabs: ["Attachments", "Tension"], body: () => renderMiniDataGrid([["Point", "Body", "Tension", "State", ""], ["A", "Bridge", "0.72", "OK", ""], ["Mid", "Segment_16", "0.64", "OK", ""], ["B", "Tower", "0.76", "OK", ""]]) },
      rightBottom: { title: "Cable Detail", tabs: ["Cable", "Solve"], body: () => renderAdditionalDetail("rope-cable-detail") },
      bottomTitle: "Cable Output",
      bottomTabs: ["Solve", "Tension", "Console"],
    },
    "physics-profiler": {
      toolbar: ["Physics Profiler", "Capture", "Islands", "Contacts", "Export"],
      layout: "metrics-graph",
      nodes: [["Broadphase", 12, 22], ["Islands", 34, 18], ["Solver", 58, 38], ["Contacts", 78, 58]],
      metrics: [["3.6", "ms"], ["24", "Islands"], ["184", "Contacts"], ["2", "Hotspots"]],
      panelTitle: "Frame Summary",
      panelRows: [["Frame", "1842"], ["Physics", "3.6 ms"], ["Contacts", "184"], ["Hotspots", "2"]],
      leftTop: { title: "Captures", tabs: ["Captures", "Frames"], body: () => navList(["Frame 1842", "Frame 1841", "Spike 1839", "Baseline", "StressTest"], 0) },
      rightTop: { title: "Solver Timings", tabs: ["Timings", "Contacts"], body: () => renderMiniDataGrid([["Phase", "Time", "Count", "State", ""], ["Broadphase", "0.8 ms", "42", "OK", ""], ["Solver", "1.9 ms", "24", "Hot", ""], ["Contacts", "0.9 ms", "184", "OK", ""]]) },
      rightBottom: { title: "Profiler Detail", tabs: ["Frame", "Solver"], body: () => renderAdditionalDetail("physics-profiler-detail") },
      bottomTitle: "Physics Profiler Output",
      bottomTabs: ["Capture", "Contacts", "Console"],
    },
    "ai-director": {
      toolbar: ["AI Director", "Simulate", "Budget", "Waves", "Export"],
      layout: "metrics-graph",
      nodes: [["Pacing", 12, 22], ["Threat", 34, 18], ["Spawn", 58, 38], ["Cooldown", 78, 58]],
      metrics: [["72", "Threat"], ["4", "Waves"], ["18", "Agents"], ["0", "Blocks"]],
      panelTitle: "Director State",
      panelRows: [["Scenario", "Hangar_Assault"], ["Threat", "72"], ["Wave", "3"], ["Pacing", "Rising"]],
      leftTop: { title: "Directors", tabs: ["Directors", "Scenarios"], body: () => navList(["Hangar_Assault", "Stealth_Patrol", "Arena_Rush", "Tutorial", "BossIntro"], 0) },
      rightTop: { title: "Threat Budget", tabs: ["Budget", "Waves"], body: () => renderMiniDataGrid([["Bucket", "Budget", "Used", "State", ""], ["Grunts", "40", "32", "OK", ""], ["Elites", "20", "18", "OK", ""], ["Drones", "12", "8", "OK", ""]]) },
      rightBottom: { title: "Director Detail", tabs: ["Director", "Waves"], body: () => renderAdditionalDetail("ai-director-detail") },
      bottomTitle: "AI Director Output",
      bottomTabs: ["Simulation", "Waves", "Console"],
    },
    "blackboard-editor": {
      toolbar: ["Blackboard", "Validate", "Trace", "Defaults", "Export"],
      layout: "table-editor",
      tableRows: [["Key", "Type", "Value", "State", ""], ["TargetActor", "Entity", "Player", "Live", ""], ["PatrolIndex", "Int", "2", "Live", ""], ["AlertLevel", "Float", "0.82", "Live", ""], ["HasCover", "Bool", "True", "Live", ""]],
      panelTitle: "Blackboard Summary",
      panelRows: [["Asset", "Guard_Patrol"], ["Keys", "18"], ["Live", "14"], ["Errors", "0"]],
      leftTop: { title: "Blackboards", tabs: ["Assets", "Agents"], body: () => navList(["Guard_Patrol", "Drone_Combat", "Civilian_Flee", "Boss_Brain", "Debug"], 0) },
      rightTop: { title: "Runtime Values", tabs: ["Values", "Owners"], body: () => renderMiniDataGrid([["Agent", "Key", "Value", "State", ""], ["Guard_01", "TargetActor", "Player", "Live", ""], ["Guard_01", "PatrolIndex", "2", "Live", ""], ["Drone_02", "AlertLevel", "0.62", "Live", ""]]) },
      rightBottom: { title: "Key Detail", tabs: ["Key", "Trace"], body: () => renderAdditionalDetail("blackboard-detail") },
      bottomTitle: "Blackboard Output",
      bottomTabs: ["Validation", "Trace", "Console"],
    },
    "eqs-query": {
      toolbar: ["EQS Query", "Run", "Tests", "Debug", "Export"],
      layout: "metrics-graph",
      nodes: [["Generate", 12, 22], ["Distance", 34, 18], ["Visibility", 58, 38], ["Score", 78, 58]],
      metrics: [["128", "Candidates"], ["12", "Best"], ["0.91", "Score"], ["2", "Rejects"]],
      panelTitle: "Query Summary",
      panelRows: [["Query", "FindCover"], ["Candidates", "128"], ["Best Score", "0.91"], ["Rejects", "2"]],
      leftTop: { title: "Queries", tabs: ["Queries", "Tests"], body: () => navList(["FindCover", "FindPatrolPoint", "FindFlank", "FindLoot", "DebugQuery"], 0) },
      rightTop: { title: "Scores", tabs: ["Scores", "Tests"], body: () => renderMiniDataGrid([["Candidate", "Distance", "Visibility", "Score", ""], ["Cover_12", "8.2", "Hidden", "0.91", ""], ["Cover_08", "6.4", "Partial", "0.82", ""], ["Cover_21", "12.1", "Hidden", "0.78", ""]]) },
      rightBottom: { title: "Query Detail", tabs: ["Query", "Candidate"], body: () => renderAdditionalDetail("eqs-query-detail") },
      bottomTitle: "EQS Output",
      bottomTabs: ["Debug", "Scores", "Console"],
    },
    "crowd-simulation": {
      toolbar: ["Crowd", "Simulate", "Avoidance", "Flow", "Export"],
      layout: "metrics-graph",
      nodes: [["Entry", 12, 22], ["Lane A", 34, 18], ["Merge", 58, 38], ["Exit", 78, 58]],
      metrics: [["240", "Agents"], ["4", "Groups"], ["18", "Blocked"], ["2.4", "ms"]],
      panelTitle: "Crowd State",
      panelRows: [["Scenario", "Plaza_Test"], ["Agents", "240"], ["Blocked", "18"], ["Cost", "2.4 ms"]],
      leftTop: { title: "Crowds", tabs: ["Scenarios", "Groups"], body: () => navList(["Plaza_Test", "Hangar_Evac", "Market_Day", "Arena_Crowd", "Debug"], 0) },
      rightTop: { title: "Groups", tabs: ["Groups", "Flow"], body: () => renderMiniDataGrid([["Group", "Agents", "Speed", "State", ""], ["Civilians", "180", "1.2", "OK", ""], ["Guards", "42", "1.6", "OK", ""], ["Blocked", "18", "0.0", "Warn", ""]]) },
      rightBottom: { title: "Crowd Detail", tabs: ["Group", "Avoidance"], body: () => renderAdditionalDetail("crowd-detail") },
      bottomTitle: "Crowd Output",
      bottomTabs: ["Simulation", "Flow", "Console"],
    },
    "smart-objects": {
      toolbar: ["Smart Objects", "Reserve", "Validate", "Slots", "Export"],
      layout: "table-editor",
      tableRows: [["Object", "Slots", "Reserved", "State", ""], ["CoverWall_A", "12", "4", "Active", ""], ["Console_Use", "2", "1", "Active", ""], ["Door_Panel", "1", "0", "Ready", ""], ["Bench_Sit", "4", "2", "Active", ""]],
      panelTitle: "Object Summary",
      panelRows: [["Object", "CoverWall_A"], ["Slots", "12"], ["Reserved", "4"], ["Cooldown", "0.4 s"]],
      leftTop: { title: "Objects", tabs: ["Objects", "Tags"], body: () => navList(["CoverWall_A", "Console_Use", "Door_Panel", "Bench_Sit", "Ladder_Climb"], 0) },
      rightTop: { title: "Reservations", tabs: ["Reservations", "Slots"], body: () => renderMiniDataGrid([["Slot", "Agent", "Action", "State", ""], ["Cover_01", "Guard_01", "TakeCover", "Reserved", ""], ["Cover_02", "Guard_02", "Peek", "Reserved", ""], ["Use_01", "Engineer", "Interact", "Reserved", ""]]) },
      rightBottom: { title: "Object Detail", tabs: ["Object", "Slots"], body: () => renderAdditionalDetail("smart-object-detail") },
      bottomTitle: "Smart Object Output",
      bottomTabs: ["Validation", "Reservations", "Console"],
    },
    "patrol-routes": {
      toolbar: ["Patrol Routes", "Preview", "Waypoint", "Validate", "Export"],
      layout: "graph",
      nodes: [["WP_01", 12, 28], ["WP_02", 34, 18], ["WP_03", 58, 38], ["WP_04", 78, 58]],
      panelTitle: "Route Summary",
      panelRows: [["Route", "GuardLoop_A"], ["Waypoints", "8"], ["Loops", "1"], ["Blocked", "0"]],
      leftTop: { title: "Routes", tabs: ["Routes", "Agents"], body: () => navList(["GuardLoop_A", "DockLoop", "RoofPatrol", "DebugRoute", "BossPath"], 0) },
      rightTop: { title: "Waypoints", tabs: ["Waypoints", "Waits"], body: () => renderMiniDataGrid([["Waypoint", "Wait", "Action", "State", ""], ["WP_01", "1.0s", "Look", "OK", ""], ["WP_02", "0.5s", "Move", "OK", ""], ["WP_03", "2.0s", "Scan", "OK", ""]]) },
      rightBottom: { title: "Route Detail", tabs: ["Route", "Waypoint"], body: () => renderAdditionalDetail("patrol-route-detail") },
      bottomTitle: "Patrol Output",
      bottomTabs: ["Validation", "Preview", "Console"],
    },
    "cover-system": {
      toolbar: ["Cover System", "Bake", "Score", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Point", "Type", "Exposure", "State", ""], ["Cover_01", "High", "0.18", "Valid", ""], ["Cover_02", "Low", "0.34", "Valid", ""], ["Cover_03", "Corner", "0.12", "Valid", ""], ["Cover_04", "Lean", "0.24", "Valid", ""]],
      panelTitle: "Cover Summary",
      panelRows: [["Set", "Hangar_Cover"], ["Points", "64"], ["Valid", "64"], ["Avg Exposure", "0.22"]],
      leftTop: { title: "Cover Sets", tabs: ["Sets", "Rules"], body: () => navList(["Hangar_Cover", "Dock_Cover", "Arena_Cover", "Tutorial", "Debug"], 0) },
      rightTop: { title: "Exposure Scores", tabs: ["Scores", "Threats"], body: () => renderMiniDataGrid([["Point", "Threat", "Exposure", "State", ""], ["Cover_01", "Player", "0.18", "OK", ""], ["Cover_02", "Drone", "0.34", "OK", ""], ["Cover_03", "Turret", "0.12", "OK", ""]]) },
      rightBottom: { title: "Cover Detail", tabs: ["Cover", "Threat"], body: () => renderAdditionalDetail("cover-detail") },
      bottomTitle: "Cover Output",
      bottomTabs: ["Bake", "Validation", "Console"],
    },
    "ai-profiler": {
      toolbar: ["AI Profiler", "Capture", "Behavior", "Perception", "Export"],
      layout: "metrics-graph",
      nodes: [["Behavior", 12, 22], ["Perception", 34, 18], ["Pathing", 58, 38], ["Tasks", 78, 58]],
      metrics: [["32", "Agents"], ["2.8", "ms"], ["18", "Events"], ["4", "Hotspots"]],
      panelTitle: "AI Capture",
      panelRows: [["Capture", "Live"], ["Agents", "32"], ["Cost", "2.8 ms"], ["Hotspots", "4"]],
      leftTop: { title: "Captures", tabs: ["Captures", "Agents"], body: () => navList(["Live Capture", "Combat Spike", "Stealth Test", "Crowd Run", "Baseline"], 0) },
      rightTop: { title: "Behavior Timings", tabs: ["Behavior", "Perception"], body: () => renderMiniDataGrid([["System", "Time", "Events", "State", ""], ["Behavior", "1.2 ms", "32", "OK", ""], ["Perception", "0.9 ms", "18", "Hot", ""], ["Pathing", "0.7 ms", "12", "OK", ""]]) },
      rightBottom: { title: "AI Profiler Detail", tabs: ["Capture", "Agent"], body: () => renderAdditionalDetail("ai-profiler-detail") },
      bottomTitle: "AI Profiler Output",
      bottomTabs: ["Capture", "Events", "Console"],
    },
    "mesh-import": {
      toolbar: ["Mesh Import", "Import", "Reimport", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Asset", "Type", "Triangles", "State", ""], ["SK_Crate.fbx", "Static Mesh", "4.8k", "Warning", ""], ["SM_Door.fbx", "Static Mesh", "2.1k", "Ready", ""], ["SK_Guard.fbx", "Skeletal", "18k", "Ready", ""], ["Collision.fbx", "UCX", "512", "Ready", ""]],
      panelTitle: "Import Summary",
      panelRows: [["Source", "SK_Crate.fbx"], ["Warnings", "3"], ["Scale", "1.0"], ["Collision", "Generated"]],
      leftTop: { title: "Imports", tabs: ["Queue", "Presets"], body: () => navList(["SK_Crate.fbx", "SM_Door.fbx", "SK_Guard.fbx", "Collision.fbx", "Batch_Props"], 0) },
      rightTop: { title: "Import Settings", tabs: ["Settings", "Warnings"], body: () => renderMiniDataGrid([["Setting", "Value", "Scope", "State", ""], ["Normals", "Import", "Mesh", "OK", ""], ["Tangents", "Generate", "Mesh", "OK", ""], ["Collision", "Auto", "Mesh", "Warn", ""]]) },
      rightBottom: { title: "Import Detail", tabs: ["Asset", "Warnings"], body: () => renderAdditionalDetail("mesh-import-detail") },
      bottomTitle: "Import Output",
      bottomTabs: ["Validation", "Import", "Console"],
    },
    "lod-chain": {
      toolbar: ["LOD Chain", "Generate", "Reduce", "Preview", "Export"],
      layout: "table-editor",
      tableRows: [["LOD", "Triangles", "Screen", "State", ""], ["LOD0", "18k", "1.00", "Source", ""], ["LOD1", "8.4k", "0.60", "Built", ""], ["LOD2", "3.2k", "0.30", "Built", ""], ["LOD3", "840", "0.12", "Built", ""]],
      panelTitle: "LOD Summary",
      panelRows: [["Mesh", "SM_Rock_A"], ["Levels", "4"], ["Reduction", "95%"], ["Errors", "0"]],
      leftTop: { title: "Meshes", tabs: ["Meshes", "Profiles"], body: () => navList(["SM_Rock_A", "SM_Crate", "SM_Door", "SM_Pipe", "SM_Cliff"], 0) },
      rightTop: { title: "Reduction", tabs: ["Reduction", "Preview"], body: () => renderMiniDataGrid([["LOD", "Reduction", "Error", "State", ""], ["LOD1", "54%", "0.08", "OK", ""], ["LOD2", "82%", "0.16", "OK", ""], ["LOD3", "95%", "0.28", "OK", ""]]) },
      rightBottom: { title: "LOD Detail", tabs: ["LOD", "Settings"], body: () => renderAdditionalDetail("lod-chain-detail") },
      bottomTitle: "LOD Output",
      bottomTabs: ["Build", "Preview", "Console"],
    },
    "redirect-map": {
      toolbar: ["Redirect Map", "Scan", "Resolve", "Replace", "Export"],
      layout: "table-editor",
      tableRows: [["Old Path", "New Path", "Refs", "State", ""], ["/Old/M_Metal", "/Materials/M_Metal", "18", "Resolved", ""], ["/Temp/Crate", "/Meshes/SM_Crate", "12", "Resolved", ""], ["/Old/UI/Icon", "/UI/Icons/save", "4", "Resolved", ""], ["/Missing/Audio", "-", "0", "Unused", ""]],
      panelTitle: "Redirect Summary",
      panelRows: [["Redirects", "42"], ["Resolved", "42"], ["Broken", "0"], ["Unused", "3"]],
      leftTop: { title: "Redirects", tabs: ["Redirects", "Owners"], body: () => navList(["Materials", "Meshes", "UI", "Audio", "Unused"], 0) },
      rightTop: { title: "Owners", tabs: ["Owners", "Refs"], body: () => renderMiniDataGrid([["Owner", "Refs", "Broken", "State", ""], ["Scenes", "18", "0", "OK", ""], ["Prefabs", "12", "0", "OK", ""], ["UI", "4", "0", "OK", ""]]) },
      rightBottom: { title: "Redirect Detail", tabs: ["Redirect", "Refs"], body: () => renderAdditionalDetail("redirect-map-detail") },
      bottomTitle: "Redirect Output",
      bottomTabs: ["Validation", "Replace", "Console"],
    },
    "texture-compression-queue": {
      toolbar: ["Texture Compression", "Run", "Pause", "Rules", "Export"],
      layout: "table-editor",
      tableRows: [["Texture", "Format", "Size", "State", ""], ["T_Rock_D", "BC7", "2048", "Queued", ""], ["T_Rock_N", "BC5", "2048", "Queued", ""], ["T_UI_Atlas", "RGBA8", "1024", "Ready", ""], ["T_Terrain_M", "BC4", "4096", "Running", ""]],
      panelTitle: "Queue Summary",
      panelRows: [["Queued", "18"], ["Running", "1"], ["Format", "BC7"], ["Warnings", "2"]],
      leftTop: { title: "Queue", tabs: ["Queue", "Rules"], body: () => navList(["T_Rock_D", "T_Rock_N", "T_UI_Atlas", "T_Terrain_M", "Batch_UI"], 0) },
      rightTop: { title: "Format Rules", tabs: ["Rules", "Platforms"], body: () => renderMiniDataGrid([["Type", "Format", "Platform", "State", ""], ["Albedo", "BC7", "Desktop", "OK", ""], ["Normal", "BC5", "Desktop", "OK", ""], ["Mask", "BC4", "Desktop", "OK", ""]]) },
      rightBottom: { title: "Compression Detail", tabs: ["Job", "Rules"], body: () => renderAdditionalDetail("texture-compression-detail") },
      bottomTitle: "Compression Output",
      bottomTabs: ["Queue", "Warnings", "Console"],
    },
    "source-asset-trace": {
      toolbar: ["Source Trace", "Trace", "Open Source", "Diff", "Export"],
      layout: "metrics-graph",
      nodes: [["Source", 12, 22], ["Importer", 34, 18], ["Derived", 58, 38], ["Consumers", 78, 58]],
      metrics: [["6", "Outputs"], ["18", "Refs"], ["1", "Stale"], ["0", "Missing"]],
      panelTitle: "Trace Summary",
      panelRows: [["Source", "crate_source.fbx"], ["Outputs", "6"], ["Refs", "18"], ["Stale", "1"]],
      leftTop: { title: "Sources", tabs: ["Sources", "Outputs"], body: () => navList(["crate_source.fbx", "guard_source.fbx", "rock_high.blend", "ui_icons.psd", "audio_mix.wproj"], 0) },
      rightTop: { title: "Generated Assets", tabs: ["Outputs", "Refs"], body: () => renderMiniDataGrid([["Asset", "Type", "Refs", "State", ""], ["SM_Crate", "Mesh", "12", "Current", ""], ["UCX_Crate", "Collision", "4", "Current", ""], ["MI_Crate", "Material", "2", "Stale", ""]]) },
      rightBottom: { title: "Source Detail", tabs: ["Source", "Outputs"], body: () => renderAdditionalDetail("source-asset-detail") },
      bottomTitle: "Source Trace Output",
      bottomTabs: ["Trace", "Diff", "Console"],
    },
    "dcc-live-link": {
      toolbar: ["DCC Live Link", "Connect", "Sync", "Pull", "Export"],
      layout: "table-editor",
      tableRows: [["Session", "DCC", "Asset", "State", ""], ["Blender", "Blender", "SM_Crate", "Connected", ""], ["Maya", "Maya", "SK_Guard", "Idle", ""], ["Houdini", "Houdini", "RockField", "Syncing", ""], ["Photoshop", "PS", "UI_Atlas", "Idle", ""]],
      panelTitle: "Session Summary",
      panelRows: [["DCC", "Blender"], ["State", "Connected"], ["Latency", "42 ms"], ["Dirty", "3"]],
      leftTop: { title: "Sessions", tabs: ["Sessions", "Assets"], body: () => navList(["Blender", "Maya", "Houdini", "Photoshop", "Substance"], 0) },
      rightTop: { title: "Sync Events", tabs: ["Events", "Conflicts"], body: () => renderMiniDataGrid([["Time", "Asset", "Event", "State", ""], ["12:00", "SM_Crate", "Push", "OK", ""], ["12:01", "M_Crate", "Dirty", "Warn", ""], ["12:02", "UCX_Crate", "Pull", "OK", ""]]) },
      rightBottom: { title: "Live Link Detail", tabs: ["Session", "Sync"], body: () => renderAdditionalDetail("dcc-live-link-detail") },
      bottomTitle: "DCC Output",
      bottomTabs: ["Sync", "Conflicts", "Console"],
    },
    "metadata-editor": {
      toolbar: ["Metadata", "Validate", "Schema", "Apply", "Export"],
      layout: "table-editor",
      tableRows: [["Field", "Type", "Value", "State", ""], ["asset.owner", "String", "Environment", "Valid", ""], ["asset.priority", "Enum", "High", "Valid", ""], ["asset.source", "Path", "crate_source.fbx", "Valid", ""], ["asset.review", "Bool", "True", "Valid", ""]],
      panelTitle: "Schema Summary",
      panelRows: [["Schema", "AssetSchema.v2"], ["Fields", "24"], ["Invalid", "0"], ["Dirty", "2"]],
      leftTop: { title: "Schemas", tabs: ["Schemas", "Assets"], body: () => navList(["AssetSchema.v2", "ImportSchema", "AudioMeta", "UIMeta", "BuildMeta"], 0) },
      rightTop: { title: "Rules", tabs: ["Rules", "Usage"], body: () => renderMiniDataGrid([["Rule", "Fields", "Errors", "State", ""], ["Required", "8", "0", "OK", ""], ["Enum", "4", "0", "OK", ""], ["Path", "3", "0", "OK", ""]]) },
      rightBottom: { title: "Metadata Detail", tabs: ["Field", "Schema"], body: () => renderAdditionalDetail("metadata-detail") },
      bottomTitle: "Metadata Output",
      bottomTabs: ["Validation", "Apply", "Console"],
    },
    "batch-process-queue": {
      toolbar: ["Batch Queue", "Run", "Pause", "Workers", "Export"],
      layout: "table-editor",
      tableRows: [["Job", "Type", "Progress", "State", ""], ["Reimport Materials", "Reimport", "42%", "Running", ""], ["Compress Textures", "Compress", "18%", "Queued", ""], ["Build LODs", "Build", "0%", "Queued", ""], ["Validate Prefabs", "Validate", "74%", "Running", ""]],
      panelTitle: "Batch Summary",
      panelRows: [["Batch", "Reimport Materials"], ["Jobs", "64"], ["Running", "2"], ["Failed", "0"]],
      leftTop: { title: "Batches", tabs: ["Batches", "Workers"], body: () => navList(["Reimport Materials", "Compress Textures", "Build LODs", "Validate Prefabs", "Cook Assets"], 0) },
      rightTop: { title: "Workers", tabs: ["Workers", "Failures"], body: () => renderMiniDataGrid([["Worker", "Job", "Load", "State", ""], ["worker-01", "Reimport", "82%", "Running", ""], ["worker-02", "Validate", "64%", "Running", ""], ["worker-03", "Idle", "0%", "Ready", ""]]) },
      rightBottom: { title: "Batch Detail", tabs: ["Job", "Worker"], body: () => renderAdditionalDetail("batch-process-detail") },
      bottomTitle: "Batch Output",
      bottomTabs: ["Queue", "Workers", "Console"],
    },
    "script-editor": {
      toolbar: ["Script", "Run", "Debug", "Format", "Save"],
      layout: "source-preview",
      diagnosticTitle: "Script compile clean",
      diagnostics: ["0 errors", "2 hints", "12 symbols"],
      leftTop: { title: "Scripts", tabs: ["Files", "Symbols"], body: () => navList(["player_controller.zs", "inventory.zs", "interaction.zs", "camera_mode.zs", "debug_tools.zs"], 0) },
      rightTop: { title: "Symbols", tabs: ["Outline", "Calls"], body: () => renderMiniDataGrid([["Symbol", "Kind", "Line", "State", ""], ["on_start", "fn", "12", "OK", ""], ["tick", "fn", "38", "Hot", ""], ["input_map", "var", "7", "OK", ""]]) },
      rightBottom: { title: "Function Detail", tabs: ["Function", "Debug"], body: () => renderAdditionalDetail("script-detail") },
      bottomTitle: "Script Output",
      bottomTabs: ["Compiler", "Debug", "Console"],
    },
    "api-browser": {
      toolbar: ["API Browser", "Search", "Open", "Copy Path", "Export"],
      layout: "table-editor",
      tableRows: [["Symbol", "Kind", "Module", "State", ""], ["UiSurfaceFrame", "Struct", "zircon_runtime::ui", "Stable", ""], ["UiCommand", "Enum", "zircon_runtime::ui", "Stable", ""], ["register_widget", "Fn", "zircon_editor", "Draft", ""], ["UiVisibility", "Enum", "zircon_runtime::ui", "Stable", ""]],
      panelTitle: "API Summary",
      panelRows: [["Namespace", "zircon_runtime::ui"], ["Symbols", "184"], ["Stable", "172"], ["Draft", "12"]],
      leftTop: { title: "Namespaces", tabs: ["Namespaces", "Crates"], body: () => navList(["zircon_runtime::ui", "zircon_editor::tools", "zircon_app::host", "zircon_resource", "zircon_math"], 0) },
      rightTop: { title: "Signatures", tabs: ["Signatures", "Examples"], body: () => renderMiniDataGrid([["Item", "Args", "Returns", "State", ""], ["measure", "ctx", "Size", "OK", ""], ["arrange", "rect", "Frame", "OK", ""], ["dispatch", "event", "Result", "OK", ""]]) },
      rightBottom: { title: "API Detail", tabs: ["Docs", "Uses"], body: () => renderAdditionalDetail("api-detail") },
      bottomTitle: "API Output",
      bottomTabs: ["Docs", "Search", "Console"],
    },
    "plugin-packaging": {
      toolbar: ["Plugin Package", "Validate", "Build", "Sign", "Publish"],
      layout: "table-editor",
      tableRows: [["Artifact", "Target", "Size", "State", ""], ["editor.tools.validation", "Win64", "4.2 MB", "Ready", ""], ["editor.tools.validation", "Linux", "4.0 MB", "Ready", ""], ["manifest.toml", "All", "8 KB", "Valid", ""], ["signature", "All", "2 KB", "Pending", ""]],
      panelTitle: "Package Summary",
      panelRows: [["Plugin", "editor.tools.validation"], ["Version", "0.18.0"], ["Targets", "2"], ["Warnings", "1"]],
      leftTop: { title: "Plugins", tabs: ["Plugins", "Profiles"], body: () => navList(["editor.tools.validation", "editor.asset.audit", "runtime.telemetry", "graphics.debug", "project.export"], 0) },
      rightTop: { title: "Manifest Checks", tabs: ["Checks", "Files"], body: () => renderMiniDataGrid([["Check", "Scope", "State", "Fix", ""], ["Manifest", "All", "OK", "-", ""], ["Icon", "Store", "Warn", "Add", ""], ["License", "All", "OK", "-", ""]]) },
      rightBottom: { title: "Package Detail", tabs: ["Package", "Signing"], body: () => renderAdditionalDetail("plugin-packaging-detail") },
      bottomTitle: "Package Output",
      bottomTabs: ["Build", "Sign", "Console"],
    },
    "module-settings": {
      toolbar: ["Module Settings", "Validate", "Dependencies", "Apply", "Export"],
      layout: "table-editor",
      tableRows: [["Module", "Feature", "Value", "State", ""], ["zircon_runtime", "ui", "On", "Valid", ""], ["zircon_runtime", "network", "Off", "Valid", ""], ["zircon_editor", "tools", "On", "Warn", ""], ["zircon_app", "launcher", "On", "Valid", ""]],
      panelTitle: "Module Summary",
      panelRows: [["Module", "zircon_runtime"], ["Features", "12"], ["Warnings", "2"], ["Cycles", "0"]],
      leftTop: { title: "Modules", tabs: ["Modules", "Features"], body: () => navList(["zircon_runtime", "zircon_editor", "zircon_app", "zircon_resource", "plugins"], 0) },
      rightTop: { title: "Dependency Rules", tabs: ["Rules", "Cycles"], body: () => renderMiniDataGrid([["Rule", "Source", "Target", "State", ""], ["Runtime Core", "editor", "runtime", "OK", ""], ["No Cycle", "app", "editor", "OK", ""], ["Tools", "editor", "resource", "Warn", ""]]) },
      rightBottom: { title: "Setting Detail", tabs: ["Setting", "Deps"], body: () => renderAdditionalDetail("module-settings-detail") },
      bottomTitle: "Module Output",
      bottomTabs: ["Validation", "Diff", "Console"],
    },
    "automation-suite": {
      toolbar: ["Automation", "Run", "Filter", "Agents", "Export"],
      layout: "table-editor",
      tableRows: [["Suite", "Tests", "Passed", "State", ""], ["Editor Smoke", "306", "306", "Passed", ""], ["Asset Import", "84", "82", "Warn", ""], ["Runtime UI", "128", "128", "Passed", ""], ["Packaging", "42", "40", "Running", ""]],
      panelTitle: "Suite Summary",
      panelRows: [["Suite", "Editor Smoke"], ["Passed", "306"], ["Failed", "0"], ["Duration", "4m 12s"]],
      leftTop: { title: "Suites", tabs: ["Suites", "Tags"], body: () => navList(["Editor Smoke", "Asset Import", "Runtime UI", "Packaging", "Nightly"], 0) },
      rightTop: { title: "Agents", tabs: ["Agents", "Artifacts"], body: () => renderMiniDataGrid([["Agent", "Suite", "Load", "State", ""], ["win-ui-01", "Editor", "82%", "Done", ""], ["linux-rt-02", "Runtime", "44%", "Done", ""], ["win-pack-01", "Package", "68%", "Run", ""]]) },
      rightBottom: { title: "Suite Detail", tabs: ["Suite", "Artifacts"], body: () => renderAdditionalDetail("automation-suite-detail") },
      bottomTitle: "Automation Output",
      bottomTabs: ["Run", "Failures", "Console"],
    },
    "build-config": {
      toolbar: ["Build Config", "Validate", "Generate", "Build", "Export"],
      layout: "table-editor",
      tableRows: [["Target", "Profile", "Features", "State", ""], ["Windows Editor", "Development", "ui,tools", "Ready", ""], ["Linux Runtime", "Release", "runtime", "Ready", ""], ["Windows Client", "Shipping", "game", "Draft", ""], ["Server", "Release", "net", "Ready", ""]],
      panelTitle: "Config Summary",
      panelRows: [["Target", "Windows Editor"], ["Profile", "Development"], ["Features", "12"], ["Warnings", "1"]],
      leftTop: { title: "Targets", tabs: ["Targets", "Profiles"], body: () => navList(["Windows Editor", "Linux Runtime", "Windows Client", "Server", "Tools"], 0) },
      rightTop: { title: "Feature Flags", tabs: ["Flags", "Env"], body: () => renderMiniDataGrid([["Flag", "Value", "Scope", "State", ""], ["ui", "On", "Editor", "OK", ""], ["tools", "On", "Editor", "OK", ""], ["net", "Off", "Client", "Warn", ""]]) },
      rightBottom: { title: "Config Detail", tabs: ["Config", "Env"], body: () => renderAdditionalDetail("build-config-detail") },
      bottomTitle: "Build Output",
      bottomTabs: ["Generate", "Build", "Console"],
    },
    "cook-rules": {
      toolbar: ["Cook Rules", "Validate", "Simulate", "Apply", "Export"],
      layout: "table-editor",
      tableRows: [["Rule", "Scope", "Platform", "State", ""], ["Texture BC7", "Textures", "Desktop", "Valid", ""], ["Strip Debug", "Scripts", "Shipping", "Valid", ""], ["Keep Editor Icons", "UI", "Editor", "Valid", ""], ["Audio Vorbis", "Audio", "Desktop", "Warn", ""]],
      panelTitle: "Rule Summary",
      panelRows: [["Profile", "Desktop"], ["Rules", "84"], ["Warnings", "2"], ["Overrides", "6"]],
      leftTop: { title: "Rules", tabs: ["Rules", "Profiles"], body: () => navList(["Desktop", "Shipping", "Editor", "Server", "Mobile"], 0) },
      rightTop: { title: "Platform Overrides", tabs: ["Overrides", "Assets"], body: () => renderMiniDataGrid([["Override", "Asset Type", "Platform", "State", ""], ["BC7", "Texture", "Desktop", "OK", ""], ["Vorbis", "Audio", "Desktop", "Warn", ""], ["Strip", "Script", "Shipping", "OK", ""]]) },
      rightBottom: { title: "Rule Detail", tabs: ["Rule", "Assets"], body: () => renderAdditionalDetail("cook-rules-detail") },
      bottomTitle: "Cook Rules Output",
      bottomTabs: ["Validate", "Simulate", "Console"],
    },
    "runtime-commands": {
      toolbar: ["Runtime Commands", "Run", "Bind", "Audit", "Export"],
      layout: "table-editor",
      tableRows: [["Command", "Context", "Shortcut", "State", ""], ["ui.reload_theme", "Editor", "Ctrl+Alt+R", "Valid", ""], ["scene.focus_selected", "Scene", "F", "Valid", ""], ["console.toggle", "Global", "Alt+4", "Valid", ""], ["asset.reimport", "Assets", "Ctrl+R", "Valid", ""]],
      panelTitle: "Command Summary",
      panelRows: [["Commands", "184"], ["Conflicts", "0"], ["Unbound", "12"], ["Contexts", "8"]],
      leftTop: { title: "Commands", tabs: ["Commands", "Contexts"], body: () => navList(["Global", "Scene", "Assets", "Console", "UI Editor"], 0) },
      rightTop: { title: "Bindings", tabs: ["Bindings", "Conflicts"], body: () => renderMiniDataGrid([["Command", "Shortcut", "Context", "State", ""], ["Reload Theme", "Ctrl+Alt+R", "Editor", "OK", ""], ["Focus", "F", "Scene", "OK", ""], ["Toggle Console", "Alt+4", "Global", "OK", ""]]) },
      rightBottom: { title: "Command Detail", tabs: ["Command", "Binding"], body: () => renderAdditionalDetail("runtime-command-detail") },
      bottomTitle: "Command Output",
      bottomTabs: ["Run", "Audit", "Console"],
    },
    "asset-migration": {
      toolbar: ["Asset Migration", "Scan", "Migrate", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Asset", "From", "To", "State", ""], ["M_Crate", "v17", "v18", "Ready", ""], ["SM_Door", "v17", "v18", "Ready", ""], ["UI_Atlas", "v16", "v18", "Warn", ""], ["AudioBank", "v17", "v18", "Ready", ""]],
      panelTitle: "Migration Summary",
      panelRows: [["Batch", "v17 -> v18"], ["Assets", "128"], ["Warnings", "3"], ["Blocked", "0"]],
      leftTop: { title: "Migrations", tabs: ["Batches", "Schemas"], body: () => navList(["v17 -> v18", "UI Schema v4", "Physics v9", "AudioBank v3", "Legacy Cleanup"], 0) },
      rightTop: { title: "Schema Checks", tabs: ["Checks", "Fields"], body: () => renderMiniDataGrid([["Check", "Assets", "State", "Fix", ""], ["Material Params", "42", "OK", "-", ""], ["Texture Color", "18", "Warn", "Auto", ""], ["Audio Bank", "12", "OK", "-", ""]]) },
      rightBottom: { title: "Migration Detail", tabs: ["Asset", "Schema"], body: () => renderAdditionalDetail("asset-migration-detail") },
      bottomTitle: "Migration Output",
      bottomTabs: ["Scan", "Migrate", "Console"],
    },
    "scene-diff": {
      toolbar: ["Scene Diff", "Compare", "Accept", "Reject", "Export"],
      layout: "table-editor",
      tableRows: [["Entity", "Change", "Owner", "State", ""], ["Door_A", "Transform", "Level", "Review", ""], ["Light_Key", "Intensity", "Lighting", "Accept", ""], ["Crate_07", "Removed", "Props", "Review", ""], ["AudioZone", "Bounds", "Audio", "Accept", ""]],
      panelTitle: "Diff Summary",
      panelRows: [["Scene", "A1_Hangar"], ["Changes", "18"], ["Conflicts", "1"], ["Accepted", "6"]],
      leftTop: { title: "Diffs", tabs: ["Changes", "Owners"], body: () => navList(["All Changes", "Level", "Lighting", "Props", "Audio"], 0) },
      rightTop: { title: "Ownership", tabs: ["Owners", "Conflicts"], body: () => renderMiniDataGrid([["Owner", "Changes", "Conflicts", "State", ""], ["Level", "6", "1", "Review", ""], ["Lighting", "4", "0", "OK", ""], ["Props", "5", "0", "Review", ""]]) },
      rightBottom: { title: "Scene Diff Detail", tabs: ["Change", "Patch"], body: () => renderAdditionalDetail("scene-diff-detail") },
      bottomTitle: "Scene Diff Output",
      bottomTabs: ["Review", "Patch", "Console"],
    },
    "prefab-diff": {
      toolbar: ["Prefab Diff", "Compare", "Apply", "Revert", "Export"],
      layout: "table-editor",
      tableRows: [["Override", "Base", "Variant", "State", ""], ["Mesh", "Door_A", "Door_B", "Changed", ""], ["Collider", "Box", "Convex", "Changed", ""], ["Audio", "Door_Open", "Door_Heavy", "Changed", ""], ["Tag", "Door", "Door.Heavy", "Ready", ""]],
      panelTitle: "Prefab Summary",
      panelRows: [["Prefab", "Door_A"], ["Variant", "B"], ["Changes", "7"], ["Conflicts", "0"]],
      leftTop: { title: "Prefabs", tabs: ["Prefabs", "Variants"], body: () => navList(["Door_A", "Crate_A", "Console_A", "AudioZone", "Trigger_A"], 0) },
      rightTop: { title: "Override Rules", tabs: ["Rules", "Refs"], body: () => renderMiniDataGrid([["Rule", "Scope", "State", "Fix", ""], ["Mesh Swap", "Variant", "OK", "-", ""], ["Collision", "Physics", "OK", "-", ""], ["Audio Ref", "Audio", "OK", "-", ""]]) },
      rightBottom: { title: "Prefab Diff Detail", tabs: ["Override", "Patch"], body: () => renderAdditionalDetail("prefab-diff-detail") },
      bottomTitle: "Prefab Diff Output",
      bottomTabs: ["Diff", "Apply", "Console"],
    },
    "performance-budget": {
      toolbar: ["Performance Budget", "Capture", "Compare", "Budget", "Export"],
      layout: "metrics-graph",
      nodes: [["Frame", 12, 20], ["Render", 34, 30], ["Physics", 58, 22], ["UI", 74, 58]],
      metrics: [["12.8", "Frame ms"], ["4.8", "Render"], ["3.6", "Physics"], ["1.2", "UI"]],
      panelTitle: "Budget Summary",
      panelRows: [["Profile", "Editor Frame"], ["Budget", "16.6 ms"], ["Actual", "12.8 ms"], ["Headroom", "3.8 ms"]],
      leftTop: { title: "Budgets", tabs: ["Budgets", "Captures"], body: () => navList(["Editor Frame", "Gameplay", "Loading", "UI Tools", "Cook"], 0) },
      rightTop: { title: "Subsystem Costs", tabs: ["Costs", "Regressions"], body: () => renderMiniDataGrid([["System", "Actual", "Budget", "State", ""], ["Render", "4.8", "6.0", "OK", ""], ["Physics", "3.6", "4.0", "OK", ""], ["UI", "1.2", "2.0", "OK", ""]]) },
      rightBottom: { title: "Budget Detail", tabs: ["Budget", "Capture"], body: () => renderAdditionalDetail("performance-budget-detail") },
      bottomTitle: "Performance Output",
      bottomTabs: ["Capture", "Regressions", "Console"],
    },
    "memory-budget": {
      toolbar: ["Memory Budget", "Snapshot", "Compare", "Budget", "Export"],
      layout: "table-editor",
      tableRows: [["Pool", "Used", "Budget", "State", ""], ["Texture Pool", "1.2 GB", "1.5 GB", "OK", ""], ["Mesh Pool", "620 MB", "700 MB", "OK", ""], ["Audio Pool", "210 MB", "256 MB", "OK", ""], ["UI Atlas", "92 MB", "80 MB", "Warn", ""]],
      panelTitle: "Memory Summary",
      panelRows: [["Snapshot", "Editor Live"], ["Used", "2.1 GB"], ["Budget", "2.5 GB"], ["Warnings", "1"]],
      leftTop: { title: "Pools", tabs: ["Pools", "Groups"], body: () => navList(["Texture Pool", "Mesh Pool", "Audio Pool", "UI Atlas", "Runtime Heap"], 0) },
      rightTop: { title: "Asset Groups", tabs: ["Groups", "Growth"], body: () => renderMiniDataGrid([["Group", "Used", "Delta", "State", ""], ["Textures", "1.2 GB", "+42 MB", "OK", ""], ["Meshes", "620 MB", "+12 MB", "OK", ""], ["UI", "92 MB", "+18 MB", "Warn", ""]]) },
      rightBottom: { title: "Memory Detail", tabs: ["Pool", "Assets"], body: () => renderAdditionalDetail("memory-budget-detail") },
      bottomTitle: "Memory Output",
      bottomTabs: ["Snapshot", "Compare", "Console"],
    },
    "dependency-cleanup": {
      toolbar: ["Dependency Cleanup", "Scan", "Select", "Clean", "Export"],
      layout: "table-editor",
      tableRows: [["Asset", "Refs", "Owner", "State", ""], ["T_OldPanel", "0", "UI", "Unused", ""], ["M_DebugGrid", "0", "Materials", "Unused", ""], ["SM_TestCrate", "0", "Meshes", "Unused", ""], ["SFX_Temp", "1", "Audio", "Review", ""]],
      panelTitle: "Cleanup Summary",
      panelRows: [["Unused", "42"], ["Broken", "0"], ["Recoverable", "42"], ["Selected", "18"]],
      leftTop: { title: "Cleanup", tabs: ["Unused", "Owners"], body: () => navList(["Unused Assets", "Zero Refs", "Stale Imports", "Temp Content", "Review"], 0) },
      rightTop: { title: "Owner Trace", tabs: ["Owners", "Refs"], body: () => renderMiniDataGrid([["Owner", "Assets", "Selected", "State", ""], ["UI", "12", "8", "OK", ""], ["Materials", "9", "4", "OK", ""], ["Audio", "4", "1", "Review", ""]]) },
      rightBottom: { title: "Cleanup Detail", tabs: ["Asset", "Refs"], body: () => renderAdditionalDetail("dependency-cleanup-detail") },
      bottomTitle: "Cleanup Output",
      bottomTabs: ["Scan", "Clean", "Console"],
    },
    "naming-rules": {
      toolbar: ["Naming Rules", "Scan", "Autofix", "Rules", "Export"],
      layout: "table-editor",
      tableRows: [["Asset", "Rule", "Expected", "State", ""], ["crate_old", "Mesh Prefix", "SM_CrateOld", "Warn", ""], ["metal01", "Material Prefix", "M_Metal01", "Warn", ""], ["button-save", "Icon Case", "icon_save", "Auto", ""], ["Audio_Beep", "SFX Prefix", "SFX_Beep", "Auto", ""]],
      panelTitle: "Naming Summary",
      panelRows: [["Warnings", "12"], ["Autofix", "5"], ["Rules", "28"], ["Blocked", "0"]],
      leftTop: { title: "Rules", tabs: ["Rules", "Scopes"], body: () => navList(["Meshes", "Materials", "Textures", "Audio", "UI"], 0) },
      rightTop: { title: "Rule Scopes", tabs: ["Scopes", "Examples"], body: () => renderMiniDataGrid([["Scope", "Rule", "Issues", "State", ""], ["Meshes", "SM_*", "3", "Warn", ""], ["Materials", "M_*", "4", "Warn", ""], ["UI", "icon_*", "2", "Auto", ""]]) },
      rightBottom: { title: "Naming Detail", tabs: ["Issue", "Rule"], body: () => renderAdditionalDetail("naming-rule-detail") },
      bottomTitle: "Naming Output",
      bottomTabs: ["Scan", "Autofix", "Console"],
    },
    "release-checklist": {
      toolbar: ["Release Checklist", "Validate", "Assign", "Approve", "Export"],
      layout: "table-editor",
      tableRows: [["Gate", "Owner", "State", "Due", ""], ["Automation", "QA", "Ready", "Today", ""], ["Cook", "Build", "Ready", "Today", ""], ["Crash Triage", "Runtime", "Review", "Today", ""], ["Notes", "Production", "Draft", "Today", ""]],
      panelTitle: "Release Summary",
      panelRows: [["Version", "0.18.0"], ["Ready", "6/8"], ["Blocking", "0"], ["Review", "2"]],
      leftTop: { title: "Gates", tabs: ["Gates", "Owners"], body: () => navList(["Automation", "Cook", "Crash Triage", "Notes", "Signoff"], 0) },
      rightTop: { title: "Approval Owners", tabs: ["Owners", "Risks"], body: () => renderMiniDataGrid([["Owner", "Gates", "Ready", "State", ""], ["QA", "2", "2", "OK", ""], ["Build", "2", "2", "OK", ""], ["Runtime", "2", "1", "Review", ""]]) },
      rightBottom: { title: "Gate Detail", tabs: ["Gate", "Approval"], body: () => renderAdditionalDetail("release-checklist-detail") },
      bottomTitle: "Release Output",
      bottomTabs: ["Validate", "Approvals", "Console"],
    },
    "gameplay-debugger": {
      toolbar: ["Gameplay Debugger", "Pause", "Step", "Watch", "Export"],
      layout: "table-editor",
      tableRows: [["Actor", "State", "Tags", "Cost", ""], ["Player_01", "Alive", "Player", "0.12 ms", ""], ["Guard_07", "Alert", "AI", "0.18 ms", ""], ["Door_A", "Locked", "Interact", "0.02 ms", ""], ["Trigger_03", "Active", "Quest", "0.01 ms", ""]],
      panelTitle: "Actor Summary",
      panelRows: [["Actor", "Player_01"], ["State", "Alive"], ["Health", "82"], ["Watches", "6"]],
      leftTop: { title: "Actors", tabs: ["Actors", "Watches"], body: () => navList(["Player_01", "Guard_07", "Door_A", "Trigger_03", "QuestVolume"], 0) },
      rightTop: { title: "Watches", tabs: ["Watches", "Events"], body: () => renderMiniDataGrid([["Watch", "Value", "Scope", "State", ""], ["health", "82", "Player", "OK", ""], ["threat", "72", "AI", "Hot", ""], ["quest.power", "True", "World", "OK", ""]]) },
      rightBottom: { title: "Actor Detail", tabs: ["Actor", "Trace"], body: () => renderAdditionalDetail("gameplay-debugger-detail") },
      bottomTitle: "Debug Output",
      bottomTabs: ["Trace", "Events", "Console"],
    },
    "replay-timeline": {
      toolbar: ["Replay Timeline", "Play", "Marker", "Scrub", "Export"],
      layout: "graph",
      nodes: [["Spawn", 10, 24], ["Fight", 34, 34], ["Crash", 58, 22], ["Recover", 78, 58]],
      panelTitle: "Replay Summary",
      panelRows: [["Replay", "Match_042"], ["Duration", "12:42"], ["Markers", "12"], ["Events", "184"]],
      leftTop: { title: "Replays", tabs: ["Replays", "Markers"], body: () => navList(["Match_042", "Crash_1842", "Playtest_A", "NetworkSpike", "Baseline"], 0) },
      rightTop: { title: "Markers", tabs: ["Markers", "Tracks"], body: () => renderMiniDataGrid([["Time", "Marker", "Track", "State", ""], ["00:18", "Spawn", "Game", "OK", ""], ["06:42", "Packet Loss", "Net", "Warn", ""], ["12:10", "Crash", "Runtime", "Hot", ""]]) },
      rightBottom: { title: "Replay Detail", tabs: ["Marker", "Track"], body: () => renderAdditionalDetail("replay-timeline-detail") },
      bottomTitle: "Replay Output",
      bottomTabs: ["Playback", "Markers", "Console"],
    },
    "network-packet-inspector": {
      toolbar: ["Packet Inspector", "Capture", "Filter", "Decode", "Export"],
      layout: "table-editor",
      tableRows: [["Packet", "Channel", "Size", "State", ""], ["#1842", "Replication", "512 B", "OK", ""], ["#1843", "RPC", "128 B", "OK", ""], ["#1844", "Input", "64 B", "OK", ""], ["#1845", "Reliable", "1.2 KB", "Warn", ""]],
      panelTitle: "Capture Summary",
      panelRows: [["Peer", "Client A"], ["Bandwidth", "1.2 KB/f"], ["Loss", "0.2%"], ["Packets", "184"]],
      leftTop: { title: "Streams", tabs: ["Streams", "Peers"], body: () => navList(["Client A", "Client B", "Server", "Replay", "Loopback"], 0) },
      rightTop: { title: "Channels", tabs: ["Channels", "Stats"], body: () => renderMiniDataGrid([["Channel", "Packets", "Bytes", "State", ""], ["Replication", "84", "42 KB", "OK", ""], ["RPC", "32", "8 KB", "OK", ""], ["Reliable", "12", "18 KB", "Warn", ""]]) },
      rightBottom: { title: "Packet Detail", tabs: ["Packet", "Decode"], body: () => renderAdditionalDetail("packet-detail") },
      bottomTitle: "Packet Output",
      bottomTabs: ["Capture", "Decode", "Console"],
    },
    "latency-map": {
      toolbar: ["Latency Map", "Probe", "Compare", "Route", "Export"],
      layout: "metrics-graph",
      nodes: [["Client", 10, 34], ["Edge", 34, 20], ["Relay", 58, 34], ["Server", 78, 58]],
      metrics: [["42", "p50 ms"], ["88", "p95 ms"], ["0.2", "loss %"], ["12", "jitter"]],
      panelTitle: "Latency Summary",
      panelRows: [["Region", "Asia"], ["p50", "42 ms"], ["p95", "88 ms"], ["Loss", "0.2%"]],
      leftTop: { title: "Regions", tabs: ["Regions", "Routes"], body: () => navList(["Asia", "US West", "Europe", "Oceania", "Local"], 0) },
      rightTop: { title: "Routes", tabs: ["Routes", "Alerts"], body: () => renderMiniDataGrid([["Route", "p50", "p95", "State", ""], ["Asia-01", "42", "88", "OK", ""], ["Asia-02", "64", "140", "Warn", ""], ["Relay-A", "38", "80", "OK", ""]]) },
      rightBottom: { title: "Latency Detail", tabs: ["Route", "Probe"], body: () => renderAdditionalDetail("latency-detail") },
      bottomTitle: "Latency Output",
      bottomTabs: ["Probe", "Alerts", "Console"],
    },
    "input-trace": {
      toolbar: ["Input Trace", "Record", "Replay", "Map", "Export"],
      layout: "table-editor",
      tableRows: [["Time", "Device", "Action", "State", ""], ["12:00.120", "Keyboard", "MoveForward", "Down", ""], ["12:00.164", "Mouse", "Look", "Delta", ""], ["12:00.220", "Keyboard", "Interact", "Pressed", ""], ["12:00.280", "Gamepad", "Confirm", "Ignored", ""]],
      panelTitle: "Trace Summary",
      panelRows: [["Devices", "2"], ["Events", "184"], ["Dropped", "0"], ["Conflicts", "1"]],
      leftTop: { title: "Devices", tabs: ["Devices", "Actions"], body: () => navList(["Keyboard", "Mouse", "Gamepad", "Touch", "Replay Input"], 0) },
      rightTop: { title: "Action Mapping", tabs: ["Actions", "Conflicts"], body: () => renderMiniDataGrid([["Action", "Device", "Binding", "State", ""], ["MoveForward", "Keyboard", "W", "OK", ""], ["Interact", "Keyboard", "E", "OK", ""], ["Confirm", "Gamepad", "A", "Ignored", ""]]) },
      rightBottom: { title: "Trace Detail", tabs: ["Event", "Action"], body: () => renderAdditionalDetail("input-trace-detail") },
      bottomTitle: "Input Output",
      bottomTabs: ["Trace", "Replay", "Console"],
    },
    "save-state-diff": {
      toolbar: ["Save State Diff", "Compare", "Patch", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Path", "Slot A", "Slot B", "State", ""], ["quest.power", "False", "True", "Changed", ""], ["player.health", "82", "64", "Changed", ""], ["inventory.key", "0", "1", "Changed", ""], ["world.flags", "184", "196", "Changed", ""]],
      panelTitle: "Diff Summary",
      panelRows: [["Base", "Slot_A"], ["Compare", "Slot_B"], ["Changes", "12"], ["Schema", "v12"]],
      leftTop: { title: "Slots", tabs: ["Slots", "Schemas"], body: () => navList(["Slot_A", "Slot_B", "Checkpoint_12", "Cloud Slot", "Replay Save"], 0) },
      rightTop: { title: "Schema Guards", tabs: ["Guards", "Paths"], body: () => renderMiniDataGrid([["Guard", "Path", "State", "Fix", ""], ["Required", "player.*", "OK", "-", ""], ["Version", "schema", "OK", "-", ""], ["Quest", "quest.*", "Warn", "Review", ""]]) },
      rightBottom: { title: "State Detail", tabs: ["State", "Patch"], body: () => renderAdditionalDetail("save-state-detail") },
      bottomTitle: "Save Diff Output",
      bottomTabs: ["Compare", "Patch", "Console"],
    },
    "repro-recorder": {
      toolbar: ["Repro Recorder", "Record", "Bookmark", "Package", "Export"],
      layout: "table-editor",
      tableRows: [["Step", "Input", "State", "Time", ""], ["1", "Open A1_Hangar", "Captured", "00:00", ""], ["2", "Start PIE", "Captured", "00:08", ""], ["3", "Interact Door_A", "Captured", "00:42", ""], ["4", "Crash", "Captured", "01:12", ""]],
      panelTitle: "Recording Summary",
      panelRows: [["Session", "Crash_1842"], ["Steps", "4"], ["Artifacts", "6"], ["State", "Recording"]],
      leftTop: { title: "Recordings", tabs: ["Recordings", "Artifacts"], body: () => navList(["Crash_1842", "Door_Bug", "NetSpike", "InputDrop", "Baseline"], 0) },
      rightTop: { title: "Environment", tabs: ["Environment", "Artifacts"], body: () => renderMiniDataGrid([["Fact", "Value", "Scope", "State", ""], ["Build", "0.18.0", "Runtime", "OK", ""], ["Map", "A1_Hangar", "Scene", "OK", ""], ["Seed", "1842", "Runtime", "OK", ""]]) },
      rightBottom: { title: "Repro Detail", tabs: ["Step", "Artifacts"], body: () => renderAdditionalDetail("repro-detail") },
      bottomTitle: "Repro Output",
      bottomTabs: ["Record", "Package", "Console"],
    },
    "qa-triage": {
      toolbar: ["QA Triage", "Assign", "Link Repro", "Prioritize", "Export"],
      layout: "table-editor",
      tableRows: [["Issue", "Severity", "Owner", "State", ""], ["QA-1842", "Blocker", "Runtime", "Open", ""], ["QA-1843", "Major", "Assets", "Review", ""], ["QA-1844", "Minor", "UI", "Assigned", ""], ["QA-1845", "Major", "Network", "Open", ""]],
      panelTitle: "Triage Summary",
      panelRows: [["Open", "18"], ["Blocking", "3"], ["Assigned", "12"], ["Needs Repro", "4"]],
      leftTop: { title: "Issues", tabs: ["Issues", "Queues"], body: () => navList(["Blockers", "Needs Repro", "Assigned", "Regression", "Resolved"], 0) },
      rightTop: { title: "Owners", tabs: ["Owners", "Repros"], body: () => renderMiniDataGrid([["Owner", "Open", "Blocking", "State", ""], ["Runtime", "6", "2", "Hot", ""], ["Assets", "4", "0", "Review", ""], ["Network", "3", "1", "Hot", ""]]) },
      rightBottom: { title: "Issue Detail", tabs: ["Issue", "Repro"], body: () => renderAdditionalDetail("qa-triage-detail") },
      bottomTitle: "QA Output",
      bottomTabs: ["Triage", "Repros", "Console"],
    },
    "render-graph": {
      toolbar: ["Render Graph", "Compile", "Capture", "Validate", "Export"],
      layout: "graph",
      nodes: [["Depth", 10, 22], ["GBuffer", 34, 24], ["Lighting", 58, 34], ["Post", 78, 58]],
      panelTitle: "Graph Summary",
      panelRows: [["Graph", "Forward+"], ["Passes", "7"], ["Barriers", "18"], ["Warnings", "1"]],
      leftTop: { title: "Passes", tabs: ["Passes", "Resources"], body: () => navList(["Depth Prepass", "GBuffer", "Lighting", "Translucency", "Post"], 2) },
      rightTop: { title: "Resource Barriers", tabs: ["Barriers", "Targets"], body: () => renderMiniDataGrid([["Resource", "From", "To", "State", ""], ["HDR_Main", "Write", "Read", "OK", ""], ["Depth", "Write", "Sample", "OK", ""], ["SSAO", "Write", "Read", "Warn", ""]]) },
      rightBottom: { title: "Pass Detail", tabs: ["Pass", "Resources"], body: () => renderAdditionalDetail("render-graph-detail") },
      bottomTitle: "Render Graph Output",
      bottomTabs: ["Compile", "Capture", "Console"],
    },
    "shader-debugger": {
      toolbar: ["Shader Debugger", "Step", "Breakpoint", "Inspect", "Export"],
      layout: "source-preview",
      diagnosticTitle: "Shader breakpoint active",
      diagnostics: ["fragment_main", "12 variables", "1 divergent branch"],
      leftTop: { title: "Shaders", tabs: ["Shaders", "Variants"], body: () => navList(["M_Metal.fragment", "M_Glass.fragment", "M_Foliage.vertex", "common_lighting", "debug_view"], 0) },
      rightTop: { title: "Variables", tabs: ["Variables", "Waves"], body: () => renderMiniDataGrid([["Variable", "Value", "Scope", "State", ""], ["roughness", "0.48", "Pixel", "OK", ""], ["normal", "0.2,0.8", "Pixel", "OK", ""], ["branch_id", "4", "Wave", "Hot", ""]]) },
      rightBottom: { title: "Shader Detail", tabs: ["Variable", "Wave"], body: () => renderAdditionalDetail("shader-debugger-detail") },
      bottomTitle: "Shader Debug Output",
      bottomTabs: ["Step", "Variables", "Console"],
    },
    "texture-streaming": {
      toolbar: ["Texture Streaming", "Refresh", "Pin", "Evict", "Export"],
      layout: "table-editor",
      tableRows: [["Texture", "Residency", "Mips", "State", ""], ["T_TerrainMega", "82%", "12", "Warn", ""], ["T_Rock_D", "100%", "8", "OK", ""], ["T_UI_Atlas", "100%", "6", "Pinned", ""], ["T_Door_N", "76%", "7", "Streaming", ""]],
      panelTitle: "Streaming Summary",
      panelRows: [["Residency", "82%"], ["Misses", "18"], ["Pinned", "12"], ["Budget", "1.5 GB"]],
      leftTop: { title: "Textures", tabs: ["Textures", "Groups"], body: () => navList(["Terrain", "Props", "UI", "Characters", "Effects"], 0) },
      rightTop: { title: "Page Requests", tabs: ["Requests", "Misses"], body: () => renderMiniDataGrid([["Texture", "Pages", "Misses", "State", ""], ["TerrainMega", "512", "18", "Warn", ""], ["Rock_D", "64", "0", "OK", ""], ["Door_N", "48", "4", "Streaming", ""]]) },
      rightBottom: { title: "Texture Detail", tabs: ["Texture", "Pages"], body: () => renderAdditionalDetail("texture-streaming-detail") },
      bottomTitle: "Streaming Output",
      bottomTabs: ["Requests", "Misses", "Console"],
    },
    "shadow-map": {
      toolbar: ["Shadow Map", "Capture", "Cascade", "Bias", "Export"],
      layout: "metrics-graph",
      nodes: [["Light", 12, 30], ["Cascade 0", 34, 20], ["Cascade 1", 58, 34], ["Atlas", 78, 58]],
      metrics: [["4", "Cascades"], ["2048", "Atlas"], ["1.8", "ms"], ["2", "Acne"]],
      panelTitle: "Shadow Summary",
      panelRows: [["Light", "KeyLight"], ["Cascades", "4"], ["Resolution", "2048"], ["Cost", "1.8 ms"]],
      leftTop: { title: "Lights", tabs: ["Lights", "Atlases"], body: () => navList(["KeyLight", "Door Strip", "Sun", "Console Glow", "Area Fill"], 0) },
      rightTop: { title: "Cascade Metrics", tabs: ["Cascades", "Issues"], body: () => renderMiniDataGrid([["Cascade", "Range", "Cost", "State", ""], ["C0", "0-12m", "0.4", "OK", ""], ["C1", "12-48m", "0.5", "OK", ""], ["C2", "48-128m", "0.6", "Warn", ""]]) },
      rightBottom: { title: "Cascade Detail", tabs: ["Cascade", "Bias"], body: () => renderAdditionalDetail("shadow-map-detail") },
      bottomTitle: "Shadow Output",
      bottomTabs: ["Capture", "Issues", "Console"],
    },
    "occlusion-culling": {
      toolbar: ["Occlusion Culling", "Capture", "Freeze", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Object", "Cell", "Visible", "State", ""], ["SM_Crate_07", "A1_04", "Yes", "Visible", ""], ["Door_A", "A1_04", "Yes", "Visible", ""], ["PipeWall", "A1_05", "No", "Culled", ""], ["Console_A", "A1_04", "Yes", "Visible", ""]],
      panelTitle: "Visibility Summary",
      panelRows: [["Cell", "A1_Hangar"], ["Visible", "184"], ["Culled", "326"], ["Queries", "64"]],
      leftTop: { title: "Cells", tabs: ["Cells", "Sets"], body: () => navList(["A1_Hangar", "Dock", "Roof", "Arena", "Tutorial"], 0) },
      rightTop: { title: "Query Batches", tabs: ["Batches", "Stats"], body: () => renderMiniDataGrid([["Batch", "Queries", "Visible", "State", ""], ["A1_04", "64", "184", "OK", ""], ["A1_05", "42", "96", "OK", ""], ["Dock", "18", "24", "Warn", ""]]) },
      rightBottom: { title: "Object Detail", tabs: ["Object", "Query"], body: () => renderAdditionalDetail("occlusion-detail") },
      bottomTitle: "Occlusion Output",
      bottomTabs: ["Capture", "Queries", "Console"],
    },
    "frame-compare": {
      toolbar: ["Frame Compare", "Load", "Compare", "Diff Passes", "Export"],
      layout: "metrics-graph",
      nodes: [["Frame 1841", 12, 26], ["Render", 34, 20], ["Physics", 58, 38], ["Frame 1842", 78, 58]],
      metrics: [["+0.8", "Frame ms"], ["+12", "Draws"], ["+42", "MB"], ["2", "Regressions"]],
      panelTitle: "Compare Summary",
      panelRows: [["Base", "1841"], ["Compare", "1842"], ["Delta", "+0.8 ms"], ["Regressions", "2"]],
      leftTop: { title: "Captures", tabs: ["Captures", "Frames"], body: () => navList(["Frame 1841", "Frame 1842", "Frame 1843", "GPU Spike", "Baseline"], 1) },
      rightTop: { title: "Pass Deltas", tabs: ["Passes", "Resources"], body: () => renderMiniDataGrid([["Pass", "Base", "Compare", "Delta", ""], ["Lighting", "4.0", "4.8", "+0.8", ""], ["UI", "1.1", "1.2", "+0.1", ""], ["Post", "0.8", "0.7", "-0.1", ""]]) },
      rightBottom: { title: "Frame Detail", tabs: ["Frame", "Diff"], body: () => renderAdditionalDetail("frame-compare-detail") },
      bottomTitle: "Frame Compare Output",
      bottomTabs: ["Compare", "Regressions", "Console"],
    },
    "material-layers": {
      toolbar: ["Material Layers", "Add Layer", "Preview", "Compile", "Export"],
      layout: "table-editor",
      tableRows: [["Layer", "Blend", "Cost", "State", ""], ["Base Metal", "Normal", "1.2", "OK", ""], ["Scratches", "Overlay", "0.8", "OK", ""], ["Dust", "Multiply", "0.4", "OK", ""], ["Wetness", "Screen", "0.9", "Warn", ""]],
      panelTitle: "Layer Summary",
      panelRows: [["Material", "M_Armor"], ["Layers", "5"], ["Cost", "3.6 ms"], ["Warnings", "1"]],
      leftTop: { title: "Layers", tabs: ["Layers", "Materials"], body: () => navList(["M_Armor", "M_Metal", "M_Glass", "M_Rock", "M_Fabric"], 0) },
      rightTop: { title: "Parameters", tabs: ["Params", "Overrides"], body: () => renderMiniDataGrid([["Param", "Value", "Layer", "State", ""], ["roughness", "0.48", "Base", "OK", ""], ["dust", "0.32", "Dust", "OK", ""], ["wetness", "0.12", "Wetness", "Warn", ""]]) },
      rightBottom: { title: "Layer Detail", tabs: ["Layer", "Params"], body: () => renderAdditionalDetail("material-layer-detail") },
      bottomTitle: "Material Layer Output",
      bottomTabs: ["Compile", "Preview", "Console"],
    },
    "gpu-memory": {
      toolbar: ["GPU Memory", "Snapshot", "Compare", "Evict", "Export"],
      layout: "table-editor",
      tableRows: [["Heap", "Used", "Budget", "State", ""], ["Textures", "1.2 GB", "1.5 GB", "OK", ""], ["Buffers", "620 MB", "800 MB", "OK", ""], ["Render Targets", "420 MB", "512 MB", "Warn", ""], ["Acceleration", "180 MB", "256 MB", "OK", ""]],
      panelTitle: "Memory Summary",
      panelRows: [["Used", "2.4 GB"], ["Budget", "3.0 GB"], ["Delta", "+512 MB"], ["Warnings", "1"]],
      leftTop: { title: "Heaps", tabs: ["Heaps", "Owners"], body: () => navList(["Textures", "Buffers", "Render Targets", "Acceleration", "Upload"], 0) },
      rightTop: { title: "Allocation Groups", tabs: ["Groups", "Growth"], body: () => renderMiniDataGrid([["Group", "Used", "Delta", "State", ""], ["Textures", "1.2 GB", "+42 MB", "OK", ""], ["Targets", "420 MB", "+128 MB", "Warn", ""], ["Buffers", "620 MB", "+64 MB", "OK", ""]]) },
      rightBottom: { title: "Allocation Detail", tabs: ["Heap", "Owners"], body: () => renderAdditionalDetail("gpu-memory-detail") },
      bottomTitle: "GPU Memory Output",
      bottomTabs: ["Snapshot", "Compare", "Console"],
    },
    retarget: {
      toolbar: ["Retarget", "Map", "Preview", "Solve", "Export"],
      layout: "table-editor",
      tableRows: [["Chain", "Source", "Target", "State", ""], ["Spine", "spine_01", "spine_02", "Mapped", ""], ["Left Arm", "upperarm_l", "arm_l", "Mapped", ""], ["Right Arm", "upperarm_r", "arm_r", "Mapped", ""], ["Legs", "thigh_l/r", "leg_l/r", "Warn", ""]],
      panelTitle: "Retarget Summary",
      panelRows: [["Source", "Humanoid_A"], ["Target", "Guard_B"], ["Bones", "68"], ["Warnings", "2"]],
      leftTop: { title: "Skeletons", tabs: ["Skeletons", "Profiles"], body: () => navList(["Humanoid_A", "Guard_B", "Creature_A", "Robot_A", "MetaHuman"], 0) },
      rightTop: { title: "Chain Rules", tabs: ["Chains", "Offsets"], body: () => renderMiniDataGrid([["Chain", "Offset", "Scale", "State", ""], ["Spine", "0.02", "1.00", "OK", ""], ["Arms", "-0.04", "0.98", "OK", ""], ["Legs", "0.08", "1.02", "Warn", ""]]) },
      rightBottom: { title: "Retarget Detail", tabs: ["Chain", "Pose"], body: () => renderAdditionalDetail("retarget-detail") },
      bottomTitle: "Retarget Output",
      bottomTabs: ["Solve", "Warnings", "Console"],
    },
    "ik-solver": {
      toolbar: ["IK Solver", "Solve", "Pin", "Preview", "Export"],
      layout: "metrics-graph",
      nodes: [["Root", 12, 22], ["Spine", 34, 20], ["Hand IK", 58, 36], ["Foot IK", 78, 58]],
      metrics: [["4", "Effectors"], ["0.38", "Solve ms"], ["1", "Warning"], ["48", "Controls"]],
      panelTitle: "Solver Summary",
      panelRows: [["Solver", "FullBodyIK"], ["Effectors", "4"], ["Cost", "0.38 ms"], ["Warnings", "1"]],
      leftTop: { title: "Solvers", tabs: ["Solvers", "Effectors"], body: () => navList(["FullBodyIK", "LeftHandIK", "FootPlacement", "LookAt", "WeaponAim"], 0) },
      rightTop: { title: "Effector Watches", tabs: ["Effectors", "Errors"], body: () => renderMiniDataGrid([["Effector", "Error", "Weight", "State", ""], ["LeftHand", "0.02", "1.00", "OK", ""], ["RightHand", "0.04", "0.90", "OK", ""], ["LeftFoot", "0.12", "0.80", "Warn", ""]]) },
      rightBottom: { title: "Solver Detail", tabs: ["Solver", "Effector"], body: () => renderAdditionalDetail("ik-solver-detail") },
      bottomTitle: "IK Output",
      bottomTabs: ["Solve", "Errors", "Console"],
    },
    "pose-library": {
      toolbar: ["Pose Library", "Search", "Tag", "Preview", "Export"],
      layout: "table-editor",
      tableRows: [["Pose", "Clip", "Cost", "State", ""], ["Pose_142", "Run_Start", "0.42", "Ready", ""], ["Pose_184", "Turn_Left", "0.38", "Ready", ""], ["Pose_210", "Aim_Up", "0.51", "Tagged", ""], ["Pose_244", "Crouch", "0.47", "Ready", ""]],
      panelTitle: "Library Summary",
      panelRows: [["Library", "Combat"], ["Poses", "842"], ["Tags", "18"], ["Missing", "0"]],
      leftTop: { title: "Libraries", tabs: ["Libraries", "Tags"], body: () => navList(["Combat", "Locomotion", "Stealth", "Cinematics", "Debug"], 0) },
      rightTop: { title: "Tags", tabs: ["Tags", "Usage"], body: () => renderMiniDataGrid([["Tag", "Count", "Use", "State", ""], ["Run", "128", "Motion", "OK", ""], ["Aim", "84", "Combat", "OK", ""], ["Crouch", "42", "Stealth", "OK", ""]]) },
      rightBottom: { title: "Pose Detail", tabs: ["Pose", "Tags"], body: () => renderAdditionalDetail("pose-detail") },
      bottomTitle: "Pose Library Output",
      bottomTabs: ["Search", "Preview", "Console"],
    },
    "mocap-cleanup": {
      toolbar: ["Mocap Cleanup", "Analyze", "Smooth", "Fix Feet", "Export"],
      layout: "table-editor",
      tableRows: [["Issue", "Frame", "Bone", "State", ""], ["Foot Slide", "142", "foot_l", "Warn", ""], ["Knee Pop", "184", "calf_r", "Warn", ""], ["Root Drift", "220", "root", "Auto", ""], ["Hand Jitter", "244", "hand_l", "Auto", ""]],
      panelTitle: "Cleanup Summary",
      panelRows: [["Take", "Take_042"], ["Issues", "18"], ["Autofix", "12"], ["Manual", "6"]],
      leftTop: { title: "Takes", tabs: ["Takes", "Filters"], body: () => navList(["Take_042", "Take_043", "Run_Cycle", "Vault_A", "Combat_A"], 0) },
      rightTop: { title: "Cleanup Filters", tabs: ["Filters", "Ranges"], body: () => renderMiniDataGrid([["Filter", "Strength", "Frames", "State", ""], ["Smooth", "0.42", "120", "OK", ""], ["Foot Lock", "0.80", "42", "Warn", ""], ["Root Align", "0.60", "64", "OK", ""]]) },
      rightBottom: { title: "Take Detail", tabs: ["Issue", "Filter"], body: () => renderAdditionalDetail("mocap-detail") },
      bottomTitle: "Mocap Output",
      bottomTabs: ["Analyze", "Fixes", "Console"],
    },
    "animation-compression": {
      toolbar: ["Animation Compression", "Analyze", "Compress", "Compare", "Export"],
      layout: "table-editor",
      tableRows: [["Clip", "Original", "Compressed", "State", ""], ["Run_Start", "4.2 MB", "1.2 MB", "OK", ""], ["Attack_A", "6.8 MB", "2.1 MB", "OK", ""], ["Crouch_Idle", "2.4 MB", "0.8 MB", "OK", ""], ["Turn_180", "3.1 MB", "1.4 MB", "Warn", ""]],
      panelTitle: "Compression Summary",
      panelRows: [["Clips", "42"], ["Saved", "68%"], ["Max Error", "0.22"], ["Warnings", "2"]],
      leftTop: { title: "Clips", tabs: ["Clips", "Profiles"], body: () => navList(["Run_Start", "Attack_A", "Crouch_Idle", "Turn_180", "Vault_A"], 0) },
      rightTop: { title: "Error Metrics", tabs: ["Errors", "Profiles"], body: () => renderMiniDataGrid([["Clip", "Error", "Ratio", "State", ""], ["Run_Start", "0.08", "71%", "OK", ""], ["Attack_A", "0.12", "69%", "OK", ""], ["Turn_180", "0.22", "55%", "Warn", ""]]) },
      rightBottom: { title: "Compression Detail", tabs: ["Clip", "Profile"], body: () => renderAdditionalDetail("animation-compression-detail") },
      bottomTitle: "Compression Output",
      bottomTabs: ["Analyze", "Compress", "Console"],
    },
    "root-motion": {
      toolbar: ["Root Motion", "Extract", "Preview", "Normalize", "Export"],
      layout: "metrics-graph",
      nodes: [["Root", 12, 30], ["Velocity", 34, 18], ["Turn", 58, 36], ["Trajectory", 78, 58]],
      metrics: [["4.2", "Meters"], ["1.8", "m/s"], ["32", "Frames"], ["0", "Drift"]],
      panelTitle: "Motion Summary",
      panelRows: [["Clip", "DashStrike"], ["Distance", "4.2 m"], ["Frames", "32"], ["Drift", "0.0"]],
      leftTop: { title: "Clips", tabs: ["Clips", "Profiles"], body: () => navList(["DashStrike", "Roll_Fwd", "Vault_A", "Attack_Lunge", "Dodge_Left"], 0) },
      rightTop: { title: "Trajectory Checks", tabs: ["Trajectory", "Curves"], body: () => renderMiniDataGrid([["Check", "Value", "Limit", "State", ""], ["Distance", "4.2", "4.5", "OK", ""], ["Drift", "0.0", "0.1", "OK", ""], ["Turn", "18 deg", "20 deg", "OK", ""]]) },
      rightBottom: { title: "Motion Detail", tabs: ["Motion", "Curves"], body: () => renderAdditionalDetail("root-motion-detail") },
      bottomTitle: "Root Motion Output",
      bottomTabs: ["Extract", "Preview", "Console"],
    },
    "event-tracks": {
      toolbar: ["Event Tracks", "Add Notify", "Validate", "Preview", "Export"],
      layout: "table-editor",
      tableRows: [["Event", "Frame", "Track", "State", ""], ["HitWindowOpen", "18", "Combat", "Valid", ""], ["SFX_Swing", "22", "Audio", "Valid", ""], ["CameraShake", "24", "Camera", "Valid", ""], ["HitWindowClose", "32", "Combat", "Valid", ""]],
      panelTitle: "Track Summary",
      panelRows: [["Montage", "Attack_Montage"], ["Notifies", "14"], ["Warnings", "0"], ["Tracks", "4"]],
      leftTop: { title: "Tracks", tabs: ["Tracks", "Montages"], body: () => navList(["Combat", "Audio", "Camera", "Gameplay", "Debug"], 0) },
      rightTop: { title: "Notify Rules", tabs: ["Rules", "Conflicts"], body: () => renderMiniDataGrid([["Rule", "Events", "Conflicts", "State", ""], ["Hit Window", "2", "0", "OK", ""], ["Audio", "4", "0", "OK", ""], ["Camera", "1", "0", "OK", ""]]) },
      rightBottom: { title: "Event Detail", tabs: ["Event", "Rule"], body: () => renderAdditionalDetail("event-track-detail") },
      bottomTitle: "Event Track Output",
      bottomTabs: ["Validate", "Preview", "Console"],
    },
    "montage-debugger": {
      toolbar: ["Montage Debugger", "Trace", "Section", "Blend", "Export"],
      layout: "table-editor",
      tableRows: [["Section", "Time", "Blend", "State", ""], ["Intro", "0.0", "0.10", "Done", ""], ["Attack_A", "0.8", "0.18", "Active", ""], ["Recover", "1.8", "0.22", "Queued", ""], ["Exit", "2.4", "0.12", "Ready", ""]],
      panelTitle: "Montage Summary",
      panelRows: [["Montage", "Attack_Montage"], ["Section", "Attack_A"], ["Blend", "0.18"], ["Events", "14"]],
      leftTop: { title: "Montages", tabs: ["Montages", "Sections"], body: () => navList(["Attack_Montage", "Dodge_Montage", "Reload_Montage", "Vault_Montage", "HitReact"], 0) },
      rightTop: { title: "Blend Watches", tabs: ["Blends", "Events"], body: () => renderMiniDataGrid([["Watch", "Value", "Target", "State", ""], ["Weight", "0.84", "1.0", "OK", ""], ["BlendOut", "0.18", "0.20", "OK", ""], ["NotifyCount", "14", "14", "OK", ""]]) },
      rightBottom: { title: "Montage Detail", tabs: ["Section", "Trace"], body: () => renderAdditionalDetail("montage-debugger-detail") },
      bottomTitle: "Montage Debug Output",
      bottomTabs: ["Trace", "Events", "Console"],
    },
    "widget-tree-debugger": {
      toolbar: ["Widget Tree", "Inspect", "Highlight", "Trace", "Export"],
      layout: "table-editor",
      tableRows: [["Widget", "Type", "Depth", "State", ""], ["Combat_HUD", "Root", "0", "Visible", ""], ["HealthBar", "Progress", "2", "Visible", ""], ["AmmoCounter", "Text", "2", "Visible", ""], ["DamageOverlay", "Image", "1", "Hidden", ""]],
      panelTitle: "Widget Summary",
      panelRows: [["Tree", "Combat_HUD"], ["Widgets", "128"], ["Invalidated", "4"], ["Cost", "1.2 ms"]],
      leftTop: { title: "Widgets", tabs: ["Widgets", "Screens"], body: () => navList(["Combat_HUD", "Inventory", "PauseMenu", "DialogBox", "Tooltip"], 0) },
      rightTop: { title: "Layout Watches", tabs: ["Watches", "Invalidation"], body: () => renderMiniDataGrid([["Widget", "Size", "Invalid", "State", ""], ["HealthBar", "240x18", "No", "OK", ""], ["AmmoCounter", "80x24", "No", "OK", ""], ["DamageOverlay", "Full", "Yes", "Warn", ""]]) },
      rightBottom: { title: "Widget Detail", tabs: ["Widget", "Layout"], body: () => renderAdditionalDetail("widget-tree-detail") },
      bottomTitle: "Widget Tree Output",
      bottomTabs: ["Inspect", "Invalidation", "Console"],
    },
    "layout-constraint-solver": {
      toolbar: ["Layout Solver", "Solve", "Pin", "Inspect", "Export"],
      layout: "table-editor",
      tableRows: [["Constraint", "Target", "Value", "State", ""], ["Min Width", "InventoryPanel", "320", "OK", ""], ["Max Width", "InventoryPanel", "720", "OK", ""], ["Anchor", "SlotGrid", "Fill", "OK", ""], ["Baseline", "ItemName", "Text", "OK", ""]],
      panelTitle: "Solve Summary",
      panelRows: [["Layout", "InventoryPanel"], ["Constraints", "42"], ["Conflicts", "0"], ["Passes", "2"]],
      leftTop: { title: "Layouts", tabs: ["Layouts", "Constraints"], body: () => navList(["InventoryPanel", "Combat_HUD", "PauseMenu", "Settings", "DialogBox"], 0) },
      rightTop: { title: "Solve Watches", tabs: ["Watches", "Conflicts"], body: () => renderMiniDataGrid([["Target", "Wanted", "Actual", "State", ""], ["SlotGrid", "Fill", "Fill", "OK", ""], ["ItemName", "120", "120", "OK", ""], ["Footer", "Auto", "Auto", "OK", ""]]) },
      rightBottom: { title: "Constraint Detail", tabs: ["Constraint", "Solve"], body: () => renderAdditionalDetail("layout-constraint-detail") },
      bottomTitle: "Layout Solve Output",
      bottomTabs: ["Solve", "Conflicts", "Console"],
    },
    "theme-variant-preview": {
      toolbar: ["Theme Preview", "Variant", "Contrast", "Apply", "Export"],
      layout: "table-editor",
      tableRows: [["Token", "Value", "Usage", "State", ""], ["surface.base", "#111416", "Panels", "OK", ""], ["accent.teal", "#3cc7d6", "Active", "OK", ""], ["text.primary", "#f4f8fb", "Text", "OK", ""], ["warning", "#f2c94c", "Alerts", "AA", ""]],
      panelTitle: "Theme Summary",
      panelRows: [["Theme", "Workbench Dark"], ["Contrast", "AA"], ["Tokens", "84"], ["Warnings", "0"]],
      leftTop: { title: "Themes", tabs: ["Themes", "Variants"], body: () => navList(["Workbench Dark", "Workbench Light", "High Contrast", "Console Dim", "Review"], 0) },
      rightTop: { title: "Contrast Checks", tabs: ["Checks", "Usage"], body: () => renderMiniDataGrid([["Pair", "Ratio", "Rule", "State", ""], ["Text/Surface", "12.8", "AA", "OK", ""], ["Accent/Base", "5.2", "AA", "OK", ""], ["Warning/Base", "8.4", "AA", "OK", ""]]) },
      rightBottom: { title: "Token Detail", tabs: ["Token", "Usage"], body: () => renderAdditionalDetail("theme-variant-detail") },
      bottomTitle: "Theme Preview Output",
      bottomTabs: ["Contrast", "Apply", "Console"],
    },
    "localization-preview": {
      toolbar: ["Localization Preview", "Locale", "Expand", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Key", "Source", "Localized", "State", ""], ["ui.start", "Start", "开始", "OK", ""], ["ui.options", "Options", "选项", "OK", ""], ["ui.inventory", "Inventory", "物品栏", "OK", ""], ["ui.missing", "Missing", "-", "Missing", ""]],
      panelTitle: "Locale Summary",
      panelRows: [["Locale", "zh-CN"], ["Coverage", "97%"], ["Missing", "1"], ["Overflow", "2"]],
      leftTop: { title: "Locales", tabs: ["Locales", "Screens"], body: () => navList(["zh-CN", "en-US", "ja-JP", "de-DE", "fr-FR"], 0) },
      rightTop: { title: "Expansion Checks", tabs: ["Expansion", "Missing"], body: () => renderMiniDataGrid([["Screen", "Overflow", "Missing", "State", ""], ["MainMenu", "0", "0", "OK", ""], ["Inventory", "2", "0", "Warn", ""], ["Settings", "0", "1", "Missing", ""]]) },
      rightBottom: { title: "String Detail", tabs: ["String", "Usage"], body: () => renderAdditionalDetail("localization-preview-detail") },
      bottomTitle: "Localization Output",
      bottomTabs: ["Validate", "Overflow", "Console"],
    },
    "focus-navigation": {
      toolbar: ["Focus Navigation", "Trace", "Validate", "Route", "Export"],
      layout: "graph",
      nodes: [["Grid", 12, 30], ["Item", 34, 18], ["Details", 58, 36], ["Footer", 78, 58]],
      panelTitle: "Focus Summary",
      panelRows: [["Screen", "Inventory"], ["Nodes", "42"], ["Dead Ends", "0"], ["Loops", "1"]],
      leftTop: { title: "Screens", tabs: ["Screens", "Modes"], body: () => navList(["Inventory", "MainMenu", "Settings", "Pause", "Dialog"], 0) },
      rightTop: { title: "Route Checks", tabs: ["Routes", "Issues"], body: () => renderMiniDataGrid([["Route", "From", "To", "State", ""], ["Right", "Grid", "Details", "OK", ""], ["Down", "Item", "Footer", "OK", ""], ["Back", "Details", "Grid", "OK", ""]]) },
      rightBottom: { title: "Focus Detail", tabs: ["Node", "Route"], body: () => renderAdditionalDetail("focus-navigation-detail") },
      bottomTitle: "Focus Output",
      bottomTabs: ["Trace", "Validate", "Console"],
    },
    "input-glyph-mapper": {
      toolbar: ["Glyph Mapper", "Map", "Preview", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Action", "Device", "Glyph", "State", ""], ["Confirm", "Xbox", "A", "Mapped", ""], ["Cancel", "Xbox", "B", "Mapped", ""], ["Inventory", "Xbox", "Y", "Mapped", ""], ["Dash", "Xbox", "RB", "Mapped", ""]],
      panelTitle: "Glyph Summary",
      panelRows: [["Device", "Xbox"], ["Prompts", "64"], ["Missing", "0"], ["Locales", "8"]],
      leftTop: { title: "Devices", tabs: ["Devices", "Sets"], body: () => navList(["Xbox", "PlayStation", "Keyboard", "SteamDeck", "Switch"], 0) },
      rightTop: { title: "Platform Mapping", tabs: ["Mapping", "Missing"], body: () => renderMiniDataGrid([["Platform", "Prompts", "Missing", "State", ""], ["Xbox", "64", "0", "OK", ""], ["PS", "64", "0", "OK", ""], ["Keyboard", "64", "0", "OK", ""]]) },
      rightBottom: { title: "Glyph Detail", tabs: ["Glyph", "Locale"], body: () => renderAdditionalDetail("input-glyph-detail") },
      bottomTitle: "Glyph Output",
      bottomTabs: ["Validate", "Preview", "Console"],
    },
    "ui-snapshot-diff": {
      toolbar: ["Snapshot Diff", "Compare", "Overlay", "Accept", "Export"],
      layout: "table-editor",
      tableRows: [["Snapshot", "Screen", "Diff", "State", ""], ["1841 -> 1842", "Inventory", "4", "Review", ""], ["1841 -> 1842", "HUD", "1", "OK", ""], ["1841 -> 1842", "Menu", "0", "OK", ""], ["1841 -> 1842", "Settings", "2", "Warn", ""]],
      panelTitle: "Diff Summary",
      panelRows: [["Base", "1841"], ["Compare", "1842"], ["Diffs", "4"], ["Accepted", "1"]],
      leftTop: { title: "Snapshots", tabs: ["Snapshots", "Screens"], body: () => navList(["Inventory", "Combat_HUD", "MainMenu", "Settings", "Dialog"], 0) },
      rightTop: { title: "Visual Checks", tabs: ["Checks", "Regions"], body: () => renderMiniDataGrid([["Region", "Delta", "Severity", "State", ""], ["SlotGrid", "4 px", "Low", "Review", ""], ["Footer", "0 px", "None", "OK", ""], ["Header", "2 px", "Low", "Warn", ""]]) },
      rightBottom: { title: "Snapshot Detail", tabs: ["Diff", "Region"], body: () => renderAdditionalDetail("ui-snapshot-detail") },
      bottomTitle: "Snapshot Output",
      bottomTabs: ["Compare", "Overlay", "Console"],
    },
    "widget-performance": {
      toolbar: ["Widget Performance", "Capture", "Invalidate", "Budget", "Export"],
      layout: "table-editor",
      tableRows: [["Widget", "Cost", "Invalidations", "State", ""], ["Combat_HUD", "1.2 ms", "4", "OK", ""], ["Inventory", "2.4 ms", "12", "Warn", ""], ["MenuFlow", "0.8 ms", "2", "OK", ""], ["Tooltip", "0.2 ms", "1", "OK", ""]],
      panelTitle: "Performance Summary",
      panelRows: [["Screen", "Combat_HUD"], ["Cost", "1.2 ms"], ["Budget", "2.0 ms"], ["Warnings", "0"]],
      leftTop: { title: "Widgets", tabs: ["Widgets", "Screens"], body: () => navList(["Combat_HUD", "Inventory", "MenuFlow", "Tooltip", "Dialog"], 0) },
      rightTop: { title: "Invalidation Groups", tabs: ["Groups", "Costs"], body: () => renderMiniDataGrid([["Group", "Cost", "Invalid", "State", ""], ["HUD Root", "0.6", "2", "OK", ""], ["Bars", "0.4", "1", "OK", ""], ["Overlay", "0.2", "1", "OK", ""]]) },
      rightBottom: { title: "Widget Perf Detail", tabs: ["Widget", "Invalidation"], body: () => renderAdditionalDetail("widget-performance-detail") },
      bottomTitle: "Widget Performance Output",
      bottomTabs: ["Capture", "Budget", "Console"],
    },
    "world-partition": {
      toolbar: ["World Partition", "Load", "Unload", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Cell", "Layer", "Memory", "State", ""], ["A1_04", "Gameplay", "184 MB", "Loaded", ""], ["A1_05", "Art", "92 MB", "Visible", ""], ["Dock_01", "World", "0 MB", "Unloaded", ""], ["Roof_02", "Gameplay", "64 MB", "Loaded", ""]],
      panelTitle: "Partition Summary",
      panelRows: [["World", "A1_Hangar"], ["Cells", "42"], ["Loaded", "12"], ["Memory", "348 MB"]],
      leftTop: { title: "Cells", tabs: ["Cells", "Layers"], body: () => navList(["A1_04", "A1_05", "Dock_01", "Roof_02", "Arena_01"], 0) },
      rightTop: { title: "Streaming Layers", tabs: ["Layers", "Rules"], body: () => renderMiniDataGrid([["Layer", "Cells", "Loaded", "State", ""], ["Gameplay", "18", "8", "OK", ""], ["Art", "14", "3", "OK", ""], ["World", "10", "1", "OK", ""]]) },
      rightBottom: { title: "Cell Detail", tabs: ["Cell", "Layer"], body: () => renderAdditionalDetail("world-partition-detail") },
      bottomTitle: "Partition Output",
      bottomTabs: ["Loads", "Memory", "Console"],
    },
    "hlod-builder": {
      toolbar: ["HLOD Builder", "Cluster", "Build", "Preview", "Export"],
      layout: "table-editor",
      tableRows: [["Cluster", "Actors", "Triangles", "State", ""], ["HLOD_A", "42", "18k", "Ready", ""], ["HLOD_B", "28", "12k", "Ready", ""], ["HLOD_C", "18", "8k", "Warn", ""], ["HLOD_D", "64", "24k", "Ready", ""]],
      panelTitle: "HLOD Summary",
      panelRows: [["Level", "Hangar_Exterior"], ["Clusters", "12"], ["Reduction", "82%"], ["Warnings", "1"]],
      leftTop: { title: "Clusters", tabs: ["Clusters", "Profiles"], body: () => navList(["HLOD_A", "HLOD_B", "HLOD_C", "HLOD_D", "Debug"], 0) },
      rightTop: { title: "Merge Rules", tabs: ["Rules", "Materials"], body: () => renderMiniDataGrid([["Rule", "Scope", "Value", "State", ""], ["Distance", "World", "120m", "OK", ""], ["Material", "Cluster", "Merge", "OK", ""], ["Collision", "Proxy", "Simple", "Warn", ""]]) },
      rightBottom: { title: "Cluster Detail", tabs: ["Cluster", "Build"], body: () => renderAdditionalDetail("hlod-detail") },
      bottomTitle: "HLOD Output",
      bottomTabs: ["Build", "Warnings", "Console"],
    },
    "level-instance": {
      toolbar: ["Level Instance", "Open", "Override", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Instance", "Source", "Overrides", "State", ""], ["DockModule_A_01", "DockModule_A", "3", "Valid", ""], ["DockModule_A_02", "DockModule_A", "1", "Valid", ""], ["HangarBay_B_01", "HangarBay_B", "4", "Warn", ""], ["RoomKit_C_04", "RoomKit_C", "0", "Valid", ""]],
      panelTitle: "Instance Summary",
      panelRows: [["Source", "DockModule_A"], ["Instances", "6"], ["Overrides", "12"], ["Warnings", "1"]],
      leftTop: { title: "Instances", tabs: ["Instances", "Sources"], body: () => navList(["DockModule_A", "HangarBay_B", "RoomKit_C", "Corridor_D", "Debug"], 0) },
      rightTop: { title: "Overrides", tabs: ["Overrides", "Refs"], body: () => renderMiniDataGrid([["Override", "Instance", "Value", "State", ""], ["Lighting", "A_01", "Night", "OK", ""], ["Props", "A_02", "Reduced", "OK", ""], ["Collision", "B_01", "Custom", "Warn", ""]]) },
      rightBottom: { title: "Instance Detail", tabs: ["Instance", "Overrides"], body: () => renderAdditionalDetail("level-instance-detail") },
      bottomTitle: "Instance Output",
      bottomTabs: ["Validation", "Overrides", "Console"],
    },
    "streaming-profiler": {
      toolbar: ["Streaming Profiler", "Capture", "Mark", "Compare", "Export"],
      layout: "metrics-graph",
      nodes: [["Request", 12, 28], ["Load", 34, 18], ["Visible", 58, 38], ["Unload", 78, 58]],
      metrics: [["184", "MB peak"], ["12", "Loads"], ["3", "Stalls"], ["0.8", "s max"]],
      panelTitle: "Streaming Summary",
      panelRows: [["Capture", "Live"], ["Peak", "184 MB"], ["Stalls", "3"], ["Max Load", "0.8 s"]],
      leftTop: { title: "Captures", tabs: ["Captures", "Cells"], body: () => navList(["Live", "Dock Run", "Arena Load", "Baseline", "Stress"], 0) },
      rightTop: { title: "Memory Bands", tabs: ["Memory", "Events"], body: () => renderMiniDataGrid([["Band", "Peak", "Events", "State", ""], ["Cells", "184 MB", "12", "OK", ""], ["Textures", "92 MB", "18", "Warn", ""], ["Audio", "24 MB", "4", "OK", ""]]) },
      rightBottom: { title: "Stream Detail", tabs: ["Event", "Memory"], body: () => renderAdditionalDetail("streaming-profiler-detail") },
      bottomTitle: "Streaming Profiler Output",
      bottomTabs: ["Capture", "Stalls", "Console"],
    },
    "scene-bookmarks": {
      toolbar: ["Scene Bookmarks", "Jump", "Save", "Share", "Export"],
      layout: "table-editor",
      tableRows: [["Bookmark", "Camera", "Owner", "State", ""], ["Entrance", "Cam_A", "Level", "Ready", ""], ["BossDoor", "Cam_B", "Design", "Ready", ""], ["LightingRef", "Cam_C", "Lighting", "Ready", ""], ["Bug_1842", "Cam_Debug", "QA", "Pinned", ""]],
      panelTitle: "Bookmark Summary",
      panelRows: [["Scene", "A1_Hangar"], ["Bookmarks", "18"], ["Pinned", "3"], ["Shared", "12"]],
      leftTop: { title: "Bookmarks", tabs: ["Bookmarks", "Sets"], body: () => navList(["Entrance", "BossDoor", "LightingRef", "Bug_1842", "DockView"], 0) },
      rightTop: { title: "Camera Targets", tabs: ["Targets", "Owners"], body: () => renderMiniDataGrid([["Target", "Owner", "FOV", "State", ""], ["Entrance", "Level", "60", "OK", ""], ["BossDoor", "Design", "55", "OK", ""], ["Bug_1842", "QA", "70", "Pinned", ""]]) },
      rightBottom: { title: "Bookmark Detail", tabs: ["Bookmark", "Camera"], body: () => renderAdditionalDetail("scene-bookmark-detail") },
      bottomTitle: "Bookmark Output",
      bottomTabs: ["Jump", "Share", "Console"],
    },
    "spawn-point-editor": {
      toolbar: ["Spawn Points", "Place", "Validate", "Simulate", "Export"],
      layout: "table-editor",
      tableRows: [["Point", "Type", "Weight", "State", ""], ["Spawn_01", "Enemy", "1.0", "Valid", ""], ["Spawn_02", "Enemy", "0.8", "Valid", ""], ["Spawn_03", "Loot", "0.2", "Warn", ""], ["Spawn_04", "Player", "1.0", "Valid", ""]],
      panelTitle: "Spawn Summary",
      panelRows: [["Set", "HangarWave_A"], ["Points", "32"], ["Valid", "31"], ["Warnings", "1"]],
      leftTop: { title: "Spawn Sets", tabs: ["Sets", "Rules"], body: () => navList(["HangarWave_A", "DockWave_B", "Tutorial", "BossArena", "Debug"], 0) },
      rightTop: { title: "Validation Checks", tabs: ["Checks", "Weights"], body: () => renderMiniDataGrid([["Check", "Points", "Issues", "State", ""], ["Reachable", "32", "0", "OK", ""], ["Overlap", "32", "1", "Warn", ""], ["Budget", "32", "0", "OK", ""]]) },
      rightBottom: { title: "Spawn Detail", tabs: ["Point", "Rules"], body: () => renderAdditionalDetail("spawn-point-detail") },
      bottomTitle: "Spawn Output",
      bottomTabs: ["Validation", "Simulation", "Console"],
    },
    "collision-matrix": {
      toolbar: ["Collision Matrix", "Edit", "Validate", "Preview", "Export"],
      layout: "table-editor",
      tableRows: [["Channel", "World", "Pawn", "Trace", ""], ["WorldStatic", "Block", "Block", "Block", ""], ["WorldDynamic", "Block", "Overlap", "Block", ""], ["Pawn", "Block", "Ignore", "Overlap", ""], ["Projectile", "Block", "Block", "Block", ""]],
      panelTitle: "Matrix Summary",
      panelRows: [["Channels", "18"], ["Rules", "324"], ["Conflicts", "0"], ["Profiles", "12"]],
      leftTop: { title: "Channels", tabs: ["Channels", "Profiles"], body: () => navList(["WorldStatic", "WorldDynamic", "Pawn", "Projectile", "Trigger"], 0) },
      rightTop: { title: "Response Rules", tabs: ["Rules", "Conflicts"], body: () => renderMiniDataGrid([["Profile", "Rules", "Conflicts", "State", ""], ["Default", "324", "0", "OK", ""], ["Projectile", "18", "0", "OK", ""], ["Trigger", "18", "0", "OK", ""]]) },
      rightBottom: { title: "Channel Detail", tabs: ["Channel", "Profile"], body: () => renderAdditionalDetail("collision-matrix-detail") },
      bottomTitle: "Collision Matrix Output",
      bottomTabs: ["Validation", "Preview", "Console"],
    },
    "environment-probes": {
      toolbar: ["Environment Probes", "Bake", "Place", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Probe", "Type", "Coverage", "State", ""], ["Probe_A12", "Irradiance", "92%", "Ready", ""], ["Probe_A13", "Reflection", "88%", "Ready", ""], ["Probe_Dock", "Irradiance", "74%", "Warn", ""], ["Probe_Roof", "Reflection", "96%", "Ready", ""]],
      panelTitle: "Probe Summary",
      panelRows: [["Set", "Hangar"], ["Probes", "64"], ["Coverage", "92%"], ["Warnings", "2"]],
      leftTop: { title: "Probes", tabs: ["Probes", "Sets"], body: () => navList(["Hangar", "Dock", "Roof", "Arena", "Debug"], 0) },
      rightTop: { title: "Bake Queue", tabs: ["Queue", "Coverage"], body: () => renderMiniDataGrid([["Probe", "Samples", "Coverage", "State", ""], ["Probe_A12", "128", "92%", "Ready", ""], ["Probe_A13", "128", "88%", "Ready", ""], ["Probe_Dock", "128", "74%", "Warn", ""]]) },
      rightBottom: { title: "Probe Detail", tabs: ["Probe", "Bake"], body: () => renderAdditionalDetail("environment-probe-detail") },
      bottomTitle: "Probe Bake Output",
      bottomTabs: ["Bake", "Coverage", "Console"],
    },
    "feature-flags": {
      toolbar: ["Feature Flags", "Rollout", "Stage", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Flag", "Audience", "Rollout", "State", ""], ["new_inventory", "Beta", "25%", "Live", ""], ["store_banner", "All", "10%", "Staged", ""], ["netcode_v2", "Internal", "100%", "Live", ""], ["holiday_event", "Asia", "0%", "Draft", ""]],
      panelTitle: "Flag Summary",
      panelRows: [["Live", "12"], ["Staged", "2"], ["Draft", "4"], ["Warnings", "1"]],
      leftTop: { title: "Flags", tabs: ["Flags", "Audiences"], body: () => navList(["new_inventory", "store_banner", "netcode_v2", "holiday_event", "debug_menu"], 0) },
      rightTop: { title: "Audience Rules", tabs: ["Rules", "Regions"], body: () => renderMiniDataGrid([["Rule", "Audience", "Users", "State", ""], ["Beta", "Opt-in", "8.4k", "OK", ""], ["All", "Global", "42k", "OK", ""], ["Asia", "Region", "12k", "Warn", ""]]) },
      rightBottom: { title: "Flag Detail", tabs: ["Flag", "Rollout"], body: () => renderAdditionalDetail("feature-flag-detail") },
      bottomTitle: "Flag Output",
      bottomTabs: ["Rollout", "Validation", "Console"],
    },
    "remote-config": {
      toolbar: ["Remote Config", "Publish", "Diff", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Key", "Value", "Scope", "State", ""], ["store.banner", "Spring", "Global", "Live", ""], ["match.timeout", "45", "Matchmaking", "Live", ""], ["ui.theme", "workbench_dark", "Editor", "Draft", ""], ["loot.multiplier", "1.25", "Event", "Staged", ""]],
      panelTitle: "Config Summary",
      panelRows: [["Version", "Live v42"], ["Drafts", "3"], ["Changed", "8"], ["Warnings", "0"]],
      leftTop: { title: "Configs", tabs: ["Configs", "Envs"], body: () => navList(["Live v42", "Draft v43", "Canary", "Staging", "Local"], 0) },
      rightTop: { title: "Diff Preview", tabs: ["Diff", "Owners"], body: () => renderMiniDataGrid([["Key", "Old", "New", "State", ""], ["store.banner", "Winter", "Spring", "OK", ""], ["loot.multiplier", "1.0", "1.25", "OK", ""], ["ui.theme", "dark", "workbench_dark", "Draft", ""]]) },
      rightBottom: { title: "Config Detail", tabs: ["Key", "Diff"], body: () => renderAdditionalDetail("remote-config-detail") },
      bottomTitle: "Config Output",
      bottomTabs: ["Validate", "Publish", "Console"],
    },
    "telemetry-query": {
      toolbar: ["Telemetry Query", "Run", "Filter", "Chart", "Export"],
      layout: "metrics-graph",
      nodes: [["Start", 12, 24], ["Store", 34, 20], ["Purchase", 58, 38], ["Return", 78, 58]],
      metrics: [["1.2k", "Events"], ["42/s", "Rate"], ["8", "Markers"], ["3", "Alerts"]],
      panelTitle: "Query Summary",
      panelRows: [["Query", "Session Funnel"], ["Events", "1.2k"], ["Rate", "42/s"], ["Alerts", "3"]],
      leftTop: { title: "Queries", tabs: ["Queries", "Dashboards"], body: () => navList(["Session Funnel", "Store Clicks", "Match Start", "Crash Near Buy", "Retention"], 0) },
      rightTop: { title: "Dimensions", tabs: ["Dimensions", "Filters"], body: () => renderMiniDataGrid([["Dimension", "Value", "Events", "State", ""], ["Region", "Asia", "420", "OK", ""], ["Build", "0.18.0", "1.2k", "OK", ""], ["Segment", "Returning", "640", "Warn", ""]]) },
      rightBottom: { title: "Query Detail", tabs: ["Query", "Chart"], body: () => renderAdditionalDetail("telemetry-query-detail") },
      bottomTitle: "Telemetry Output",
      bottomTabs: ["Run", "Alerts", "Console"],
    },
    "patch-planner": {
      toolbar: ["Patch Planner", "Plan", "Gate", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Patch", "Type", "Risk", "State", ""], ["Fix Crash 1842", "Runtime", "High", "Ready", ""], ["Update Locale", "UI", "Low", "Ready", ""], ["Texture Fix", "Assets", "Medium", "Review", ""], ["Net Tweak", "Online", "High", "Staged", ""]],
      panelTitle: "Patch Summary",
      panelRows: [["Version", "0.18.1"], ["Changes", "24"], ["High Risk", "3"], ["Blockers", "0"]],
      leftTop: { title: "Patches", tabs: ["Patches", "Branches"], body: () => navList(["0.18.1 Hotfix", "0.18.0 Release", "0.17.9 LTS", "Canary", "Rollback"], 0) },
      rightTop: { title: "Release Gates", tabs: ["Gates", "Owners"], body: () => renderMiniDataGrid([["Gate", "Owner", "State", "Risk", ""], ["Crash Triage", "Runtime", "Ready", "High", ""], ["QA Smoke", "QA", "Ready", "Low", ""], ["Store Cert", "Release", "Review", "Med", ""]]) },
      rightBottom: { title: "Patch Detail", tabs: ["Patch", "Risk"], body: () => renderAdditionalDetail("patch-planner-detail") },
      bottomTitle: "Patch Output",
      bottomTabs: ["Validation", "Build", "Console"],
    },
    "dlc-catalog": {
      toolbar: ["DLC Catalog", "Pack", "Entitlements", "Validate", "Export"],
      layout: "table-editor",
      tableRows: [["Pack", "SKU", "Items", "State", ""], ["FounderPack", "dlc_founder", "3", "Ready", ""], ["ArenaPack", "dlc_arena", "8", "Live", ""], ["SkinSet_A", "dlc_skin_a", "12", "Draft", ""], ["Soundtrack", "dlc_ost", "18", "Ready", ""]],
      panelTitle: "Catalog Summary",
      panelRows: [["Packs", "8"], ["Ready", "3"], ["Live", "4"], ["Drafts", "1"]],
      leftTop: { title: "DLC", tabs: ["Packs", "SKUs"], body: () => navList(["FounderPack", "ArenaPack", "SkinSet_A", "Soundtrack", "Deluxe"], 0) },
      rightTop: { title: "Entitlements", tabs: ["Entitlements", "Stores"], body: () => renderMiniDataGrid([["Store", "SKU", "Price", "State", ""], ["Steam", "dlc_founder", "$9.99", "Ready", ""], ["Xbox", "dlc_founder", "$9.99", "Ready", ""], ["PSN", "dlc_founder", "$9.99", "Review", ""]]) },
      rightBottom: { title: "DLC Detail", tabs: ["Pack", "Store"], body: () => renderAdditionalDetail("dlc-detail") },
      bottomTitle: "DLC Output",
      bottomTabs: ["Package", "Validate", "Console"],
    },
    "crash-symbolication": {
      toolbar: ["Crash Symbolication", "Resolve", "Symbols", "Group", "Export"],
      layout: "table-editor",
      tableRows: [["Crash", "Platform", "Resolved", "State", ""], ["Crash_1842", "Win64", "Yes", "Hot", ""], ["Crash_1818", "Linux", "No", "Symbols", ""], ["Crash_1760", "Android", "Yes", "Grouped", ""], ["Crash_1722", "Win64", "Yes", "Review", ""]],
      panelTitle: "Crash Summary",
      panelRows: [["Crashes", "184"], ["Resolved", "92%"], ["Missing Symbols", "6"], ["Hotspots", "3"]],
      leftTop: { title: "Crashes", tabs: ["Crashes", "Groups"], body: () => navList(["Crash_1842", "Crash_1818", "Crash_1760", "Crash_1722", "Renderer"], 0) },
      rightTop: { title: "Symbols", tabs: ["Symbols", "Builds"], body: () => renderMiniDataGrid([["Build", "Platform", "Symbols", "State", ""], ["0.18.0", "Win64", "Ready", "OK", ""], ["0.18.0", "Linux", "Missing", "Warn", ""], ["0.18.0", "Android", "Ready", "OK", ""]]) },
      rightBottom: { title: "Crash Detail", tabs: ["Stack", "Frames"], body: () => renderAdditionalDetail("symbolication-detail") },
      bottomTitle: "Symbolication Output",
      bottomTabs: ["Resolve", "Symbols", "Console"],
    },
    "player-segment": {
      toolbar: ["Player Segment", "Refresh", "Rules", "Preview", "Export"],
      layout: "metrics-graph",
      nodes: [["All", 12, 30], ["Returning", 36, 20], ["High Value", 58, 42], ["At Risk", 78, 58]],
      metrics: [["42k", "Users"], ["18%", "Convert"], ["3", "Cohorts"], ["2", "Alerts"]],
      panelTitle: "Segment Summary",
      panelRows: [["Segment", "Returning"], ["Users", "42k"], ["Regions", "6"], ["Alerts", "2"]],
      leftTop: { title: "Segments", tabs: ["Segments", "Rules"], body: () => navList(["Returning Players", "High Value", "New Users", "At Risk", "Lapsed"], 0) },
      rightTop: { title: "Cohorts", tabs: ["Cohorts", "Rules"], body: () => renderMiniDataGrid([["Cohort", "Users", "Rate", "State", ""], ["Returning", "42k", "18%", "Active", ""], ["High Value", "4.2k", "42%", "Active", ""], ["At Risk", "8.1k", "6%", "Warn", ""]]) },
      rightBottom: { title: "Segment Detail", tabs: ["Segment", "Rules"], body: () => renderAdditionalDetail("segment-detail") },
      bottomTitle: "Segment Output",
      bottomTabs: ["Refresh", "Preview", "Console"],
    },
    "experiment-console": {
      toolbar: ["Experiment Console", "Launch", "Traffic", "Analyze", "Export"],
      layout: "table-editor",
      tableRows: [["Experiment", "Variant", "Traffic", "State", ""], ["Store Banner", "A/B/C", "10%", "Live", ""], ["Match CTA", "A/B", "25%", "Staged", ""], ["Quest Prompt", "A/B", "0%", "Draft", ""], ["Offer Tile", "A/B/C", "5%", "Live", ""]],
      panelTitle: "Experiment Summary",
      panelRows: [["Live", "3"], ["Variants", "8"], ["Traffic", "10%"], ["Warnings", "1"]],
      leftTop: { title: "Experiments", tabs: ["Experiments", "Metrics"], body: () => navList(["Store Banner", "Match CTA", "Quest Prompt", "Offer Tile", "Onboarding"], 0) },
      rightTop: { title: "Metrics", tabs: ["Metrics", "Variants"], body: () => renderMiniDataGrid([["Metric", "A", "B", "State", ""], ["Click", "8.2%", "9.1%", "Live", ""], ["Buy", "1.8%", "2.0%", "Live", ""], ["Exit", "4.2%", "4.0%", "Watch", ""]]) },
      rightBottom: { title: "Experiment Detail", tabs: ["Experiment", "Traffic"], body: () => renderAdditionalDetail("experiment-detail") },
      bottomTitle: "Experiment Output",
      bottomTabs: ["Analyze", "Traffic", "Console"],
    },
  };
  return configs[kind] ?? configs["prefab-editor"];
}

function additionalDetailConfig(kind) {
  const map = {
    "prefab-detail": ["Prefab Overrides", "AudioZone.prefab / Variant A", [["Root", "AudioZone"], ["Overrides", "3"], ["References", "8"]], "Validate"],
    "vfx-detail": ["Emitter State", "Spark Burst emitter", [["Rate", "240/s"], ["Lifetime", "2.5 s"], ["Renderer", "Sprite"]], "Simulate"],
    "shader-detail": ["Shader Variants", "unlit.zshader", [["Keywords", "3"], ["Stage", "Fragment"], ["Target", "SPIR-V"]], "Compile"],
    "terrain-detail": ["Brush Settings", "Smooth brush", [["Size", "18 m"], ["Strength", "0.42"], ["Layer", "Dust"]], "Apply"],
    "audio-detail": ["Audio Event", "Ambience_Hangar", [["Bus", "Ambience"], ["Loop", "On"], ["Loudness", "-18 LUFS"]], "Preview"],
    "behavior-detail": ["Behavior Node", "Move To target", [["Node", "Move To"], ["Status", "Running"], ["Cost", "0.16 ms"]], "Trace"],
    "lighting-detail": ["Bake Settings", "A1_Hangar preview bake", [["Quality", "Preview"], ["Samples", "128"], ["Denoise", "On"]], "Start Bake"],
    "physics-detail": ["Collision Body", "Box_01 rigid body", [["Shape", "Box"], ["Mass", "12.0"], ["Layer", "World"]], "Simulate"],
    "level-streaming-detail": ["Streaming Cell", "A1_North / Gameplay", [["State", "Loaded"], ["Memory", "184 MB"], ["Priority", "High"]], "Reload Cell"],
    "sequencer-detail": ["Sequencer Key", "Shot 010 camera cut", [["Frame", "142"], ["Camera", "Cam_A"], ["Ease", "Bezier"]], "Set Key"],
    "navmesh-detail": ["Agent Profile", "Humanoid navigation", [["Radius", "0.42 m"], ["Height", "1.8 m"], ["Slope", "45 deg"]], "Bake"],
    "render-pipeline-detail": ["Render Pass", "Forward+ lighting pass", [["Target", "HDR_Main"], ["Batches", "42"], ["Cost", "3.1 ms"]], "Compile"],
    "input-mapping-detail": ["Input Binding", "Play / Ctrl+P", [["Device", "Keyboard"], ["Context", "Global"], ["Conflicts", "0"]], "Validate"],
    "data-table-detail": ["Data Row", "mat.metal", [["Schema", "AssetRef"], ["Status", "Valid"], ["Refs", "3"]], "Validate"],
    "network-replication-detail": ["Peer Detail", "Client A", [["Ping", "42 ms"], ["Loss", "0.2%"], ["Dirty", "12"]], "Capture"],
    "localization-detail": ["String Detail", "asset.missing / zh-CN", [["Locale", "zh-CN"], ["Status", "Missing"], ["Fallback", "en-US"]], "Review"],
    "visual-script-detail": ["Script Node", "Branch / DoorOpen", [["Pins", "3"], ["Breakpoints", "1"], ["Compile", "Clean"]], "Debug"],
    "state-machine-detail": ["Transition Detail", "Walk -> Run", [["Condition", "Speed > 3.0"], ["Blend", "0.18 s"], ["Priority", "2"]], "Preview"],
    "skeleton-mesh-detail": ["Bone Detail", "spine_02", [["Influences", "4"], ["Socket", "WeaponBack"], ["LOD", "0"]], "Paint Weights"],
    "texture-detail": ["Texture Detail", "T_Grid_01.png", [["Format", "BC7"], ["Mips", "8"], ["sRGB", "On"]], "Recompress"],
    "material-instance-detail": ["Instance Detail", "MI_Metal_Rough", [["Parent", "M_Metal"], ["Overrides", "3"], ["Users", "14"]], "Compile"],
    "prefab-variant-detail": ["Override Detail", "Variant B / Trigger", [["Overrides", "7"], ["References", "12"], ["Status", "Dirty"]], "Apply"],
    "level-audit-detail": ["Audit Issue", "Crate_07 missing collision", [["Severity", "Error"], ["Fixable", "Yes"], ["Rule", "Physics"]], "Fix Issue"],
    "test-runner-detail": ["Test Failure", "viewport_resize", [["Suite", "Editor Host"], ["Line", "184"], ["Expected", "1672x941"]], "Debug Test"],
    "frame-debugger-detail": ["Draw Detail", "Lighting pass / draw #064", [["Pipeline", "Forward+"], ["Target", "HDR_Main"], ["Cost", "0.42 ms"]], "Open State"],
    "memory-profiler-detail": ["Allocation Detail", "Renderer heap / texture pool", [["Size", "412 MB"], ["Delta", "+18 MB"], ["Stack", "3 frames"]], "Open Stack"],
    "asset-dependency-detail": ["Asset Path", "A1_Hangar -> M_Metal", [["Depth", "2"], ["Refs", "18"], ["Missing", "0"]], "Trace Path"],
    "reference-finder-detail": ["Owner Reference", "A1_Hangar.scene / Props", [["Use", "Mesh"], ["Writable", "Yes"], ["Replace", "Allowed"]], "Replace Ref"],
    "cook-package-detail": ["Package Detail", "Windows Development", [["Step", "Cook Assets"], ["Progress", "62%"], ["Warnings", "2"]], "Open Artifacts"],
    "crash-session-detail": ["Stack Detail", "Surface acquire panic", [["Frame", "1842"], ["Thread", "Render"], ["Recovered", "Yes"]], "Open Stack"],
    "log-analysis-detail": ["Log Detail", "Texture fallback warning", [["Channel", "Assets"], ["Hits", "2"], ["Owner", "Importer"]], "Create Task"],
    "automation-report-detail": ["Report Failure", "viewport_resize smoke", [["Suite", "Editor"], ["Artifacts", "3"], ["Owner", "UI Host"]], "Open Artifact"],
    "layout-manager-detail": ["Layout Detail", "default-workbench", [["Zones", "5"], ["Splits", "2"], ["Dirty", "No"]], "Apply Layout"],
    "theme-token-detail": ["Token Detail", "accent.teal", [["Value", "#3cc7d6"], ["Usage", "Active"], ["Contrast", "AA"]], "Preview Token"],
    "command-center-detail": ["Command Detail", "Toggle Console", [["Shortcut", "Alt+4"], ["Context", "Workbench"], ["Conflicts", "0"]], "Edit Binding"],
    "module-graph-detail": ["Module Detail", "zircon_runtime", [["Modules", "18"], ["Cycles", "0"], ["Warnings", "2"]], "Open Boundary"],
    "asset-validation-detail": ["Repair Detail", "T_Grid_01 compression", [["Severity", "Warning"], ["Fix", "Auto"], ["Rule", "Texture"]], "Repair Asset"],
    "hot-reload-detail": ["Reload Detail", "design.css", [["Module", "Editor UI"], ["Risk", "Medium"], ["Rollback", "Yes"]], "Reload Module"],
    "project-history-detail": ["Change Detail", "Added frame debugger page", [["Files", "4"], ["Assets", "1"], ["Recover", "Yes"]], "Open Diff"],
    "task-board-detail": ["Task Detail", "Workbench PNG batch", [["Owner", "UI"], ["State", "In Review"], ["Priority", "High"]], "Open Review"],
    "source-control-detail": ["Submit Detail", "Workbench PNGs changelist", [["Files", "14"], ["Conflicts", "0"], ["Shelved", "2"]], "Submit"],
    "review-comments-detail": ["Comment Detail", "Right drawer density", [["Thread", "Open"], ["Owner", "UI"], ["Blocking", "Yes"]], "Reply"],
    "build-farm-detail": ["Worker Detail", "win-builder-01", [["State", "Running"], ["Job", "editor-ui"], ["Health", "92%"]], "Open Job"],
    "release-notes-detail": ["Release Change", "Workbench PNG export", [["Version", "0.18.0"], ["Section", "Editor UI"], ["State", "Ready"]], "Mark Ready"],
    "project-settings-detail": ["Setting Detail", "Default Map", [["Value", "A1_Hangar"], ["Scope", "Runtime"], ["Dirty", "No"]], "Apply Setting"],
    "plugin-development-detail": ["Plugin Detail", "editor.tools.validation", [["Modules", "3"], ["Hooks", "6"], ["Warnings", "1"]], "Build Plugin"],
    "remote-device-detail": ["Device Detail", "Windows-DevKit-01", [["Status", "Connected"], ["Ping", "42 ms"], ["Build", "Current"]], "Deploy"],
    "session-sync-detail": ["Sync Conflict", "Console Filter", [["Owner", "BuildBot"], ["Peers", "3"], ["Conflict", "Yes"]], "Resolve"],
    "cutscene-detail": ["Shot Detail", "Intro_Hangar / Shot 020", [["Camera", "Cam_Crane"], ["Duration", "6.4 s"], ["Keys", "18"]], "Set Key"],
    "dialogue-detail": ["Line Detail", "Captain / L_002", [["Locale", "zh-CN"], ["State", "Review"], ["Branches", "2"]], "Preview Line"],
    "quest-detail": ["Condition Detail", "Restore Power gate", [["Objective", "Restore Power"], ["Gate", "PowerOn"], ["Reward", "XP 120"]], "Simulate"],
    "camera-rig-detail": ["Lens Detail", "Crane_A / 35mm", [["Focus", "4.2 m"], ["Aperture", "2.8"], ["Noise", "0.18"]], "Preview"],
    "control-rig-detail": ["Solver Detail", "Spine IK", [["Controls", "48"], ["Solve", "0.38 ms"], ["Warning", "1"]], "Bake Pose"],
    "motion-matching-detail": ["Pose Detail", "Pose 142 / Run_Start", [["Cost", "0.42"], ["Clip", "Run_Start"], ["Frame", "18"]], "Preview Pose"],
    "facial-animation-detail": ["Expression Detail", "Captain_Line_03", [["Curves", "52"], ["Phonemes", "18"], ["Quality", "92%"]], "Solve"],
    "blend-space-detail": ["Sample Detail", "Run_Fwd", [["Speed", "6.0"], ["Direction", "0 deg"], ["Weight", "0.84"]], "Preview Sample"],
    "foliage-detail": ["Instance Detail", "Grass_A / Meadow Fill", [["Radius", "12 m"], ["Density", "0.68"], ["Instances", "18k"]], "Paint"],
    "scatter-detail": ["Rule Detail", "Slope Limit", [["Accepted", "3.2k"], ["Reject", "2.3k"], ["Weight", "0.70"]], "Preview Rule"],
    "volume-detail": ["Volume Detail", "FogZone_A", [["Shape", "Box"], ["Overlaps", "6"], ["Priority", "High"]], "Apply Bounds"],
    "weather-detail": ["Preset Detail", "StormFront_02", [["Rain", "38%"], ["Wind", "12 m/s"], ["Clouds", "0.72"]], "Simulate"],
    "post-process-detail": ["Pass Detail", "Color Grade / LUT", [["Tone", "Filmic"], ["LUT", "Cool_01"], ["Cost", "0.24 ms"]], "Preview"],
    "particle-library-detail": ["Emitter Detail", "P_Sparks_Library", [["Particles", "1.8k"], ["Tags", "Metal"], ["State", "Ready"]], "Simulate"],
    "collision-proxy-detail": ["Proxy Detail", "PipeWall convex hull", [["Hulls", "12"], ["Verts", "184"], ["Warnings", "2"]], "Bake Proxy"],
    "level-variant-detail": ["Variant Detail", "Hangar_DayNight / Night", [["Changes", "18"], ["Conflicts", "0"], ["Actors", "9"]], "Apply Variant"],
    "gameplay-ability-detail": ["Ability Detail", "DashStrike", [["Cost", "25 stamina"], ["Cooldown", "4.5 s"], ["Tags", "Ability.Dash"]], "Activate"],
    "gameplay-effect-detail": ["Effect Detail", "Burning", [["Duration", "6 s"], ["Stacks", "3"], ["Modifiers", "8"]], "Apply Effect"],
    "ai-perception-detail": ["Agent Detail", "Guard_01", [["Stimuli", "12"], ["Target", "Player"], ["State", "Alert"]], "Trace Agent"],
    "spawn-rules-detail": ["Rule Detail", "HangarWave_A", [["Waves", "5"], ["Budget", "120"], ["Valid", "Yes"]], "Simulate"],
    "gameplay-tags-detail": ["Tag Detail", "Ability.Dash", [["Uses", "12"], ["Owner", "Gameplay"], ["Conflicts", "0"]], "Rename Tag"],
    "save-data-detail": ["Slot Detail", "Slot_A", [["Schema", "v12"], ["Size", "1.8 MB"], ["Dirty", "No"]], "Validate Slot"],
    "world-state-detail": ["Flag Detail", "Power.On", [["Scope", "Quest"], ["Value", "True"], ["Dirty", "Yes"]], "Patch Flag"],
    "telemetry-dashboard-detail": ["Metric Detail", "Gameplay Ability", [["Events", "1.2k"], ["Rate", "42/s"], ["Markers", "8"]], "Open Trace"],
    "lobby-detail": ["Lobby Detail", "Dev Room", [["Members", "4"], ["Ready", "3"], ["Privacy", "Friends"]], "Start Session"],
    "matchmaking-detail": ["Ticket Detail", "Ranked_2v2 / Ticket 184", [["Skill", "1420"], ["Wait", "42 s"], ["Region", "Asia"]], "Simulate"],
    "server-browser-detail": ["Server Detail", "asia-dev-01", [["Players", "12/32"], ["Ping", "42 ms"], ["Build", "Current"]], "Join Server"],
    "replay-detail": ["Replay Detail", "Match_042", [["Duration", "12:42"], ["Markers", "12"], ["Size", "84 MB"]], "Open Replay"],
    "achievements-detail": ["Achievement Detail", "No Damage Run", [["Points", "30"], ["State", "Draft"], ["Progress", "12%"]], "Validate"],
    "entitlements-detail": ["Entitlement Detail", "FounderPack", [["Grant", "Owned"], ["Items", "3"], ["Audit", "Clean"]], "Grant"],
    "user-profile-detail": ["Profile Detail", "Player_042", [["Presence", "Online"], ["Friends", "18"], ["Privacy", "Friends"]], "Sync Profile"],
    "online-diagnostics-detail": ["Diagnostic Detail", "Presence heartbeat", [["State", "Warn"], ["Latency", "88 ms"], ["Region", "Asia"]], "Open Trace"],
    "hud-detail": ["Widget Detail", "HealthBar", [["Layer", "Gameplay"], ["Anchor", "TopLeft"], ["Binding", "player.health"]], "Preview Widget"],
    "menu-flow-detail": ["Transition Detail", "Main -> Options", [["Input", "Click"], ["Motion", "Fade 120 ms"], ["Guard", "None"]], "Preview Route"],
    "font-atlas-detail": ["Glyph Detail", "U+4E2D", [["Range", "CJK"], ["Packed", "Yes"], ["Fallback", "No"]], "Open Glyph"],
    "icon-library-detail": ["Icon Detail", "save", [["Size", "24"], ["Refs", "18"], ["Export", "SVG+PNG"]], "Export Icon"],
    "ui-binding-detail": ["Binding Detail", "ItemName.text", [["Source", "inventory.name"], ["Type", "String"], ["Trace", "Live"]], "Trace Binding"],
    "accessibility-detail": ["Issue Detail", "IconButton_12 missing label", [["Rule", "Name"], ["Severity", "Error"], ["Fix", "Add label"]], "Apply Fix"],
    "input-prompts-detail": ["Prompt Detail", "Press A", [["Action", "Confirm"], ["Device", "Xbox"], ["Locale", "en-US"]], "Preview Prompt"],
    "ui-motion-detail": ["Motion Detail", "Panel_Open", [["Duration", "180 ms"], ["Ease", "OutCubic"], ["Events", "2"]], "Play Clip"],
    "shader-permutations-detail": ["Permutation Detail", "Metal_Debug", [["Keywords", "DEBUG_VIEW"], ["Stage", "Fragment"], ["Warnings", "1"]], "Compile Variant"],
    "render-target-detail": ["Target Detail", "HDR_Main", [["Format", "RGBA16F"], ["Memory", "42 MB"], ["Samples", "1x"]], "Capture Target"],
    "gpu-profiler-detail": ["Pass Detail", "Lighting", [["Time", "4.8 ms"], ["Draws", "42"], ["State", "Hot"]], "Open Markers"],
    "light-probes-detail": ["Probe Detail", "Probe A12", [["Set", "Hangar"], ["Coverage", "92%"], ["Bake", "Ready"]], "Bake Probe"],
    "reflection-capture-detail": ["Capture Detail", "Hangar_Cubemap", [["Shape", "Box"], ["Faces", "6"], ["Resolution", "512"]], "Capture"],
    "decal-detail": ["Decal Detail", "WarningStripe", [["Material", "M_Decal_Warn"], ["Sort", "12"], ["Placements", "12"]], "Place Decal"],
    "virtual-texture-detail": ["Texture Detail", "TerrainMega", [["Residency", "82%"], ["Pages", "512"], ["Misses", "18"]], "Inspect Pages"],
    "material-audit-detail": ["Material Detail", "M_Glass", [["Cost", "4.2 ms"], ["Rule", "Overdraw"], ["Fix", "Manual"]], "Open Material"],
    "sound-cue-detail": ["Cue Detail", "Weapon_Fire", [["Nodes", "6"], ["Variations", "4"], ["Peak", "-3.2 dB"]], "Preview Cue"],
    "audio-mixer-detail": ["Bus Detail", "Master", [["Loudness", "-18 LUFS"], ["Peak", "-3.0 dB"], ["Effects", "2"]], "Solo Bus"],
    "music-system-detail": ["State Detail", "Combat_Loop", [["BPM", "128"], ["Bars", "8"], ["Stingers", "3"]], "Preview State"],
    "audio-occlusion-detail": ["Occlusion Detail", "Trace_01", [["Obstacle", "Door_A"], ["LPF", "0.52"], ["Volume", "-6 dB"]], "Simulate Trace"],
    "voice-bank-detail": ["Voice Detail", "CAP_002", [["Speaker", "Captain"], ["Locale", "zh-CN"], ["State", "Missing"]], "Import Line"],
    "subtitle-timing-detail": ["Subtitle Detail", "SUB_003", [["Start", "00:06.00"], ["End", "00:08.10"], ["State", "Overlap"]], "Align Cue"],
    "lip-sync-detail": ["Lip Sync Detail", "Captain_Line_03", [["Phonemes", "18"], ["Visemes", "12"], ["Quality", "92%"]], "Preview Clip"],
    "audio-profiler-detail": ["Event Detail", "Weapon_Fire", [["Voices", "4"], ["CPU", "0.22 ms"], ["Bus", "SFX"]], "Open Event"],
    "rigid-body-detail": ["Body Detail", "Crate_07", [["Mass", "12 kg"], ["Friction", "0.62"], ["Layer", "World"]], "Simulate Body"],
    "physics-constraints-detail": ["Constraint Detail", "Door_Hinge", [["Joints", "8"], ["Swing", "-45..90"], ["Broken", "0"]], "Simulate Joint"],
    "destruction-detail": ["Cluster Detail", "Cluster A", [["Chunks", "18"], ["Threshold", "120"], ["Damage", "Ready"]], "Preview Break"],
    "cloth-detail": ["Cloth Detail", "Cape_A", [["Verts", "184"], ["Pinned", "60"], ["Wind", "0.35"]], "Simulate Cloth"],
    "vehicle-physics-detail": ["Vehicle Detail", "Rover_01", [["Mass", "1240 kg"], ["Wheels", "4"], ["Top Speed", "82 km/h"]], "Drive Test"],
    "fluid-detail": ["Fluid Detail", "SteamVent_A", [["Particles", "18k"], ["Step", "2.8 ms"], ["Cache", "Ready"]], "Simulate Fluid"],
    "rope-cable-detail": ["Cable Detail", "BridgeCable_A", [["Segments", "32"], ["Tension", "0.72"], ["Length", "18 m"]], "Solve Cable"],
    "physics-profiler-detail": ["Profiler Detail", "Frame 1842", [["Physics", "3.6 ms"], ["Contacts", "184"], ["Hotspots", "2"]], "Open Capture"],
    "ai-director-detail": ["Director Detail", "Hangar_Assault", [["Threat", "72"], ["Wave", "3"], ["Pacing", "Rising"]], "Simulate Director"],
    "blackboard-detail": ["Key Detail", "TargetActor", [["Type", "Entity"], ["Value", "Player"], ["State", "Live"]], "Trace Key"],
    "eqs-query-detail": ["Query Detail", "FindCover", [["Candidates", "128"], ["Best Score", "0.91"], ["Rejects", "2"]], "Run Query"],
    "crowd-detail": ["Crowd Detail", "Civilians", [["Agents", "180"], ["Blocked", "18"], ["Cost", "2.4 ms"]], "Simulate Crowd"],
    "smart-object-detail": ["Object Detail", "CoverWall_A", [["Slots", "12"], ["Reserved", "4"], ["Cooldown", "0.4 s"]], "Reserve Slot"],
    "patrol-route-detail": ["Route Detail", "GuardLoop_A", [["Waypoints", "8"], ["Loops", "1"], ["Blocked", "0"]], "Preview Route"],
    "cover-detail": ["Cover Detail", "Cover_01", [["Type", "High"], ["Exposure", "0.18"], ["State", "Valid"]], "Score Cover"],
    "ai-profiler-detail": ["Profiler Detail", "Live Capture", [["Agents", "32"], ["Cost", "2.8 ms"], ["Hotspots", "4"]], "Open Capture"],
    "mesh-import-detail": ["Import Detail", "SK_Crate.fbx", [["Warnings", "3"], ["Triangles", "4.8k"], ["Collision", "Generated"]], "Import"],
    "lod-chain-detail": ["LOD Detail", "SM_Rock_A / LOD2", [["Triangles", "3.2k"], ["Reduction", "82%"], ["Error", "0.16"]], "Preview LOD"],
    "redirect-map-detail": ["Redirect Detail", "/Old/M_Metal -> /Materials/M_Metal", [["Refs", "18"], ["Broken", "0"], ["State", "Resolved"]], "Replace Refs"],
    "texture-compression-detail": ["Compression Detail", "T_Rock_D", [["Format", "BC7"], ["Size", "2048"], ["State", "Queued"]], "Run Job"],
    "source-asset-detail": ["Source Detail", "crate_source.fbx", [["Outputs", "6"], ["Refs", "18"], ["Stale", "1"]], "Open Source"],
    "dcc-live-link-detail": ["Live Link Detail", "Blender / SM_Crate", [["Latency", "42 ms"], ["Dirty", "3"], ["State", "Connected"]], "Sync Now"],
    "metadata-detail": ["Metadata Detail", "asset.owner", [["Type", "String"], ["Value", "Environment"], ["State", "Valid"]], "Apply Field"],
    "batch-process-detail": ["Batch Detail", "Reimport Materials", [["Jobs", "64"], ["Running", "2"], ["Failed", "0"]], "Open Job"],
    "script-detail": ["Function Detail", "tick(delta_time)", [["Line", "38"], ["Cost", "0.08 ms"], ["Breakpoints", "1"]], "Debug"],
    "api-detail": ["API Detail", "UiSurfaceFrame", [["Crate", "zircon_runtime"], ["Uses", "42"], ["Stability", "Stable"]], "Copy Path"],
    "plugin-packaging-detail": ["Package Detail", "editor.tools.validation", [["Version", "0.18.0"], ["Targets", "2"], ["Signed", "Pending"]], "Build Package"],
    "module-settings-detail": ["Setting Detail", "zircon_runtime / ui", [["Value", "On"], ["Dependents", "4"], ["Warnings", "0"]], "Apply Setting"],
    "automation-suite-detail": ["Suite Detail", "Editor Smoke", [["Tests", "306"], ["Passed", "306"], ["Duration", "4m 12s"]], "Open Artifacts"],
    "build-config-detail": ["Config Detail", "Windows Editor", [["Profile", "Development"], ["Features", "12"], ["Warnings", "1"]], "Generate"],
    "cook-rules-detail": ["Rule Detail", "Texture BC7", [["Scope", "Textures"], ["Platform", "Desktop"], ["State", "Valid"]], "Simulate Rule"],
    "runtime-command-detail": ["Command Detail", "ui.reload_theme", [["Shortcut", "Ctrl+Alt+R"], ["Context", "Editor"], ["Conflicts", "0"]], "Run Command"],
    "asset-migration-detail": ["Migration Detail", "M_Crate / v17 -> v18", [["Assets", "128"], ["Warnings", "3"], ["Blocked", "0"]], "Migrate Asset"],
    "scene-diff-detail": ["Scene Diff Detail", "Door_A transform", [["Change", "Transform"], ["Owner", "Level"], ["State", "Review"]], "Accept Change"],
    "prefab-diff-detail": ["Prefab Diff Detail", "Door_A Variant B", [["Changes", "7"], ["Conflicts", "0"], ["Refs", "12"]], "Apply Diff"],
    "performance-budget-detail": ["Budget Detail", "Editor Frame", [["Budget", "16.6 ms"], ["Actual", "12.8 ms"], ["Headroom", "3.8 ms"]], "Open Capture"],
    "memory-budget-detail": ["Memory Detail", "Texture Pool", [["Used", "1.2 GB"], ["Budget", "1.5 GB"], ["Headroom", "300 MB"]], "Open Assets"],
    "dependency-cleanup-detail": ["Cleanup Detail", "T_OldPanel", [["Refs", "0"], ["Owner", "UI"], ["Recoverable", "Yes"]], "Select Asset"],
    "naming-rule-detail": ["Naming Detail", "crate_old", [["Rule", "Mesh Prefix"], ["Expected", "SM_CrateOld"], ["Autofix", "Yes"]], "Autofix"],
    "release-checklist-detail": ["Gate Detail", "Crash Triage", [["Owner", "Runtime"], ["State", "Review"], ["Blocking", "No"]], "Approve Gate"],
    "gameplay-debugger-detail": ["Actor Detail", "Player_01", [["Health", "82"], ["State", "Alive"], ["Watches", "6"]], "Trace Actor"],
    "replay-timeline-detail": ["Replay Marker", "Crash / 12:10", [["Replay", "Match_042"], ["Events", "184"], ["Markers", "12"]], "Jump To"],
    "packet-detail": ["Packet Detail", "#1845 Reliable", [["Size", "1.2 KB"], ["Channel", "Reliable"], ["Loss", "0.2%"]], "Decode Packet"],
    "latency-detail": ["Route Detail", "Asia-01", [["p50", "42 ms"], ["p95", "88 ms"], ["Loss", "0.2%"]], "Probe Route"],
    "input-trace-detail": ["Input Event", "Interact / Pressed", [["Device", "Keyboard"], ["Binding", "E"], ["Dropped", "No"]], "Replay Event"],
    "save-state-detail": ["State Detail", "quest.power", [["Slot A", "False"], ["Slot B", "True"], ["Schema", "v12"]], "Patch State"],
    "repro-detail": ["Repro Step", "Interact Door_A", [["Session", "Crash_1842"], ["Step", "3"], ["Artifacts", "6"]], "Open Artifact"],
    "qa-triage-detail": ["Issue Detail", "QA-1842", [["Severity", "Blocker"], ["Owner", "Runtime"], ["Repro", "Linked"]], "Assign Issue"],
    "render-graph-detail": ["Pass Detail", "Lighting pass", [["Resources", "6"], ["Barriers", "18"], ["Cost", "4.8 ms"]], "Open Pass"],
    "shader-debugger-detail": ["Shader Detail", "fragment_main", [["Variables", "12"], ["Wave", "4"], ["Branch", "Divergent"]], "Step Shader"],
    "texture-streaming-detail": ["Texture Detail", "T_TerrainMega", [["Residency", "82%"], ["Pages", "512"], ["Misses", "18"]], "Pin Texture"],
    "shadow-map-detail": ["Cascade Detail", "Cascade 2", [["Range", "48-128m"], ["Cost", "0.6 ms"], ["Issues", "2"]], "Tune Bias"],
    "occlusion-detail": ["Object Detail", "SM_Crate_07", [["Cell", "A1_04"], ["Visible", "Yes"], ["Queries", "64"]], "Freeze Query"],
    "frame-compare-detail": ["Frame Detail", "1841 -> 1842", [["Delta", "+0.8 ms"], ["Draws", "+12"], ["Regressions", "2"]], "Open Diff"],
    "material-layer-detail": ["Layer Detail", "Wetness", [["Blend", "Screen"], ["Cost", "0.9"], ["State", "Warn"]], "Preview Layer"],
    "gpu-memory-detail": ["Allocation Detail", "Render Targets", [["Used", "420 MB"], ["Budget", "512 MB"], ["Delta", "+128 MB"]], "Open Owners"],
    "retarget-detail": ["Retarget Detail", "Legs chain", [["Source", "thigh_l/r"], ["Target", "leg_l/r"], ["State", "Warn"]], "Preview Chain"],
    "ik-solver-detail": ["Solver Detail", "FullBodyIK", [["Effectors", "4"], ["Cost", "0.38 ms"], ["Warning", "1"]], "Solve IK"],
    "pose-detail": ["Pose Detail", "Pose_142", [["Clip", "Run_Start"], ["Cost", "0.42"], ["Tags", "Run"]], "Preview Pose"],
    "mocap-detail": ["Take Detail", "Take_042 / Foot Slide", [["Frame", "142"], ["Bone", "foot_l"], ["Fix", "Foot Lock"]], "Apply Fix"],
    "animation-compression-detail": ["Compression Detail", "Turn_180", [["Error", "0.22"], ["Ratio", "55%"], ["State", "Warn"]], "Compare Clip"],
    "root-motion-detail": ["Motion Detail", "DashStrike", [["Distance", "4.2 m"], ["Frames", "32"], ["Drift", "0.0"]], "Preview Motion"],
    "event-track-detail": ["Event Detail", "HitWindowOpen", [["Frame", "18"], ["Track", "Combat"], ["State", "Valid"]], "Preview Event"],
    "montage-debugger-detail": ["Montage Detail", "Attack_A section", [["Blend", "0.18"], ["Weight", "0.84"], ["Events", "14"]], "Trace Section"],
    "widget-tree-detail": ["Widget Detail", "HealthBar", [["Type", "Progress"], ["Depth", "2"], ["State", "Visible"]], "Highlight"],
    "layout-constraint-detail": ["Constraint Detail", "Min Width", [["Target", "InventoryPanel"], ["Value", "320"], ["State", "OK"]], "Inspect Solve"],
    "theme-variant-detail": ["Token Detail", "accent.teal", [["Value", "#3cc7d6"], ["Usage", "Active"], ["Contrast", "AA"]], "Preview Token"],
    "localization-preview-detail": ["String Detail", "ui.inventory", [["Locale", "zh-CN"], ["Overflow", "No"], ["State", "OK"]], "Open Usage"],
    "focus-navigation-detail": ["Focus Detail", "SlotGrid", [["Routes", "4"], ["Dead Ends", "0"], ["Loops", "1"]], "Trace Route"],
    "input-glyph-detail": ["Glyph Detail", "Confirm / Xbox A", [["Prompts", "64"], ["Missing", "0"], ["Locales", "8"]], "Preview Glyph"],
    "ui-snapshot-detail": ["Snapshot Detail", "Inventory SlotGrid", [["Diff", "4 px"], ["Severity", "Low"], ["State", "Review"]], "Overlay Diff"],
    "widget-performance-detail": ["Widget Perf Detail", "Combat_HUD", [["Cost", "1.2 ms"], ["Budget", "2.0 ms"], ["Invalid", "4"]], "Open Capture"],
    "world-partition-detail": ["Cell Detail", "A1_04", [["Layer", "Gameplay"], ["Memory", "184 MB"], ["State", "Loaded"]], "Load Cell"],
    "hlod-detail": ["Cluster Detail", "HLOD_C", [["Actors", "18"], ["Triangles", "8k"], ["State", "Warn"]], "Preview Cluster"],
    "level-instance-detail": ["Instance Detail", "DockModule_A_01", [["Source", "DockModule_A"], ["Overrides", "3"], ["State", "Valid"]], "Open Instance"],
    "streaming-profiler-detail": ["Stream Detail", "A1_04 load", [["Peak", "184 MB"], ["Stall", "0.8 s"], ["Events", "12"]], "Open Event"],
    "scene-bookmark-detail": ["Bookmark Detail", "Bug_1842", [["Camera", "Cam_Debug"], ["Owner", "QA"], ["State", "Pinned"]], "Jump"],
    "spawn-point-detail": ["Spawn Detail", "Spawn_03", [["Type", "Loot"], ["Weight", "0.2"], ["State", "Warn"]], "Simulate Point"],
    "collision-matrix-detail": ["Channel Detail", "Projectile", [["Rules", "18"], ["Conflicts", "0"], ["Profile", "Default"]], "Preview Channel"],
    "environment-probe-detail": ["Probe Detail", "Probe_Dock", [["Coverage", "74%"], ["Samples", "128"], ["State", "Warn"]], "Bake Probe"],
    "feature-flag-detail": ["Flag Detail", "new_inventory", [["Audience", "Beta"], ["Rollout", "25%"], ["State", "Live"]], "Open Rollout"],
    "remote-config-detail": ["Config Detail", "store.banner", [["Value", "Spring"], ["Scope", "Global"], ["State", "Live"]], "Open Diff"],
    "telemetry-query-detail": ["Query Detail", "Session Funnel", [["Events", "1.2k"], ["Rate", "42/s"], ["Alerts", "3"]], "Run Query"],
    "patch-planner-detail": ["Patch Detail", "Fix Crash 1842", [["Risk", "High"], ["Owner", "Runtime"], ["State", "Ready"]], "Open Patch"],
    "dlc-detail": ["DLC Detail", "FounderPack", [["Items", "3"], ["SKU", "dlc_founder"], ["State", "Ready"]], "Open Pack"],
    "symbolication-detail": ["Crash Detail", "Crash_1842", [["Resolved", "Yes"], ["Platform", "Win64"], ["Hits", "42"]], "Open Stack"],
    "segment-detail": ["Segment Detail", "Returning Players", [["Users", "42k"], ["Region", "Global"], ["State", "Active"]], "Open Segment"],
    "experiment-detail": ["Experiment Detail", "Store Banner", [["Variants", "3"], ["Traffic", "10%"], ["State", "Live"]], "Open Metrics"],
  };
  const [title, subtitle, rows, action] = map[kind] ?? map["prefab-detail"];
  return { title, subtitle, rows, action };
}

function additionalOutputConfig(kind) {
  const map = {
    "prefab-output": [["Validation", "References", "Console"], ["Prefab variant clean", "3 overrides resolved", "No missing references"]],
    "vfx-output": [["Simulation", "Warnings", "Console"], ["Emitter warmup complete", "Bounds updated", "GPU particles: 1,024"]],
    "shader-output": [["Compiler", "Variants", "Console"], ["Compiled unlit.zshader", "Variant debug_view enabled", "0 errors / 2 warnings"]],
    "terrain-output": [["Bake", "Paint", "Console"], ["Height cache updated", "Layer Dust painted", "Navigation mesh dirty"]],
    "audio-output": [["Analysis", "Mixer", "Console"], ["Peak -3.2 dB", "Loop region valid", "Ambience bus active"]],
    "behavior-output": [["Trace", "Blackboard", "Console"], ["Selector ticked", "Move To running", "Blackboard PatrolIndex = 2"]],
    "lighting-output": [["Jobs", "Warnings", "Console"], ["Bake preview queued", "Reflection probes dirty", "Denoiser ready"]],
    "physics-output": [["Simulation", "Contacts", "Console"], ["Contact pairs: 12", "Solver stable", "Collision proxy valid"]],
    "level-streaming-output": [["Loads", "Memory", "Console"], ["A1_North loaded", "Memory budget: 184 MB", "Dock exterior unloaded"]],
    "sequencer-output": [["Timeline", "Curves", "Render"], ["Shot 010 key set", "Camera cut valid", "Preview render queued"]],
    "navmesh-output": [["Bake", "Warnings", "Console"], ["Nav tiles rebuilt", "2 jump links updated", "No disconnected islands"]],
    "render-pipeline-output": [["Compile", "Capture", "Console"], ["Forward+ graph compiled", "HDR_Main target ready", "UI pass batched"]],
    "input-mapping-output": [["Validation", "Conflicts", "Console"], ["Context Editor_Default valid", "0 conflicts", "4 bindings changed"]],
    "data-table-output": [["Validation", "Import", "Console"], ["items.zdata validated", "1 optional field empty", "CSV export ready"]],
    "network-replication-output": [["Replication", "RPC", "Console"], ["Client A snapshot sent", "RPC queue drained", "Bandwidth 1.2 KB/frame"]],
    "localization-output": [["Validation", "Export", "Console"], ["zh-CN coverage 97%", "1 missing string", "PO export ready"]],
    "visual-script-output": [["Compiler", "Trace", "Console"], ["DoorController compiled", "2 breakpoints armed", "No unresolved pins"]],
    "state-machine-output": [["Validation", "Preview", "Console"], ["Walk -> Run valid", "1 unreachable state warning", "Preview clock running"]],
    "skeleton-mesh-output": [["Import", "Weights", "Console"], ["SK_Guard import clean", "4 weight warnings", "Socket preview active"]],
    "texture-output": [["Analysis", "Compression", "Console"], ["BC7 compression ready", "Mip chain complete", "Alpha coverage 92%"]],
    "material-instance-output": [["Shader", "Usage", "Console"], ["MI_Metal_Rough compiled", "3 overrides changed", "14 assets using instance"]],
    "prefab-variant-output": [["Validation", "Diff", "Console"], ["Variant B has 7 overrides", "1 removed child", "References valid"]],
    "level-audit-output": [["Run", "Fixes", "Console"], ["12 issues found", "8 automatic fixes available", "1 blocking error"]],
    "test-runner-output": [["Run", "Failures", "Console"], ["42 passed", "1 failed: viewport_resize", "Run duration 2.4 s"]],
    "frame-debugger-output": [["Events", "Markers", "Console"], ["Frame 1842 captured", "UI pass selected", "128 draw calls"]],
    "memory-profiler-output": [["Snapshots", "Leaks", "Console"], ["Snapshot compared", "18 leak suspects", "Renderer heap +18 MB"]],
    "asset-dependency-output": [["Trace", "Cycles", "Console"], ["146 transitive dependencies", "0 missing assets", "No cycles found"]],
    "reference-finder-output": [["Search", "Replace", "Console"], ["18 references found", "12 replaceable", "Scope: Project"]],
    "cook-package-output": [["Cook", "Warnings", "Console"], ["Cook step 62%", "2 texture warnings", "Package queue running"]],
    "crash-session-output": [["Timeline", "Recovery", "Console"], ["Crash replay loaded", "Autosave checkpoint valid", "Recovery path available"]],
    "log-analysis-output": [["Triage", "Patterns", "Console"], ["2 fallback warnings", "1 audio listener issue", "Triage task ready"]],
    "automation-report-output": [["Summary", "Failures", "Console"], ["306 passed", "6 failed", "18 artifacts available"]],
    "layout-manager-output": [["Apply", "Diff", "Console"], ["default-workbench valid", "0 geometry conflicts", "Drawer states saved"]],
    "theme-token-output": [["Validation", "Contrast", "Console"], ["Token set valid", "AA contrast passed", "Gradients reduced"]],
    "command-center-output": [["Audit", "Conflicts", "Console"], ["184 commands scanned", "0 conflicts", "12 unbound commands"]],
    "module-graph-output": [["Analysis", "Boundaries", "Console"], ["18 modules analyzed", "0 cycles", "2 boundary warnings"]],
    "asset-validation-output": [["Run", "Repairs", "Console"], ["428 assets scanned", "7 warnings", "5 automatic repairs"]],
    "hot-reload-output": [["Reload", "Warnings", "Console"], ["3 modules dirty", "Patch queued", "Rollback point saved"]],
    "project-history-output": [["Activity", "Diff", "Console"], ["12 recent changes", "3 docs touched", "Recovery snapshot ready"]],
    "task-board-output": [["Activity", "Reviews", "Console"], ["14 open tasks", "3 in review", "1 blocked item"]],
    "source-control-output": [["Sync", "Submit", "Console"], ["14 files modified", "0 conflicts", "2 shelves available"]],
    "review-comments-output": [["Activity", "Resolved", "Console"], ["6 unresolved threads", "1 blocking comment", "3 replies today"]],
    "build-farm-output": [["Queue", "Workers", "Console"], ["4 agents online", "7 jobs queued", "1 failing worker"]],
    "release-notes-output": [["Draft", "Publish", "Console"], ["0.18.0 draft updated", "4 sections ready", "2 release notes pending"]],
    "project-settings-output": [["Validation", "Changes", "Console"], ["Project settings valid", "1 build warning", "No dirty settings"]],
    "plugin-development-output": [["Build", "Reload", "Console"], ["Plugin build ready", "1 manifest warning", "Hot reload available"]],
    "remote-device-output": [["Deploy", "Telemetry", "Console"], ["Device connected", "Install running", "Telemetry stream active"]],
    "session-sync-output": [["Events", "Conflicts", "Console"], ["3 peers connected", "1 conflict pending", "Selection state synced"]],
    "cutscene-output": [["Timeline", "Curves", "Render"], ["Shot 020 key updated", "Camera cut valid", "Preview render queued"]],
    "dialogue-output": [["Validation", "Localization", "Console"], ["42 dialogue lines scanned", "1 missing localized line", "Branch flow valid"]],
    "quest-output": [["Validation", "Simulation", "Console"], ["6 objectives valid", "2 branches simulated", "0 blocking errors"]],
    "camera-rig-output": [["Preview", "Keys", "Console"], ["Crane_A preview active", "Focus key inserted", "No framing warnings"]],
    "control-rig-output": [["Solve", "Bake", "Console"], ["Control rig solved", "1 channel warning", "Bake target ready"]],
    "motion-matching-output": [["Analysis", "Queries", "Console"], ["842 poses indexed", "Best pose cost 0.42", "Query preview running"]],
    "facial-animation-output": [["Solve", "Curves", "Console"], ["52 facial curves solved", "18 phonemes aligned", "1 blink adjusted"]],
    "blend-space-output": [["Preview", "Samples", "Console"], ["9 samples normalized", "Blend preview running", "No empty grid cells"]],
    "foliage-output": [["Paint", "Density", "Console"], ["18k instances updated", "Density map clean", "No invalid meshes"]],
    "scatter-output": [["Validation", "Preview", "Console"], ["42 rules evaluated", "3.2k placements accepted", "2 mask warnings"]],
    "volume-output": [["Overlap", "Warnings", "Console"], ["6 actors overlapping", "FogZone bounds valid", "1 priority warning"]],
    "weather-output": [["Simulation", "Curves", "Console"], ["Storm curve simulated", "Rain blend at 38%", "Wind gust key added"]],
    "post-process-output": [["Preview", "Validation", "Console"], ["Filmic look previewed", "LUT contrast valid", "0 pass errors"]],
    "particle-library-output": [["Simulation", "Usage", "Console"], ["Emitter simulated", "12 assets using emitter", "No missing materials"]],
    "collision-proxy-output": [["Analyze", "Bake", "Console"], ["12 proxies generated", "2 hull warnings", "Bake artifact ready"]],
    "level-variant-output": [["Diff", "Preview", "Console"], ["18 changes previewed", "0 variant conflicts", "Night variant applied"]],
    "gameplay-ability-output": [["Simulation", "Trace", "Console"], ["DashStrike activation valid", "Cooldown timer ready", "No blocked tags"]],
    "gameplay-effect-output": [["Validation", "Apply", "Console"], ["Burning effect validated", "8 modifiers resolved", "Stacking limit 3"]],
    "ai-perception-output": [["Trace", "Stimuli", "Console"], ["Guard_01 sensed Player", "12 stimuli active", "Memory updated"]],
    "spawn-rules-output": [["Simulation", "Validation", "Console"], ["5 waves simulated", "Spawn budget valid", "No blocked points"]],
    "gameplay-tags-output": [["Validation", "Rename", "Console"], ["286 tags validated", "0 conflicts", "14 unused tags"]],
    "save-data-output": [["Validation", "Migration", "Console"], ["Slot_A schema valid", "Slot_B migrated to v12", "Cloud sync clean"]],
    "world-state-output": [["Diff", "Patch", "Console"], ["184 flags compared", "12 dirty flags", "0 patch conflicts"]],
    "telemetry-dashboard-output": [["Capture", "Markers", "Console"], ["1.2k events captured", "8 markers inserted", "Telemetry export ready"]],
    "lobby-output": [["Session", "Invites", "Console"], ["Dev Room synced", "3 members ready", "Invite token refreshed"]],
    "matchmaking-output": [["Simulation", "Tickets", "Console"], ["128 tickets queued", "Match quality 0.82", "Backfill rule ready"]],
    "server-browser-output": [["Ping", "Refresh", "Console"], ["42 servers refreshed", "Avg ping 64 ms", "8 full servers hidden"]],
    "replay-output": [["Playback", "Markers", "Console"], ["Replay Match_042 opened", "12 markers indexed", "Playback ready"]],
    "achievements-output": [["Validation", "Publish", "Console"], ["38 achievements validated", "2 drafts pending", "0 conflicts"]],
    "entitlements-output": [["Audit", "Grant", "Console"], ["FounderPack owned", "1 missing entitlement", "Grant queue pending"]],
    "user-profile-output": [["Sync", "Friends", "Console"], ["Profile synced", "18 friends loaded", "Privacy rules valid"]],
    "online-diagnostics-output": [["Checks", "Trace", "Console"], ["Auth token pass", "Presence heartbeat warning", "Trace export ready"]],
    "hud-output": [["Preview", "Bindings", "Console"], ["Combat_HUD preview ready", "18 widgets anchored", "0 invalid bindings"]],
    "menu-flow-output": [["Validation", "Preview", "Console"], ["9 screens validated", "18 transitions checked", "0 dead ends"]],
    "font-atlas-output": [["Bake", "Coverage", "Console"], ["Inter_UI atlas baked", "99.2% coverage", "0 missing glyphs"]],
    "icon-library-output": [["Audit", "Export", "Console"], ["286 icons scanned", "14 unused icons", "Export manifest ready"]],
    "ui-binding-output": [["Validation", "Trace", "Console"], ["24 bindings valid", "7 data sources resolved", "Live trace attached"]],
    "accessibility-output": [["Audit", "Fixes", "Console"], ["42 checks completed", "1 error / 3 warnings", "2 quick fixes available"]],
    "input-prompts-output": [["Localization", "Preview", "Console"], ["64 prompts validated", "8 locales covered", "No missing glyphs"]],
    "ui-motion-output": [["Timeline", "Events", "Console"], ["Panel_Open previewed", "4 curves evaluated", "2 timeline events fired"]],
    "shader-permutations-output": [["Compiler", "Variants", "Console"], ["128 variants scanned", "42 variants stripped", "1 debug warning"]],
    "render-target-output": [["Capture", "Memory", "Console"], ["HDR_Main captured", "4 attachments inspected", "42 MB allocated"]],
    "gpu-profiler-output": [["Capture", "Markers", "Console"], ["Frame 1842 captured", "Lighting pass hot", "12.8 ms GPU frame"]],
    "light-probes-output": [["Bake", "Validation", "Console"], ["64 probes queued", "Coverage 92%", "0 leak warnings"]],
    "reflection-capture-output": [["Capture", "Validation", "Console"], ["Hangar_Cubemap captured", "6 faces valid", "1 stale capture"]],
    "decal-output": [["Validation", "Placement", "Console"], ["12 decal placements valid", "1 sort warning", "Projection bounds updated"]],
    "virtual-texture-output": [["Streaming", "Feedback", "Console"], ["512 pages tracked", "82% residency", "18 feedback misses"]],
    "material-audit-output": [["Audit", "Fixes", "Console"], ["184 materials scanned", "7 warnings", "5 automatic fixes available"]],
    "sound-cue-output": [["Preview", "Analysis", "Console"], ["Weapon_Fire previewed", "4 variations weighted", "Peak -3.2 dB"]],
    "audio-mixer-output": [["Meters", "Snapshots", "Console"], ["Gameplay Mix active", "Music ducking applied", "Master peak -3.0 dB"]],
    "music-system-output": [["Preview", "Transitions", "Console"], ["Combat_Loop playing", "3 stingers armed", "Transition grid valid"]],
    "audio-occlusion-output": [["Simulation", "Trace", "Console"], ["12 traces simulated", "4 blocked paths", "Average LPF 0.42"]],
    "voice-bank-output": [["Import", "Validation", "Console"], ["248 lines scanned", "1 missing zh-CN line", "Voice bank manifest ready"]],
    "subtitle-timing-output": [["Validation", "Preview", "Console"], ["42 subtitle cues checked", "1 overlap warning", "Reading speed valid"]],
    "lip-sync-output": [["Solve", "Preview", "Console"], ["18 phonemes solved", "12 viseme curves generated", "Quality 92%"]],
    "audio-profiler-output": [["Capture", "Events", "Console"], ["64 voices captured", "18 audio events", "Mixer CPU 3.2 ms"]],
    "rigid-body-output": [["Simulation", "Contacts", "Console"], ["Crate_07 simulated", "12 contact pairs", "Solver stable"]],
    "physics-constraints-output": [["Solver", "Breakage", "Console"], ["Door_Hinge stable", "0 broken joints", "Motor limit valid"]],
    "destruction-output": [["Fracture", "Simulation", "Console"], ["42 chunks generated", "6 clusters built", "Damage threshold ready"]],
    "cloth-output": [["Solve", "Paint", "Console"], ["Cape_A solved", "184 verts evaluated", "No constraint tears"]],
    "vehicle-physics-output": [["Test", "Telemetry", "Console"], ["Rover_01 drive test ready", "4 wheels grounded", "Grip curve valid"]],
    "fluid-output": [["Simulation", "Cache", "Console"], ["18k particles simulated", "Cache ready", "0 leak warnings"]],
    "rope-cable-output": [["Solve", "Tension", "Console"], ["32 segments solved", "Tension 0.72", "Anchors valid"]],
    "physics-profiler-output": [["Capture", "Contacts", "Console"], ["Frame 1842 captured", "184 contacts analyzed", "Physics cost 3.6 ms"]],
    "ai-director-output": [["Simulation", "Waves", "Console"], ["Hangar_Assault simulated", "Threat budget 72", "Wave 3 pacing rising"]],
    "blackboard-output": [["Validation", "Trace", "Console"], ["18 keys validated", "14 live values traced", "0 missing keys"]],
    "eqs-query-output": [["Debug", "Scores", "Console"], ["128 candidates scored", "Best cover score 0.91", "2 candidates rejected"]],
    "crowd-output": [["Simulation", "Flow", "Console"], ["240 agents simulated", "18 blocked agents", "Flow lanes stable"]],
    "smart-object-output": [["Validation", "Reservations", "Console"], ["12 slots validated", "4 slots reserved", "0 stale reservations"]],
    "patrol-route-output": [["Validation", "Preview", "Console"], ["8 waypoints validated", "GuardLoop_A preview ready", "0 blocked edges"]],
    "cover-output": [["Bake", "Validation", "Console"], ["64 cover points baked", "Average exposure 0.22", "0 invalid points"]],
    "ai-profiler-output": [["Capture", "Events", "Console"], ["32 agents captured", "18 perception events", "AI cost 2.8 ms"]],
    "mesh-import-output": [["Validation", "Import", "Console"], ["SK_Crate.fbx validated", "3 import warnings", "Collision generated"]],
    "lod-chain-output": [["Build", "Preview", "Console"], ["4 LOD levels built", "LOD3 reduced 95%", "No reduction errors"]],
    "redirect-map-output": [["Validation", "Replace", "Console"], ["42 redirects resolved", "0 broken references", "3 unused redirects"]],
    "texture-compression-output": [["Queue", "Warnings", "Console"], ["18 textures queued", "1 compression job running", "BC7 desktop rules active"]],
    "source-asset-output": [["Trace", "Diff", "Console"], ["6 derived assets traced", "1 stale material found", "18 references mapped"]],
    "dcc-live-link-output": [["Sync", "Conflicts", "Console"], ["Blender session connected", "3 dirty assets pending", "No sync conflicts"]],
    "metadata-output": [["Validation", "Apply", "Console"], ["24 metadata fields valid", "2 dirty fields applied", "Schema AssetSchema.v2 clean"]],
    "batch-process-output": [["Queue", "Workers", "Console"], ["64 batch jobs queued", "2 workers running", "0 failed jobs"]],
    "script-output": [["Compiler", "Debug", "Console"], ["player_controller.zs compiled", "0 errors / 2 hints", "Breakpoint armed at line 38"]],
    "api-output": [["Docs", "Search", "Console"], ["184 symbols indexed", "UiSurfaceFrame opened", "42 usages found"]],
    "plugin-packaging-output": [["Build", "Sign", "Console"], ["2 package targets built", "Manifest valid", "Signature pending"]],
    "module-settings-output": [["Validation", "Diff", "Console"], ["12 module flags validated", "0 dependency cycles", "2 warnings remain"]],
    "automation-suite-output": [["Run", "Failures", "Console"], ["306 editor tests passed", "4m 12s duration", "Artifacts captured"]],
    "build-config-output": [["Generate", "Build", "Console"], ["Windows Editor config generated", "12 features enabled", "1 warning in environment"]],
    "cook-rules-output": [["Validate", "Simulate", "Console"], ["84 cook rules validated", "6 platform overrides applied", "2 audio warnings"]],
    "runtime-command-output": [["Run", "Audit", "Console"], ["ui.reload_theme executed", "184 commands audited", "0 shortcut conflicts"]],
    "asset-migration-output": [["Scan", "Migrate", "Console"], ["128 assets scanned", "3 schema warnings", "0 blocked migrations"]],
    "scene-diff-output": [["Review", "Patch", "Console"], ["18 scene changes compared", "1 ownership conflict", "6 changes accepted"]],
    "prefab-diff-output": [["Diff", "Apply", "Console"], ["7 prefab overrides compared", "0 conflicts", "Patch ready"]],
    "performance-budget-output": [["Capture", "Regressions", "Console"], ["Frame capture loaded", "12.8 ms actual", "0 budget failures"]],
    "memory-budget-output": [["Snapshot", "Compare", "Console"], ["2.1 GB memory snapshot", "UI Atlas over budget", "Texture pool within budget"]],
    "dependency-cleanup-output": [["Scan", "Clean", "Console"], ["42 unused assets found", "18 selected for cleanup", "0 broken refs"]],
    "naming-rule-output": [["Scan", "Autofix", "Console"], ["12 naming warnings", "5 autofix candidates", "0 blocked renames"]],
    "release-checklist-output": [["Validate", "Approvals", "Console"], ["6 release gates ready", "2 gates in review", "0 blocking issues"]],
    "gameplay-debugger-output": [["Trace", "Events", "Console"], ["Player_01 trace attached", "6 watches live", "0 debug errors"]],
    "replay-timeline-output": [["Playback", "Markers", "Console"], ["Replay Match_042 loaded", "12 markers indexed", "Playback at 06:42"]],
    "packet-output": [["Capture", "Decode", "Console"], ["184 packets captured", "Reliable channel warning", "0 decode errors"]],
    "latency-output": [["Probe", "Alerts", "Console"], ["Asia probes refreshed", "p50 latency 42 ms", "1 route warning"]],
    "input-trace-output": [["Trace", "Replay", "Console"], ["184 input events captured", "0 dropped events", "1 ignored gamepad action"]],
    "save-state-output": [["Compare", "Patch", "Console"], ["12 state changes compared", "Schema v12 valid", "Patch preview ready"]],
    "repro-output": [["Record", "Package", "Console"], ["4 repro steps captured", "6 artifacts attached", "Package ready for QA"]],
    "qa-triage-output": [["Triage", "Repros", "Console"], ["18 issues triaged", "3 blockers open", "4 issues need repro"]],
    "render-graph-output": [["Compile", "Capture", "Console"], ["Forward+ graph compiled", "18 barriers validated", "1 SSAO warning"]],
    "shader-debugger-output": [["Step", "Variables", "Console"], ["Breakpoint hit in fragment_main", "12 variables inspected", "1 divergent branch"]],
    "texture-streaming-output": [["Requests", "Misses", "Console"], ["512 pages tracked", "18 residency misses", "Texture budget within limit"]],
    "shadow-map-output": [["Capture", "Issues", "Console"], ["4 cascades captured", "2 bias issues found", "Shadow cost 1.8 ms"]],
    "occlusion-output": [["Capture", "Queries", "Console"], ["184 visible objects", "326 objects culled", "64 queries evaluated"]],
    "frame-compare-output": [["Compare", "Regressions", "Console"], ["Frame delta +0.8 ms", "2 regressions flagged", "12 draw calls added"]],
    "material-layer-output": [["Compile", "Preview", "Console"], ["M_Armor layers compiled", "1 wetness warning", "5 layers previewed"]],
    "gpu-memory-output": [["Snapshot", "Compare", "Console"], ["2.4 GB GPU memory used", "Render targets +128 MB", "1 heap warning"]],
    "retarget-output": [["Solve", "Warnings", "Console"], ["68 bones mapped", "2 leg chain warnings", "Preview pose solved"]],
    "ik-solver-output": [["Solve", "Errors", "Console"], ["FullBodyIK solved", "4 effectors evaluated", "1 foot error warning"]],
    "pose-library-output": [["Search", "Preview", "Console"], ["842 poses indexed", "Pose_142 previewed", "18 tags available"]],
    "mocap-output": [["Analyze", "Fixes", "Console"], ["18 mocap issues detected", "12 autofix candidates", "Foot lock applied"]],
    "animation-compression-output": [["Analyze", "Compress", "Console"], ["42 clips analyzed", "68% size saved", "2 compression warnings"]],
    "root-motion-output": [["Extract", "Preview", "Console"], ["Root curve extracted", "4.2 m trajectory", "0 drift detected"]],
    "event-track-output": [["Validate", "Preview", "Console"], ["14 notifies validated", "0 event conflicts", "Attack preview ready"]],
    "montage-debugger-output": [["Trace", "Events", "Console"], ["Attack_Montage trace live", "Section Attack_A active", "14 notifies watched"]],
    "widget-tree-output": [["Inspect", "Invalidation", "Console"], ["128 widgets inspected", "4 invalidations tracked", "Combat_HUD highlighted"]],
    "layout-constraint-output": [["Solve", "Conflicts", "Console"], ["42 constraints solved", "0 layout conflicts", "2 solve passes"]],
    "theme-variant-output": [["Contrast", "Apply", "Console"], ["84 theme tokens checked", "AA contrast passed", "Workbench Dark previewed"]],
    "localization-preview-output": [["Validate", "Overflow", "Console"], ["zh-CN coverage 97%", "2 overflow warnings", "1 missing string"]],
    "focus-navigation-output": [["Trace", "Validate", "Console"], ["42 focus nodes validated", "0 dead ends", "1 loop reviewed"]],
    "input-glyph-output": [["Validate", "Preview", "Console"], ["64 prompts mapped", "0 missing glyphs", "8 locales validated"]],
    "ui-snapshot-output": [["Compare", "Overlay", "Console"], ["4 UI snapshot diffs", "1 diff accepted", "Overlay ready"]],
    "widget-performance-output": [["Capture", "Budget", "Console"], ["Combat_HUD captured", "1.2 ms widget cost", "0 budget failures"]],
    "world-partition-output": [["Loads", "Memory", "Console"], ["42 partition cells scanned", "12 cells loaded", "348 MB streaming memory"]],
    "hlod-output": [["Build", "Warnings", "Console"], ["12 HLOD clusters built", "82% triangle reduction", "1 proxy collision warning"]],
    "level-instance-output": [["Validation", "Overrides", "Console"], ["6 level instances validated", "12 overrides resolved", "1 collision override warning"]],
    "streaming-profiler-output": [["Capture", "Stalls", "Console"], ["Live streaming capture active", "3 load stalls found", "184 MB peak memory"]],
    "scene-bookmark-output": [["Jump", "Share", "Console"], ["18 bookmarks indexed", "3 bookmarks pinned", "Bug_1842 jumped"]],
    "spawn-point-output": [["Validation", "Simulation", "Console"], ["32 spawn points checked", "1 overlap warning", "HangarWave_A simulated"]],
    "collision-matrix-output": [["Validation", "Preview", "Console"], ["18 channels validated", "0 collision conflicts", "12 profiles scanned"]],
    "environment-probe-output": [["Bake", "Coverage", "Console"], ["64 probes queued", "92% average coverage", "2 probe coverage warnings"]],
    "feature-flag-output": [["Rollout", "Validation", "Console"], ["12 flags scanned", "2 staged rollouts", "1 audience warning"]],
    "remote-config-output": [["Validate", "Publish", "Console"], ["Live v42 diff checked", "8 keys changed", "0 validation errors"]],
    "telemetry-query-output": [["Run", "Alerts", "Console"], ["1.2k events queried", "3 telemetry alerts", "Chart refreshed"]],
    "patch-planner-output": [["Validation", "Build", "Console"], ["24 patch changes validated", "3 high risk changes", "0 release blockers"]],
    "dlc-output": [["Package", "Validate", "Console"], ["8 DLC packs scanned", "3 store SKUs ready", "1 entitlement review"]],
    "symbolication-output": [["Resolve", "Symbols", "Console"], ["184 crashes grouped", "92% stacks resolved", "6 symbol files missing"]],
    "segment-output": [["Refresh", "Preview", "Console"], ["42k users refreshed", "3 cohorts active", "2 segment alerts"]],
    "experiment-output": [["Analyze", "Traffic", "Console"], ["3 experiments live", "8 variants analyzed", "1 metric watch warning"]],
  };
  const [tabs, rows] = map[kind] ?? map["prefab-output"];
  return { tabs, rows };
}

function renderToolPropertyPanel(title, rows) {
  const panel = el("div", "tool-property-panel");
  panel.innerHTML = `<div class="section-title">${title}</div>`;
  rows.forEach(([label, value]) => panel.append(fieldRow(label, value)));
  return panel;
}

function renderAudioGraph() {
  const graph = el("div", "audio-graph");
  ["Input", "Filter", "Loop", "Bus", "Output"].forEach((label, index) => {
    const node = el("div", "audio-node");
    node.style.left = `${8 + index * 18}%`;
    node.style.top = `${20 + (index % 2) * 28}%`;
    node.textContent = label;
    graph.append(node);
  });
  return graph;
}

function renderShell(design) {
  const frame = el("div", "design-frame");
  frame.append(renderTopbar(design), renderMainTabs(design), renderWorkspace(design), renderStatusbar(design));
  return frame;
}

function renderTopbar(design) {
  const top = el("div", "topbar");
  const left = el("div", "top-group");
  ["☰", "▱", "▰", "▣"].forEach((label) => left.append(iconButton(label, label === "▣" ? "ghost" : "")));
  left.append(el("div", "divider"));
  ["↶", "↷"].forEach((label) => left.append(iconButton(label, "ghost")));

  const center = el("div", "top-center");
  ["Pointer", "Move", "Rotate", "Scale"].forEach((label, index) => {
    const btn = iconButton(index === 0 ? "⌁" : index === 1 ? "✣" : index === 2 ? "⟳" : "□", index === 0 ? "active" : "ghost");
    btn.title = label;
    center.append(btn);
  });
  center.append(el("div", "divider"));
  ["Snap", "10°", "0.25"].forEach((label) => center.append(pill(label)));

  const right = el("div", "top-actions");
  right.append(pill(mainTabFor(design).label), button("Play", "secondary-btn"));
  ["▦", "☼", "⋮"].forEach((label) => right.append(iconButton(label, "ghost")));

  top.append(left, center, right);
  return top;
}

function renderMainTabs(design) {
  const tabs = el("div", "main-tabs");
  const activeTab = mainTabFor(design);
  const baseTabs = [
    { id: "scene", group: "scene", label: "Scene Editor", meta: "A1_Hangar.scene" },
    { id: "material", group: "material", label: "Material Editor", meta: "M_Metal.zmat" },
    { id: "montage", group: "montage", label: "Montage Editor", meta: "Idle_Run.blend" },
    { id: "ui", group: "ui", label: "UI Asset Editor", meta: "workbench_shell.v2.ui" },
    { id: "assets", group: "assets", label: "Asset Browser", meta: "crate://content" },
    { id: "diagnostics", group: "diagnostics", label: "Diagnostics", meta: "Live Session" },
    { id: "project", group: "project", label: "Project", meta: "Sandbox" },
  ];
  const tabData = baseTabs.map((tab) => (tab.group === activeTab.group ? { ...tab, ...activeTab } : tab));
  tabData.forEach(({ id, label, meta }) => {
    const tab = el("div", `main-tab ${activeTab.id === id ? "active" : ""}`);
    tab.innerHTML = `<span>${label}</span><small>${meta}</small>`;
    tabs.append(tab);
  });
  return tabs;
}

function mainTabFor(design) {
  const explicit = {
    "material-lab": { id: "material", group: "material", label: "Material Editor", meta: "M_Metal.zmat" },
    animation: { id: "montage", group: "montage", label: "Montage Editor", meta: "Idle_Run.blend" },
    "ui-asset-editor": { id: "ui", group: "ui", label: "UI Asset Editor", meta: "workbench_shell.v2.ui" },
    "prefab-editor": { id: "prefab", group: "scene", label: "Prefab Editor", meta: "AudioZone.prefab" },
    "terrain-editor": { id: "terrain", group: "scene", label: "Terrain Editor", meta: "Valley_01" },
    "lighting-bake": { id: "lighting", group: "scene", label: "Lighting Bake", meta: "A1_Hangar" },
    "physics-collision": { id: "physics", group: "scene", label: "Physics Collision", meta: "Box_01" },
    "level-streaming": { id: "streaming", group: "scene", label: "Level Streaming", meta: "A1_Hangar" },
    sequencer: { id: "sequencer", group: "montage", label: "Sequencer", meta: "Intro_Hangar" },
    "navmesh-ai": { id: "navmesh", group: "scene", label: "NavMesh AI", meta: "Agent_Humanoid" },
    "render-pipeline": { id: "render", group: "diagnostics", label: "Render Pipeline", meta: "Forward+" },
    "input-mapping": { id: "input", group: "project", label: "Input Mapping", meta: "Editor_Default" },
    "data-table": { id: "data", group: "assets", label: "Data Table", meta: "items.zdata" },
    "network-replication": { id: "network", group: "diagnostics", label: "Network Replication", meta: "3 peers" },
    localization: { id: "localization", group: "assets", label: "Localization", meta: "zh-CN" },
    "visual-script": { id: "visual-script", group: "assets", label: "Visual Script", meta: "DoorController.zvs" },
    "state-machine": { id: "state-machine", group: "project", label: "State Machine", meta: "CharacterLocomotion" },
    "skeleton-mesh": { id: "skeleton", group: "scene", label: "Skeleton Mesh", meta: "SK_Guard" },
    "texture-editor": { id: "texture", group: "assets", label: "Texture Editor", meta: "T_Grid_01.png" },
    "material-instance": { id: "material-instance", group: "material", label: "Material Instance", meta: "MI_Metal_Rough" },
    "prefab-variant": { id: "prefab-variant", group: "scene", label: "Prefab Variant", meta: "AudioZone / B" },
    "level-audit": { id: "level-audit", group: "diagnostics", label: "Level Audit", meta: "12 issues" },
    "test-runner": { id: "test-runner", group: "diagnostics", label: "Test Runner", meta: "42 passed" },
    "frame-debugger": { id: "frame-debugger", group: "diagnostics", label: "Frame Debugger", meta: "Capture #1842" },
    "memory-profiler": { id: "memory-profiler", group: "diagnostics", label: "Memory Profiler", meta: "1.42 GB" },
    "asset-dependency": { id: "asset-dependency", group: "assets", label: "Asset Dependency", meta: "A1_Hangar.scene" },
    "reference-finder": { id: "reference-finder", group: "assets", label: "Reference Finder", meta: "18 refs" },
    "cook-package": { id: "cook-package", group: "project", label: "Cook Package", meta: "Windows Dev" },
    "crash-session-replay": { id: "crash-replay", group: "diagnostics", label: "Crash Replay", meta: "renderer-panic" },
    "log-analysis": { id: "log-analysis", group: "diagnostics", label: "Log Analysis", meta: "2 warnings" },
    "automation-report": { id: "automation-report", group: "diagnostics", label: "Automation Report", meta: "nightly" },
    "layout-manager": { id: "layout-manager", group: "project", label: "Layout Manager", meta: "default-workbench" },
    "theme-token": { id: "theme-token", group: "assets", label: "Theme Tokens", meta: "workbench-strict" },
    "command-center": { id: "command-center", group: "project", label: "Command Center", meta: "184 commands" },
    "module-graph": { id: "module-graph", group: "project", label: "Module Graph", meta: "zircon_runtime" },
    "asset-validation": { id: "asset-validation", group: "assets", label: "Asset Validation", meta: "7 warnings" },
    "hot-reload": { id: "hot-reload", group: "diagnostics", label: "Hot Reload", meta: "3 dirty" },
    "project-history": { id: "project-history", group: "project", label: "Project History", meta: "12 changes" },
    "task-board": { id: "task-board", group: "project", label: "Task Board", meta: "Sprint 04" },
    "source-control": { id: "source-control", group: "project", label: "Source Control", meta: "14 modified" },
    "review-comments": { id: "review-comments", group: "project", label: "Review Comments", meta: "6 unresolved" },
    "build-farm": { id: "build-farm", group: "diagnostics", label: "Build Farm", meta: "4 agents" },
    "release-notes": { id: "release-notes", group: "project", label: "Release Notes", meta: "0.18.0" },
    "project-settings-page": { id: "project-settings-page", group: "project", label: "Project Settings", meta: "Sandbox" },
    "plugin-development": { id: "plugin-development", group: "project", label: "Plugin Dev", meta: "validation plugin" },
    "remote-device": { id: "remote-device", group: "project", label: "Remote Device", meta: "DevKit-01" },
    "session-sync": { id: "session-sync", group: "project", label: "Session Sync", meta: "3 peers" },
    "cutscene-editor": { id: "cutscene", group: "montage", label: "Cutscene Editor", meta: "Intro_Hangar" },
    "dialogue-editor": { id: "dialogue", group: "assets", label: "Dialogue Editor", meta: "Hangar_Intro" },
    "quest-editor": { id: "quest", group: "project", label: "Quest Editor", meta: "Restore Power" },
    "camera-rig": { id: "camera-rig", group: "scene", label: "Camera Rig", meta: "Crane_A" },
    "control-rig": { id: "control-rig", group: "scene", label: "Control Rig", meta: "SK_Guard" },
    "motion-matching": { id: "motion-matching", group: "montage", label: "Motion Matching", meta: "Locomotion_DB" },
    "facial-animation": { id: "facial", group: "montage", label: "Facial Animation", meta: "Captain_Line_03" },
    "blend-space": { id: "blend-space", group: "montage", label: "Blend Space", meta: "BS_Locomotion_2D" },
    "foliage-editor": { id: "foliage", group: "scene", label: "Foliage Editor", meta: "Valley_01" },
    "scatter-editor": { id: "scatter", group: "scene", label: "Scatter Editor", meta: "RockField_A" },
    "volume-editor": { id: "volume", group: "scene", label: "Volume Editor", meta: "FogZone_A" },
    "weather-editor": { id: "weather", group: "scene", label: "Weather Editor", meta: "StormFront_02" },
    "post-process": { id: "post-process", group: "scene", label: "Post Process", meta: "Hangar_Night" },
    "particle-library": { id: "particle-library", group: "assets", label: "Particle Library", meta: "64 emitters" },
    "collision-proxy": { id: "collision-proxy", group: "scene", label: "Collision Proxy", meta: "12 proxies" },
    "level-variant": { id: "level-variant", group: "scene", label: "Level Variant", meta: "Night" },
    "gameplay-ability": { id: "gameplay-ability", group: "project", label: "Gameplay Ability", meta: "DashStrike" },
    "gameplay-effect": { id: "gameplay-effect", group: "project", label: "Gameplay Effect", meta: "Burning" },
    "ai-perception": { id: "ai-perception", group: "scene", label: "AI Perception", meta: "Guard_01" },
    "spawn-rules": { id: "spawn-rules", group: "project", label: "Spawn Rules", meta: "HangarWave_A" },
    "gameplay-tags": { id: "gameplay-tags", group: "project", label: "Gameplay Tags", meta: "286 tags" },
    "save-data": { id: "save-data", group: "project", label: "Save Data", meta: "schema v12" },
    "world-state": { id: "world-state", group: "project", label: "World State", meta: "184 flags" },
    "telemetry-dashboard": { id: "telemetry", group: "diagnostics", label: "Telemetry", meta: "Live Session" },
    "lobby-editor": { id: "lobby", group: "project", label: "Lobby", meta: "Dev Room" },
    "matchmaking-editor": { id: "matchmaking", group: "project", label: "Matchmaking", meta: "Ranked_2v2" },
    "server-browser": { id: "server-browser", group: "project", label: "Server Browser", meta: "42 servers" },
    "replay-browser": { id: "replay", group: "diagnostics", label: "Replay Browser", meta: "Match_042" },
    "achievements-editor": { id: "achievements", group: "project", label: "Achievements", meta: "38 definitions" },
    "entitlements-editor": { id: "entitlements", group: "project", label: "Entitlements", meta: "FounderPack" },
    "user-profile-editor": { id: "user-profile", group: "project", label: "User Profile", meta: "Player_042" },
    "online-diagnostics": { id: "online-diagnostics", group: "diagnostics", label: "Online Diagnostics", meta: "1 warning" },
    "hud-editor": { id: "hud", group: "ui", label: "HUD Editor", meta: "Combat_HUD" },
    "menu-flow": { id: "menu-flow", group: "ui", label: "Menu Flow", meta: "MainMenu" },
    "font-atlas": { id: "font-atlas", group: "ui", label: "Font Atlas", meta: "Inter_UI" },
    "icon-library": { id: "icons", group: "ui", label: "Icon Library", meta: "286 icons" },
    "ui-binding-editor": { id: "ui-binding-editor", group: "ui", label: "UI Binding", meta: "inventory_panel" },
    "accessibility-audit": { id: "accessibility", group: "ui", label: "Accessibility", meta: "3 issues" },
    "input-prompts": { id: "input-prompts", group: "ui", label: "Input Prompts", meta: "Gamepad_Xbox" },
    "ui-motion": { id: "ui-motion", group: "ui", label: "UI Motion", meta: "Panel_Open" },
    "shader-permutations": { id: "shader-perms", group: "assets", label: "Shader Permutations", meta: "128 variants" },
    "render-targets": { id: "render-targets", group: "diagnostics", label: "Render Targets", meta: "HDR_Main" },
    "gpu-profiler": { id: "gpu-profiler", group: "diagnostics", label: "GPU Profiler", meta: "12.8 ms" },
    "light-probes": { id: "light-probes", group: "scene", label: "Light Probes", meta: "64 probes" },
    "reflection-capture": { id: "reflection", group: "scene", label: "Reflection Capture", meta: "Hangar_Cubemap" },
    "decal-editor": { id: "decal", group: "scene", label: "Decal Editor", meta: "WarningStripe" },
    "virtual-texture": { id: "virtual-texture", group: "assets", label: "Virtual Texture", meta: "82% resident" },
    "material-audit": { id: "material-audit", group: "diagnostics", label: "Material Audit", meta: "7 warnings" },
    "sound-cue": { id: "sound-cue", group: "assets", label: "Sound Cue", meta: "Weapon_Fire" },
    "audio-mixer": { id: "audio-mixer", group: "assets", label: "Audio Mixer", meta: "-18 LUFS" },
    "music-system": { id: "music-system", group: "assets", label: "Music System", meta: "Combat_Loop" },
    "audio-occlusion": { id: "audio-occlusion", group: "scene", label: "Audio Occlusion", meta: "12 traces" },
    "voice-bank": { id: "voice-bank", group: "assets", label: "Voice Bank", meta: "248 lines" },
    "subtitle-timing": { id: "subtitle", group: "assets", label: "Subtitle Timing", meta: "42 cues" },
    "lip-sync": { id: "lip-sync", group: "montage", label: "Lip Sync", meta: "18 phonemes" },
    "audio-profiler": { id: "audio-profiler", group: "diagnostics", label: "Audio Profiler", meta: "64 voices" },
    "rigid-body": { id: "rigid-body", group: "scene", label: "Rigid Body", meta: "Crate_07" },
    "physics-constraints": { id: "constraints", group: "scene", label: "Physics Constraints", meta: "Door_Hinge" },
    "destruction-editor": { id: "destruction", group: "scene", label: "Destruction", meta: "42 chunks" },
    "cloth-simulation": { id: "cloth", group: "scene", label: "Cloth Simulation", meta: "Cape_A" },
    "vehicle-physics": { id: "vehicle", group: "scene", label: "Vehicle Physics", meta: "Rover_01" },
    "fluid-simulation": { id: "fluid", group: "scene", label: "Fluid Simulation", meta: "18k particles" },
    "rope-cable": { id: "rope-cable", group: "scene", label: "Rope Cable", meta: "32 segments" },
    "physics-profiler": { id: "physics-profiler", group: "diagnostics", label: "Physics Profiler", meta: "3.6 ms" },
    "ai-director": { id: "ai-director", group: "project", label: "AI Director", meta: "72 threat" },
    "blackboard-editor": { id: "blackboard", group: "project", label: "Blackboard", meta: "18 keys" },
    "eqs-query": { id: "eqs", group: "scene", label: "EQS Query", meta: "128 candidates" },
    "crowd-simulation": { id: "crowd", group: "scene", label: "Crowd Simulation", meta: "240 agents" },
    "smart-objects": { id: "smart-objects", group: "scene", label: "Smart Objects", meta: "12 slots" },
    "patrol-routes": { id: "patrol-routes", group: "scene", label: "Patrol Routes", meta: "8 waypoints" },
    "cover-system": { id: "cover-system", group: "scene", label: "Cover System", meta: "64 points" },
    "ai-profiler": { id: "ai-profiler", group: "diagnostics", label: "AI Profiler", meta: "32 agents" },
    "mesh-import": { id: "mesh-import", group: "assets", label: "Mesh Import", meta: "SK_Crate.fbx" },
    "lod-chain": { id: "lod-chain", group: "assets", label: "LOD Chain", meta: "SM_Rock_A" },
    "redirect-map": { id: "redirect-map", group: "project", label: "Redirect Map", meta: "42 redirects" },
    "texture-compression-queue": { id: "texture-compression", group: "assets", label: "Texture Compression", meta: "18 queued" },
    "source-asset-trace": { id: "source-trace", group: "assets", label: "Source Asset Trace", meta: "6 outputs" },
    "dcc-live-link": { id: "dcc-live-link", group: "assets", label: "DCC Live Link", meta: "Connected" },
    "metadata-editor": { id: "metadata", group: "project", label: "Metadata Editor", meta: "24 fields" },
    "batch-process-queue": { id: "batch-process", group: "diagnostics", label: "Batch Queue", meta: "64 jobs" },
    "script-editor": { id: "script", group: "project", label: "Script Editor", meta: "0 errors" },
    "api-browser": { id: "api-browser", group: "project", label: "API Browser", meta: "184 symbols" },
    "plugin-packaging": { id: "plugin-packaging", group: "project", label: "Plugin Packaging", meta: "Ready" },
    "module-settings": { id: "module-settings", group: "project", label: "Module Settings", meta: "2 warnings" },
    "automation-suite": { id: "automation-suite", group: "diagnostics", label: "Automation Suite", meta: "306 passed" },
    "build-config": { id: "build-config", group: "project", label: "Build Config", meta: "Development" },
    "cook-rules": { id: "cook-rules", group: "project", label: "Cook Rules", meta: "84 rules" },
    "runtime-commands": { id: "runtime-commands", group: "diagnostics", label: "Runtime Commands", meta: "0 conflicts" },
    "asset-migration": { id: "asset-migration", group: "project", label: "Asset Migration", meta: "128 assets" },
    "scene-diff": { id: "scene-diff", group: "scene", label: "Scene Diff", meta: "18 changes" },
    "prefab-diff": { id: "prefab-diff", group: "project", label: "Prefab Diff", meta: "7 changes" },
    "performance-budget": { id: "performance-budget", group: "diagnostics", label: "Performance Budget", meta: "12.8 ms" },
    "memory-budget": { id: "memory-budget", group: "diagnostics", label: "Memory Budget", meta: "2.1 GB" },
    "dependency-cleanup": { id: "dependency-cleanup", group: "project", label: "Dependency Cleanup", meta: "42 unused" },
    "naming-rules": { id: "naming-rules", group: "project", label: "Naming Rules", meta: "12 warnings" },
    "release-checklist": { id: "release-checklist", group: "project", label: "Release Checklist", meta: "6/8 ready" },
    "gameplay-debugger": { id: "gameplay-debugger", group: "scene", label: "Gameplay Debugger", meta: "Live" },
    "replay-timeline": { id: "replay-timeline", group: "diagnostics", label: "Replay Timeline", meta: "12:42" },
    "network-packet-inspector": { id: "packet-inspector", group: "diagnostics", label: "Packet Inspector", meta: "1.2 KB/f" },
    "latency-map": { id: "latency-map", group: "diagnostics", label: "Latency Map", meta: "42 ms" },
    "input-trace": { id: "input-trace", group: "diagnostics", label: "Input Trace", meta: "184 events" },
    "save-state-diff": { id: "save-state-diff", group: "project", label: "Save State Diff", meta: "12 flags" },
    "repro-recorder": { id: "repro-recorder", group: "diagnostics", label: "Repro Recorder", meta: "Recording" },
    "qa-triage": { id: "qa-triage", group: "project", label: "QA Triage", meta: "3 blocking" },
    "render-graph": { id: "render-graph", group: "scene", label: "Render Graph", meta: "7 passes" },
    "shader-debugger": { id: "shader-debugger", group: "assets", label: "Shader Debugger", meta: "breakpoint" },
    "texture-streaming": { id: "texture-streaming", group: "assets", label: "Texture Streaming", meta: "82%" },
    "shadow-map": { id: "shadow-map", group: "scene", label: "Shadow Map", meta: "4 cascades" },
    "occlusion-culling": { id: "occlusion-culling", group: "scene", label: "Occlusion Culling", meta: "184 visible" },
    "frame-compare": { id: "frame-compare", group: "diagnostics", label: "Frame Compare", meta: "+0.8 ms" },
    "material-layers": { id: "material-layers", group: "assets", label: "Material Layers", meta: "5 layers" },
    "gpu-memory": { id: "gpu-memory", group: "diagnostics", label: "GPU Memory", meta: "2.4 GB" },
    retarget: { id: "retarget", group: "montage", label: "Retarget", meta: "68 bones" },
    "ik-solver": { id: "ik-solver", group: "montage", label: "IK Solver", meta: "4 effectors" },
    "pose-library": { id: "pose-library", group: "montage", label: "Pose Library", meta: "842 poses" },
    "mocap-cleanup": { id: "mocap-cleanup", group: "montage", label: "Mocap Cleanup", meta: "18 issues" },
    "animation-compression": { id: "animation-compression", group: "montage", label: "Animation Compression", meta: "68% saved" },
    "root-motion": { id: "root-motion", group: "montage", label: "Root Motion", meta: "4.2 m" },
    "event-tracks": { id: "event-tracks", group: "montage", label: "Event Tracks", meta: "14 notifies" },
    "montage-debugger": { id: "montage-debugger", group: "diagnostics", label: "Montage Debugger", meta: "Section B" },
    "widget-tree-debugger": { id: "widget-tree-debugger", group: "ui", label: "Widget Tree Debugger", meta: "128 widgets" },
    "layout-constraint-solver": { id: "layout-constraint-solver", group: "ui", label: "Layout Solver", meta: "0 conflicts" },
    "theme-variant-preview": { id: "theme-variant-preview", group: "ui", label: "Theme Preview", meta: "AA" },
    "localization-preview": { id: "localization-preview", group: "ui", label: "Localization Preview", meta: "97%" },
    "focus-navigation": { id: "focus-navigation", group: "ui", label: "Focus Navigation", meta: "0 dead ends" },
    "input-glyph-mapper": { id: "input-glyph-mapper", group: "ui", label: "Input Glyph Mapper", meta: "64 prompts" },
    "ui-snapshot-diff": { id: "ui-snapshot-diff", group: "ui", label: "UI Snapshot Diff", meta: "4 diffs" },
    "widget-performance": { id: "widget-performance", group: "diagnostics", label: "Widget Performance", meta: "1.2 ms" },
    "world-partition": { id: "world-partition", group: "scene", label: "World Partition", meta: "42 cells" },
    "hlod-builder": { id: "hlod-builder", group: "scene", label: "HLOD Builder", meta: "12 clusters" },
    "level-instance": { id: "level-instance", group: "scene", label: "Level Instance", meta: "6 instances" },
    "streaming-profiler": { id: "streaming-profiler", group: "diagnostics", label: "Streaming Profiler", meta: "184 MB" },
    "scene-bookmarks": { id: "scene-bookmarks", group: "scene", label: "Scene Bookmarks", meta: "18 marks" },
    "spawn-point-editor": { id: "spawn-point-editor", group: "scene", label: "Spawn Point Editor", meta: "32 points" },
    "collision-matrix": { id: "collision-matrix", group: "project", label: "Collision Matrix", meta: "0 conflicts" },
    "environment-probes": { id: "environment-probes", group: "scene", label: "Environment Probes", meta: "64 probes" },
    "feature-flags": { id: "feature-flags", group: "project", label: "Feature Flags", meta: "12 flags" },
    "remote-config": { id: "remote-config", group: "project", label: "Remote Config", meta: "Live v42" },
    "telemetry-query": { id: "telemetry-query", group: "diagnostics", label: "Telemetry Query", meta: "1.2k events" },
    "patch-planner": { id: "patch-planner", group: "project", label: "Patch Planner", meta: "0.18.1" },
    "dlc-catalog": { id: "dlc-catalog", group: "project", label: "DLC Catalog", meta: "8 packs" },
    "crash-symbolication": { id: "crash-symbolication", group: "diagnostics", label: "Crash Symbolication", meta: "184 crashes" },
    "player-segment": { id: "player-segment", group: "project", label: "Player Segment", meta: "42k users" },
    "experiment-console": { id: "experiment-console", group: "project", label: "Experiment Console", meta: "3 live" },
    "vfx-editor": { id: "vfx", group: "assets", label: "VFX Editor", meta: "P_Sparks.fx" },
    "shader-editor": { id: "shader", group: "assets", label: "Shader Editor", meta: "unlit.zshader" },
    "audio-editor": { id: "audio", group: "assets", label: "Audio Editor", meta: "Ambience_Hangar" },
    "behavior-tree": { id: "behavior", group: "project", label: "Behavior Tree", meta: "Guard_Patrol.bt" },
    "asset-browser": { id: "assets", group: "assets", label: "Asset Browser", meta: "crate://content" },
    "plugin-manager": { id: "plugins", group: "project", label: "Plugin Manager", meta: "12 installed" },
    "build-export": { id: "build", group: "project", label: "Build Export", meta: "Windows desktop" },
    welcome: { id: "welcome", group: "project", label: "Welcome", meta: "No project open" },
    project: { id: "project", group: "project", label: "Project", meta: "Sandbox" },
  };
  if (explicit[design.center]) return explicit[design.center];
  if (["console", "performance", "runtime-diagnostics"].includes(design.center) || ["console", "performance-events"].includes(design.bottom)) {
    return { id: "diagnostics", group: "diagnostics", label: "Diagnostics", meta: "Live Session" };
  }
  return { id: "scene", group: "scene", label: "Scene Editor", meta: "A1_Hangar.scene" };
}

function renderWorkspace(design) {
  const workspace = el("div", "workspace-grid");
  const drawers = drawerLayoutFor(design);
  const leftStack = el("div", "drawer-stack left");
  const rightStack = el("div", "drawer-stack right");

  leftStack.append(renderDrawer("left-top", drawers.leftTop), renderDrawer("left-bottom", drawers.leftBottom));
  rightStack.append(renderDrawer("right-top", drawers.rightTop), renderDrawer("right-bottom", drawers.rightBottom));

  workspace.append(leftStack, renderMainEditorWindow(design), rightStack, renderBottomDock(design, drawers.bottom));
  return workspace;
}

function renderMainEditorWindow(design) {
  const editor = el("div", "main-editor-window");
  const header = el("div", "editor-window-header");
  const tab = mainTabFor(design);
  header.innerHTML = `<div><strong>${tab.label}</strong><span>${design.status}</span></div>`;
  const actions = el("div", "inline");
  actions.append(selectBtn("Layout"), selectBtn("Split"), iconButton("⋮", "ghost"));
  header.append(actions);

  const body = el("div", "main-editor-body");
  const center = renderCenterPanel(design);
  while (center.firstChild) body.append(center.firstChild);
  editor.append(header, body);
  return editor;
}

function renderDrawer(slot, config) {
  const drawer = el("div", `tool-drawer ${slot} ${config.compact ? "compact" : ""}`);
  const header = el("div", "drawer-header");
  header.innerHTML = `<div><strong>${config.title}</strong><span>${config.zone}</span></div>`;
  const actions = el("div", "drawer-actions");
  actions.append(iconButton("−", "ghost"), iconButton("↗", "ghost"));
  header.append(actions);

  const tabs = el("div", "drawer-tabs");
  config.tabs.forEach((tab, index) => {
    const item = el("div", `drawer-tab ${index === 0 ? "active" : ""}`);
    item.textContent = tab;
    tabs.append(item);
  });

  const body = el("div", "drawer-body");
  body.append(config.body);
  drawer.append(header, tabs, body);
  return drawer;
}

function renderBottomDock(design, config) {
  const dock = el("div", "bottom-dock");
  const header = el("div", "drawer-header dock-header");
  header.innerHTML = `<div><strong>${config.title}</strong><span>${config.zone}</span></div>`;
  const tabs = el("div", "drawer-tabs dock-tabs");
  config.tabs.forEach((tab, index) => {
    const item = el("div", `drawer-tab ${index === 0 ? "active" : ""}`);
    item.textContent = tab;
    tabs.append(item);
  });
  const body = el("div", "dock-body");
  body.append(config.body ?? renderBottomPanelContent(design));
  dock.append(header, tabs, body);
  return dock;
}

function renderBottomPanelContent(design) {
  const bottom = renderBottomPanel(design);
  const body = el("div");
  while (bottom.firstChild) body.append(bottom.firstChild);
  return body;
}

function drawerLayoutFor(design) {
  const fileTree = { title: "Files", zone: "Left Bottom", tabs: ["Project", "Assets"], body: renderAssetTree() };
  const output = { title: "Output", zone: "Bottom", tabs: ["Console", "Problems", "Tasks"], body: renderBottomPanelContent(design) };

  if (isAdditionalEditor(design.center)) {
    return additionalDrawerLayoutFor(design, fileTree);
  }

  if (design.center === "material-lab") {
    return {
      leftTop: { title: "Material Nodes", zone: "Left Top", tabs: ["Palette", "Pinned"], body: renderMaterialCatalog() },
      leftBottom: fileTree,
      rightTop: { title: "Parameters", zone: "Right Top", tabs: ["State", "Tokens"], body: renderComponentState() },
      rightBottom: { title: "Preview Asset", zone: "Right Bottom", tabs: ["Inspector", "Usage"], body: renderAssetDetail() },
      bottom: { title: "Material Output", zone: "Bottom", tabs: ["Preview Log", "Shader", "Console"], body: renderMaterialLog() },
    };
  }

  if (design.center === "ui-asset-editor") {
    return {
      leftTop: { title: "Widget Palette", zone: "Left Top", tabs: ["Controls", "Prefabs"], body: renderUiAssetTree() },
      leftBottom: fileTree,
      rightTop: { title: "UI Tree", zone: "Right Top", tabs: ["Hierarchy", "Slots"], body: renderUiAssetTree() },
      rightBottom: { title: "Properties", zone: "Right Bottom", tabs: ["Inspector", "Bindings"], body: renderUiAssetInspector() },
      bottom: { title: "Diagnostics", zone: "Bottom", tabs: ["Compiler", "Bindings", "Output"], body: renderUiAssetDiagnostics() },
    };
  }

  if (design.center === "animation") {
    return {
      leftTop: { title: "Clip Library", zone: "Left Top", tabs: ["Montages", "Clips"], body: renderAnimationRig() },
      leftBottom: fileTree,
      rightTop: { title: "Skeleton", zone: "Right Top", tabs: ["Hierarchy", "Sockets"], body: renderHierarchy() },
      rightBottom: { title: "Animation List", zone: "Right Bottom", tabs: ["Tracks", "Events"], body: renderAnimationProperties() },
      bottom: { title: "Timeline", zone: "Bottom", tabs: ["Dope Sheet", "Curves", "Events"], body: renderAnimationTimeline() },
    };
  }

  if (design.center === "asset-browser") {
    return {
      leftTop: { title: "Prefab Shelf", zone: "Left Top", tabs: ["Prefabs", "Favorites"], body: renderPrefabShelf() },
      leftBottom: fileTree,
      rightTop: { title: "Hierarchy", zone: "Right Top", tabs: ["Scene", "Selection"], body: renderHierarchy() },
      rightBottom: { title: "Asset Inspector", zone: "Right Bottom", tabs: ["Details", "Import"], body: renderAssetDetail() },
      bottom: { title: "Asset Output", zone: "Bottom", tabs: ["Table", "Import Queue", "Console"], body: renderAssetTablePanel() },
    };
  }

  if (design.center === "performance" || design.center === "runtime-diagnostics" || design.bottom === "console") {
    return {
      leftTop: { title: "Capture Tools", zone: "Left Top", tabs: ["Captures", "Channels"], body: design.center === "runtime-diagnostics" ? renderRuntimeChannels() : renderCaptureList() },
      leftBottom: fileTree,
      rightTop: { title: "Runtime Tree", zone: "Right Top", tabs: ["Systems", "Hierarchy"], body: renderDiagnostics() },
      rightBottom: { title: "Frame Detail", zone: "Right Bottom", tabs: ["Frame", "Events"], body: design.center === "performance" ? renderPerformanceDetail() : renderConsoleDetail() },
      bottom: output,
    };
  }

  if (["project", "plugin-manager", "build-export", "welcome"].includes(design.center)) {
    return {
      leftTop: { title: "Project Tools", zone: "Left Top", tabs: ["Actions", "Templates"], body: renderProjectTree() },
      leftBottom: fileTree,
      rightTop: { title: "Project Structure", zone: "Right Top", tabs: ["Modules", "Plugins"], body: design.center === "plugin-manager" ? renderPluginList() : renderProjectActions() },
      rightBottom: { title: "Details", zone: "Right Bottom", tabs: ["Inspector", "Status"], body: design.center === "build-export" ? renderBuildDetail() : design.center === "plugin-manager" ? renderPluginDetail() : renderWelcomeDetail() },
      bottom: { title: "Project Output", zone: "Bottom", tabs: ["Activity", "Build", "Console"], body: renderBottomPanelContent(design) },
    };
  }

  return {
    leftTop: { title: "Prefabs", zone: "Left Top", tabs: ["Place", "Recent"], body: renderPrefabShelf() },
    leftBottom: fileTree,
    rightTop: { title: "Hierarchy", zone: "Right Top", tabs: ["Scene", "Layers"], body: renderHierarchy(design.context) },
    rightBottom: { title: "Inspector", zone: "Right Bottom", tabs: ["Properties", "History"], body: renderInspector(design.right) },
    bottom: output,
  };
}

function renderPrefabShelf() {
  const shelf = el("div", "prefab-shelf");
  [
    ["Crate", "Box_01"],
    ["Railing", "SM_Railing"],
    ["Light", "AreaLight"],
    ["Audio", "AudioZone"],
    ["Start", "PlayerStart"],
    ["VFX", "P_Sparks"],
  ].forEach(([name, meta], index) => {
    const item = el("div", `prefab-item ${index === 0 ? "selected" : ""}`);
    item.innerHTML = `<div class="prefab-icon"></div><div><strong>${name}</strong><span>${meta}</span></div>`;
    shelf.append(item);
  });
  return shelf;
}

function renderRail(active) {
  const rail = el("div", "rail");
  [
    ["scene", "▶"],
    ["hierarchy", "◇"],
    ["assets", "⌘"],
    ["console", "▤"],
    ["project", "⚙"],
    ["inspector", "?"],
  ].forEach(([id, label]) => {
    const node = el("div", `rail-btn ${active === id ? "active" : ""}`);
    node.textContent = label;
    rail.append(node);
  });
  return rail;
}

function renderLeftPanel(design) {
  if (design.center === "asset-browser") return renderPanel("left-panel", ["Folders", "Labels"], "Folders", renderAssetTree());
  if (design.center === "project") return renderPanel("left-panel", ["Project", "Recent"], "Project", renderProjectTree());
  if (design.center === "material-lab") return renderPanel("left-panel", ["Catalog", "Pinned"], "Catalog", renderMaterialCatalog());
  if (design.center === "ui-asset-editor") return renderPanel("left-panel", ["UI Tree", "Refs"], "UI Tree", renderUiAssetTree());
  if (design.center === "animation") return renderPanel("left-panel", ["Rig", "Clips"], "Rig", renderAnimationRig());
  if (design.center === "performance") return renderPanel("left-panel", ["Captures", "Markers"], "Captures", renderCaptureList());
  if (design.center === "runtime-diagnostics") return renderPanel("left-panel", ["Channels", "Filters"], "Channels", renderRuntimeChannels());
  if (design.center === "plugin-manager") return renderPanel("left-panel", ["Plugins", "Groups"], "Plugins", renderPluginList());
  if (design.center === "build-export") return renderPanel("left-panel", ["Targets", "Profiles"], "Targets", renderBuildTargets());
  if (design.center === "welcome") return renderPanel("left-panel", ["Recent", "Templates"], "Recent", renderWelcomeRecent());
  return renderPanel("left-panel", ["Scene", "Layers"], design.leftTab, renderHierarchy(design.context));
}

function renderCenterPanel(design) {
  const panel = el("div", "panel center-panel");
  if (design.center === "asset-browser") {
    panel.append(renderAssetBrowser());
    return panel;
  }
  if (design.center === "project") {
    panel.append(renderProjectDashboard());
    return panel;
  }
  if (design.center === "material-lab") {
    panel.append(renderMaterialLabPage());
    return panel;
  }
  if (design.center === "ui-asset-editor") {
    panel.append(renderUiAssetEditorPage());
    return panel;
  }
  if (design.center === "animation") {
    panel.append(renderAnimationPage());
    return panel;
  }
  if (design.center === "performance") {
    panel.append(renderPerformancePage());
    return panel;
  }
  if (design.center === "runtime-diagnostics") {
    panel.append(renderRuntimeDiagnosticsPage());
    return panel;
  }
  if (design.center === "plugin-manager") {
    panel.append(renderPluginManagerPage());
    return panel;
  }
  if (design.center === "build-export") {
    panel.append(renderBuildExportPage());
    return panel;
  }
  if (design.center === "welcome") {
    panel.append(renderWelcomePage());
    return panel;
  }
  if (isAdditionalEditor(design.center)) {
    panel.append(renderAdditionalEditorPage(design.center));
    return panel;
  }
  panel.append(renderViewport(design.center === "scene-small"));
  return panel;
}

function renderRightPanel(design) {
  if (design.right === "asset-detail") return renderPanel("right-panel", ["Details", "Import"], "Details", renderAssetDetail());
  if (design.right === "diagnostics") return renderPanel("right-panel", ["Runtime", "History"], "Runtime", renderDiagnostics());
  if (design.right === "project-actions") return renderPanel("right-panel", ["Actions", "Settings"], "Actions", renderProjectActions());
  if (design.right === "component-state") return renderPanel("right-panel", ["State", "Tokens"], "State", renderComponentState());
  if (design.right === "ui-asset-inspector") return renderPanel("right-panel", ["Inspector", "Bindings"], "Inspector", renderUiAssetInspector());
  if (design.right === "animation-properties") return renderPanel("right-panel", ["Clip", "Track"], "Clip", renderAnimationProperties());
  if (design.right === "performance-detail") return renderPanel("right-panel", ["Frame", "GPU"], "Frame", renderPerformanceDetail());
  if (design.right === "plugin-detail") return renderPanel("right-panel", ["Plugin", "Health"], "Plugin", renderPluginDetail());
  if (design.right === "build-detail") return renderPanel("right-panel", ["Profile", "Output"], "Profile", renderBuildDetail());
  if (design.right === "welcome-detail") return renderPanel("right-panel", ["Help", "Version"], "Help", renderWelcomeDetail());
  if (isAdditionalDetail(design.right)) return renderPanel("right-panel", ["Details", "State"], "Details", renderAdditionalDetail(design.right));
  return renderPanel("right-panel", ["Inspector", "History"], "Inspector", renderInspector(design.right));
}

function renderBottomPanel(design) {
  const panel = el("div", "panel bottom-panel");
  if (design.bottom === "console") {
    panel.append(renderConsole());
    return panel;
  }
  if (design.bottom === "asset-table") {
    panel.append(renderAssetTablePanel());
    return panel;
  }
  if (design.bottom === "project-activity") {
    panel.append(renderProjectActivity());
    return panel;
  }
  if (design.bottom === "hierarchy-log") {
    panel.append(renderHierarchyLog());
    return panel;
  }
  if (design.bottom === "material-log") {
    panel.append(renderMaterialLog());
    return panel;
  }
  if (design.bottom === "ui-asset-diagnostics") {
    panel.append(renderUiAssetDiagnostics());
    return panel;
  }
  if (design.bottom === "animation-timeline") {
    panel.append(renderAnimationTimeline());
    return panel;
  }
  if (design.bottom === "performance-events") {
    panel.append(renderPerformanceEvents());
    return panel;
  }
  if (design.bottom === "plugin-log") {
    panel.append(renderPluginLog());
    return panel;
  }
  if (design.bottom === "build-log") {
    panel.append(renderBuildLog());
    return panel;
  }
  if (design.bottom === "welcome-status") {
    panel.append(renderWelcomeStatus());
    return panel;
  }
  if (isAdditionalOutput(design.bottom)) {
    panel.append(renderAdditionalOutput(design.bottom));
    return panel;
  }
  panel.append(renderComponentLab());
  return panel;
}

function renderPanel(positionClass, tabs, active, body) {
  const panel = el("div", `panel ${positionClass}`);
  const header = el("div", "panel-header");
  const tabRow = el("div", "panel-tabs");
  tabs.forEach((tab) => {
    const node = el("div", `panel-tab ${tab === active ? "active" : ""}`);
    node.textContent = tab;
    tabRow.append(node);
  });
  const actions = el("div", "inline");
  actions.append(iconButton("+", "ghost"), iconButton("⋮", "ghost"));
  header.append(tabRow, actions);
  const content = el("div", "panel-content");
  content.append(body);
  panel.append(header, content);
  return panel;
}

function renderHierarchy(context) {
  const wrap = el("div");
  const search = el("div", "search-row");
  search.append(field("Search..."), squareButton("⌕"), squareButton("+"));
  const tree = el("div", "tree");
  treeRows.forEach(([name, depth, selected]) => {
    const row = el("div", `tree-row depth-${depth} ${selected ? "selected" : ""}`);
    row.innerHTML = `<span>${depth < 2 ? "⌄" : ""}</span><span class="name">${name}</span><span class="muted">◉</span><span class="muted">▣</span>`;
    tree.append(row);
  });
  wrap.append(search, tree);
  if (context === "hierarchy") {
    const menu = renderContextMenu(["Create Child", "Duplicate", "Rename", "Focus Selection", "Delete"], 112, 242);
    wrap.append(menu);
  }
  return wrap;
}

function renderViewport(small = false) {
  const shell = el("div", "viewport-shell");
  const canvas = el("div", "scene-canvas");
  const wall = el("div", "scene-wall");
  const toolbar = el("div", "viewport-toolbar");
  const left = el("div", "tool-cluster");
  left.append(selectBtn("Perspective"), selectBtn("Lit"));
  const right = el("div", "tool-cluster");
  ["⌘", "⧉", "⌁", "U"].forEach((label, index) => right.append(miniButton(label, index === 1)));
  right.append(selectBtn("10°"), selectBtn("0.25"));
  toolbar.append(left, right);
  shell.append(canvas, wall, toolbar, el("div", "crate"), el("div", "crate big"), renderGizmo(), renderWorldAxis());
  const mini = el("div", "viewport-mini-axis");
  mini.textContent = "Y  X";
  shell.append(mini);
  if (small) shell.style.filter = "saturate(0.9) brightness(0.86)";
  return shell;
}

function renderGizmo() {
  const node = el("div", "gizmo");
  node.innerHTML = `<div class="axis-line x"></div><div class="axis-line y"></div><div class="axis-line z"></div><div class="axis-label x">X</div><div class="axis-label y">Y</div><div class="axis-label z">Z</div>`;
  return node;
}

function renderWorldAxis() {
  const node = el("div", "world-axis");
  node.innerHTML = `<div class="cube-face"></div><span class="axis-label y">Y</span><span class="axis-label x">X</span><span class="axis-label z">Z</span>`;
  return node;
}

function renderInspector(mode = "inspector") {
  const root = el("div", "inspector-root");
  root.append(objectHeader());
  const form = el("div", "form-grid");
  form.append(formField("Tag", "Untagged"), formField("Layer", "Default"));
  root.append(form);
  root.append(transformSection(mode === "inspector-deep"));
  root.append(meshSection(mode === "inspector-deep"));
  if (mode !== "inspector-compact") root.append(lightingSection());
  root.append(button("+ Add Component", "secondary-btn"));
  return root;
}

function objectHeader() {
  const row = el("div", "object-row");
  row.innerHTML = `<span class="cube-icon"></span><strong>Props</strong><span class="muted" style="margin-left:auto">Static</span><span class="box"></span><span class="muted">⋮</span>`;
  return row;
}

function transformSection(deep) {
  const sec = el("div", "section");
  sec.append(sectionHead("Transform", true));
  ["Position", "Rotation", "Scale"].forEach((label, index) => {
    const values = index === 0 ? ["128.4", "64.2", "-32.7"] : index === 1 ? ["0°", "90°", "0°"] : ["1.00", "1.00", "1.00"];
    sec.append(axisValues(label, values));
  });
  if (deep) sec.append(axisValues("Pivot", ["Center", "Local", "On"]));
  return sec;
}

function meshSection(deep) {
  const sec = el("div", "section");
  sec.append(sectionHead("Mesh Renderer", true));
  sec.append(fieldRow("Mesh", "Box_01"));
  sec.append(fieldRow("Material", "M_Metal", true));
  if (deep) {
    sec.append(fieldRow("Shader", "unlit.zshader"));
    sec.append(fieldRow("Variant", "SRP Preview"));
  }
  return sec;
}

function lightingSection() {
  const sec = el("div", "section");
  sec.append(sectionHead("Lighting", false));
  sec.append(fieldRow("Cast Shadows", "On"));
  const row = el("div", "checkbox-row");
  row.innerHTML = `<span class="box checked"></span><span>Receive Shadows</span>`;
  sec.append(row);
  return sec;
}

function renderComponentLab() {
  const root = el("div");
  const tabs = el("div", "bottom-tabs");
  tabs.innerHTML = `<span class="panel-tab active">UI Components</span><span class="panel-tab">Console</span>`;
  const body = el("div", "bottom-body");
  body.append(renderButtonsCol(), renderInputsCol(), renderSliderCol(), renderListCol());
  root.append(tabs, body);
  return root;
}

function renderButtonsCol() {
  const col = el("div", "lab-col");
  col.innerHTML = `<div class="lab-title">Buttons</div>`;
  const buttons = el("div", "button-grid");
  buttons.append(button("Primary", "primary-btn"), button("Secondary", "secondary-btn"), button("Tertiary", "secondary-btn"), button("Outline", "secondary-btn"));
  const icons = el("div", "icon-grid");
  ["+", "▰", "▣", "⌫", "◉", "⊘", "▣", "⋯"].forEach((label) => icons.append(miniButton(label)));
  col.append(buttons, spacer(12), icons);
  return col;
}

function renderInputsCol() {
  const col = el("div", "lab-col");
  col.innerHTML = `<div class="lab-title">Inputs And Checks</div>`;
  const stack = el("div", "control-stack");
  stack.append(field("Text field"), field("Focused input"), field("Disabled input"));
  ["Checkbox", "Checkbox", "Radio option", "Radio option"].forEach((label, index) => {
    const row = el("div", index > 1 ? "radio-row" : "checkbox-row");
    row.innerHTML = `<span class="${index > 1 ? "radio" : "box"} ${index === 0 || index === 2 ? "checked" : ""}"></span><span>${label}</span>`;
    stack.append(row);
  });
  col.append(stack);
  return col;
}

function renderSliderCol() {
  const col = el("div", "lab-col");
  col.innerHTML = `<div class="lab-title">Sliders And Tabs</div>`;
  col.append(slider("Value", "0.75"), slider("Range", "0.80"), slider("Steps", "3"));
  const segs = el("div", "inline");
  ["Left", "Center", "Right"].forEach((label) => {
    const node = el("div", `seg ${label === "Center" ? "active" : ""}`);
    node.textContent = label;
    node.style.height = "30px";
    node.style.minWidth = "72px";
    node.style.display = "grid";
    node.style.placeItems = "center";
    node.style.borderColor = "#30383e";
    segs.append(node);
  });
  col.append(spacer(12), segs);
  return col;
}

function renderListCol() {
  const col = el("div", "lab-col");
  col.innerHTML = `<div class="lab-title">List</div>`;
  const table = el("div", "table");
  ["List item", "Selected item", "Disabled item", "More Tools"].forEach((label, index) => {
    const row = el("div", `table-row ${index === 1 ? "selected" : ""}`);
    row.innerHTML = `<span>${label}</span><span></span><span></span><span>${index === 1 ? "Selected" : ""}</span><span>${index === 1 ? "✓" : "›"}</span>`;
    table.append(row);
  });
  col.append(table, spacer(12), renderAlerts());
  return col;
}

function renderAlerts() {
  const wrap = el("div", "control-stack");
  [["Info Alert", "info"], ["Success Alert", "green"], ["Warning Alert", "yellow"], ["Error Alert", "red"]].forEach(([label, tone]) => {
    const row = el("div", "field");
    row.style.borderColor = tone === "red" ? "rgba(235,106,93,.5)" : tone === "yellow" ? "rgba(226,179,72,.5)" : "rgba(53,199,208,.45)";
    row.textContent = label;
    wrap.append(row);
  });
  return wrap;
}

function renderAssetBrowser() {
  const root = el("div", "asset-workspace");
  const toolbar = el("div", "asset-toolbar");
  toolbar.append(button("Folder", "secondary-btn"), button("Asset", "secondary-btn"), field("Search assets..."), selectBtn("Kind: all"), selectBtn("Grid"), button("Import", "primary-btn"));
  const body = el("div", "asset-body");
  body.append(renderAssetTree(), renderAssetGrid(), renderAssetDetail());
  root.append(toolbar, body);
  return root;
}

function renderAssetTree() {
  const wrap = el("div", "asset-sidebar");
  ["crate://", "content", "scenes", "materials", "meshes", "textures", "ui"].forEach((name, index) => {
    const row = el("div", `list-row ${index === 2 ? "selected" : ""}`);
    row.style.padding = "0 8px";
    row.textContent = `${index < 2 ? "⌄" : "  "} ${name}`;
    wrap.append(row);
  });
  return wrap;
}

function renderAssetGrid() {
  const grid = el("div", "asset-grid");
  assetItems.forEach(([name, kind], index) => {
    const tile = el("div", `asset-tile ${index === 1 ? "selected" : ""}`);
    tile.innerHTML = `<div class="thumb-img ${kind === "material" ? "material" : kind === "scene" ? "scene" : ""}"></div><div class="asset-name">${name}</div><div class="asset-meta">${kind}</div>`;
    grid.append(tile);
  });
  return grid;
}

function renderAssetDetail() {
  const detail = el("div", "asset-detail");
  detail.innerHTML = `<div class="section-title">Box_01.mesh</div><div class="muted" style="margin:4px 0 12px">Mesh asset / 2.4 MB</div>`;
  detail.append(fieldRow("Import", "glTF 2.0"));
  detail.append(fieldRow("Triangles", "1,248"));
  detail.append(fieldRow("Materials", "M_Metal"));
  detail.append(fieldRow("Readiness", "Ready"));
  detail.append(spacer(10), button("Reimport", "primary-btn"), spacer(8), button("Locate References", "secondary-btn"));
  return detail;
}

function renderAssetTablePanel() {
  const root = el("div");
  const tabs = el("div", "bottom-tabs");
  tabs.innerHTML = `<span class="panel-tab active">References</span><span class="panel-tab">Import Log</span><span class="panel-tab">Package</span>`;
  const table = el("div", "table");
  table.style.margin = "12px";
  [["Name", "Type", "Size", "Modified", ""], ["Box_01.mesh", "Mesh", "2.4 MB", "2m ago", "⋮"], ["M_Metal.zmat", "Material", "512 KB", "10m ago", "⋮"], ["T_Grid_01.png", "Texture", "1.2 MB", "1h ago", "⋮"]].forEach((cols, index) => {
    const row = el("div", `table-row ${index === 0 ? "head" : index === 1 ? "selected" : ""}`);
    row.innerHTML = cols.map((col) => `<span>${col}</span>`).join("");
    table.append(row);
  });
  root.append(tabs, table);
  return root;
}

function renderConsole() {
  const root = el("div", "console-workspace");
  root.style.gridTemplateRows = "42px 1fr";
  const toolbar = el("div", "console-toolbar");
  toolbar.style.gap = "8px";
  toolbar.style.padding = "6px 10px";
  toolbar.style.background = "#151a1e";
  toolbar.style.borderBottom = "1px solid var(--line)";
  toolbar.append(button("Clear", "secondary-btn"), selectBtn("All"), selectBtn("Warnings"), field("Filter console..."), button("Open Detail", "primary-btn"));
  root.append(toolbar, renderLogList());
  return root;
}

function renderLogList() {
  const list = el("div", "log-list");
  logRows.forEach(([time, level, message]) => {
    const row = el("div", `log-row ${level.toLowerCase()}`);
    row.innerHTML = `<span>${time}</span><span>${level}</span><span>${message}</span>`;
    list.append(row);
  });
  return list;
}

function renderDiagnostics() {
  const wrap = el("div", "control-stack");
  [["Frame", "16.6 ms"], ["Draw calls", "128"], ["UI nodes", "318"], ["Warnings", "2"], ["GPU", "WGPU preview"]].forEach(([label, value]) => wrap.append(fieldRow(label, value)));
  wrap.append(spacer(10), renderLogList());
  return wrap;
}

function renderProjectDashboard() {
  const root = el("div", "project-workspace");
  root.append(renderProjectTree());
  const main = el("div", "project-main");
  const grid = el("div", "dashboard-grid");
  [["42", "Assets"], ["6", "Open scenes"], ["2", "Warnings"], ["98%", "Shader ready"], ["12", "Plugins"], ["4", "Tasks"]].forEach(([value, label]) => {
    const metric = el("div", "metric");
    metric.innerHTML = `<div class="metric-value">${value}</div><div class="metric-label">${label}</div>`;
    grid.append(metric);
  });
  main.append(grid, renderActionList());
  root.append(main, renderProjectActions());
  return root;
}

function renderProjectTree() {
  const side = el("div", "project-side");
  ["Overview", "Scenes", "Assets", "Builds", "Plugins", "Settings"].forEach((label, index) => {
    const row = el("div", `list-row ${index === 0 ? "selected" : ""}`);
    row.style.padding = "0 8px";
    row.textContent = label;
    side.append(row);
  });
  return side;
}

function renderActionList() {
  const list = el("div", "action-list");
  [["Open Scene", "A1_Hangar.scene", "Open"], ["Import Asset", "Bring model, texture, or audio into content", "Import"], ["Build Desktop", "Package current profile for Windows", "Build"], ["Run Tests", "Validate runtime and editor contracts", "Run"]].forEach(([title, desc, action]) => {
    const item = el("div", "action-item");
    item.innerHTML = `<span class="square-btn">▣</span><div><strong>${title}</strong><div class="muted">${desc}</div></div><button class="secondary-btn">${action}</button>`;
    list.append(item);
  });
  return list;
}

function renderProjectActions() {
  const wrap = el("div", "asset-detail");
  wrap.innerHTML = `<div class="section-title">Project Actions</div><div class="muted" style="margin:4px 0 12px">Zircon Labs / Sandbox</div>`;
  ["Open Assets", "Asset Browser", "Build Export", "Plugin Manager", "Project Settings"].forEach((label, index) => wrap.append(button(label, index === 0 ? "primary-btn" : "secondary-btn"), spacer(8)));
  return wrap;
}

function renderProjectActivity() {
  const root = el("div");
  const tabs = el("div", "bottom-tabs");
  tabs.innerHTML = `<span class="panel-tab active">Activity</span><span class="panel-tab">Tasks</span><span class="panel-tab">Package</span>`;
  const list = el("div", "log-list");
  ["Opened A1_Hangar.scene", "Imported Box_01.mesh", "Compiled unlit.zshader", "Updated UI surface frame", "Saved Project Overview layout"].forEach((msg, index) => {
    const row = el("div", `log-row ${index === 2 ? "warn" : "info"}`);
    row.innerHTML = `<span>12:0${index}:00</span><span>${index === 2 ? "Warn" : "Info"}</span><span>${msg}</span>`;
    list.append(row);
  });
  root.append(tabs, list);
  return root;
}

function renderHierarchyLog() {
  const root = el("div");
  const tabs = el("div", "bottom-tabs");
  tabs.innerHTML = `<span class="panel-tab active">Selection</span><span class="panel-tab">Operations</span><span class="panel-tab">Console</span>`;
  const body = el("div", "bottom-body");
  body.style.gridTemplateColumns = "420px 1fr";
  const left = el("div", "lab-col");
  left.append(renderHierarchy());
  const right = el("div", "lab-col");
  right.append(renderLogList());
  body.append(left, right);
  root.append(tabs, body);
  return root;
}

function renderMaterialCatalog() {
  return navList(["Inputs", "Surfaces", "Navigation", "Data Display", "Charts", "MUI X"], 1);
}

function renderUiAssetTree() {
  return treeList([
    ["WorkbenchShell", 0, false],
    ["TopBar", 1, false],
    ["ActivityRail", 1, false],
    ["DocumentHost", 1, true],
    ["SceneViewport", 2, false],
    ["InspectorSlot", 2, false],
    ["StatusBar", 1, false],
  ]);
}

function renderAnimationRig() {
  return treeList([
    ["CharacterRoot", 0, false],
    ["Hips", 1, true],
    ["Spine", 2, false],
    ["Head", 3, false],
    ["Arm.L", 2, false],
    ["Arm.R", 2, false],
    ["Leg.L", 2, false],
    ["Leg.R", 2, false],
  ]);
}

function renderCaptureList() {
  return navList(["Live frame", "Startup capture", "Idle hover", "Click scenario", "UI rebuild", "GPU pass"], 0);
}

function renderRuntimeChannels() {
  return navList(["Render", "Assets", "Tasks", "Input", "Plugins", "UI Surface", "Audio"], 0);
}

function renderPluginList() {
  return navList(["Hybrid GI", "Virtual Geometry", "Sound Runtime", "Texture Importer", "Navigation", "Solari"], 2);
}

function renderBuildTargets() {
  return navList(["Windows Desktop", "Linux Desktop", "Editor Debug", "Runtime Shipping", "Package Preview"], 0);
}

function renderWelcomeRecent() {
  return navList(["Zircon Sandbox", "Material Lab", "Audio Prototype", "Navigation Testbed", "Empty Project"], 0);
}

function renderMaterialLabPage() {
  const page = el("div", "tool-page");
  page.append(toolToolbar(["Material Lab", "Compact", "Dark", "Selected state", "Export PNG"]));
  const grid = el("div", "component-matrix");
  [
    ["Buttons", renderButtonsCol()],
    ["Inputs", renderInputsCol()],
    ["Selection", renderSliderCol()],
    ["Lists", renderListCol()],
    ["Data Grid", renderMiniDataGrid()],
    ["Charts", renderMiniChart()],
  ].forEach(([title, content]) => grid.append(toolCard(title, content)));
  page.append(grid);
  return page;
}

function renderUiAssetEditorPage() {
  const page = el("div", "tool-page");
  page.append(toolToolbar(["UI Asset", "Source", "Preview", "Compile", "Save"]));
  const split = el("div", "editor-split");
  split.append(codePanel(), previewPanel(), diagnosticsPanel("UI asset compile clean", ["14 bindings", "0 missing resources", "318 nodes"]));
  page.append(split);
  return page;
}

function renderAnimationPage() {
  const page = el("div", "tool-page");
  page.append(toolToolbar(["Animation", "Dope Sheet", "Curves", "Loop", "Record"]));
  const split = el("div", "animation-grid");
  split.append(renderGraphCanvas(), renderCurvePanel());
  page.append(split);
  return page;
}

function renderPerformancePage() {
  const page = el("div", "tool-page");
  page.append(toolToolbar(["Performance", "Capture", "Compare", "Markers", "Export"]));
  const grid = el("div", "perf-grid");
  grid.append(renderFrameChart(), renderPerfMetrics(), renderPassBreakdown());
  page.append(grid);
  return page;
}

function renderRuntimeDiagnosticsPage() {
  const page = el("div", "tool-page");
  page.append(toolToolbar(["Runtime Diagnostics", "Live", "Frame", "Assets", "Tasks"]));
  const grid = el("div", "runtime-grid");
  grid.append(renderDiagnosticTimeline(), renderDiagnosticTable());
  page.append(grid);
  return page;
}

function renderPluginManagerPage() {
  const page = el("div", "tool-page");
  page.append(toolToolbar(["Plugin Manager", "Installed", "Native", "Runtime", "Refresh"]));
  const grid = el("div", "plugin-grid");
  ["Hybrid GI", "Virtual Geometry", "Sound Runtime", "Texture Importer", "Navigation", "Solari"].forEach((name, index) => {
    const card = el("div", `plugin-card ${index === 2 ? "selected" : ""}`);
    card.innerHTML = `<strong>${name}</strong><span class="muted">${index === 2 ? "warning: backend route missing" : "ready"}</span><span class="pill">${index % 2 ? "Runtime" : "Editor"}</span>`;
    grid.append(card);
  });
  page.append(grid);
  return page;
}

function renderBuildExportPage() {
  const page = el("div", "tool-page");
  page.append(toolToolbar(["Build Export", "Validate", "Package", "Run", "Open Folder"]));
  const grid = el("div", "build-grid");
  grid.append(renderBuildPipeline(), renderBuildSettings());
  page.append(grid);
  return page;
}

function renderWelcomePage() {
  const page = el("div", "welcome-page");
  const recent = el("div", "welcome-panel");
  recent.innerHTML = `<div class="section-title">Recent Projects</div>`;
  recent.append(navList(["Zircon Sandbox", "Material Lab", "Audio Prototype", "Navigation Testbed"], 0));
  const create = el("div", "welcome-panel callout");
  create.innerHTML = `<div class="section-title">Create Project</div>`;
  create.append(fieldRow("Name", "Zircon Project"), fieldRow("Location", "Documents/ZirconProjects"), fieldRow("Template", "3D Editor Sandbox"), button("Create Project", "primary-btn"));
  const info = el("div", "welcome-panel");
  info.innerHTML = `<div class="section-title">Engine Status</div>`;
  info.append(fieldRow("Version", "0.1 workbench"), fieldRow("Renderer", "WGPU"), fieldRow("UI", "Retained host"), fieldRow("Assets", "Ready"));
  page.append(recent, create, info);
  return page;
}

function renderComponentState() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Selected", "Button / Primary"), fieldRow("State", "Hovered + Focused"), fieldRow("Token", "material_accent"), fieldRow("Density", "Compact 32px"));
  wrap.append(renderMiniChart());
  return wrap;
}

function renderUiAssetInspector() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Node", "DocumentHost"), fieldRow("Component", "Slot"), fieldRow("Binding", "workbench.document"), fieldRow("Clip", "true"), fieldRow("Dirty domain", "Layout"));
  return wrap;
}

function renderAnimationProperties() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Clip", "Idle_Run"), fieldRow("Duration", "2.4 s"), fieldRow("Sample rate", "30 fps"), fieldRow("Root motion", "Enabled"), fieldRow("Blend", "0.18"));
  return wrap;
}

function renderPerformanceDetail() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Frame", "16.6 ms"), fieldRow("UI layout", "1.8 ms"), fieldRow("Paint", "2.2 ms"), fieldRow("GPU", "10.4 ms"), fieldRow("Idle", "2.2 ms"));
  wrap.append(renderFrameChart());
  return wrap;
}

function renderPluginDetail() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Plugin", "Sound Runtime"), fieldRow("ABI", "native-dynamic v3"), fieldRow("Status", "Warning"), fieldRow("Services", "12"), fieldRow("Last load", "12:04:31"));
  wrap.append(button("Open Diagnostics", "primary-btn"));
  return wrap;
}

function renderBuildDetail() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Target", "Windows Desktop"), fieldRow("Profile", "Editor Debug"), fieldRow("Assets", "Package all"), fieldRow("Renderer", "Default"), fieldRow("Output", "target/export/windows"));
  wrap.append(button("Validate Profile", "primary-btn"));
  return wrap;
}

function renderWelcomeDetail() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Engine", "Zircon"), fieldRow("Workspace", "E:/Git/ZirconEngine"), fieldRow("Last opened", "Today"), fieldRow("Templates", "6 available"));
  wrap.append(renderAlerts());
  return wrap;
}

function renderMaterialLog() {
  return tabbedLog(["Component States", "Token Audit", "Visual Diff"], ["Button hover token resolved", "Data grid selection uses teal active row", "Chart surface uses panel fill"]);
}

function renderUiAssetDiagnostics() {
  return tabbedLog(["Diagnostics", "Bindings", "Resources"], ["Compiled workbench_shell.v2.ui.toml", "Binding route Workbench.OpenScene valid", "No missing icon references"]);
}

function renderAnimationTimeline() {
  const root = el("div", "timeline-panel");
  const tabs = el("div", "bottom-tabs");
  tabs.innerHTML = `<span class="panel-tab active">Timeline</span><span class="panel-tab">Curves</span><span class="panel-tab">Events</span>`;
  root.append(tabs, renderTimelineBody());
  return root;
}

function renderPerformanceEvents() {
  return tabbedLog(["Events", "Frames", "GPU"], ["Frame 142 captured", "UI layout invalidated: 18 nodes", "Render pass submitted: 7 batches"]);
}

function renderPluginLog() {
  return tabbedLog(["Plugin Log", "ABI", "Services"], ["Sound Runtime loaded with warning", "Texture Importer ready", "Navigation runtime waiting for scene"]);
}

function renderBuildLog() {
  return tabbedLog(["Build Log", "Validation", "Artifacts"], ["Profile validated", "Shader package ready", "Export task waiting for target folder"]);
}

function renderWelcomeStatus() {
  return tabbedLog(["Status", "Templates", "Recent"], ["No project open", "3D Editor Sandbox template selected", "Recent projects scanned"]);
}

function renderFocusLayer(design) {
  const layer = el("div", "focus-layer");
  const wide = ["scene-toolbar", "scene-gizmo", "asset-grid", "console-filter", "project-dashboard"].includes(design.focus);
  const compact = ["hierarchy-context-menu", "inspector-transform", "inspector-material", "asset-import", "console-detail", "project-actions"].includes(design.focus);
  const win = el("div", `focus-window ${wide ? "wide" : ""} ${compact ? "compact" : ""}`);
  const title = el("div", "focus-title");
  title.innerHTML = `<strong>${design.title}</strong><span class="muted">Main-tab drawer panel detail</span>`;
  const body = el("div", "focus-body");
  body.append(renderFocusBody(design.focus));
  win.append(title, body);
  layer.append(win);
  return layer;
}

function renderFocusBody(focus) {
  if (focus === "scene-toolbar") {
    const wrap = el("div", "control-stack");
    const viewport = renderViewport();
    viewport.classList.add("callout");
    viewport.style.height = "520px";
    wrap.append(viewport, renderToolbarSpec());
    return wrap;
  }
  if (focus === "scene-gizmo") {
    const grid = el("div", "focus-grid-2");
    const viewport = renderViewport();
    viewport.classList.add("callout");
    grid.append(viewport, renderGizmoSpec());
    return grid;
  }
  if (focus === "hierarchy-selection") return renderHierarchy();
  if (focus === "hierarchy-context-menu") {
    const wrap = el("div");
    wrap.style.position = "relative";
    wrap.append(renderHierarchy("hierarchy"));
    return wrap;
  }
  if (focus === "inspector-transform") {
    const wrap = el("div", "inspector-root");
    wrap.append(objectHeader(), transformSection(true));
    return wrap;
  }
  if (focus === "inspector-material") {
    const wrap = el("div", "inspector-root");
    wrap.append(objectHeader(), meshSection(true), lightingSection(), button("+ Add Component", "secondary-btn"));
    return wrap;
  }
  if (focus === "asset-grid") return renderAssetBrowser();
  if (focus === "asset-import") return renderAssetDetail();
  if (focus === "console-filter") return renderConsole();
  if (focus === "console-detail") return renderConsoleDetail();
  if (focus === "project-dashboard") return renderProjectDashboard();
  if (focus === "project-actions") return renderProjectActions();
  if (focus === "material-components") return renderMaterialLabPage();
  if (focus === "ui-asset-tree") return renderUiAssetEditorPage();
  if (focus === "animation-timeline") return renderAnimationTimeline();
  if (focus === "performance-frame") return renderPerformancePage();
  if (focus === "runtime-events") return renderRuntimeDiagnosticsPage();
  if (focus === "plugin-detail") return renderPluginDetail();
  if (focus === "build-targets") return renderBuildExportPage();
  if (focus === "welcome-new-project") return renderWelcomePage();
  return renderViewport();
}

function renderToolbarSpec() {
  const row = el("div", "table");
  [["Control", "State", "Size", "Purpose", ""], ["Perspective", "Default", "104x30", "Camera projection", ""], ["Lit", "Default", "62x30", "Shading mode", ""], ["Grid", "Active", "30x30", "Grid overlay", ""], ["Snap", "Active", "92x30", "Snap step", ""]].forEach((cols, index) => {
    const node = el("div", `table-row ${index === 0 ? "head" : index === 3 ? "selected" : ""}`);
    node.innerHTML = cols.map((col) => `<span>${col}</span>`).join("");
    row.append(node);
  });
  return row;
}

function renderGizmoSpec() {
  const spec = el("div", "control-stack");
  spec.append(fieldRow("Selection", "Props / Box_01"));
  spec.append(fieldRow("Gizmo", "Translate"));
  spec.append(fieldRow("X Axis", "red handle"));
  spec.append(fieldRow("Y Axis", "green handle"));
  spec.append(fieldRow("Z Axis", "blue handle"));
  spec.append(fieldRow("Bounds", "teal outline"));
  return spec;
}

function renderConsoleDetail() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Level", "Warning"), fieldRow("Source", "Texture Importer"), fieldRow("Asset", "T_Grid_01.png"));
  const detail = el("div", "log-list");
  detail.style.height = "300px";
  detail.innerHTML = `<div class="log-row warn"><span>12:04:25</span><span>Warn</span><span>Texture T_Grid_01 uses fallback sampler</span></div><div style="padding:14px;line-height:1.7;color:#cbd5d9">Sampler descriptor is missing anisotropy metadata. The runtime preview falls back to linear clamp while the asset importer keeps the original package unchanged.</div>`;
  wrap.append(detail, button("Open Import Settings", "primary-btn"));
  return wrap;
}

function navList(items, selectedIndex) {
  const wrap = el("div", "control-stack");
  items.forEach((item, index) => {
    const row = el("div", `list-row ${index === selectedIndex ? "selected" : ""}`);
    row.style.padding = "0 8px";
    row.textContent = item;
    wrap.append(row);
  });
  return wrap;
}

function treeList(rows) {
  const wrap = el("div", "tree");
  rows.forEach(([name, depth, selected]) => {
    const row = el("div", `tree-row depth-${depth} ${selected ? "selected" : ""}`);
    row.innerHTML = `<span>${depth < 2 ? "⌄" : ""}</span><span class="name">${name}</span><span class="muted">◉</span><span class="muted">▣</span>`;
    wrap.append(row);
  });
  return wrap;
}

function toolToolbar(items) {
  const toolbar = el("div", "asset-toolbar");
  items.forEach((item, index) => {
    toolbar.append(index === 0 ? selectBtn(item) : index === items.length - 1 ? button(item, "primary-btn") : button(item, "secondary-btn"));
  });
  return toolbar;
}

function toolCard(title, content) {
  const card = el("div", "tool-card");
  const head = el("div", "tool-card-head");
  head.textContent = title;
  const body = el("div", "tool-card-body");
  body.append(content);
  card.append(head, body);
  return card;
}

function renderMiniDataGrid(rows = [["Component", "Variant", "State", "Token", ""], ["Button", "Primary", "Hover", "accent", ""], ["TextField", "Outlined", "Focus", "outline", ""], ["DataGrid", "Dense", "Selected", "selected", ""]]) {
  const table = el("div", "table");
  rows.forEach((cols, index) => {
    const row = el("div", `table-row ${index === 0 ? "head" : index === 3 ? "selected" : ""}`);
    row.innerHTML = cols.map((col) => `<span>${col}</span>`).join("");
    table.append(row);
  });
  return table;
}

function renderMiniChart() {
  const chart = el("div", "mini-chart");
  [38, 64, 46, 78, 54, 88, 62].forEach((height) => {
    const bar = el("span");
    bar.style.height = `${height}%`;
    chart.append(bar);
  });
  return chart;
}

function codePanel() {
  const code = el("div", "code-panel");
  code.innerHTML = `<pre>[asset]
kind = "view"
id = "editor.host.workbench_shell"

[nodes.document_host]
component = "Slot"
control_id = "DocumentHost"
layout = { clip = true }</pre>`;
  return code;
}

function previewPanel() {
  const preview = el("div", "preview-panel");
  preview.innerHTML = `<div class="mini-frame"></div><div class="muted">Live retained host preview</div>`;
  return preview;
}

function diagnosticsPanel(title, rows) {
  const panel = el("div", "asset-detail");
  panel.innerHTML = `<div class="section-title">${title}</div>`;
  rows.forEach((row) => panel.append(fieldRow("Check", row)));
  return panel;
}

function renderGraphCanvas(nodes = [["BlendSpace", 40, 40], ["Idle", 220, 40], ["Run", 400, 40], ["AimOffset", 40, 180], ["Output", 220, 180]]) {
  const canvas = el("div", "graph-canvas");
  nodes.forEach((nodeData, index) => {
    const [label, x, y] = nodeData;
    const node = el("div", `graph-node ${index === 0 ? "selected" : ""}`);
    node.style.left = `${x}${x <= 100 ? "%" : "px"}`;
    node.style.top = `${y}${y <= 100 ? "%" : "px"}`;
    node.textContent = label;
    canvas.append(node);
  });
  return canvas;
}

function renderDataEditorGrid(rows) {
  const wrap = el("div", "data-editor-grid");
  const toolbar = el("div", "data-editor-toolbar");
  toolbar.append(field("Filter rows..."), selectBtn("View: All"), button("Add Row", "secondary-btn"));
  wrap.append(toolbar, renderMiniDataGrid(rows));
  return wrap;
}

function renderMetricsGraph(nodes, metrics) {
  const wrap = el("div", "metrics-graph");
  wrap.append(renderGraphCanvas(nodes));
  const metricsRow = el("div", "metric-grid compact");
  metrics.forEach(([value, label]) => {
    const card = el("div", "metric");
    card.innerHTML = `<div class="metric-value">${value}</div><div class="metric-label">${label}</div>`;
    metricsRow.append(card);
  });
  wrap.append(metricsRow);
  return wrap;
}

function renderCurvePanel() {
  const panel = el("div", "curve-panel");
  panel.append(renderMiniChart(), fieldRow("Selected key", "Frame 42"), fieldRow("Interpolation", "Bezier"), fieldRow("Value", "0.82"));
  return panel;
}

function renderFrameChart() {
  const chart = el("div", "frame-chart");
  [42, 48, 56, 65, 51, 38, 44, 80, 52, 46, 40, 62, 55, 49].forEach((height, index) => {
    const bar = el("span", index === 7 ? "warn-bar" : "");
    bar.style.height = `${height}%`;
    chart.append(bar);
  });
  return chart;
}

function renderPerfMetrics() {
  const wrap = el("div", "metric-grid");
  [["16.6", "Frame ms"], ["1.8", "Layout ms"], ["7", "Draw passes"], ["318", "UI nodes"]].forEach(([value, label]) => {
    const card = el("div", "metric");
    card.innerHTML = `<div class="metric-value">${value}</div><div class="metric-label">${label}</div>`;
    wrap.append(card);
  });
  return wrap;
}

function renderPassBreakdown() {
  const table = el("div", "table");
  [["Pass", "Time", "Batches", "State", ""], ["Scene", "8.4ms", "42", "Ready", ""], ["UI", "2.2ms", "18", "Ready", ""], ["Post", "1.6ms", "6", "Ready", ""]].forEach((cols, index) => {
    const row = el("div", `table-row ${index === 0 ? "head" : ""}`);
    row.innerHTML = cols.map((col) => `<span>${col}</span>`).join("");
    table.append(row);
  });
  return table;
}

function renderDiagnosticTimeline() {
  const wrap = el("div", "diagnostic-timeline");
  logRows.forEach(([time, level, message]) => {
    const row = el("div", `log-row ${level.toLowerCase()}`);
    row.innerHTML = `<span>${time}</span><span>${level}</span><span>${message}</span>`;
    wrap.append(row);
  });
  return wrap;
}

function renderDiagnosticTable() {
  const table = el("div", "table");
  [["Channel", "Events", "Warnings", "Last", ""], ["Render", "128", "0", "now", ""], ["Assets", "42", "1", "2s", ""], ["Tasks", "18", "0", "4s", ""], ["UI", "318", "0", "now", ""]].forEach((cols, index) => {
    const row = el("div", `table-row ${index === 0 ? "head" : index === 2 ? "selected" : ""}`);
    row.innerHTML = cols.map((col) => `<span>${col}</span>`).join("");
    table.append(row);
  });
  return table;
}

function renderBuildPipeline() {
  const pipe = el("div", "pipeline");
  ["Validate", "Compile", "Cook Assets", "Package", "Sign"].forEach((step, index) => {
    const node = el("div", `pipeline-step ${index < 3 ? "done" : index === 3 ? "active" : ""}`);
    node.innerHTML = `<strong>${step}</strong><span>${index < 3 ? "Complete" : index === 3 ? "Running" : "Queued"}</span>`;
    pipe.append(node);
  });
  return pipe;
}

function renderBuildSettings() {
  const wrap = el("div", "control-stack");
  wrap.append(fieldRow("Platform", "Windows Desktop"), fieldRow("Profile", "Editor Debug"), fieldRow("Renderer", "Default"), fieldRow("Assets", "All referenced"), button("Start Export", "primary-btn"));
  return wrap;
}

function renderTimelineBody() {
  const body = el("div", "timeline-body");
  ["Root Motion", "Hips", "Spine", "Arm.L", "Arm.R", "Events"].forEach((track, index) => {
    const row = el("div", "timeline-row");
    row.innerHTML = `<span>${track}</span><div class="timeline-track">${Array.from({ length: 5 }, (_, key) => `<i style="left:${14 + key * 17 + index * 2}%"></i>`).join("")}</div>`;
    body.append(row);
  });
  return body;
}

function tabbedLog(tabs, rows) {
  const root = el("div");
  const tabRow = el("div", "bottom-tabs");
  tabRow.innerHTML = tabs.map((tab, index) => `<span class="panel-tab ${index === 0 ? "active" : ""}">${tab}</span>`).join("");
  const list = el("div", "log-list");
  rows.forEach((message, index) => {
    const row = el("div", `log-row ${index === 0 ? "info" : index === 1 ? "warn" : "info"}`);
    row.innerHTML = `<span>12:0${index}:00</span><span>${index === 1 ? "Warn" : "Info"}</span><span>${message}</span>`;
    list.append(row);
  });
  root.append(tabRow, list);
  return root;
}

function renderStatusbar(design) {
  const bar = el("div", "statusbar");
  bar.innerHTML = `<div class="status-group"><span><span class="dot"></span>${design.status}</span><span><span class="dot warn"></span>2 Warnings</span><span><span class="dot info"></span>0 Messages</span></div><div class="status-group"><span>Grid: 10 cm</span><span>Snap: On</span><span>100%</span></div>`;
  return bar;
}

function sectionHead(title, checked) {
  const head = el("div", "section-head");
  head.innerHTML = `<strong>${title}</strong><span class="${checked ? "box checked" : "box"}"></span>`;
  return head;
}

function formField(label, value) {
  const wrap = el("div");
  wrap.innerHTML = `<div class="label">${label}</div>`;
  wrap.append(field(value));
  return wrap;
}

function fieldRow(label, value, swatch = false) {
  const row = el("div", "field-row");
  row.innerHTML = `<span class="muted">${label}</span><span class="field">${swatch ? '<span class="material-swatch"></span>' : ""}${value}</span>`;
  return row;
}

function axisValues(label, values) {
  const row = el("div", "field-row");
  row.innerHTML = `<span class="muted">${label}</span><span class="axis-values"><span>X</span><span class="value">${values[0]}</span><span>Y</span><span class="value">${values[1]}</span><span>Z</span><span class="value">${values[2]}</span></span>`;
  return row;
}

function slider(label, value) {
  const node = el("div", "slider");
  node.innerHTML = `<span>${label}</span><span class="track"><span class="thumb"></span></span><span class="value">${value}</span>`;
  return node;
}

function renderContextMenu(items, left, top) {
  const menu = el("div", "context-menu");
  menu.style.left = `${left}px`;
  menu.style.top = `${top}px`;
  items.forEach((item, index) => {
    const row = el("div", `menu-row ${index === 1 ? "active" : ""}`);
    row.innerHTML = `<span>${item}</span><span>${index < 3 ? "Ctrl" : ""}</span>`;
    menu.append(row);
  });
  return menu;
}

function iconButton(label, extra = "") {
  const btn = el("div", `icon-btn ${extra}`);
  btn.textContent = label;
  return btn;
}

function squareButton(label) {
  const btn = el("div", "square-btn");
  btn.textContent = label;
  return btn;
}

function miniButton(label, active = false) {
  const btn = el("div", `mini-btn ${active ? "active" : ""}`);
  btn.textContent = label;
  return btn;
}

function selectBtn(label) {
  const btn = el("div", "select-btn");
  btn.innerHTML = `<span>${label}</span><span class="muted">⌄</span>`;
  return btn;
}

function button(label, className) {
  const btn = el("button", className);
  btn.textContent = label;
  return btn;
}

function field(label) {
  const node = el("div", "field");
  node.textContent = label;
  return node;
}

function pill(label) {
  const node = el("div", "pill");
  node.textContent = label;
  return node;
}

function spacer(height) {
  const node = el("div");
  node.style.height = `${height}px`;
  return node;
}

function el(tag, className = "") {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}

if (typeof document !== "undefined") {
  main();
}

export { ALL_DESIGNS, FULL_DESIGNS, FOCUS_DESIGNS };

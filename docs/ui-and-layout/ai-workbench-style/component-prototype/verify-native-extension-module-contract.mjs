import { readFileSync } from "node:fs";
import { extensionModules } from "./src/modules/modules.js";

const expectedExtensionEventsPerWorkspace = 20;
const expectedSharedCompositeEventCount = 8;
const previewActionIdPattern = /^[a-z0-9_]+(?:\.[a-z0-9_]+)+$/;
const allowedEventKinds = new Set(["Click", "Change", "Submit"]);
const extensionEventInteractiveComponents = new Set([
  "WorkbenchButton",
  "WorkbenchIconButton",
  "WorkbenchDropdown",
  "WorkbenchField",
  "WorkbenchListRow",
  "WorkbenchPropertyRow",
  "WorkbenchTab",
  "WorkbenchTableRow",
  "WorkbenchTimelineStrip",
  "WorkbenchSearchInput",
]);
const extensionEventKindComponents = new Map([
  ["Click", new Set(["WorkbenchButton", "WorkbenchIconButton", "WorkbenchListRow", "WorkbenchPropertyRow", "WorkbenchTab", "WorkbenchTableRow", "WorkbenchTimelineStrip"])],
  ["Change", new Set(["WorkbenchDropdown", "WorkbenchField", "WorkbenchSearchInput"])],
  ["Submit", new Set(["WorkbenchDropdown", "WorkbenchField", "WorkbenchSearchInput"])],
]);

const requiredWorkspaceComponents = [
  "WorkbenchDropdown",
  "WorkbenchField",
  "WorkbenchListRow",
  "WorkbenchTab",
  "WorkbenchTableRow",
];

function routeSegment(value) {
  return String(value ?? "")
    .trim()
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_|_$/g, "");
}

function workbenchExtensionRoutePrefix(nativeId) {
  return `workbench.extension.${routeSegment(nativeId)}.`;
}

const extensionWorkspaceContracts = [
  {
    moduleId: "terrain-editor",
    nativeId: "TerrainEditor",
    label: "Terrain Editor",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui",
    workspaceRoot: "terrain_editor_workspace",
    workspaceControlId: "WorkbenchExtensionTerrainEditorWorkspace",
    openerControlId: "WorkbenchAssetsTerrainEditorButton",
    openerBindingId: "WorkbenchExtension/TerrainEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionTerrainEditorOutputRow",
    childPrefix: "terrain_editor",
  },
  {
    moduleId: "foliage-editor",
    nativeId: "FoliageEditor",
    label: "Foliage Editor",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui",
    workspaceRoot: "foliage_editor_workspace",
    workspaceControlId: "WorkbenchExtensionFoliageEditorWorkspace",
    openerControlId: "WorkbenchAssetsFoliageEditorButton",
    openerBindingId: "WorkbenchExtension/FoliageEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionFoliageEditorOutputRow",
    childPrefix: "foliage_editor",
  },
  {
    moduleId: "level-streaming",
    nativeId: "LevelStreaming",
    label: "Level Streaming",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui",
    workspaceRoot: "level_streaming_workspace",
    workspaceControlId: "WorkbenchExtensionLevelStreamingWorkspace",
    openerControlId: "WorkbenchAssetsLevelStreamingButton",
    openerBindingId: "WorkbenchExtension/LevelStreamingOpen",
    feedbackOutputControlId: "WorkbenchExtensionLevelStreamingOutputRow",
    childPrefix: "level_streaming",
  },
  {
    moduleId: "level-variant",
    nativeId: "LevelVariant",
    label: "Level Variant",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui",
    workspaceRoot: "level_variant_workspace",
    workspaceControlId: "WorkbenchExtensionLevelVariantWorkspace",
    openerControlId: "WorkbenchAssetsLevelVariantButton",
    openerBindingId: "WorkbenchExtension/LevelVariantOpen",
    feedbackOutputControlId: "WorkbenchExtensionLevelVariantOutputRow",
    childPrefix: "level_variant",
  },
  {
    moduleId: "prefab-editor",
    nativeId: "PrefabEditor",
    label: "Prefab Editor",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui",
    workspaceRoot: "prefab_editor_workspace",
    workspaceControlId: "WorkbenchExtensionPrefabEditorWorkspace",
    openerControlId: "WorkbenchAssetsPrefabEditorButton",
    openerBindingId: "WorkbenchExtension/PrefabEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionPrefabEditorOutputRow",
    childPrefix: "prefab_editor",
  },
  {
    moduleId: "scatter-editor",
    nativeId: "ScatterEditor",
    label: "Scatter Editor",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui",
    workspaceRoot: "scatter_editor_workspace",
    workspaceControlId: "WorkbenchExtensionScatterEditorWorkspace",
    openerControlId: "WorkbenchAssetsScatterEditorButton",
    openerBindingId: "WorkbenchExtension/ScatterEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionScatterEditorOutputRow",
    childPrefix: "scatter_editor",
  },
  {
    moduleId: "volume-editor",
    nativeId: "VolumeEditor",
    label: "Volume Editor",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui",
    workspaceRoot: "volume_editor_workspace",
    workspaceControlId: "WorkbenchExtensionVolumeEditorWorkspace",
    openerControlId: "WorkbenchAssetsVolumeEditorButton",
    openerBindingId: "WorkbenchExtension/VolumeEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionVolumeEditorOutputRow",
    childPrefix: "volume_editor",
  },
  {
    moduleId: "weather-editor",
    nativeId: "WeatherEditor",
    label: "Weather Editor",
    workspaceFile: "workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui",
    workspaceRoot: "weather_editor_workspace",
    workspaceControlId: "WorkbenchExtensionWeatherEditorWorkspace",
    openerControlId: "WorkbenchAssetsWeatherEditorButton",
    openerBindingId: "WorkbenchExtension/WeatherEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionWeatherEditorOutputRow",
    childPrefix: "weather_editor",
  },
  {
    moduleId: "spawn-rules",
    nativeId: "SpawnRules",
    label: "Spawn Rules",
    workspaceFile: "workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui",
    workspaceRoot: "spawn_rules_workspace",
    workspaceControlId: "WorkbenchExtensionSpawnRulesWorkspace",
    openerControlId: "WorkbenchAssetsSpawnRulesButton",
    openerBindingId: "WorkbenchExtension/SpawnRulesOpen",
    feedbackOutputControlId: "WorkbenchExtensionSpawnRulesOutputRow",
    childPrefix: "spawn_rules",
  },
  {
    moduleId: "world-state",
    nativeId: "WorldState",
    label: "World State",
    workspaceFile: "workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui",
    workspaceRoot: "world_state_workspace",
    workspaceControlId: "WorkbenchExtensionWorldStateWorkspace",
    openerControlId: "WorkbenchAssetsWorldStateButton",
    openerBindingId: "WorkbenchExtension/WorldStateOpen",
    feedbackOutputControlId: "WorkbenchExtensionWorldStateOutputRow",
    childPrefix: "world_state",
  },
  {
    moduleId: "collision-proxy",
    nativeId: "CollisionProxy",
    label: "Collision Proxy",
    workspaceFile: "workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui",
    workspaceRoot: "collision_proxy_workspace",
    workspaceControlId: "WorkbenchExtensionCollisionProxyWorkspace",
    openerControlId: "WorkbenchAssetsCollisionProxyButton",
    openerBindingId: "WorkbenchExtension/CollisionProxyOpen",
    feedbackOutputControlId: "WorkbenchExtensionCollisionProxyOutputRow",
    childPrefix: "collision_proxy",
  },
  {
    moduleId: "physics-collision",
    nativeId: "PhysicsCollision",
    label: "Physics Collision",
    workspaceFile: "workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui",
    workspaceRoot: "physics_collision_workspace",
    workspaceControlId: "WorkbenchExtensionPhysicsCollisionWorkspace",
    openerControlId: "WorkbenchAssetsPhysicsCollisionButton",
    openerBindingId: "WorkbenchExtension/PhysicsCollisionOpen",
    feedbackOutputControlId: "WorkbenchExtensionPhysicsCollisionOutputRow",
    childPrefix: "physics_collision",
  },
  {
    moduleId: "navmesh-ai",
    nativeId: "NavmeshAi",
    label: "Navmesh AI",
    workspaceFile: "workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui",
    workspaceRoot: "navmesh_ai_workspace",
    workspaceControlId: "WorkbenchExtensionNavmeshAiWorkspace",
    openerControlId: "WorkbenchAssetsNavmeshAiButton",
    openerBindingId: "WorkbenchExtension/NavmeshAiOpen",
    feedbackOutputControlId: "WorkbenchExtensionNavmeshAiOutputRow",
    childPrefix: "navmesh_ai",
  },
  {
    moduleId: "lobby-editor",
    nativeId: "LobbyEditor",
    label: "Lobby Editor",
    workspaceFile: "workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui",
    workspaceRoot: "lobby_editor_workspace",
    workspaceControlId: "WorkbenchExtensionLobbyEditorWorkspace",
    openerControlId: "WorkbenchAssetsLobbyEditorButton",
    openerBindingId: "WorkbenchExtension/LobbyEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionLobbyEditorOutputRow",
    childPrefix: "lobby_editor",
  },
  {
    moduleId: "matchmaking-editor",
    nativeId: "MatchmakingEditor",
    label: "Matchmaking Editor",
    workspaceFile: "workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui",
    workspaceRoot: "matchmaking_editor_workspace",
    workspaceControlId: "WorkbenchExtensionMatchmakingEditorWorkspace",
    openerControlId: "WorkbenchAssetsMatchmakingEditorButton",
    openerBindingId: "WorkbenchExtension/MatchmakingEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionMatchmakingEditorOutputRow",
    childPrefix: "matchmaking_editor",
  },
  {
    moduleId: "shader-editor",
    nativeId: "ShaderEditor",
    label: "Shader Editor",
    workspaceFile: "workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui",
    workspaceRoot: "shader_editor_workspace",
    workspaceControlId: "WorkbenchExtensionShaderEditorWorkspace",
    openerControlId: "WorkbenchRenderShaderEditorButton",
    openerBindingId: "WorkbenchExtension/ShaderEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionShaderOutputRow",
    childPrefix: "shader",
  },
  {
    moduleId: "lighting-bake",
    nativeId: "LightingBake",
    label: "Lighting Bake",
    workspaceFile: "workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui",
    workspaceRoot: "lighting_bake_workspace",
    workspaceControlId: "WorkbenchExtensionLightingBakeWorkspace",
    openerControlId: "WorkbenchRenderLightingBakeButton",
    openerBindingId: "WorkbenchExtension/LightingBakeOpen",
    feedbackOutputControlId: "WorkbenchExtensionLightingBakeOutputRow",
    childPrefix: "lighting_bake",
    openerSourceName: "renderWorkspace",
  },
  {
    moduleId: "post-process",
    nativeId: "PostProcess",
    label: "Post Process",
    workspaceFile: "workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui",
    workspaceRoot: "post_process_workspace",
    workspaceControlId: "WorkbenchExtensionPostProcessWorkspace",
    openerControlId: "WorkbenchRenderPostProcessButton",
    openerBindingId: "WorkbenchExtension/PostProcessOpen",
    feedbackOutputControlId: "WorkbenchExtensionPostProcessOutputRow",
    childPrefix: "post_process",
    openerSourceName: "renderWorkspace",
  },
  {
    moduleId: "sequencer",
    nativeId: "Sequencer",
    label: "Sequencer",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui",
    workspaceRoot: "sequencer_workspace",
    workspaceControlId: "WorkbenchExtensionSequencerWorkspace",
    openerControlId: "WorkbenchAbilitySequencerButton",
    openerBindingId: "WorkbenchExtension/SequencerOpen",
    feedbackOutputControlId: "WorkbenchExtensionSequencerOutputRow",
    childPrefix: "sequencer",
    openerSourceName: "abilityWorkspace",
  },
  {
    moduleId: "montage-editor",
    nativeId: "MontageEditor",
    label: "Montage Editor",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui",
    workspaceRoot: "montage_editor_workspace",
    workspaceControlId: "WorkbenchExtensionMontageEditorWorkspace",
    openerControlId: "WorkbenchAbilityMontageEditorButton",
    openerBindingId: "WorkbenchExtension/MontageEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionMontageEditorOutputRow",
    childPrefix: "montage_editor",
    openerSourceName: "abilityWorkspace",
  },
  {
    moduleId: "blend-space",
    nativeId: "BlendSpace",
    label: "Blend Space",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui",
    workspaceRoot: "blend_space_workspace",
    workspaceControlId: "WorkbenchExtensionBlendSpaceWorkspace",
    openerControlId: "WorkbenchAbilityBlendSpaceButton",
    openerBindingId: "WorkbenchExtension/BlendSpaceOpen",
    feedbackOutputControlId: "WorkbenchExtensionBlendSpaceOutputRow",
    childPrefix: "blend_space",
    openerSourceName: "abilityWorkspace",
    compositeSourceNames: ["blendSpaceDetails"],
    componentSourceNames: ["animationTransportControls"],
  },
  {
    moduleId: "pose-library",
    nativeId: "PoseLibrary",
    label: "Pose Library",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui",
    workspaceRoot: "pose_library_workspace",
    workspaceControlId: "WorkbenchExtensionPoseLibraryWorkspace",
    openerControlId: "WorkbenchAbilityPoseLibraryButton",
    openerBindingId: "WorkbenchExtension/PoseLibraryOpen",
    feedbackOutputControlId: "WorkbenchExtensionPoseLibraryOutputRow",
    childPrefix: "pose_library",
    openerSourceName: "abilityWorkspace",
  },
  {
    moduleId: "retarget",
    nativeId: "Retarget",
    label: "Retarget",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui",
    workspaceRoot: "retarget_workspace",
    workspaceControlId: "WorkbenchExtensionRetargetWorkspace",
    openerControlId: "WorkbenchAbilityRetargetButton",
    openerBindingId: "WorkbenchExtension/RetargetOpen",
    feedbackOutputControlId: "WorkbenchExtensionRetargetOutputRow",
    childPrefix: "retarget",
    openerSourceName: "abilityWorkspace",
  },
  {
    moduleId: "control-rig",
    nativeId: "ControlRig",
    label: "Control Rig",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui",
    workspaceRoot: "control_rig_workspace",
    workspaceControlId: "WorkbenchExtensionControlRigWorkspace",
    openerControlId: "WorkbenchAbilityControlRigButton",
    openerBindingId: "WorkbenchExtension/ControlRigOpen",
    feedbackOutputControlId: "WorkbenchExtensionControlRigOutputRow",
    childPrefix: "control_rig",
    openerSourceName: "abilityWorkspace",
  },
  {
    moduleId: "motion-matching",
    nativeId: "MotionMatching",
    label: "Motion Matching",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui",
    workspaceRoot: "motion_matching_workspace",
    workspaceControlId: "WorkbenchExtensionMotionMatchingWorkspace",
    openerControlId: "WorkbenchAbilityMotionMatchingButton",
    openerBindingId: "WorkbenchExtension/MotionMatchingOpen",
    feedbackOutputControlId: "WorkbenchExtensionMotionMatchingOutputRow",
    childPrefix: "motion_matching",
    openerSourceName: "abilityWorkspace",
  },
  {
    moduleId: "animation-compression",
    nativeId: "AnimationCompression",
    label: "Animation Compression",
    workspaceFile: "workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui",
    workspaceRoot: "animation_compression_workspace",
    workspaceControlId: "WorkbenchExtensionAnimationCompressionWorkspace",
    openerControlId: "WorkbenchAbilityAnimationCompressionButton",
    openerBindingId: "WorkbenchExtension/AnimationCompressionOpen",
    feedbackOutputControlId: "WorkbenchExtensionAnimationCompressionOutputRow",
    childPrefix: "animation_compression",
    openerSourceName: "abilityWorkspace",
  },
  {
    moduleId: "data-table",
    nativeId: "DataTable",
    label: "Data Table",
    workspaceFile: "workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui",
    workspaceRoot: "data_table_workspace",
    workspaceControlId: "WorkbenchExtensionDataTableWorkspace",
    openerControlId: "WorkbenchAssetsDataTableButton",
    openerBindingId: "WorkbenchExtension/DataTableOpen",
    feedbackOutputControlId: "WorkbenchExtensionDataTableOutputRow",
    childPrefix: "data_table",
  },
  {
    moduleId: "source-control",
    nativeId: "SourceControl",
    label: "Source Control",
    workspaceFile: "workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui",
    workspaceRoot: "source_control_workspace",
    workspaceControlId: "WorkbenchExtensionSourceControlWorkspace",
    openerControlId: "WorkbenchAssetsSourceControlButton",
    openerBindingId: "WorkbenchExtension/SourceControlOpen",
    feedbackOutputControlId: "WorkbenchExtensionSourceControlOutputRow",
    childPrefix: "source_control",
  },
  {
    moduleId: "build-export",
    nativeId: "BuildExport",
    label: "Build Export",
    workspaceFile: "workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui",
    workspaceRoot: "build_export_workspace",
    workspaceControlId: "WorkbenchExtensionBuildExportWorkspace",
    openerControlId: "WorkbenchAssetsBuildExportButton",
    openerBindingId: "WorkbenchExtension/BuildExportOpen",
    feedbackOutputControlId: "WorkbenchExtensionBuildExportOutputRow",
    childPrefix: "build_export",
  },
  {
    moduleId: "automation-report",
    nativeId: "AutomationReport",
    label: "Automation Report",
    workspaceFile: "workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui",
    workspaceRoot: "automation_report_workspace",
    workspaceControlId: "WorkbenchExtensionAutomationReportWorkspace",
    openerControlId: "WorkbenchAssetsAutomationReportButton",
    openerBindingId: "WorkbenchExtension/AutomationReportOpen",
    feedbackOutputControlId: "WorkbenchExtensionAutomationReportOutputRow",
    childPrefix: "automation_report",
  },
  {
    moduleId: "project-overview",
    nativeId: "ProjectOverview",
    label: "Project Overview",
    workspaceFile: "workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui",
    workspaceRoot: "project_overview_workspace",
    workspaceControlId: "WorkbenchExtensionProjectOverviewWorkspace",
    openerControlId: "WorkbenchAssetsProjectOverviewButton",
    openerBindingId: "WorkbenchExtension/ProjectOverviewOpen",
    feedbackOutputControlId: "WorkbenchExtensionProjectOverviewOutputRow",
    childPrefix: "project_overview",
  },
  {
    moduleId: "plugin-manager",
    nativeId: "PluginManager",
    label: "Plugin Manager",
    workspaceFile: "workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui",
    workspaceRoot: "plugin_manager_workspace",
    workspaceControlId: "WorkbenchExtensionPluginManagerWorkspace",
    openerControlId: "WorkbenchAssetsPluginManagerButton",
    openerBindingId: "WorkbenchExtension/PluginManagerOpen",
    feedbackOutputControlId: "WorkbenchExtensionPluginManagerOutputRow",
    childPrefix: "plugin_manager",
  },
  {
    moduleId: "save-data",
    nativeId: "SaveData",
    label: "Save Data",
    workspaceFile: "workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui",
    workspaceRoot: "save_data_workspace",
    workspaceControlId: "WorkbenchExtensionSaveDataWorkspace",
    openerControlId: "WorkbenchAssetsSaveDataButton",
    openerBindingId: "WorkbenchExtension/SaveDataOpen",
    feedbackOutputControlId: "WorkbenchExtensionSaveDataOutputRow",
    childPrefix: "save_data",
  },
  {
    moduleId: "particle-library",
    nativeId: "ParticleLibrary",
    label: "Particle Library",
    workspaceFile: "workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui",
    workspaceRoot: "particle_library_workspace",
    workspaceControlId: "WorkbenchExtensionParticleLibraryWorkspace",
    openerControlId: "WorkbenchVfxParticleLibraryButton",
    openerBindingId: "WorkbenchExtension/ParticleLibraryOpen",
    feedbackOutputControlId: "WorkbenchExtensionParticleLibraryOutputRow",
    childPrefix: "particle_library",
    openerSourceName: "vfxWorkspace",
  },
  {
    moduleId: "ui-asset-editor",
    nativeId: "UiAssetEditor",
    label: "UI Asset Editor",
    workspaceFile: "workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui",
    workspaceRoot: "ui_asset_editor_workspace",
    workspaceControlId: "WorkbenchExtensionUiAssetEditorWorkspace",
    openerControlId: "WorkbenchHudUiAssetEditorButton",
    openerBindingId: "WorkbenchExtension/UiAssetEditorOpen",
    feedbackOutputControlId: "WorkbenchExtensionUiAssetEditorOutputRow",
    childPrefix: "ui_asset_editor",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "ui-binding",
    nativeId: "UiBinding",
    label: "UI Binding",
    workspaceFile: "workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui",
    workspaceRoot: "ui_binding_workspace",
    workspaceControlId: "WorkbenchExtensionUiBindingWorkspace",
    openerControlId: "WorkbenchHudUiBindingButton",
    openerBindingId: "WorkbenchExtension/UiBindingOpen",
    feedbackOutputControlId: "WorkbenchExtensionUiBindingOutputRow",
    childPrefix: "ui_binding",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "icon-library",
    nativeId: "IconLibrary",
    label: "Icon Library",
    workspaceFile: "workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui",
    workspaceRoot: "icon_library_workspace",
    workspaceControlId: "WorkbenchExtensionIconLibraryWorkspace",
    openerControlId: "WorkbenchHudIconLibraryButton",
    openerBindingId: "WorkbenchExtension/IconLibraryOpen",
    feedbackOutputControlId: "WorkbenchExtensionIconLibraryOutputRow",
    childPrefix: "icon_library",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "accessibility-audit",
    nativeId: "AccessibilityAudit",
    label: "Accessibility Audit",
    workspaceFile: "workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui",
    workspaceRoot: "accessibility_audit_workspace",
    workspaceControlId: "WorkbenchExtensionAccessibilityAuditWorkspace",
    openerControlId: "WorkbenchHudAccessibilityAuditButton",
    openerBindingId: "WorkbenchExtension/AccessibilityAuditOpen",
    feedbackOutputControlId: "WorkbenchExtensionAccessibilityAuditOutputRow",
    childPrefix: "accessibility_audit",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "menu-flow",
    nativeId: "MenuFlow",
    label: "Menu Flow",
    workspaceFile: "workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui",
    workspaceRoot: "menu_flow_workspace",
    workspaceControlId: "WorkbenchExtensionMenuFlowWorkspace",
    openerControlId: "WorkbenchHudMenuFlowButton",
    openerBindingId: "WorkbenchExtension/MenuFlowOpen",
    feedbackOutputControlId: "WorkbenchExtensionMenuFlowOutputRow",
    childPrefix: "menu_flow",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "font-atlas",
    nativeId: "FontAtlas",
    label: "Font Atlas",
    workspaceFile: "workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui",
    workspaceRoot: "font_atlas_workspace",
    workspaceControlId: "WorkbenchExtensionFontAtlasWorkspace",
    openerControlId: "WorkbenchHudFontAtlasButton",
    openerBindingId: "WorkbenchExtension/FontAtlasOpen",
    feedbackOutputControlId: "WorkbenchExtensionFontAtlasOutputRow",
    childPrefix: "font_atlas",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "console-diagnostics",
    nativeId: "ConsoleDiagnostics",
    label: "Console Diagnostics",
    workspaceFile: "workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui",
    workspaceRoot: "console_diagnostics_workspace",
    workspaceControlId: "WorkbenchExtensionConsoleDiagnosticsWorkspace",
    openerControlId: "WorkbenchHudConsoleDiagnosticsButton",
    openerBindingId: "WorkbenchExtension/ConsoleDiagnosticsOpen",
    feedbackOutputControlId: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
    childPrefix: "console_diagnostics",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "runtime-diagnostics",
    nativeId: "RuntimeDiagnostics",
    label: "Runtime Diagnostics",
    workspaceFile: "workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui",
    workspaceRoot: "runtime_diagnostics_workspace",
    workspaceControlId: "WorkbenchExtensionRuntimeDiagnosticsWorkspace",
    openerControlId: "WorkbenchHudRuntimeDiagnosticsButton",
    openerBindingId: "WorkbenchExtension/RuntimeDiagnosticsOpen",
    feedbackOutputControlId: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
    childPrefix: "runtime_diagnostics",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "performance",
    nativeId: "Performance",
    label: "Performance",
    workspaceFile: "workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui",
    workspaceRoot: "performance_workspace",
    workspaceControlId: "WorkbenchExtensionPerformanceWorkspace",
    openerControlId: "WorkbenchHudPerformanceButton",
    openerBindingId: "WorkbenchExtension/PerformanceOpen",
    feedbackOutputControlId: "WorkbenchExtensionPerformanceOutputRow",
    childPrefix: "performance",
    openerSourceName: "hudWorkspace",
  },
  {
    moduleId: "telemetry-dashboard",
    nativeId: "TelemetryDashboard",
    label: "Telemetry Dashboard",
    workspaceFile: "workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui",
    workspaceRoot: "telemetry_dashboard_workspace",
    workspaceControlId: "WorkbenchExtensionTelemetryDashboardWorkspace",
    openerControlId: "WorkbenchHudTelemetryDashboardButton",
    openerBindingId: "WorkbenchExtension/TelemetryDashboardOpen",
    feedbackOutputControlId: "WorkbenchExtensionTelemetryDashboardOutputRow",
    childPrefix: "telemetry_dashboard",
    openerSourceName: "hudWorkspace",
  },
];

const sources = {
  matrix: readLocal("./web-native-handoff-matrix.md"),
  workbenchWindow: readRepo("../../../../zircon_editor/assets/ui/editor/windows/workbench_window.zui"),
  moduleWorkspace: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui"),
  assetsWorkspace: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui"),
  abilityWorkspace: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui"),
  vfxWorkspace: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui"),
  additionalModuleWorkspaces: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui"),
  renderWorkspace: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui"),
  hudWorkspace: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui"),
  extensionWorkspaceHost: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui"),
  blendSpaceDetails: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_blend_space_details.zui"),
  animationTransportControls: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/composites/animation/workbench_transport_controls.zui"),
  extensionWorkspaceFiles: [
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui"),
    },
    {
      sourceName: "workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui",
      source: readRepo("../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui"),
    },
  ],
  coreModuleBindings: readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs"),
  extensionModuleBindings: [
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/types.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/install.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_state.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/simulation_physics.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/online_sessions.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/runtime_state.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics/observability.rs"),
    readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs"),
  ].join("\n"),
  windowTemplateBindings: readRepo("../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs"),
  previewActions: [
    readRepo("../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions/quality_and_observability.rs"),
  ].join("\n"),
  assetEditorMenu: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/asset_editor_menu.rs"),
  abilityEditorMenu: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/ability_editor_menu.rs"),
  renderEditorMenu: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/render_editor_menu.rs"),
  hudEditorMenu: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/hud_editor_menu.rs"),
  componentizedWindow: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs"),
  referenceMenuActions: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs"),
  moduleFieldEdit: readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs"),
  extensionNavigation: [
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/types.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_state.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/render_asset_vfx.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/simulation_physics.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/online_sessions.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/runtime_state.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/data_production.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics/observability.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/level_tools.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/prefab_and_scatter.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/terrain_and_foliage.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/volume_and_weather.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_transport.rs"),
  ].join("\n"),
  extensionFeedback: [
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/data_production.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/simulation_physics.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/online_sessions.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/runtime_state.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics.rs"),
    readRepo("../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics/observability.rs"),
  ].join("\n"),
};

const failures = [];
const extensionWorkspaceSource = sources.extensionWorkspaceFiles
  .map((workspaceFile) => workspaceFile.source)
  .join("\n");
const extensionEvents = [
  ...extensionEventsFromZui("workbench/modules/core/index/workbench_module_workspace.zui", sources.moduleWorkspace),
  ...extensionEventsFromZui("workbench/modules/core/assets/workbench_assets_workspace.zui", sources.assetsWorkspace),
  ...extensionEventsFromZui("workbench/modules/core/gameplay/workbench_ability_workspace.zui", sources.abilityWorkspace),
  ...extensionEventsFromZui("workbench/modules/core/rendering/workbench_vfx_workspace.zui", sources.vfxWorkspace),
  ...extensionEventsFromZui("workbench/modules/core/index/workbench_additional_module_workspaces.zui", sources.additionalModuleWorkspaces),
  ...extensionEventsFromZui("workbench/modules/core/rendering/workbench_render_workspace.zui", sources.renderWorkspace),
  ...extensionEventsFromZui("workbench/modules/core/ui/workbench_hud_workspace.zui", sources.hudWorkspace),
  ...sources.extensionWorkspaceFiles.flatMap((workspaceFile) =>
    extensionEventsFromZui(workspaceFile.sourceName, workspaceFile.source),
  ),
  ...extensionEventsFromZui(
    "workbench/composites/animation/workbench_blend_space_details.zui",
    sources.blendSpaceDetails,
  ),
  ...extensionEventsFromZui(
    "workbench/composites/animation/workbench_transport_controls.zui",
    sources.animationTransportControls,
  ),
];
const extensionBindings = extensionBindingsFromRust(sources.extensionModuleBindings);
const previewActions = previewActionsFromRust(sources.previewActions);
const extensionPreviewActions = new Set([...previewActions].filter(isWorkbenchExtensionPreviewAction));
const extensionBindingIds = new Set(extensionEvents.map((event) => event.bindingId));
const bindingActionIds = new Set([...extensionBindings.values()].map((binding) => binding.actionId));
const assetEditorMenuActionIds = new Set(
  [...sources.assetEditorMenu.matchAll(/extension_action_id:\s*"([^"]+)"/g)].map((match) => match[1]),
);
const abilityEditorMenuActionIds = new Set(
  [...sources.abilityEditorMenu.matchAll(/extension_action_id:\s*"([^"]+)"/g)].map((match) => match[1]),
);
const renderEditorMenuActionIds = new Set(
  [...sources.renderEditorMenu.matchAll(/extension_action_id:\s*"([^"]+)"/g)].map((match) => match[1]),
);
const hudEditorMenuActionIds = new Set(
  [...sources.hudEditorMenu.matchAll(/extension_action_id:\s*"([^"]+)"/g)].map((match) => match[1]),
);
const editorMenuActionIds = new Set([
  ...assetEditorMenuActionIds,
  ...abilityEditorMenuActionIds,
  ...renderEditorMenuActionIds,
  ...hudEditorMenuActionIds,
]);
const extensionWorkspaceNodes = parseZuiNodes(extensionWorkspaceSource);
const seenBindingIds = new Set();
const expectedExtensionEventCount =
  extensionWorkspaceContracts.length * expectedExtensionEventsPerWorkspace
  + expectedSharedCompositeEventCount
  - editorMenuActionIds.size;
const expectedExtensionPreviewActionCount =
  extensionWorkspaceContracts.length * expectedExtensionEventsPerWorkspace
  + expectedSharedCompositeEventCount;

for (const contract of extensionWorkspaceContracts) {
  if (!extensionModules.some((module) => module.id === contract.moduleId && module.extension === true)) {
    failures.push(`${contract.moduleId} should remain a prototype extension module in the browser registry`);
  }

  assertIncludes(sources.matrix, `| ${contract.moduleId} | prototype-only |`, `${contract.moduleId} matrix state`);
  assertIncludes(sources.matrix, "native-extension workspace evidence", `${contract.moduleId} matrix evidence`);
  assertIncludes(
    sources.moduleWorkspace,
    "workbench/modules/extensions/index/workbench_extension_module_workspaces.zui#WorkbenchExtensionModuleWorkspaces",
    "module workspace extension import",
  );
  assertIncludes(sources.moduleWorkspace, "WorkbenchExtensionModuleWorkspacesHost", "module workspace extension host");
  assertIncludes(sources.extensionWorkspaceHost, `${contract.workspaceFile}#${contract.workspaceControlId}`, `${contract.moduleId} split workspace import`);
  assertIncludes(sources.extensionWorkspaceHost, `${contract.workspaceControlId}Host`, `${contract.moduleId} split workspace host`);
  const extensionOpenAction = `${workbenchExtensionRoutePrefix(contract.nativeId)}open`;
  if (assetEditorMenuActionIds.has(extensionOpenAction)) {
    assertIncludes(
      sources.workbenchWindow,
      `menu.item.assets.${routeSegment(contract.nativeId)}`,
      `${contract.moduleId} asset editor menu item`,
    );
    assertIncludes(
      sources.assetEditorMenu,
      `extension_action_id: "${extensionOpenAction}"`,
      `${contract.moduleId} asset editor menu dispatch`,
    );
  } else if (abilityEditorMenuActionIds.has(extensionOpenAction)) {
    assertIncludes(
      sources.workbenchWindow,
      `menu.item.ability.${routeSegment(contract.nativeId)}`,
      `${contract.moduleId} ability editor menu item`,
    );
    assertIncludes(
      sources.abilityEditorMenu,
      `extension_action_id: "${extensionOpenAction}"`,
      `${contract.moduleId} ability editor menu dispatch`,
    );
  } else if (renderEditorMenuActionIds.has(extensionOpenAction)) {
    assertIncludes(
      sources.renderWorkspace,
      "WorkbenchModule/RenderToolsOpen",
      `${contract.moduleId} Render Tools trigger`,
    );
    assertIncludes(
      sources.workbenchWindow,
      "WorkbenchRenderToolsMenu",
      `${contract.moduleId} Render Tools menu`,
    );
    assertIncludes(
      sources.workbenchWindow,
      `menu.item.render.${routeSegment(contract.nativeId)}`,
      `${contract.moduleId} Render Tools menu item`,
    );
    assertIncludes(
      sources.renderEditorMenu,
      `extension_action_id: "${extensionOpenAction}"`,
      `${contract.moduleId} Render Tools menu dispatch`,
    );
  } else if (hudEditorMenuActionIds.has(extensionOpenAction)) {
    assertIncludes(
      sources.hudWorkspace,
      "WorkbenchModule/HudToolsOpen",
      `${contract.moduleId} HUD Tools trigger`,
    );
    assertIncludes(
      sources.workbenchWindow,
      "WorkbenchHudToolsMenu",
      `${contract.moduleId} HUD Tools menu`,
    );
    assertIncludes(
      sources.workbenchWindow,
      `menu.item.hud.${routeSegment(contract.nativeId)}`,
      `${contract.moduleId} HUD Tools menu item`,
    );
    assertIncludes(
      sources.hudEditorMenu,
      `extension_action_id: "${extensionOpenAction}"`,
      `${contract.moduleId} HUD Tools menu dispatch`,
    );
  } else {
    const openerSource =
      sources[contract.openerSourceName ?? (contract.openerBindingId === "WorkbenchExtension/ShaderEditorOpen"
        ? "renderWorkspace"
        : "assetsWorkspace")];
    assertIncludes(openerSource, contract.openerControlId, `${contract.moduleId} opener control`);
    assertIncludes(openerSource, contract.openerBindingId, `${contract.moduleId} opener binding`);
  }

  const workspaceNode = extensionWorkspaceNodes.get(contract.workspaceRoot);
  if (!workspaceNode) {
    failures.push(`${contract.moduleId} workspace root ${contract.workspaceRoot} is missing`);
    continue;
  }
  if (workspaceNode.controlId !== contract.workspaceControlId) {
    failures.push(
      `${contract.moduleId} workspace root should use ${contract.workspaceControlId}, found ${workspaceNode.controlId}`,
    );
  }
  if (workspaceNode.component !== "HorizontalGroup") {
    failures.push(`${contract.moduleId} workspace should use HorizontalGroup, found ${workspaceNode.component}`);
  }
  assertIncludes(workspaceNode.block, '"workbench-extension-module-body"', `${contract.moduleId} workspace classes`);
  assertIncludes(workspaceNode.block, 'visibility = "collapsed"', `${contract.moduleId} workspace default visibility`);
  for (const regionName of [
    `${contract.childPrefix}_rail_gap`,
    `${contract.childPrefix}_left`,
    `${contract.childPrefix}_center`,
    `${contract.childPrefix}_right`,
  ]) {
    if (!workspaceNode.children.includes(regionName)) {
      failures.push(`${contract.moduleId} workspace should include ${regionName}`);
    }
  }

  const workspaceDescendants = collectDescendantNodes(extensionWorkspaceNodes, contract.workspaceRoot);
  for (const sourceName of contract.compositeSourceNames ?? []) {
    workspaceDescendants.push(...parseZuiNodes(sources[sourceName]).values());
  }
  const workspaceComponents = new Set(workspaceDescendants.map((node) => node.component).filter(Boolean));
  for (const sourceName of contract.componentSourceNames ?? []) {
    for (const node of parseZuiNodes(sources[sourceName]).values()) {
      if (node.component) {
        workspaceComponents.add(node.component);
      }
    }
  }
  if (!workspaceComponents.has("WorkbenchButton") && !workspaceComponents.has("WorkbenchIconButton")) {
    failures.push(`${contract.moduleId} workspace is missing a button command control`);
  }
  for (const component of requiredWorkspaceComponents) {
    if (!workspaceComponents.has(component)) {
      failures.push(`${contract.moduleId} workspace is missing ${component}`);
    }
  }
  const workspaceEvents = workspaceDescendants.flatMap((node) => node.events);
  if (workspaceEvents.length < expectedExtensionEventsPerWorkspace - 1) {
    failures.push(
      `${contract.moduleId} workspace should expose at least ${expectedExtensionEventsPerWorkspace - 1} WorkbenchExtension events, found ${workspaceEvents.length}`,
    );
  }
  for (const event of workspaceEvents) {
    if (!event.bindingId.startsWith(`WorkbenchExtension/${contract.nativeId}`)) {
      failures.push(`${contract.moduleId} workspace event ${event.bindingId} is outside ${contract.nativeId}`);
    }
    const expectedRoutePrefix = workbenchExtensionRoutePrefix(contract.nativeId);
    if (!event.route.startsWith(expectedRoutePrefix)) {
      failures.push(`${contract.moduleId} workspace event ${event.bindingId} route ${event.route} is outside ${expectedRoutePrefix}`);
    }
  }

  assertIncludes(sources.extensionNavigation, contract.workspaceControlId, `${contract.moduleId} navigation workspace`);
  assertIncludes(sources.extensionFeedback, contract.feedbackOutputControlId, `${contract.moduleId} feedback output`);
  assertIncludes(sources.componentizedWindow, "apply_workbench_extension_workspace", `${contract.moduleId} workspace route`);
  assertIncludes(sources.referenceMenuActions, "apply_workbench_extension_module_command_feedback", `${contract.moduleId} feedback route`);
}

if (extensionEvents.length !== expectedExtensionEventCount) {
  failures.push(`expected ${expectedExtensionEventCount} WorkbenchExtension events, found ${extensionEvents.length}`);
}
if (extensionBindings.size !== expectedExtensionEventCount) {
  failures.push(`expected ${expectedExtensionEventCount} WorkbenchExtension bindings, found ${extensionBindings.size}`);
}
if (extensionPreviewActions.size !== expectedExtensionPreviewActionCount) {
  failures.push(
    `expected ${expectedExtensionPreviewActionCount} WorkbenchExtension preview actions, found ${extensionPreviewActions.size}`,
  );
}

for (const event of extensionEvents) {
  if (!event.controlId) {
    failures.push(`${event.sourceName}:${event.nodeName} ${event.bindingId} has no control_id`);
  }
  if (!extensionEventInteractiveComponents.has(event.component)) {
    failures.push(
      `${event.sourceName}:${event.nodeName} ${event.bindingId} is attached to non-interactive component ${event.component || "(missing)"}`,
    );
  }
  if (!allowedEventKinds.has(event.eventKind)) {
    failures.push(`${event.sourceName}:${event.nodeName} ${event.bindingId} uses unsupported event ${event.eventKind}`);
  } else if (!extensionEventKindComponents.get(event.eventKind)?.has(event.component)) {
    failures.push(
      `${event.sourceName}:${event.nodeName} ${event.bindingId} uses ${event.eventKind} on ${event.component || "(missing)"}`,
    );
  }
  if (!event.route.startsWith("workbench.extension.")) {
    failures.push(`${event.sourceName}:${event.nodeName} ${event.bindingId} route ${event.route} is outside workbench.extension.*`);
  }
  if (seenBindingIds.has(event.bindingId)) {
    failures.push(`${event.bindingId} is declared more than once`);
  }
  seenBindingIds.add(event.bindingId);

  const binding = extensionBindings.get(event.bindingId);
  if (!binding) {
    failures.push(`${event.bindingId} is declared in ZUI but missing from extension native bindings`);
    continue;
  }
  if (binding.eventKind !== event.eventKind) {
    failures.push(`${event.bindingId} is declared as ${event.eventKind} but native binding is ${binding.eventKind}`);
  }
}

for (const [bindingId, binding] of extensionBindings) {
  if (!extensionBindingIds.has(bindingId)) {
    failures.push(`${bindingId} exists in extension native bindings but has no ZUI declaration`);
  }
  if (!previewActions.has(binding.actionId)) {
    failures.push(`${bindingId} resolves to unregistered preview action ${binding.actionId}`);
  }
  if (!extensionPreviewActions.has(binding.actionId)) {
    failures.push(`${bindingId} resolves outside the Workbench extension preview namespace: ${binding.actionId}`);
  }
  if (!sources.extensionNavigation.includes(binding.actionId)) {
    failures.push(`${binding.actionId} is missing from extension retained navigation`);
  }
}

for (const actionId of extensionPreviewActions) {
  if (
    !bindingActionIds.has(actionId)
    && !assetEditorMenuActionIds.has(actionId)
    && !abilityEditorMenuActionIds.has(actionId)
    && !renderEditorMenuActionIds.has(actionId)
    && !hudEditorMenuActionIds.has(actionId)
  ) {
    failures.push(`${actionId} is registered as a Workbench extension preview action but has no native binding`);
  }
}

assertNotIncludes(
  sources.coreModuleBindings,
  "WorkbenchExtension",
  "core module binding file should not own extension bindings",
);
assertIncludes(
  sources.windowTemplateBindings,
  "workbench_extension_module_template_bindings",
  "window binding installer extension module",
);
assertIncludes(
  sources.moduleFieldEdit,
  'WORKBENCH_EXTENSION_BINDING_PREFIX: &str = "WorkbenchExtension/"',
  "module field edit extension prefix",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.terrain_editor.open",
  "extension feedback terrain editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.terrain_editor.build.invoke",
  "extension feedback terrain editor action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.foliage_editor.open",
  "extension feedback foliage editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.foliage_editor.build.invoke",
  "extension feedback foliage editor action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.level_streaming.open",
  "extension feedback level streaming open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.level_streaming.load.invoke",
  "extension feedback level streaming action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.level_variant.open",
  "extension feedback level variant open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.level_variant.apply.invoke",
  "extension feedback level variant action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.prefab_editor.open",
  "extension feedback prefab editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.prefab_editor.validate.invoke",
  "extension feedback prefab editor validate action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.scatter_editor.open",
  "extension feedback scatter editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.scatter_editor.validate.invoke",
  "extension feedback scatter editor validate action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.volume_editor.open",
  "extension feedback volume editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.volume_editor.validate.invoke",
  "extension feedback volume editor validate action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.weather_editor.open",
  "extension feedback weather editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.weather_editor.build.invoke",
  "extension feedback weather editor build action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.spawn_rules.open",
  "extension feedback spawn rules open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.spawn_rules.validate.invoke",
  "extension feedback spawn rules action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.world_state.open",
  "extension feedback world state open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.world_state.validate.invoke",
  "extension feedback world state action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.shader_editor.compile.invoke",
  "extension feedback compile action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.shader_editor.open",
  "extension feedback open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.lighting_bake.open",
  "extension feedback lighting bake open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.lighting_bake.bake.invoke",
  "extension feedback lighting bake action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.post_process.open",
  "extension feedback post process open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.post_process.apply.invoke",
  "extension feedback post process action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.sequencer.open",
  "extension feedback sequencer open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.sequencer.validate.invoke",
  "extension feedback sequencer action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.montage_editor.open",
  "extension feedback montage editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.montage_editor.apply.invoke",
  "extension feedback montage editor action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.blend_space.open",
  "extension feedback blend space open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.blend_space.apply.invoke",
  "extension feedback blend space action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.pose_library.open",
  "extension feedback pose library open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.pose_library.apply.invoke",
  "extension feedback pose library action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.retarget.open",
  "extension feedback retarget open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.retarget.apply.invoke",
  "extension feedback retarget action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.control_rig.open",
  "extension feedback control rig open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.control_rig.validate.invoke",
  "extension feedback control rig action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.motion_matching.open",
  "extension feedback motion matching open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.motion_matching.rebuild.invoke",
  "extension feedback motion matching action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.animation_compression.open",
  "extension feedback animation compression open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.animation_compression.compress.invoke",
  "extension feedback animation compression action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.automation_report.open",
  "extension feedback automation report open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.automation_report.publish.invoke",
  "extension feedback automation report action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.project_overview.open",
  "extension feedback project overview open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.project_overview.publish.invoke",
  "extension feedback project overview action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.save_data.open",
  "extension feedback save data open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.save_data.save_slot.invoke",
  "extension feedback save data action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.particle_library.open",
  "extension feedback particle library open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.particle_library.compile.invoke",
  "extension feedback particle library compile action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.ui_asset_editor.open",
  "extension feedback ui asset editor open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.ui_asset_editor.validate.invoke",
  "extension feedback ui asset editor validate action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.ui_binding.open",
  "extension feedback ui binding open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.ui_binding.validate.invoke",
  "extension feedback ui binding validate action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.icon_library.open",
  "extension feedback icon library open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.icon_library.validate.invoke",
  "extension feedback icon library validate action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.accessibility_audit.open",
  "extension feedback accessibility audit open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.accessibility_audit.audit_screen.invoke",
  "extension feedback accessibility audit action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.menu_flow.open",
  "extension feedback menu flow open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.menu_flow.validate_focus.invoke",
  "extension feedback menu flow action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.font_atlas.open",
  "extension feedback font atlas open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.font_atlas.bake_atlas.invoke",
  "extension feedback font atlas action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.console_diagnostics.open",
  "extension feedback console diagnostics open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.console_diagnostics.filter_console.invoke",
  "extension feedback console diagnostics action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.runtime_diagnostics.open",
  "extension feedback runtime diagnostics open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.runtime_diagnostics.capture_snapshot.invoke",
  "extension feedback runtime diagnostics action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.performance.open",
  "extension feedback performance open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.performance.capture_frame.invoke",
  "extension feedback performance action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.telemetry_dashboard.open",
  "extension feedback telemetry dashboard open action",
);
assertIncludes(
  sources.extensionFeedback,
  "workbench.extension.telemetry_dashboard.filter_telemetry.invoke",
  "extension feedback telemetry dashboard action",
);

console.log(
  `native extension module contract: modules=${extensionWorkspaceContracts.length} events=${extensionEvents.length} routed=${extensionEvents.filter((event) => event.route).length} unique=${seenBindingIds.size} bindings=${extensionBindings.size} menuMapped=${editorMenuActionIds.size} nativeEvidence=${extensionBindings.size + editorMenuActionIds.size} extensionPreviewActions=${extensionPreviewActions.size}`,
);

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`fail ${failure}`);
  }
  process.exit(1);
}

console.log("ok native Workbench extension module event contract");

function readLocal(path) {
  return readFileSync(new URL(path, import.meta.url), "utf8");
}

function readRepo(path) {
  return readLocal(path);
}

function nodeBlocksFromZui(source) {
  return source.split(/(?=^\[nodes\.[^\]]+\]$)/gm);
}

function zuiExtensionEventsFromBlock(block) {
  const events = [];
  for (const eventMatch of block.matchAll(
    /\{\s*id\s*=\s*"(WorkbenchExtension\/[^"]+)"\s*,\s*event\s*=\s*"([^"]+)"([^}]*)\}/g,
  )) {
    const [, bindingId, eventKind, tail] = eventMatch;
    const route = tail.match(/route\s*=\s*"([^"]+)"/)?.[1] ?? "";
    events.push({ bindingId, eventKind, route });
  }
  return events;
}

function parseZuiNodes(source) {
  const nodes = new Map();
  for (const block of nodeBlocksFromZui(source)) {
    const name = block.match(/^\[nodes\.([^\]]+)\]$/m)?.[1];
    if (!name) {
      continue;
    }
    const component = block.match(/^\s*component\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const controlId = block.match(/^\s*control_id\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const children = [...block.matchAll(/\{\s*node\s*=\s*"([^"]+)"\s*\}/g)].map((match) => match[1]);
    const events = zuiExtensionEventsFromBlock(block);
    nodes.set(name, { name, component, controlId, children, events, block });
  }
  return nodes;
}

function extensionEventsFromZui(sourceName, source) {
  const events = [];
  for (const block of nodeBlocksFromZui(source)) {
    const nodeName = block.match(/^\[nodes\.([^\]]+)\]$/m)?.[1];
    if (!nodeName) {
      continue;
    }
    const controlId = block.match(/^\s*control_id\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const component = block.match(/^\s*component\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    for (const event of zuiExtensionEventsFromBlock(block)) {
      events.push({ sourceName, nodeName, component, controlId, ...event });
    }
  }
  return events;
}

function extensionBindingsFromRust(source) {
  const bindings = new Map();
  for (const match of source.matchAll(/\b(click|change|submit)\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,?\s*\)/g)) {
    const [, helper, controlKey, actionId] = match;
    if (isWorkbenchExtensionPreviewAction(actionId)) {
      bindings.set(`WorkbenchExtension/${controlKey}`, {
        eventKind: helperToEventKind(helper),
        actionId,
      });
    }
  }
  for (const match of source.matchAll(/\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,?\s*\)/g)) {
    const [, controlKey, actionId] = match;
    if (isWorkbenchExtensionPreviewAction(actionId) && !bindings.has(`WorkbenchExtension/${controlKey}`)) {
      bindings.set(`WorkbenchExtension/${controlKey}`, { eventKind: "Click", actionId });
    }
  }
  for (const match of source.matchAll(
    /\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*EditorUiEventKind::(Change|Submit)\s*,?\s*\)/g,
  )) {
    const [, controlKey, actionId, eventKind] = match;
    if (isWorkbenchExtensionPreviewAction(actionId)) {
      bindings.set(`WorkbenchExtension/${controlKey}`, { eventKind, actionId });
    }
  }
  return bindings;
}

function helperToEventKind(helper) {
  switch (helper) {
    case "click":
      return "Click";
    case "change":
      return "Change";
    case "submit":
      return "Submit";
    default:
      throw new Error(`unknown extension binding helper ${helper}`);
  }
}

function previewActionsFromRust(source) {
  const ids = new Set();
  const invalidIds = new Set();
  for (const listMatch of source.matchAll(/PREVIEW_ACTION_IDS:\s*&\[&str\]\s*=\s*&\[([\s\S]*?)\];/g)) {
    for (const idMatch of listMatch[1].matchAll(/"([^"]+)"/g)) {
      const id = idMatch[1];
      if (!previewActionIdPattern.test(id)) {
        invalidIds.add(id);
        continue;
      }
      ids.add(id);
    }
  }
  if (invalidIds.size > 0) {
    throw new Error(
      `preview action ids must use dotted functional paths: ${[...invalidIds].join(", ")}`,
    );
  }
  if (ids.size === 0) {
    throw new Error("preview action arrays were not found");
  }
  return ids;
}

function isWorkbenchExtensionPreviewAction(actionId) {
  return actionId.startsWith("workbench.extension.");
}

function collectDescendantNodes(nodes, rootName, visited = new Set()) {
  if (visited.has(rootName)) {
    return [];
  }
  visited.add(rootName);
  const node = nodes.get(rootName);
  if (!node) {
    return [];
  }
  return [
    node,
    ...node.children.flatMap((childName) => collectDescendantNodes(nodes, childName, visited)),
  ];
}

function assertIncludes(source, needle, message) {
  if (!source.includes(needle)) {
    failures.push(`${message} is missing ${needle}`);
  }
}

function assertNotIncludes(source, needle, message) {
  if (source.includes(needle)) {
    failures.push(`${message} should not include ${needle}`);
  }
}


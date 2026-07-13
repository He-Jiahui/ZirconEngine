import { existsSync, readFileSync } from "node:fs";
import { nativeModules, extensionModules, webModuleTabs } from "./src/modules/modules.js";
import {
  coreReferenceSamples,
  extensionReferenceSamples,
  supplementalReferenceSamples
} from "./src/foundation/reference-samples.js";

const matrixUrl = new URL("./web-native-handoff-matrix.md", import.meta.url);
const moduleComponentsUrl = new URL("./src/modules/shared/module-components.js", import.meta.url);
const componentLabModuleUrl = new URL("./src/modules/component-lab/module.js", import.meta.url);
const componentLabDataUrl = new URL("./src/modules/component-lab/data.js", import.meta.url);
const moduleSharedComponentUrls = [
  moduleComponentsUrl,
  new URL("./src/modules/shared/actions.js", import.meta.url),
  new URL("./src/modules/shared/bottom-output.js", import.meta.url),
  new URL("./src/modules/shared/panels.js", import.meta.url),
  new URL("./src/modules/shared/regions.js", import.meta.url),
  new URL("./src/modules/shared/rows.js", import.meta.url),
  new URL("./src/modules/shared/utils.js", import.meta.url),
  new URL("./src/modules/shared/visuals.js", import.meta.url)
];
const actionPathsUrl = new URL("./src/foundation/action-paths.js", import.meta.url);
const actionNamingContractUrl = new URL("./verify-action-naming-contract.mjs", import.meta.url);
const moduleWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui",
  import.meta.url
);
const effectWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui",
  import.meta.url
);
const materialWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui",
  import.meta.url
);
const behaviorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui",
  import.meta.url
);
const assetsWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui",
  import.meta.url
);
const vfxWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui",
  import.meta.url
);
const additionalModuleWorkspacesUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui",
  import.meta.url
);
const abilityWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui",
  import.meta.url
);
const tagsWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui",
  import.meta.url
);
const perceptionWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_perception_workspace.zui",
  import.meta.url
);
const renderWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui",
  import.meta.url
);
const hudWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui",
  import.meta.url
);
const extensionWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui",
  import.meta.url
);
const generatedBottomPanelUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui",
  import.meta.url
);
const generatedBottomDrawerUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_drawer.zui",
  import.meta.url
);
const generatedBottomBodyUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/host/generated_bottom_body.zui",
  import.meta.url
);
const extensionTerrainEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui",
  import.meta.url
);
const extensionFoliageEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui",
  import.meta.url
);
const extensionLevelStreamingWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui",
  import.meta.url
);
const extensionLevelVariantWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui",
  import.meta.url
);
const extensionPrefabEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui",
  import.meta.url
);
const extensionScatterEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui",
  import.meta.url
);
const extensionVolumeEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui",
  import.meta.url
);
const extensionWeatherEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui",
  import.meta.url
);
const extensionSpawnRulesWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui",
  import.meta.url
);
const extensionWorldStateWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui",
  import.meta.url
);
const extensionCollisionProxyWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui",
  import.meta.url
);
const extensionPhysicsCollisionWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui",
  import.meta.url
);
const extensionNavmeshAiWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui",
  import.meta.url
);
const extensionLobbyEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui",
  import.meta.url
);
const extensionMatchmakingEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui",
  import.meta.url
);
const extensionShaderWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui",
  import.meta.url
);
const extensionLightingBakeWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui",
  import.meta.url
);
const extensionPostProcessWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui",
  import.meta.url
);
const extensionSequencerWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui",
  import.meta.url
);
const extensionMontageEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui",
  import.meta.url
);
const extensionBlendSpaceWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui",
  import.meta.url
);
const extensionPoseLibraryWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui",
  import.meta.url
);
const extensionRetargetWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui",
  import.meta.url
);
const extensionControlRigWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui",
  import.meta.url
);
const extensionMotionMatchingWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui",
  import.meta.url
);
const extensionAnimationCompressionWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui",
  import.meta.url
);
const extensionDataTableWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui",
  import.meta.url
);
const extensionFontAtlasWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui",
  import.meta.url
);
const extensionConsoleDiagnosticsWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui",
  import.meta.url
);
const extensionRuntimeDiagnosticsWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui",
  import.meta.url
);
const extensionPerformanceWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui",
  import.meta.url
);
const extensionTelemetryDashboardWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui",
  import.meta.url
);
const extensionSourceControlWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui",
  import.meta.url
);
const extensionBuildExportWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui",
  import.meta.url
);
const extensionAutomationReportWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui",
  import.meta.url
);
const extensionProjectOverviewWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui",
  import.meta.url
);
const extensionPluginManagerWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui",
  import.meta.url
);
const extensionSaveDataWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui",
  import.meta.url
);
const extensionParticleLibraryWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui",
  import.meta.url
);
const extensionUiAssetEditorWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui",
  import.meta.url
);
const extensionUiBindingWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui",
  import.meta.url
);
const extensionIconLibraryWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui",
  import.meta.url
);
const extensionAccessibilityAuditWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui",
  import.meta.url
);
const extensionMenuFlowWorkspaceUrl = new URL(
  "../../../../zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui",
  import.meta.url
);
const coreModuleBindingsUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs",
  import.meta.url
);
const extensionModuleBindingsUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings.rs",
  import.meta.url
);
const extensionModuleBindingTypesUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/types.rs",
  import.meta.url
);
const extensionModuleBindingInstallUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/install.rs",
  import.meta.url
);
const extensionModuleBindingGameplayAnimationUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs",
  import.meta.url
);
const extensionModuleBindingGameplayStateUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_state.rs",
  import.meta.url
);
const extensionModuleBindingRenderAssetVfxUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs",
  import.meta.url
);
const extensionModuleBindingSimulationPhysicsUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/simulation_physics.rs",
  import.meta.url
);
const extensionModuleBindingOnlineSessionsUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/online_sessions.rs",
  import.meta.url
);
const extensionModuleBindingRuntimeStateUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/runtime_state.rs",
  import.meta.url
);
const extensionModuleBindingUiDiagnosticsUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics.rs",
  import.meta.url
);
const extensionModuleBindingUiDiagnosticsObservabilityUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics/observability.rs",
  import.meta.url
);
const extensionModuleBindingWorldBuildingUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs",
  import.meta.url
);
const generatedBottomBindingsUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs",
  import.meta.url
);
const generatedBottomViewDescriptorUrl = new URL(
  "../../../../zircon_editor/src/ui/host/builtin_views/activity_views/generated_bottom_view_descriptor.rs",
  import.meta.url
);
const shellViewInstancesUrl = new URL(
  "../../../../zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs",
  import.meta.url
);
const layoutDrawersUrl = new URL(
  "../../../../zircon_editor/src/ui/host/builtin_layout/layout_drawers.rs",
  import.meta.url
);
const paneDataConversionUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs",
  import.meta.url
);
const applyPresentationUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs",
  import.meta.url
);
const hostContractPanesUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs",
  import.meta.url
);
const windowBindingsUrl = new URL(
  "../../../../zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs",
  import.meta.url
);
const extensionNavigationUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs",
  import.meta.url
);
const extensionNavigationSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs",
  import.meta.url
);
const extensionNavigationSpecTypesUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/types.rs",
  import.meta.url
);
const extensionNavigationGameplayAnimationSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs",
  import.meta.url
);
const extensionNavigationGameplayStateSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_state.rs",
  import.meta.url
);
const extensionNavigationRenderAssetVfxSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/render_asset_vfx.rs",
  import.meta.url
);
const extensionNavigationSimulationPhysicsSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/simulation_physics.rs",
  import.meta.url
);
const extensionNavigationOnlineSessionsSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/online_sessions.rs",
  import.meta.url
);
const extensionNavigationRuntimeStateSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/runtime_state.rs",
  import.meta.url
);
const extensionNavigationDataProductionSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/data_production.rs",
  import.meta.url
);
const extensionNavigationUiDiagnosticsSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics.rs",
  import.meta.url
);
const extensionNavigationUiDiagnosticsObservabilitySpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics/observability.rs",
  import.meta.url
);
const extensionNavigationWorldBuildingSpecsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building.rs",
  import.meta.url
);
const extensionFeedbackUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs",
  import.meta.url
);
const extensionFeedbackDataProductionUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/data_production.rs",
  import.meta.url
);
const extensionFeedbackGameplayStateUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs",
  import.meta.url
);
const extensionFeedbackSimulationPhysicsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/simulation_physics.rs",
  import.meta.url
);
const extensionFeedbackOnlineSessionsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/online_sessions.rs",
  import.meta.url
);
const extensionFeedbackRuntimeStateUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/runtime_state.rs",
  import.meta.url
);
const extensionFeedbackUiDiagnosticsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics.rs",
  import.meta.url
);
const extensionFeedbackUiDiagnosticsObservabilityUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics/observability.rs",
  import.meta.url
);
const generatedBottomNavigationUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs",
  import.meta.url
);
const generatedBottomFeedbackUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs",
  import.meta.url
);
const generatedBottomLifecycleUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs",
  import.meta.url
);
const previewActionsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions.rs",
  import.meta.url
);
const extensionPreviewActionsUrl = new URL(
  "../../../../zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs",
  import.meta.url
);

if (!existsSync(matrixUrl)) {
  console.error("fail web-native handoff matrix document is missing");
  process.exit(1);
}

const matrix = readFileSync(matrixUrl, "utf8");
const moduleComponentsSource = moduleSharedComponentUrls
  .map((url) => readFileSync(url, "utf8"))
  .join("\n");
const componentLabModuleSource = readFileSync(componentLabModuleUrl, "utf8");
const componentLabDataSource = readFileSync(componentLabDataUrl, "utf8");
const actionPathsSource = readFileSync(actionPathsUrl, "utf8");
const actionNamingContractSource = readFileSync(actionNamingContractUrl, "utf8");
const moduleWorkspaceSource = readFileSync(moduleWorkspaceUrl, "utf8");
const effectWorkspaceSource = readFileSync(effectWorkspaceUrl, "utf8");
const materialWorkspaceSource = readFileSync(materialWorkspaceUrl, "utf8");
const behaviorWorkspaceSource = readFileSync(behaviorWorkspaceUrl, "utf8");
const assetsWorkspaceSource = readFileSync(assetsWorkspaceUrl, "utf8");
const vfxWorkspaceSource = readFileSync(vfxWorkspaceUrl, "utf8");
const additionalModuleWorkspacesSource = readFileSync(additionalModuleWorkspacesUrl, "utf8");
const abilityWorkspaceSource = readFileSync(abilityWorkspaceUrl, "utf8");
const tagsWorkspaceSource = readFileSync(tagsWorkspaceUrl, "utf8");
const perceptionWorkspaceSource = readFileSync(perceptionWorkspaceUrl, "utf8");
const renderWorkspaceSource = readFileSync(renderWorkspaceUrl, "utf8");
const hudWorkspaceSource = readFileSync(hudWorkspaceUrl, "utf8");
const extensionWorkspaceSource = readFileSync(extensionWorkspaceUrl, "utf8");
const generatedBottomPanelSource = readFileSync(generatedBottomPanelUrl, "utf8");
const generatedBottomDrawerSource = readFileSync(generatedBottomDrawerUrl, "utf8");
const generatedBottomBodySource = readFileSync(generatedBottomBodyUrl, "utf8");
const extensionTerrainEditorWorkspaceSource = readFileSync(extensionTerrainEditorWorkspaceUrl, "utf8");
const extensionFoliageEditorWorkspaceSource = readFileSync(extensionFoliageEditorWorkspaceUrl, "utf8");
const extensionLevelStreamingWorkspaceSource = readFileSync(extensionLevelStreamingWorkspaceUrl, "utf8");
const extensionLevelVariantWorkspaceSource = readFileSync(extensionLevelVariantWorkspaceUrl, "utf8");
const extensionPrefabEditorWorkspaceSource = readFileSync(extensionPrefabEditorWorkspaceUrl, "utf8");
const extensionScatterEditorWorkspaceSource = readFileSync(extensionScatterEditorWorkspaceUrl, "utf8");
const extensionVolumeEditorWorkspaceSource = readFileSync(extensionVolumeEditorWorkspaceUrl, "utf8");
const extensionWeatherEditorWorkspaceSource = readFileSync(extensionWeatherEditorWorkspaceUrl, "utf8");
const extensionSpawnRulesWorkspaceSource = readFileSync(extensionSpawnRulesWorkspaceUrl, "utf8");
const extensionWorldStateWorkspaceSource = readFileSync(extensionWorldStateWorkspaceUrl, "utf8");
const extensionCollisionProxyWorkspaceSource = readFileSync(extensionCollisionProxyWorkspaceUrl, "utf8");
const extensionPhysicsCollisionWorkspaceSource = readFileSync(extensionPhysicsCollisionWorkspaceUrl, "utf8");
const extensionNavmeshAiWorkspaceSource = readFileSync(extensionNavmeshAiWorkspaceUrl, "utf8");
const extensionLobbyEditorWorkspaceSource = readFileSync(extensionLobbyEditorWorkspaceUrl, "utf8");
const extensionMatchmakingEditorWorkspaceSource = readFileSync(extensionMatchmakingEditorWorkspaceUrl, "utf8");
const extensionShaderWorkspaceSource = readFileSync(extensionShaderWorkspaceUrl, "utf8");
const extensionLightingBakeWorkspaceSource = readFileSync(extensionLightingBakeWorkspaceUrl, "utf8");
const extensionPostProcessWorkspaceSource = readFileSync(extensionPostProcessWorkspaceUrl, "utf8");
const extensionSequencerWorkspaceSource = readFileSync(extensionSequencerWorkspaceUrl, "utf8");
const extensionMontageEditorWorkspaceSource = readFileSync(extensionMontageEditorWorkspaceUrl, "utf8");
const extensionBlendSpaceWorkspaceSource = readFileSync(extensionBlendSpaceWorkspaceUrl, "utf8");
const extensionPoseLibraryWorkspaceSource = readFileSync(extensionPoseLibraryWorkspaceUrl, "utf8");
const extensionRetargetWorkspaceSource = readFileSync(extensionRetargetWorkspaceUrl, "utf8");
const extensionControlRigWorkspaceSource = readFileSync(extensionControlRigWorkspaceUrl, "utf8");
const extensionMotionMatchingWorkspaceSource = readFileSync(extensionMotionMatchingWorkspaceUrl, "utf8");
const extensionAnimationCompressionWorkspaceSource = readFileSync(extensionAnimationCompressionWorkspaceUrl, "utf8");
const extensionDataTableWorkspaceSource = readFileSync(extensionDataTableWorkspaceUrl, "utf8");
const extensionFontAtlasWorkspaceSource = readFileSync(extensionFontAtlasWorkspaceUrl, "utf8");
const extensionConsoleDiagnosticsWorkspaceSource = readFileSync(extensionConsoleDiagnosticsWorkspaceUrl, "utf8");
const extensionRuntimeDiagnosticsWorkspaceSource = readFileSync(extensionRuntimeDiagnosticsWorkspaceUrl, "utf8");
const extensionPerformanceWorkspaceSource = readFileSync(extensionPerformanceWorkspaceUrl, "utf8");
const extensionTelemetryDashboardWorkspaceSource = readFileSync(extensionTelemetryDashboardWorkspaceUrl, "utf8");
const extensionSourceControlWorkspaceSource = readFileSync(extensionSourceControlWorkspaceUrl, "utf8");
const extensionBuildExportWorkspaceSource = readFileSync(extensionBuildExportWorkspaceUrl, "utf8");
const extensionAutomationReportWorkspaceSource = readFileSync(extensionAutomationReportWorkspaceUrl, "utf8");
const extensionProjectOverviewWorkspaceSource = readFileSync(extensionProjectOverviewWorkspaceUrl, "utf8");
const extensionPluginManagerWorkspaceSource = readFileSync(extensionPluginManagerWorkspaceUrl, "utf8");
const extensionSaveDataWorkspaceSource = readFileSync(extensionSaveDataWorkspaceUrl, "utf8");
const extensionParticleLibraryWorkspaceSource = readFileSync(extensionParticleLibraryWorkspaceUrl, "utf8");
const extensionUiAssetEditorWorkspaceSource = readFileSync(extensionUiAssetEditorWorkspaceUrl, "utf8");
const extensionUiBindingWorkspaceSource = readFileSync(extensionUiBindingWorkspaceUrl, "utf8");
const extensionIconLibraryWorkspaceSource = readFileSync(extensionIconLibraryWorkspaceUrl, "utf8");
const extensionAccessibilityAuditWorkspaceSource = readFileSync(extensionAccessibilityAuditWorkspaceUrl, "utf8");
const extensionMenuFlowWorkspaceSource = readFileSync(extensionMenuFlowWorkspaceUrl, "utf8");
const extensionWorkspaceEvidenceSource = [
  extensionTerrainEditorWorkspaceSource,
  extensionFoliageEditorWorkspaceSource,
  extensionLevelStreamingWorkspaceSource,
  extensionLevelVariantWorkspaceSource,
  extensionPrefabEditorWorkspaceSource,
  extensionScatterEditorWorkspaceSource,
  extensionVolumeEditorWorkspaceSource,
  extensionWeatherEditorWorkspaceSource,
  extensionSpawnRulesWorkspaceSource,
  extensionWorldStateWorkspaceSource,
  extensionCollisionProxyWorkspaceSource,
  extensionPhysicsCollisionWorkspaceSource,
  extensionNavmeshAiWorkspaceSource,
  extensionLobbyEditorWorkspaceSource,
  extensionMatchmakingEditorWorkspaceSource,
  extensionShaderWorkspaceSource,
  extensionLightingBakeWorkspaceSource,
  extensionPostProcessWorkspaceSource,
  extensionSequencerWorkspaceSource,
  extensionMontageEditorWorkspaceSource,
  extensionBlendSpaceWorkspaceSource,
  extensionPoseLibraryWorkspaceSource,
  extensionRetargetWorkspaceSource,
  extensionControlRigWorkspaceSource,
  extensionMotionMatchingWorkspaceSource,
  extensionAnimationCompressionWorkspaceSource,
  extensionDataTableWorkspaceSource,
  extensionFontAtlasWorkspaceSource,
  extensionConsoleDiagnosticsWorkspaceSource,
  extensionRuntimeDiagnosticsWorkspaceSource,
  extensionPerformanceWorkspaceSource,
  extensionTelemetryDashboardWorkspaceSource,
  extensionSourceControlWorkspaceSource,
  extensionBuildExportWorkspaceSource,
  extensionAutomationReportWorkspaceSource,
  extensionProjectOverviewWorkspaceSource,
  extensionPluginManagerWorkspaceSource,
  extensionSaveDataWorkspaceSource,
  extensionParticleLibraryWorkspaceSource,
  extensionUiAssetEditorWorkspaceSource,
  extensionUiBindingWorkspaceSource,
  extensionIconLibraryWorkspaceSource,
  extensionAccessibilityAuditWorkspaceSource,
  extensionMenuFlowWorkspaceSource
].join("\n");
const coreModuleBindingsSource = readFileSync(coreModuleBindingsUrl, "utf8");
const extensionModuleBindingsSource = [
  readFileSync(extensionModuleBindingsUrl, "utf8"),
  readFileSync(extensionModuleBindingTypesUrl, "utf8"),
  readFileSync(extensionModuleBindingInstallUrl, "utf8"),
  readFileSync(extensionModuleBindingGameplayAnimationUrl, "utf8"),
  readFileSync(extensionModuleBindingGameplayStateUrl, "utf8"),
  readFileSync(extensionModuleBindingRenderAssetVfxUrl, "utf8"),
  readFileSync(extensionModuleBindingSimulationPhysicsUrl, "utf8"),
  readFileSync(extensionModuleBindingOnlineSessionsUrl, "utf8"),
  readFileSync(extensionModuleBindingRuntimeStateUrl, "utf8"),
  readFileSync(extensionModuleBindingUiDiagnosticsUrl, "utf8"),
  readFileSync(extensionModuleBindingUiDiagnosticsObservabilityUrl, "utf8"),
  readFileSync(extensionModuleBindingWorldBuildingUrl, "utf8")
].join("\n");
const generatedBottomBindingsSource = readFileSync(generatedBottomBindingsUrl, "utf8");
const generatedBottomViewDescriptorSource = readFileSync(generatedBottomViewDescriptorUrl, "utf8");
const shellViewInstancesSource = readFileSync(shellViewInstancesUrl, "utf8");
const layoutDrawersSource = readFileSync(layoutDrawersUrl, "utf8");
const paneDataConversionSource = readFileSync(paneDataConversionUrl, "utf8");
const applyPresentationSource = readFileSync(applyPresentationUrl, "utf8");
const hostContractPanesSource = readFileSync(hostContractPanesUrl, "utf8");
const windowBindingsSource = readFileSync(windowBindingsUrl, "utf8");
const extensionNavigationSource = [
  readFileSync(extensionNavigationUrl, "utf8"),
  readFileSync(extensionNavigationSpecsUrl, "utf8"),
  readFileSync(extensionNavigationSpecTypesUrl, "utf8"),
  readFileSync(extensionNavigationGameplayAnimationSpecsUrl, "utf8"),
  readFileSync(extensionNavigationGameplayStateSpecsUrl, "utf8"),
  readFileSync(extensionNavigationRenderAssetVfxSpecsUrl, "utf8"),
  readFileSync(extensionNavigationSimulationPhysicsSpecsUrl, "utf8"),
  readFileSync(extensionNavigationOnlineSessionsSpecsUrl, "utf8"),
  readFileSync(extensionNavigationRuntimeStateSpecsUrl, "utf8"),
  readFileSync(extensionNavigationDataProductionSpecsUrl, "utf8"),
  readFileSync(extensionNavigationUiDiagnosticsSpecsUrl, "utf8"),
  readFileSync(extensionNavigationUiDiagnosticsObservabilitySpecsUrl, "utf8"),
  readFileSync(extensionNavigationWorldBuildingSpecsUrl, "utf8")
].join("\n");
const extensionFeedbackSource = [
  readFileSync(extensionFeedbackUrl, "utf8"),
  readFileSync(extensionFeedbackDataProductionUrl, "utf8"),
  readFileSync(extensionFeedbackGameplayStateUrl, "utf8"),
  readFileSync(extensionFeedbackSimulationPhysicsUrl, "utf8"),
  readFileSync(extensionFeedbackOnlineSessionsUrl, "utf8"),
  readFileSync(extensionFeedbackRuntimeStateUrl, "utf8"),
  readFileSync(extensionFeedbackUiDiagnosticsUrl, "utf8"),
  readFileSync(extensionFeedbackUiDiagnosticsObservabilityUrl, "utf8")
].join("\n");
const generatedBottomNavigationSource = readFileSync(generatedBottomNavigationUrl, "utf8");
const generatedBottomFeedbackSource = readFileSync(generatedBottomFeedbackUrl, "utf8");
const generatedBottomLifecycleSource = readFileSync(generatedBottomLifecycleUrl, "utf8");
const previewActionsSource = [
  readFileSync(previewActionsUrl, "utf8"),
  readFileSync(extensionPreviewActionsUrl, "utf8")
].join("\n");

const componentFamilies = [
  "layout",
  "button",
  "icon-button",
  "text-field",
  "selection-control",
  "tabs-segmented",
  "dropdown",
  "slider",
  "list-view",
  "tree-view",
  "table-view",
  "popup-menu",
  "feedback",
  "drawer-window-panel",
  "generated-bottom-panel"
];

const primitiveZuiAssets = [
  "workbench/primitives/inputs/workbench_button.zui",
  "workbench/primitives/inputs/workbench_icon_button.zui",
  "workbench/primitives/chrome/workbench_rail_button.zui",
  "workbench/primitives/inputs/workbench_field.zui",
  "workbench/primitives/inputs/workbench_checkbox.zui",
  "workbench/primitives/inputs/workbench_radio.zui",
  "workbench/primitives/inputs/workbench_toggle.zui",
  "workbench/primitives/inputs/workbench_tab.zui",
  "workbench/primitives/inputs/workbench_segmented_control.zui",
  "workbench/primitives/inputs/workbench_dropdown.zui",
  "workbench/primitives/inputs/workbench_slider.zui",
  "workbench/primitives/data/workbench_list_row.zui",
  "workbench/primitives/data/workbench_tree_row.zui",
  "workbench/primitives/data/workbench_table_row.zui",
  "workbench/primitives/feedback/workbench_popup_menu.zui",
  "workbench/primitives/feedback/workbench_tooltip.zui",
  "workbench/primitives/feedback/workbench_toast.zui",
  "workbench/primitives/data/workbench_property_row.zui",
  "workbench/primitives/data/workbench_component_property_row.zui",
  "workbench/primitives/chrome/workbench_chip.zui",
  "workbench/primitives/chrome/workbench_axis_value_field.zui",
  "workbench/primitives/chrome/workbench_section_title.zui",
  "workbench/primitives/feedback/workbench_status_item.zui"
];

const shellSurfaceZuiAssets = [
  "workbench/shell/workbench_activity_rail.zui",
  "workbench/shell/workbench_top_toolbar.zui",
  "workbench/shell/workbench_scene_tree_panel.zui",
  "workbench/shell/workbench_viewport_panel.zui",
  "workbench/shell/workbench_inspector_panel.zui",
  "workbench/shell/workbench_status_bar.zui",
  "workbench/shell/workbench_main_band.zui"
];

const requiredCommands = [
  "node verify-interaction-contract.mjs",
  "node verify-action-naming-contract.mjs",
  "node validate-control-routes.mjs",
  "node validate-responsive.mjs",
  "node verify-native-component-contract.mjs",
  "node verify-native-module-contract.mjs",
  "node verify-native-extension-module-contract.mjs",
  "node verify-native-generated-bottom-contract.mjs",
  "node verify-web-native-handoff.mjs"
];

const failures = [];

check(matrix.startsWith("---\nrelated_code:"), "matrix uses required docs header");
check(matrix.includes("doc_type: workflow-detail"), "matrix is classified as a workflow detail");
check(matrix.includes("docs/ui-and-layout/ai-workbench-style/component-prototype/modules.js"), "matrix header mentions modules.js");
check(matrix.includes("docs/ui-and-layout/ai-workbench-style/component-prototype/src/modules/component-lab/module.js"), "matrix header mentions Component Lab module");
check(matrix.includes("docs/ui-and-layout/ai-workbench-style/component-prototype/src/modules/extensions/extension-modules.js"), "matrix header mentions extension-modules.js");
check(matrix.includes("docs/ui-and-layout/ai-workbench-style/component-prototype/extension-surfaces.js"), "matrix header mentions extension-surfaces.js");
check(matrix.includes("docs/ui-and-layout/ai-workbench-style/component-prototype/src/modules/extensions/extension-handoff.js"), "matrix header mentions extension-handoff.js");
check(matrix.includes("docs/ui-and-layout/ai-workbench-style/component-prototype/action-paths.js"), "matrix header mentions action-paths.js");
check(matrix.includes("docs/ui-and-layout/ai-workbench-style/component-prototype/verify-action-naming-contract.mjs"), "matrix header mentions action naming contract verifier");
check(actionPathsSource.includes("export function actionPath") && actionPathsSource.includes("export function actionRouteKey"), "action path helper exports naming and route-key helpers");
check(actionNamingContractSource.includes("dottedActionPattern") && actionNamingContractSource.includes("collectNativeActionIds"), "action naming contract scans dotted web/native action ids");
check(
  matrix.includes("workbench.collection.menu.*") &&
    matrix.includes("workbench.module.table.*") &&
    matrix.includes("workbench.generated_bottom.*") &&
    matrix.includes("workbench.component_lab.*"),
  "matrix records dotted browser action namespaces"
);
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui"), "matrix header mentions native module workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui"), "matrix header mentions split native Effect workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui"), "matrix header mentions split native Material workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui"), "matrix header mentions split native Behavior workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui"), "matrix header mentions split native Assets workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui"), "matrix header mentions split native VFX workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui"), "matrix header mentions split native Ability workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui"), "matrix header mentions split native Tags workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_perception_workspace.zui"), "matrix header mentions split native Perception workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_render_workspace.zui"), "matrix header mentions split native Render workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui"), "matrix header mentions split native HUD workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui"), "matrix header mentions native extension module workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui"), "matrix header mentions terrain-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui"), "matrix header mentions foliage-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui"), "matrix header mentions level-streaming extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui"), "matrix header mentions level-variant extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui"), "matrix header mentions prefab-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui"), "matrix header mentions scatter-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui"), "matrix header mentions volume-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui"), "matrix header mentions weather-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui"), "matrix header mentions spawn-rules extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui"), "matrix header mentions world-state extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui"), "matrix header mentions collision-proxy extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui"), "matrix header mentions physics-collision extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui"), "matrix header mentions navmesh-ai extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui"), "matrix header mentions lobby-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui"), "matrix header mentions matchmaking-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui"), "matrix header mentions shader extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui"), "matrix header mentions lighting-bake extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui"), "matrix header mentions post-process extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui"), "matrix header mentions sequencer extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui"), "matrix header mentions montage-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui"), "matrix header mentions blend-space extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui"), "matrix header mentions pose-library extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui"), "matrix header mentions retarget extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui"), "matrix header mentions control-rig extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui"), "matrix header mentions motion-matching extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui"), "matrix header mentions animation-compression extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui"), "matrix header mentions data-table extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui"), "matrix header mentions font-atlas extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui"), "matrix header mentions console-diagnostics extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui"), "matrix header mentions runtime-diagnostics extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui"), "matrix header mentions performance extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui"), "matrix header mentions telemetry-dashboard extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui"), "matrix header mentions source-control extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui"), "matrix header mentions build-export extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui"), "matrix header mentions automation-report extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui"), "matrix header mentions project-overview extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui"), "matrix header mentions plugin-manager extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui"), "matrix header mentions save-data extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui"), "matrix header mentions particle-library extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui"), "matrix header mentions ui-asset-editor extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui"), "matrix header mentions ui-binding extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui"), "matrix header mentions icon-library extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui"), "matrix header mentions accessibility-audit extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui"), "matrix header mentions menu-flow extension workspace zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui"), "matrix header mentions generated bottom panel zui");
check(matrix.includes("zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_drawer.zui"), "matrix header mentions generated bottom drawer zui");
check(matrix.includes("zircon_editor/assets/ui/editor/host/generated_bottom_body.zui"), "matrix header mentions generated bottom shell body zui");
check(matrix.includes("zircon_editor/src/ui/host/builtin_views/activity_views/generated_bottom_view_descriptor.rs"), "matrix header mentions generated bottom activity view descriptor");
check(matrix.includes("zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs"), "matrix header mentions generated bottom shell view instance");
check(matrix.includes("zircon_editor/src/ui/host/builtin_layout/layout_drawers.rs"), "matrix header mentions generated bottom drawer layout");
check(matrix.includes("zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs"), "matrix header mentions retained generated bottom pane conversion");
check(matrix.includes("zircon_editor/src/ui/retained_host/ui/apply_presentation.rs"), "matrix header mentions generated bottom presentation application");
check(matrix.includes("zircon_editor/src/ui/retained_host/host_contract/data/panes.rs"), "matrix header mentions generated bottom host contract pane data");
check(matrix.includes("zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs"), "matrix header mentions retained painter template nodes");
check(matrix.includes("zircon_runtime/src/ui/surface/render/extract.rs"), "matrix header mentions runtime render extract");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings.rs"), "matrix header mentions native extension module bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/types.rs"), "matrix header mentions native extension module binding types");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/install.rs"), "matrix header mentions native extension module binding installer");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs"), "matrix header mentions native extension module gameplay/animation bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_state.rs"), "matrix header mentions native extension module gameplay/state bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs"), "matrix header mentions native extension module render/asset/vfx bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/simulation_physics.rs"), "matrix header mentions native extension module simulation/physics bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/online_sessions.rs"), "matrix header mentions native extension module online/session bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/runtime_state.rs"), "matrix header mentions native extension module runtime/state bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics.rs"), "matrix header mentions native extension module ui/diagnostics bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics/observability.rs"), "matrix header mentions native extension module diagnostics observability bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs"), "matrix header mentions native extension module world-building bindings");
check(matrix.includes("zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs"), "matrix header mentions native generated bottom bindings");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs"), "matrix header mentions native extension module navigation");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs"), "matrix header mentions native extension module navigation specs index");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/types.rs"), "matrix header mentions native extension module navigation spec types");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs"), "matrix header mentions native extension module gameplay/animation specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_state.rs"), "matrix header mentions native extension module gameplay/state specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/render_asset_vfx.rs"), "matrix header mentions native extension module render/asset/vfx specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/simulation_physics.rs"), "matrix header mentions native extension module simulation/physics specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/online_sessions.rs"), "matrix header mentions native extension module online/session specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/runtime_state.rs"), "matrix header mentions native extension module runtime/state specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/data_production.rs"), "matrix header mentions native extension module data/production specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics.rs"), "matrix header mentions native extension module ui/diagnostics specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics/observability.rs"), "matrix header mentions native extension module diagnostics observability specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building.rs"), "matrix header mentions native extension module world-building specs");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs"), "matrix header mentions native extension module feedback");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/simulation_physics.rs"), "matrix header mentions native extension module simulation/physics feedback");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/online_sessions.rs"), "matrix header mentions native extension module online/session feedback");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/runtime_state.rs"), "matrix header mentions native extension module runtime/state feedback");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics.rs"), "matrix header mentions native extension module ui/diagnostics feedback");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics/observability.rs"), "matrix header mentions native extension module diagnostics observability feedback");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs"), "matrix header mentions native generated bottom navigation");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs"), "matrix header mentions native generated bottom feedback");
check(matrix.includes("zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_lifecycle.rs"), "matrix header mentions native generated bottom lifecycle");

check(matrix.includes("## Component Family Handoff"), "component family section exists");
for (const family of componentFamilies) {
  check(matrix.includes(`| ${family} |`), `component family ${family} is listed`);
}
for (const asset of primitiveZuiAssets) {
  check(matrix.includes(asset), `matrix records primitive zui asset ${asset}`);
}
for (const asset of shellSurfaceZuiAssets) {
  check(matrix.includes(asset), `matrix records shell surface zui asset ${asset}`);
}
check(matrix.includes("workbench/shell/workbench_component_drawer.zui"), "matrix records component drawer low-level zui composition");
check(matrix.includes("32 declarative `.zui` component assets"), "matrix records native component zui asset count");
check(matrix.includes("7 shell surface `.zui` assets"), "matrix records native shell surface zui asset count");
check(matrix.includes("builtin import graph"), "matrix records native builtin import graph coverage");
check(matrix.includes("primitive and shell surface set"), "matrix records focused workbench primitive and shell surface governance test");
const componentLabTab = webModuleTabs.find((module) => module.id === "component-lab");
check(
  componentLabTab?.webOnly === true &&
    componentLabTab.label === "Component Lab" &&
    !nativeModules.some((module) => module.id === "component-lab") &&
    !extensionModules.some((module) => module.id === "component-lab"),
  "Component Lab is browser-only and excluded from native/extension registries"
);
check(
  componentLabModuleSource.includes('id: "component-lab"') &&
    componentLabModuleSource.includes("webOnly: true") &&
    componentLabDataSource.includes("component-lab-right:inputs") &&
    componentLabDataSource.includes("component-lab-main:collections") &&
    componentLabDataSource.includes("module-bottom-component-lab:responsive"),
  "Component Lab source records web-only module metadata and explicit panel routes"
);
check(
  matrix.includes("| component-lab | web-only | Component Lab | `src/modules/component-lab/` |"),
  "matrix records Component Lab web-only module row"
);
check(moduleComponentsSource.includes("function generatedBottomPanel"), "web generated bottom panel function exists");
check(moduleComponentsSource.includes("data-generated-bottom-panel"), "web generated bottom panels expose handoff marker");
check(matrix.includes("data-generated-bottom-panel"), "matrix names generated bottom panel marker");
check(matrix.includes("visible shell bottom drawer pane body evidence recorded; module lifecycle remains state owner"), "matrix records generated bottom panels as visible shell drawer body evidence without full promotion");
check(
  moduleWorkspaceSource.includes("workbench/modules/generated/workbench_generated_bottom_drawer.zui#WorkbenchGeneratedBottomDrawer") &&
    moduleWorkspaceSource.includes("WorkbenchGeneratedBottomDrawerHost") &&
    generatedBottomDrawerSource.includes("workbench/modules/generated/workbench_generated_bottom_panel.zui#WorkbenchGeneratedBottomPanel") &&
    generatedBottomDrawerSource.includes('props = { visibility = "collapsed" }') &&
    generatedBottomDrawerSource.includes("WorkbenchGeneratedBottomPanelHost"),
  "native module workspace hosts generated bottom drawer evidence"
);
check(
  generatedBottomBodySource.includes('id = "editor.host.pane.generated_bottom.body"') &&
    generatedBottomBodySource.includes("workbench/modules/generated/workbench_generated_bottom_panel.zui#WorkbenchGeneratedBottomPanel") &&
    generatedBottomBodySource.includes("GeneratedBottomPaneBodyRoot") &&
  generatedBottomBodySource.includes("GeneratedBottomPanePanelHost"),
  "native shell generated bottom pane body hosts shared generated bottom panel"
);
check(
  generatedBottomPanelSource.includes('control_id = "WorkbenchGeneratedBottomPanel"') &&
    generatedBottomPanelSource.includes('props = { visibility = "visible" }') &&
    !generatedBottomPanelSource.includes('props = { visibility = "collapsed" }'),
  "native generated bottom shared panel remains visible for direct shell pane hosting"
);
check(
  generatedBottomViewDescriptorSource.includes('ViewDescriptorId::new("editor.generated_bottom")') &&
    generatedBottomViewDescriptorSource.includes("ActivityDrawerSlot::Bottom") &&
    generatedBottomViewDescriptorSource.includes("ViewContentKind::GeneratedBottom") &&
    generatedBottomViewDescriptorSource.includes('"pane.generated_bottom.body"') &&
    generatedBottomViewDescriptorSource.includes("PanePayloadKind::GeneratedBottomV1"),
  "native generated bottom activity view targets bottom drawer pane body"
);
check(
  shellViewInstancesSource.includes('ViewInstanceId::new("editor.generated_bottom#1")') &&
    shellViewInstancesSource.includes("ViewHost::Drawer(ActivityDrawerSlot::Bottom)") &&
    layoutDrawersSource.includes('ViewInstanceId::new("editor.generated_bottom#1")') &&
    layoutDrawersSource.includes("ActivityDrawerSlot::Bottom"),
  "native shell bottom drawer includes generated bottom view instance"
);
check(
  paneDataConversionSource.includes("to_host_contract_generated_bottom_pane_from_host_pane") &&
    paneDataConversionSource.includes("project_pane_template_nodes") &&
    applyPresentationSource.includes("to_host_contract_generated_bottom_pane") &&
    applyPresentationSource.includes("GeneratedBottomV1") &&
    hostContractPanesSource.includes("GeneratedBottomPaneData"),
  "retained host contract projects generated bottom shell pane nodes"
);
check(
  moduleWorkspaceSource.includes("workbench/modules/core/gameplay/workbench_effect_workspace.zui#WorkbenchEffectWorkspace") &&
    moduleWorkspaceSource.includes("WorkbenchEffectWorkspaceHost") &&
    effectWorkspaceSource.includes("[components.WorkbenchEffectWorkspace]") &&
    effectWorkspaceSource.includes("WorkbenchModule/EffectApply"),
  "native module workspace hosts split Effect workspace evidence"
);
check(
  moduleWorkspaceSource.includes("workbench/modules/core/rendering/workbench_material_workspace.zui#WorkbenchMaterialWorkspace") &&
    moduleWorkspaceSource.includes("WorkbenchMaterialWorkspaceHost") &&
    materialWorkspaceSource.includes("[components.WorkbenchMaterialWorkspace]") &&
    materialWorkspaceSource.includes("WorkbenchModule/MaterialCompile"),
  "native module workspace hosts split Material workspace evidence"
);
check(
  moduleWorkspaceSource.includes("workbench/modules/core/ai/workbench_behavior_workspace.zui#WorkbenchBehaviorWorkspace") &&
    moduleWorkspaceSource.includes("WorkbenchBehaviorWorkspaceHost") &&
    behaviorWorkspaceSource.includes("[components.WorkbenchBehaviorWorkspace]") &&
    behaviorWorkspaceSource.includes("WorkbenchModule/BehaviorValidate"),
  "native module workspace hosts split Behavior workspace evidence"
);
check(
  moduleWorkspaceSource.includes("workbench/modules/core/assets/workbench_assets_workspace.zui#WorkbenchAssetsWorkspace") &&
    moduleWorkspaceSource.includes("WorkbenchAssetsWorkspaceHost") &&
    assetsWorkspaceSource.includes("[components.WorkbenchAssetsWorkspace]") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/DataTableOpen") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/SourceControlOpen") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/BuildExportOpen") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/AutomationReportOpen") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/SaveDataOpen"),
  "native module workspace hosts split Assets workspace evidence"
);
check(
  moduleWorkspaceSource.includes("workbench/modules/core/rendering/workbench_vfx_workspace.zui#WorkbenchVfxWorkspace") &&
    moduleWorkspaceSource.includes("WorkbenchVfxWorkspaceHost") &&
    vfxWorkspaceSource.includes("[components.WorkbenchVfxWorkspace]") &&
    vfxWorkspaceSource.includes("WorkbenchModule/VfxSimulate") &&
    vfxWorkspaceSource.includes("WorkbenchVfxParticleLibraryButton") &&
    vfxWorkspaceSource.includes("WorkbenchExtension/ParticleLibraryOpen"),
  "native module workspace hosts split VFX workspace evidence"
);
check(
  additionalModuleWorkspacesSource.includes("workbench/modules/core/gameplay/workbench_ability_workspace.zui#WorkbenchAbilityWorkspace") &&
    additionalModuleWorkspacesSource.includes("WorkbenchAbilityWorkspaceHost") &&
    abilityWorkspaceSource.includes("[components.WorkbenchAbilityWorkspace]") &&
    abilityWorkspaceSource.includes("WorkbenchModule/AbilityPlaytest"),
  "native additional module workspace hosts split Ability workspace evidence"
);
check(
  additionalModuleWorkspacesSource.includes("workbench/modules/core/gameplay/workbench_tags_workspace.zui#WorkbenchTagsWorkspace") &&
    additionalModuleWorkspacesSource.includes("WorkbenchTagsWorkspaceHost") &&
    tagsWorkspaceSource.includes("[components.WorkbenchTagsWorkspace]") &&
    tagsWorkspaceSource.includes("WorkbenchModule/TagsAdd"),
  "native additional module workspace hosts split Tags workspace evidence"
);
check(
  additionalModuleWorkspacesSource.includes("workbench/modules/core/ai/workbench_perception_workspace.zui#WorkbenchPerceptionWorkspace") &&
    additionalModuleWorkspacesSource.includes("WorkbenchPerceptionWorkspaceHost") &&
    perceptionWorkspaceSource.includes("[components.WorkbenchPerceptionWorkspace]") &&
    perceptionWorkspaceSource.includes("WorkbenchModule/PerceptionSimulate"),
  "native additional module workspace hosts split Perception workspace evidence"
);
check(
  abilityWorkspaceSource.includes("WorkbenchAbilitySequencerButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/SequencerOpen") &&
    abilityWorkspaceSource.includes("WorkbenchAbilityMontageEditorButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/MontageEditorOpen") &&
    abilityWorkspaceSource.includes("WorkbenchAbilityBlendSpaceButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/BlendSpaceOpen") &&
    abilityWorkspaceSource.includes("WorkbenchAbilityPoseLibraryButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/PoseLibraryOpen") &&
    abilityWorkspaceSource.includes("WorkbenchAbilityRetargetButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/RetargetOpen") &&
    abilityWorkspaceSource.includes("WorkbenchAbilityControlRigButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/ControlRigOpen") &&
    abilityWorkspaceSource.includes("WorkbenchAbilityMotionMatchingButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/MotionMatchingOpen") &&
    abilityWorkspaceSource.includes("WorkbenchAbilityAnimationCompressionButton") &&
    abilityWorkspaceSource.includes("WorkbenchExtension/AnimationCompressionOpen"),
  "gameplay ability workspace exposes animation extension openers"
);
check(
  additionalModuleWorkspacesSource.includes("workbench/modules/core/rendering/workbench_render_workspace.zui#WorkbenchRenderWorkspace") &&
    additionalModuleWorkspacesSource.includes("WorkbenchRenderWorkspaceHost") &&
    renderWorkspaceSource.includes("[components.WorkbenchRenderWorkspace]") &&
    renderWorkspaceSource.includes("WorkbenchModule/RenderCompile"),
  "native additional module workspace hosts split Render workspace evidence"
);
check(
  additionalModuleWorkspacesSource.includes("workbench/modules/core/ui/workbench_hud_workspace.zui#WorkbenchHudWorkspace") &&
    additionalModuleWorkspacesSource.includes("WorkbenchHudWorkspaceHost") &&
    hudWorkspaceSource.includes("[components.WorkbenchHudWorkspace]") &&
    hudWorkspaceSource.includes("WorkbenchModule/HudPreview") &&
    hudWorkspaceSource.includes("WorkbenchHudConsoleDiagnosticsButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/ConsoleDiagnosticsOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudRuntimeDiagnosticsButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/RuntimeDiagnosticsOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudTelemetryDashboardButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/TelemetryDashboardOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudPerformanceButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/PerformanceOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudFontAtlasButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/FontAtlasOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudUiAssetEditorButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/UiAssetEditorOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudUiBindingButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/UiBindingOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudIconLibraryButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/IconLibraryOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudMenuFlowButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/MenuFlowOpen") &&
    hudWorkspaceSource.includes("WorkbenchHudAccessibilityAuditButton") &&
    hudWorkspaceSource.includes("WorkbenchExtension/AccessibilityAuditOpen"),
  "native additional module workspace hosts split HUD workspace evidence"
);
check(
  generatedBottomPanelSource.includes("WorkbenchGeneratedBottomPanel") &&
    generatedBottomPanelSource.includes("workbench-generated-bottom-route-row") &&
    generatedBottomPanelSource.includes("WorkbenchGeneratedBottom/GameplayEffectCompileLog") &&
    generatedBottomPanelSource.includes("WorkbenchGeneratedBottom/RenderPipelineWarnings"),
  "native generated bottom panel declares retained route rows"
);
check(
  generatedBottomBindingsSource.includes("insert_workbench_generated_bottom_bindings") &&
    generatedBottomBindingsSource.includes("workbench.generated_bottom.gameplay_effect_compile_log.select") &&
    generatedBottomBindingsSource.includes("workbench.generated_bottom.mode.commit"),
  "native generated bottom template bindings exist"
);
check(
  windowBindingsSource.includes("workbench_generated_bottom_template_bindings") &&
    windowBindingsSource.includes("insert_workbench_generated_bottom_bindings"),
  "workbench window installs generated bottom template bindings"
);
check(
  generatedBottomNavigationSource.includes("GENERATED_BOTTOM_ROUTE_TARGETS") &&
    generatedBottomNavigationSource.includes("module-bottom-gameplay-effect:compile-log") &&
    generatedBottomNavigationSource.includes("module-bottom-hud-editor:compile-log"),
  "retained generated bottom navigation covers web panel routes"
);
check(
  generatedBottomFeedbackSource.includes("apply_workbench_generated_bottom_feedback") &&
    generatedBottomFeedbackSource.includes("WorkbenchGeneratedBottomSelectedRoute"),
  "retained generated bottom feedback updates shared panel content"
);
check(
  generatedBottomLifecycleSource.includes("open_workbench_generated_bottom_drawer") &&
    generatedBottomLifecycleSource.includes("close_workbench_generated_bottom_drawer") &&
    generatedBottomLifecycleSource.includes("WorkbenchGeneratedBottomDrawerHost"),
  "retained generated bottom lifecycle owns drawer visibility"
);
check(
  previewActionsSource.includes("workbench.generated_bottom.gameplay_effect_compile_log.select") &&
    previewActionsSource.includes("workbench.generated_bottom.hud_editor_compile_log.select") &&
    previewActionsSource.includes("workbench.generated_bottom.mode.commit"),
  "preview action registry includes generated bottom actions"
);

check(matrix.includes("## Module Handoff"), "module handoff section exists");
for (const module of nativeModules) {
  check(matrix.includes(`| ${module.id} | native-covered |`), `native module ${module.id} is marked native-covered`);
}
for (const module of extensionModules) {
  check(matrix.includes(`| ${module.id} | prototype-only |`), `extension module ${module.id} is marked prototype-only`);
}
check(
  matrix.includes("| terrain-editor | prototype-only | Terrain Editor | `workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui` |"),
  "terrain-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| foliage-editor | prototype-only | Foliage Editor | `workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui` |"),
  "foliage-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| level-streaming | prototype-only | Level Streaming | `workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui` |"),
  "level-streaming row points at the native extension workspace evidence"
);
check(
  matrix.includes("| level-variant | prototype-only | Level Variant | `workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui` |"),
  "level-variant row points at the native extension workspace evidence"
);
check(
  matrix.includes("| prefab-editor | prototype-only | Prefab Editor | `workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui` |"),
  "prefab-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| scatter-editor | prototype-only | Scatter Editor | `workbench/modules/extensions/world/workbench_extension_scatter_editor_workspace.zui` |"),
  "scatter-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| volume-editor | prototype-only | Volume Editor | `workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui` |"),
  "volume-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| weather-editor | prototype-only | Weather Editor | `workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui` |"),
  "weather-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| spawn-rules | prototype-only | Spawn Rules | `workbench/modules/extensions/gameplay/workbench_extension_spawn_rules_workspace.zui` |"),
  "spawn-rules row points at the native extension workspace evidence"
);
check(
  matrix.includes("| world-state | prototype-only | World State | `workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui` |"),
  "world-state row points at the native extension workspace evidence"
);
check(
  matrix.includes("| collision-proxy | prototype-only | Collision Proxy | `workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui` |"),
  "collision-proxy row points at the native extension workspace evidence"
);
check(
  matrix.includes("| physics-collision | prototype-only | Physics Collision | `workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui` |"),
  "physics-collision row points at the native extension workspace evidence"
);
check(
  matrix.includes("| navmesh-ai | prototype-only | Navmesh AI | `workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui` |"),
  "navmesh-ai row points at the native extension workspace evidence"
);
check(
  matrix.includes("| lobby-editor | prototype-only | Lobby Editor | `workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui` |"),
  "lobby-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| matchmaking-editor | prototype-only | Matchmaking Editor | `workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui` |"),
  "matchmaking-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| shader-editor | prototype-only | Shader Editor | `workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui` |"),
  "shader-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| lighting-bake | prototype-only | Lighting Bake | `workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui` |"),
  "lighting-bake row points at the native extension workspace evidence"
);
check(
  matrix.includes("| post-process | prototype-only | Post Process | `workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui` |"),
  "post-process row points at the native extension workspace evidence"
);
check(
  matrix.includes("| sequencer | prototype-only | Sequencer | `workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui` |"),
  "sequencer row points at the native extension workspace evidence"
);
check(
  matrix.includes("| montage-editor | prototype-only | Montage Editor | `workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui` |"),
  "montage-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| blend-space | prototype-only | Blend Space | `workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui` |"),
  "blend-space row points at the native extension workspace evidence"
);
check(
  matrix.includes("| pose-library | prototype-only | Pose Library | `workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui` |"),
  "pose-library row points at the native extension workspace evidence"
);
check(
  matrix.includes("| retarget | prototype-only | Retarget | `workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui` |"),
  "retarget row points at the native extension workspace evidence"
);
check(
  matrix.includes("| control-rig | prototype-only | Control Rig | `workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui` |"),
  "control-rig row points at the native extension workspace evidence"
);
check(
  matrix.includes("| motion-matching | prototype-only | Motion Matching | `workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui` |"),
  "motion-matching row points at the native extension workspace evidence"
);
check(
  matrix.includes("| animation-compression | prototype-only | Animation Compression | `workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui` |"),
  "animation-compression row points at the native extension workspace evidence"
);
check(
  matrix.includes("| data-table | prototype-only | Data Table | `workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui` |"),
  "data-table row points at the native extension workspace evidence"
);
check(
  matrix.includes("| source-control | prototype-only | Source Control | `workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui` |"),
  "source-control row points at the native extension workspace evidence"
);
check(
  matrix.includes("| build-export | prototype-only | Build Export | `workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui` |"),
  "build-export row points at the native extension workspace evidence"
);
check(
  matrix.includes("| automation-report | prototype-only | Automation Report | `workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui` |"),
  "automation-report row points at the native extension workspace evidence"
);
check(
  matrix.includes("| project-overview | prototype-only | Project Overview | `workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui` |"),
  "project-overview row points at the native extension workspace evidence"
);
check(
  matrix.includes("| plugin-manager | prototype-only | Plugin Manager | `workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui` |"),
  "plugin-manager row points at the native extension workspace evidence"
);
check(
  matrix.includes("| particle-library | prototype-only | Particle Library | `workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui` |"),
  "particle-library row points at the native extension workspace evidence"
);
check(
  matrix.includes("| ui-asset-editor | prototype-only | UI Asset Editor | `workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui` |"),
  "ui-asset-editor row points at the native extension workspace evidence"
);
check(
  matrix.includes("| ui-binding | prototype-only | UI Binding | `workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui` |"),
  "ui-binding row points at the native extension workspace evidence"
);
check(
  matrix.includes("| icon-library | prototype-only | Icon Library | `workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui` |"),
  "icon-library row points at the native extension workspace evidence"
);
check(
  matrix.includes("| accessibility-audit | prototype-only | Accessibility Audit | `workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui` |"),
  "accessibility-audit row points at the native extension workspace evidence"
);
check(
  matrix.includes("| font-atlas | prototype-only | Font Atlas | `workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui` |"),
  "font-atlas row points at the native extension workspace evidence"
);
check(
  matrix.includes("| console-diagnostics | prototype-only | Console Diagnostics | `workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui` |"),
  "console-diagnostics row points at the native extension workspace evidence"
);
check(
  matrix.includes("| runtime-diagnostics | prototype-only | Runtime Diagnostics | `workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui` |"),
  "runtime-diagnostics row points at the native extension workspace evidence"
);
check(
  matrix.includes("| telemetry-dashboard | prototype-only | Telemetry Dashboard | `workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui` |"),
  "telemetry-dashboard row points at the native extension workspace evidence"
);
check(
  matrix.includes("| performance | prototype-only | Performance | `workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui` |"),
  "performance row points at the native extension workspace evidence"
);
check(
  matrix.includes("| menu-flow | prototype-only | Menu Flow | `workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui` |"),
  "menu-flow row points at the native extension workspace evidence"
);
check(
  matrix.includes("| save-data | prototype-only | Save Data | `workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui` |"),
  "save-data row points at the native extension workspace evidence"
);
check(matrix.includes("native-extension workspace evidence"), "matrix names native extension workspace evidence");
check(
  moduleWorkspaceSource.includes("workbench/modules/extensions/index/workbench_extension_module_workspaces.zui#WorkbenchExtensionModuleWorkspaces") &&
    moduleWorkspaceSource.includes("WorkbenchExtensionModuleWorkspacesHost"),
  "native module workspace hosts extension module workspaces"
);
check(
  additionalModuleWorkspacesSource.includes("workbench/modules/core/rendering/workbench_render_workspace.zui#WorkbenchRenderWorkspace") &&
    additionalModuleWorkspacesSource.includes("WorkbenchRenderWorkspaceHost") &&
    renderWorkspaceSource.includes("WorkbenchRenderShaderEditorButton") &&
    renderWorkspaceSource.includes("WorkbenchExtension/ShaderEditorOpen") &&
    renderWorkspaceSource.includes("WorkbenchRenderLightingBakeButton") &&
    renderWorkspaceSource.includes("WorkbenchExtension/LightingBakeOpen") &&
    renderWorkspaceSource.includes("WorkbenchRenderPostProcessButton") &&
    renderWorkspaceSource.includes("WorkbenchExtension/PostProcessOpen"),
  "render pipeline workspace exposes rendering extension openers through the split Render workspace"
);
check(
  assetsWorkspaceSource.includes("WorkbenchAssetsDataTableButton") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsTerrainEditorButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/TerrainEditorOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsFoliageEditorButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/FoliageEditorOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsLevelStreamingButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/LevelStreamingOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsLevelVariantButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/LevelVariantOpen") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/DataTableOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsSourceControlButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/SourceControlOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsBuildExportButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/BuildExportOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsAutomationReportButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/AutomationReportOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsProjectOverviewButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/ProjectOverviewOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsPhysicsCollisionButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/PhysicsCollisionOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsNavmeshAiButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/NavmeshAiOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsPluginManagerButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/PluginManagerOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsSaveDataButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/SaveDataOpen") &&
    assetsWorkspaceSource.includes("WorkbenchAssetsWorldStateButton") &&
    assetsWorkspaceSource.includes("WorkbenchExtension/WorldStateOpen"),
  "asset browser workspace exposes data extension openers"
);
check(
    extensionWorkspaceSource.includes("WorkbenchExtensionModuleWorkspaces") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/world/workbench_extension_terrain_editor_workspace.zui#WorkbenchExtensionTerrainEditorWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/world/workbench_extension_foliage_editor_workspace.zui#WorkbenchExtensionFoliageEditorWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/world/workbench_extension_level_streaming_workspace.zui#WorkbenchExtensionLevelStreamingWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui#WorkbenchExtensionLevelVariantWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/rendering/workbench_extension_shader_editor_workspace.zui#WorkbenchExtensionShaderEditorWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui#WorkbenchExtensionLightingBakeWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui#WorkbenchExtensionPostProcessWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui#WorkbenchExtensionSequencerWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui#WorkbenchExtensionMontageEditorWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui#WorkbenchExtensionBlendSpaceWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui#WorkbenchExtensionPoseLibraryWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui#WorkbenchExtensionRetargetWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui#WorkbenchExtensionControlRigWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui#WorkbenchExtensionMotionMatchingWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/animation/workbench_extension_animation_compression_workspace.zui#WorkbenchExtensionAnimationCompressionWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/gameplay/workbench_extension_world_state_workspace.zui#WorkbenchExtensionWorldStateWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui#WorkbenchExtensionCollisionProxyWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui#WorkbenchExtensionPhysicsCollisionWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui#WorkbenchExtensionNavmeshAiWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/multiplayer/workbench_extension_lobby_editor_workspace.zui#WorkbenchExtensionLobbyEditorWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/multiplayer/workbench_extension_matchmaking_editor_workspace.zui#WorkbenchExtensionMatchmakingEditorWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/data/workbench_extension_data_table_workspace.zui#WorkbenchExtensionDataTableWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/ui/workbench_extension_font_atlas_workspace.zui#WorkbenchExtensionFontAtlasWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/diagnostics/workbench_extension_console_diagnostics_workspace.zui#WorkbenchExtensionConsoleDiagnosticsWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui#WorkbenchExtensionRuntimeDiagnosticsWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/diagnostics/workbench_extension_performance_workspace.zui#WorkbenchExtensionPerformanceWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/diagnostics/workbench_extension_telemetry_dashboard_workspace.zui#WorkbenchExtensionTelemetryDashboardWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/production/workbench_extension_source_control_workspace.zui#WorkbenchExtensionSourceControlWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/production/workbench_extension_build_export_workspace.zui#WorkbenchExtensionBuildExportWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/production/workbench_extension_automation_report_workspace.zui#WorkbenchExtensionAutomationReportWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/production/workbench_extension_project_overview_workspace.zui#WorkbenchExtensionProjectOverviewWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/production/workbench_extension_plugin_manager_workspace.zui#WorkbenchExtensionPluginManagerWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/data/workbench_extension_save_data_workspace.zui#WorkbenchExtensionSaveDataWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui#WorkbenchExtensionParticleLibraryWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui#WorkbenchExtensionUiAssetEditorWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/ui/workbench_extension_ui_binding_workspace.zui#WorkbenchExtensionUiBindingWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui#WorkbenchExtensionIconLibraryWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/ui/workbench_extension_accessibility_audit_workspace.zui#WorkbenchExtensionAccessibilityAuditWorkspace") &&
    extensionWorkspaceSource.includes("workbench/modules/extensions/ui/workbench_extension_menu_flow_workspace.zui#WorkbenchExtensionMenuFlowWorkspace"),
  "native extension workspace host imports split extension workspace components"
);
check(
  extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionTerrainEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/TerrainEditorBuild") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionFoliageEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/FoliageEditorBuild") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionLevelStreamingWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/LevelStreamingLoad") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionLevelVariantWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/LevelVariantApply") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionPrefabEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/PrefabEditorValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionScatterEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/ScatterEditorValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionVolumeEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/VolumeEditorValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionWeatherEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/WeatherEditorBuild") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionSpawnRulesWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/SpawnRulesValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionWorldStateWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/WorldStateValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionCollisionProxyWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/CollisionProxyBake") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionPhysicsCollisionWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/PhysicsCollisionValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionNavmeshAiWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/NavmeshAiQueryPath") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionLobbyEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/LobbyEditorSimulateLobby") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionMatchmakingEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/MatchmakingEditorSimulateMatch") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionShaderEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/ShaderEditorCompile") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionLightingBakeWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/LightingBakeBake") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionPostProcessWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/PostProcessApply") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionSequencerWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/SequencerValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionMontageEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/MontageEditorApply") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionBlendSpaceWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/BlendSpaceApply") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionPoseLibraryWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/PoseLibraryApply") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionRetargetWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/RetargetApply") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionControlRigWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/ControlRigValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionMotionMatchingWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/MotionMatchingRebuild") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionAnimationCompressionWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/AnimationCompressionCompress") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionDataTableWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/DataTableSave") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionFontAtlasWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/FontAtlasBakeAtlas") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionConsoleDiagnosticsWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/ConsoleDiagnosticsFilterConsole") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionRuntimeDiagnosticsWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/RuntimeDiagnosticsCaptureSnapshot") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionPerformanceWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/PerformanceCaptureFrame") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionTelemetryDashboardWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/TelemetryDashboardFilterTelemetry") &&
    extensionWorkspaceSource.includes("WorkbenchExtensionShaderEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionSourceControlWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/SourceControlSubmit") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionBuildExportWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/BuildExportPackage") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionAutomationReportWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/AutomationReportPublish") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionProjectOverviewWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/ProjectOverviewPublish") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionPluginManagerWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/PluginManagerHotReload") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionSaveDataWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/SaveDataSaveSlot") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionParticleLibraryWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/ParticleLibraryCompile") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionUiAssetEditorWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/UiAssetEditorValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionUiBindingWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/UiBindingValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionIconLibraryWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/IconLibraryValidate") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionAccessibilityAuditWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/AccessibilityAuditAuditScreen") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtensionMenuFlowWorkspace") &&
    extensionWorkspaceEvidenceSource.includes("WorkbenchExtension/MenuFlowValidateFocus"),
  "native extension workspaces declare retained controls and events"
);
check(
  !coreModuleBindingsSource.includes("insert_workbench_extension_module_bindings") &&
    extensionModuleBindingsSource.includes("insert_workbench_extension_module_bindings") &&
    extensionModuleBindingsSource.includes("workbench.extension.terrain_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.terrain_editor.strength.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.foliage_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.foliage_editor.radius.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.level_streaming.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.level_streaming.distance.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.level_variant.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.level_variant.capture.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.prefab_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.prefab_editor.instance.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.scatter_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.scatter_editor.density.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.volume_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.volume_editor.priority.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.weather_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.weather_editor.blend_time.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.spawn_rules.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.spawn_rules.seed.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.world_state.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.world_state.authority.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.navmesh_ai.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.navmesh_ai.cost.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.physics_collision.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.physics_collision.mass.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.lobby_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.lobby_editor.max_players.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.shader_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.shader_editor.live_compile.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.lighting_bake.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.lighting_bake.denoise.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.post_process.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.post_process.blend.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.sequencer.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.sequencer.work_range.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.montage_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.montage_editor.blend.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.blend_space.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.blend_space.interpolation.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.pose_library.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.pose_library.mirror.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.retarget.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.retarget.solver.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.control_rig.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.control_rig.weight.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.motion_matching.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.motion_matching.cost.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.animation_compression.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.animation_compression.tolerance.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.data_table.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.data_table.version.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.automation_report.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.automation_report.retry.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.project_overview.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.project_overview.health.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.plugin_manager.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.plugin_manager.version.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.save_data.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.save_data.compression.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.font_atlas.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.font_atlas.range.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.console_diagnostics.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.console_diagnostics.regex.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.runtime_diagnostics.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.runtime_diagnostics.filter.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.performance.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.performance.threshold.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.telemetry_dashboard.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.telemetry_dashboard.segment.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.source_control.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.source_control.gate.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.build_export.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.build_export.compression.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.particle_library.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.particle_library.bounds.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.ui_asset_editor.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.ui_asset_editor.theme.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.ui_binding.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.ui_binding.converter.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.icon_library.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.icon_library.color_token.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.accessibility_audit.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.accessibility_audit.rule_set.commit") &&
    extensionModuleBindingsSource.includes("workbench.extension.menu_flow.open") &&
    extensionModuleBindingsSource.includes("workbench.extension.menu_flow.transition.commit"),
  "extension module template bindings are split from core module bindings"
);
check(
  windowBindingsSource.includes("workbench_extension_module_template_bindings") &&
    windowBindingsSource.includes("insert_workbench_extension_module_bindings"),
  "workbench window installs extension module template bindings"
);
check(
  extensionNavigationSource.includes("EXTENSION_MODULE_WORKSPACE_CONTROLS") &&
    extensionNavigationSource.includes("workbench.extension.terrain_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionTerrainEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.foliage_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionFoliageEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.level_streaming.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionLevelStreamingWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.level_variant.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionLevelVariantWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.prefab_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionPrefabEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.scatter_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionScatterEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.volume_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionVolumeEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.weather_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionWeatherEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.spawn_rules.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionSpawnRulesWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.world_state.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionWorldStateWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.navmesh_ai.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionNavmeshAiWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.physics_collision.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionPhysicsCollisionWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.lobby_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionLobbyEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.matchmaking_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionMatchmakingEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.shader_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionShaderEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.lighting_bake.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionLightingBakeWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.post_process.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionPostProcessWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.sequencer.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionSequencerWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.montage_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionMontageEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.blend_space.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionBlendSpaceWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.pose_library.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionPoseLibraryWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.retarget.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionRetargetWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.control_rig.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionControlRigWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.motion_matching.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionMotionMatchingWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.animation_compression.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionAnimationCompressionWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.data_table.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionDataTableWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.font_atlas.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionFontAtlasWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.console_diagnostics.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionConsoleDiagnosticsWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.runtime_diagnostics.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionRuntimeDiagnosticsWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.performance.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionPerformanceWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.telemetry_dashboard.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionTelemetryDashboardWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.source_control.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionSourceControlWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.build_export.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionBuildExportWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.automation_report.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionAutomationReportWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.project_overview.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionProjectOverviewWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.plugin_manager.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionPluginManagerWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.save_data.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionSaveDataWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.particle_library.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionParticleLibraryWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.ui_asset_editor.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionUiAssetEditorWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.ui_binding.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionUiBindingWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.icon_library.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionIconLibraryWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.accessibility_audit.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionAccessibilityAuditWorkspace") &&
    extensionNavigationSource.includes("workbench.extension.menu_flow.open") &&
    extensionNavigationSource.includes("WorkbenchExtensionMenuFlowWorkspace"),
  "extension navigation maps extension actions to retained controls"
);
check(
    extensionFeedbackSource.includes("apply_workbench_extension_module_command_feedback") &&
    extensionFeedbackSource.includes("WorkbenchExtensionTerrainEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.terrain_editor.build.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionFoliageEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.foliage_editor.build.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionLevelStreamingOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.level_streaming.load.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionLevelVariantOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.level_variant.apply.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionPrefabEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.prefab_editor.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionScatterEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.scatter_editor.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionVolumeEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.volume_editor.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionWeatherEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.weather_editor.build.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionSpawnRulesOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.spawn_rules.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionWorldStateOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.world_state.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionCollisionProxyOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.collision_proxy.bake.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionPhysicsCollisionOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.physics_collision.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionNavmeshAiOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.navmesh_ai.query_path.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionLobbyEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.lobby_editor.simulate_lobby.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionShaderOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.shader_editor.compile.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionLightingBakeOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.lighting_bake.bake.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionPostProcessOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.post_process.apply.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionSequencerOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.sequencer.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionMontageEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.montage_editor.apply.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionBlendSpaceOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.blend_space.apply.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionPoseLibraryOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.pose_library.apply.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionRetargetOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.retarget.apply.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionControlRigOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.control_rig.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionMotionMatchingOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.motion_matching.rebuild.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionAnimationCompressionOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.animation_compression.compress.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionDataTableOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.data_table.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionFontAtlasOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.font_atlas.bake_atlas.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionConsoleDiagnosticsOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.console_diagnostics.filter_console.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionRuntimeDiagnosticsOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.runtime_diagnostics.capture_snapshot.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionPerformanceOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.performance.capture_frame.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionTelemetryDashboardOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.telemetry_dashboard.filter_telemetry.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionSourceControlOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.source_control.submit.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionBuildExportOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.build_export.package.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionAutomationReportOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.automation_report.publish.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionProjectOverviewOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.project_overview.publish.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionPluginManagerOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.plugin_manager.hot_reload.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionSaveDataOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.save_data.save_slot.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionParticleLibraryOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.particle_library.compile.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionUiAssetEditorOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.ui_asset_editor.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionUiBindingOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.ui_binding.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionIconLibraryOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.icon_library.validate.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionAccessibilityAuditOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.accessibility_audit.audit_screen.invoke") &&
    extensionFeedbackSource.includes("WorkbenchExtensionMenuFlowOutputRow") &&
    extensionFeedbackSource.includes("workbench.extension.menu_flow.validate_focus.invoke"),
  "extension feedback updates extension status and output controls"
);
check(
    previewActionsSource.includes("workbench.extension.terrain_editor.open") &&
    previewActionsSource.includes("workbench.extension.terrain_editor.strength.commit") &&
    previewActionsSource.includes("workbench.extension.foliage_editor.open") &&
    previewActionsSource.includes("workbench.extension.foliage_editor.radius.commit") &&
    previewActionsSource.includes("workbench.extension.level_streaming.open") &&
    previewActionsSource.includes("workbench.extension.level_streaming.distance.commit") &&
    previewActionsSource.includes("workbench.extension.level_variant.open") &&
    previewActionsSource.includes("workbench.extension.level_variant.capture.commit") &&
    previewActionsSource.includes("workbench.extension.prefab_editor.open") &&
    previewActionsSource.includes("workbench.extension.prefab_editor.instance.commit") &&
    previewActionsSource.includes("workbench.extension.scatter_editor.open") &&
    previewActionsSource.includes("workbench.extension.scatter_editor.density.commit") &&
    previewActionsSource.includes("workbench.extension.volume_editor.open") &&
    previewActionsSource.includes("workbench.extension.volume_editor.priority.commit") &&
    previewActionsSource.includes("workbench.extension.weather_editor.open") &&
    previewActionsSource.includes("workbench.extension.weather_editor.blend_time.commit") &&
    previewActionsSource.includes("workbench.extension.spawn_rules.open") &&
    previewActionsSource.includes("workbench.extension.spawn_rules.seed.commit") &&
    previewActionsSource.includes("workbench.extension.world_state.open") &&
    previewActionsSource.includes("workbench.extension.world_state.authority.commit") &&
    previewActionsSource.includes("workbench.extension.collision_proxy.open") &&
    previewActionsSource.includes("workbench.extension.collision_proxy.lod.commit") &&
    previewActionsSource.includes("workbench.extension.physics_collision.open") &&
    previewActionsSource.includes("workbench.extension.physics_collision.mass.commit") &&
    previewActionsSource.includes("workbench.extension.navmesh_ai.open") &&
    previewActionsSource.includes("workbench.extension.navmesh_ai.cost.commit") &&
    previewActionsSource.includes("workbench.extension.lobby_editor.open") &&
    previewActionsSource.includes("workbench.extension.lobby_editor.max_players.commit") &&
    previewActionsSource.includes("workbench.extension.shader_editor.open") &&
    previewActionsSource.includes("workbench.extension.shader_editor.compile.invoke") &&
    previewActionsSource.includes("workbench.extension.shader_editor.live_compile.commit") &&
    previewActionsSource.includes("workbench.extension.lighting_bake.open") &&
    previewActionsSource.includes("workbench.extension.lighting_bake.denoise.commit") &&
    previewActionsSource.includes("workbench.extension.post_process.open") &&
    previewActionsSource.includes("workbench.extension.post_process.blend.commit") &&
    previewActionsSource.includes("workbench.extension.sequencer.open") &&
    previewActionsSource.includes("workbench.extension.sequencer.work_range.commit") &&
    previewActionsSource.includes("workbench.extension.montage_editor.open") &&
    previewActionsSource.includes("workbench.extension.montage_editor.blend.commit") &&
    previewActionsSource.includes("workbench.extension.blend_space.open") &&
    previewActionsSource.includes("workbench.extension.blend_space.interpolation.commit") &&
    previewActionsSource.includes("workbench.extension.pose_library.open") &&
    previewActionsSource.includes("workbench.extension.pose_library.mirror.commit") &&
    previewActionsSource.includes("workbench.extension.retarget.open") &&
    previewActionsSource.includes("workbench.extension.retarget.solver.commit") &&
    previewActionsSource.includes("workbench.extension.control_rig.open") &&
    previewActionsSource.includes("workbench.extension.control_rig.weight.commit") &&
    previewActionsSource.includes("workbench.extension.motion_matching.open") &&
    previewActionsSource.includes("workbench.extension.motion_matching.cost.commit") &&
    previewActionsSource.includes("workbench.extension.animation_compression.open") &&
    previewActionsSource.includes("workbench.extension.animation_compression.tolerance.commit") &&
    previewActionsSource.includes("workbench.extension.data_table.open") &&
    previewActionsSource.includes("workbench.extension.data_table.version.commit") &&
    previewActionsSource.includes("workbench.extension.font_atlas.open") &&
    previewActionsSource.includes("workbench.extension.font_atlas.range.commit") &&
    previewActionsSource.includes("workbench.extension.console_diagnostics.open") &&
    previewActionsSource.includes("workbench.extension.console_diagnostics.regex.commit") &&
    previewActionsSource.includes("workbench.extension.runtime_diagnostics.open") &&
    previewActionsSource.includes("workbench.extension.runtime_diagnostics.filter.commit") &&
    previewActionsSource.includes("workbench.extension.performance.open") &&
    previewActionsSource.includes("workbench.extension.performance.threshold.commit") &&
    previewActionsSource.includes("workbench.extension.telemetry_dashboard.open") &&
    previewActionsSource.includes("workbench.extension.telemetry_dashboard.segment.commit") &&
    previewActionsSource.includes("workbench.extension.source_control.open") &&
    previewActionsSource.includes("workbench.extension.source_control.gate.commit") &&
    previewActionsSource.includes("workbench.extension.build_export.open") &&
    previewActionsSource.includes("workbench.extension.build_export.compression.commit") &&
    previewActionsSource.includes("workbench.extension.automation_report.open") &&
    previewActionsSource.includes("workbench.extension.automation_report.retry.commit") &&
    previewActionsSource.includes("workbench.extension.project_overview.open") &&
    previewActionsSource.includes("workbench.extension.project_overview.health.commit") &&
    previewActionsSource.includes("workbench.extension.plugin_manager.open") &&
    previewActionsSource.includes("workbench.extension.plugin_manager.version.commit") &&
    previewActionsSource.includes("workbench.extension.save_data.open") &&
    previewActionsSource.includes("workbench.extension.save_data.compression.commit") &&
    previewActionsSource.includes("workbench.extension.particle_library.open") &&
    previewActionsSource.includes("workbench.extension.particle_library.bounds.commit") &&
    previewActionsSource.includes("workbench.extension.ui_asset_editor.open") &&
    previewActionsSource.includes("workbench.extension.ui_asset_editor.theme.commit") &&
    previewActionsSource.includes("workbench.extension.ui_binding.open") &&
    previewActionsSource.includes("workbench.extension.ui_binding.converter.commit") &&
    previewActionsSource.includes("workbench.extension.icon_library.open") &&
    previewActionsSource.includes("workbench.extension.icon_library.color_token.commit") &&
    previewActionsSource.includes("workbench.extension.accessibility_audit.open") &&
    previewActionsSource.includes("workbench.extension.accessibility_audit.rule_set.commit") &&
    previewActionsSource.includes("workbench.extension.menu_flow.open") &&
    previewActionsSource.includes("workbench.extension.menu_flow.transition.commit"),
  "preview action registry includes extension actions"
);

check(matrix.includes("## Reference Manifest Handoff"), "reference manifest section exists");
check(matrix.includes(`| core references | ${coreReferenceSamples.length} | native-covered |`), "core reference count is recorded");
check(matrix.includes(`| extension references | ${extensionReferenceSamples.length} | prototype-only |`), "extension reference count is recorded");
check(matrix.includes(`| supplemental references | ${supplementalReferenceSamples.length} | supporting |`), "supplemental reference count is recorded");

check(matrix.includes("## Retained/Taffy Migration Gates"), "retained taffy migration gates section exists");
for (const command of requiredCommands) {
  check(matrix.includes(command), `verification command is recorded: ${command}`);
}
check(matrix.includes("Do not promote prototype-only extension modules into native-covered status until matching .zui workspace, binding, preview action, and retained route evidence exist."), "prototype-only promotion rule is explicit");

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`fail ${failure}`);
  }
  process.exit(1);
}

console.log(`web-native handoff matrix: native=${nativeModules.length} webTabs=${webModuleTabs.length} extension=${extensionModules.length} componentFamilies=${componentFamilies.length} nativeExtensionEvidence=44`);
console.log("ok web-native handoff matrix");

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}


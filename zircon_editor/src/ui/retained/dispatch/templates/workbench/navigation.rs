use std::{collections::HashMap, sync::LazyLock};

#[path = "navigation/specs.rs"]
mod specs;

use specs::EXTENSION_MODULE_NAVIGATION_SPECS;
pub(super) use specs::EXTENSION_MODULE_WORKSPACE_CONTROLS;

const EMPTY_CONTROLS: &[&str] = &[];

pub(super) fn is_workbench_extension_action(action_id: &str) -> bool {
    extension_action_index().contains_key(action_id)
}

pub(super) fn workbench_extension_workspace_control_id(action_id: &str) -> Option<&'static str> {
    extension_action_index()
        .get(action_id)
        .and_then(|route| route.workspace_control_id)
}

/// Canonical sources loaded only when their registered extension is selected.
pub(super) fn workbench_extension_workspace_document(
    action_id: &str,
) -> Option<(&'static str, &'static str)> {
    match action_id {
        "workbench.extension.particle_library.open" => Some((
            "res://ui/rendering/extension_particle_library_workspace.zui",
            "WorkbenchExtensionParticleLibraryWorkspace",
        )),
        "workbench.extension.terrain_editor.open" => Some((
            "res://ui/editor/world/extension_terrain_editor_workspace.zui",
            "WorkbenchExtensionTerrainEditorWorkspace",
        )),
        "workbench.extension.foliage_editor.open" => Some((
            "res://ui/editor/world/extension_foliage_editor_workspace.zui",
            "WorkbenchExtensionFoliageEditorWorkspace",
        )),
        "workbench.extension.level_streaming.open" => Some((
            "res://ui/editor/world/extension_level_streaming_workspace.zui",
            "WorkbenchExtensionLevelStreamingWorkspace",
        )),
        "workbench.extension.level_variant.open" => Some((
            "res://ui/editor/world/extension_level_variant_workspace.zui",
            "WorkbenchExtensionLevelVariantWorkspace",
        )),
        "workbench.extension.prefab_editor.open" => Some((
            "res://ui/editor/world/extension_prefab_editor_workspace.zui",
            "WorkbenchExtensionPrefabEditorWorkspace",
        )),
        "workbench.extension.scatter_editor.open" => Some((
            "res://ui/editor/world/extension_scatter_editor_workspace.zui",
            "WorkbenchExtensionScatterEditorWorkspace",
        )),
        "workbench.extension.volume_editor.open" => Some((
            "res://ui/editor/world/extension_volume_editor_workspace.zui",
            "WorkbenchExtensionVolumeEditorWorkspace",
        )),
        "workbench.extension.weather_editor.open" => Some((
            "res://ui/editor/world/extension_weather_editor_workspace.zui",
            "WorkbenchExtensionWeatherEditorWorkspace",
        )),
        "workbench.extension.spawn_rules.open" => Some((
            "res://ui/editor/gameplay/extension_spawn_rules_workspace.zui",
            "WorkbenchExtensionSpawnRulesWorkspace",
        )),
        "workbench.extension.world_state.open" => Some((
            "res://ui/editor/gameplay/extension_world_state_workspace.zui",
            "WorkbenchExtensionWorldStateWorkspace",
        )),
        "workbench.extension.collision_proxy.open" => Some((
            "res://ui/simulation/extension_collision_proxy_workspace.zui",
            "WorkbenchExtensionCollisionProxyWorkspace",
        )),
        "workbench.extension.physics_collision.open" => Some((
            "res://ui/simulation/extension_physics_collision_workspace.zui",
            "WorkbenchExtensionPhysicsCollisionWorkspace",
        )),
        "workbench.extension.navmesh_ai.open" => Some((
            "res://ui/editor/simulation/extension_navmesh_ai_workspace.zui",
            "WorkbenchExtensionNavmeshAiWorkspace",
        )),
        "workbench.extension.lobby_editor.open" => Some((
            "res://ui/editor/multiplayer/extension_lobby_editor_workspace.zui",
            "WorkbenchExtensionLobbyEditorWorkspace",
        )),
        "workbench.extension.matchmaking_editor.open" => Some((
            "res://ui/multiplayer/extension_matchmaking_editor_workspace.zui",
            "WorkbenchExtensionMatchmakingEditorWorkspace",
        )),
        "workbench.extension.data_table.open" => Some((
            "res://ui/editor/modules/data/extension_data_table_workspace.zui",
            "WorkbenchExtensionDataTableWorkspace",
        )),
        "workbench.extension.source_control.open" => Some((
            "res://ui/editor/production/extension_source_control_workspace.zui",
            "WorkbenchExtensionSourceControlWorkspace",
        )),
        "workbench.extension.build_export.open" => Some((
            "res://ui/editor/production/extension_build_export_workspace.zui",
            "WorkbenchExtensionBuildExportWorkspace",
        )),
        "workbench.extension.automation_report.open" => Some((
            "res://ui/production/extension_automation_report_workspace.zui",
            "WorkbenchExtensionAutomationReportWorkspace",
        )),
        "workbench.extension.project_overview.open" => Some((
            "res://ui/production/extension_project_overview_workspace.zui",
            "WorkbenchExtensionProjectOverviewWorkspace",
        )),
        "workbench.extension.plugin_manager.open" => Some((
            "res://ui/editor/production/extension_plugin_manager_workspace.zui",
            "WorkbenchExtensionPluginManagerWorkspace",
        )),
        "workbench.extension.save_data.open" => Some((
            "res://ui/editor/modules/data/extension_save_data_workspace.zui",
            "WorkbenchExtensionSaveDataWorkspace",
        )),
        "workbench.extension.sequencer.open" => Some((
            "res://ui/editor/animation/extension_sequencer_workspace.zui",
            "WorkbenchExtensionSequencerWorkspace",
        )),
        "workbench.extension.montage_editor.open" => Some((
            "res://ui/editor/animation/extension_montage_editor_workspace.zui",
            "WorkbenchExtensionMontageEditorWorkspace",
        )),
        "workbench.extension.blend_space.open" => Some((
            "res://ui/editor/animation/extension_blend_space_workspace.zui",
            "WorkbenchExtensionBlendSpaceWorkspace",
        )),
        "workbench.extension.pose_library.open" => Some((
            "res://ui/editor/animation/extension_pose_library_workspace.zui",
            "WorkbenchExtensionPoseLibraryWorkspace",
        )),
        "workbench.extension.retarget.open" => Some((
            "res://ui/editor/animation/extension_retarget_workspace.zui",
            "WorkbenchExtensionRetargetWorkspace",
        )),
        "workbench.extension.control_rig.open" => Some((
            "res://ui/editor/animation/extension_control_rig_workspace.zui",
            "WorkbenchExtensionControlRigWorkspace",
        )),
        "workbench.extension.motion_matching.open" => Some((
            "res://ui/editor/animation/extension_motion_matching_workspace.zui",
            "WorkbenchExtensionMotionMatchingWorkspace",
        )),
        "workbench.extension.animation_compression.open" => Some((
            "res://ui/animation/extension_animation_compression_workspace.zui",
            "WorkbenchExtensionAnimationCompressionWorkspace",
        )),
        "workbench.extension.shader_editor.open" => Some((
            "res://ui/editor/rendering/extension_shader_editor_workspace.zui",
            "WorkbenchExtensionShaderEditorWorkspace",
        )),
        "workbench.extension.lighting_bake.open" => Some((
            "res://ui/editor/rendering/extension_lighting_bake_workspace.zui",
            "WorkbenchExtensionLightingBakeWorkspace",
        )),
        "workbench.extension.post_process.open" => Some((
            "res://ui/editor/rendering/extension_post_process_workspace.zui",
            "WorkbenchExtensionPostProcessWorkspace",
        )),
        "workbench.extension.console_diagnostics.open" => Some((
            "res://ui/diagnostics/extension_console_diagnostics_workspace.zui",
            "WorkbenchExtensionConsoleDiagnosticsWorkspace",
        )),
        "workbench.extension.runtime_diagnostics.open" => Some((
            "res://ui/diagnostics/extension_runtime_diagnostics_workspace.zui",
            "WorkbenchExtensionRuntimeDiagnosticsWorkspace",
        )),
        "workbench.extension.telemetry_dashboard.open" => Some((
            "res://ui/diagnostics/extension_telemetry_dashboard_workspace.zui",
            "WorkbenchExtensionTelemetryDashboardWorkspace",
        )),
        "workbench.extension.performance.open" => Some((
            "res://ui/editor/modules/diagnostics/extension_perf_workspace.zui",
            "WorkbenchExtensionPerformanceWorkspace",
        )),
        "workbench.extension.font_atlas.open" => Some((
            "res://ui/editor/modules/ui/extension_font_atlas_workspace.zui",
            "WorkbenchExtensionFontAtlasWorkspace",
        )),
        "workbench.extension.menu_flow.open" => Some((
            "res://ui/editor/modules/ui/extension_menu_flow_workspace.zui",
            "WorkbenchExtensionMenuFlowWorkspace",
        )),
        "workbench.extension.accessibility_audit.open" => Some((
            "res://ui/editor/ui/extension_accessibility_audit_workspace.zui",
            "WorkbenchExtensionAccessibilityAuditWorkspace",
        )),
        "workbench.extension.icon_library.open" => Some((
            "res://ui/editor/modules/ui/extension_icon_library_workspace.zui",
            "WorkbenchExtensionIconLibraryWorkspace",
        )),
        "workbench.extension.ui_binding.open" => Some((
            "res://ui/editor/modules/ui/extension_ui_binding_workspace.zui",
            "WorkbenchExtensionUiBindingWorkspace",
        )),
        "workbench.extension.ui_asset_editor.open" => Some((
            "res://ui/editor/ui/extension_ui_asset_editor_workspace.zui",
            "WorkbenchExtensionUiAssetEditorWorkspace",
        )),
        _ => None,
    }
}

pub(super) fn workbench_extension_panel_row_control_id(action_id: &str) -> Option<&'static str> {
    extension_action_index()
        .get(action_id)
        .and_then(|route| route.row_control_id)
}

pub(super) fn workbench_extension_panel_row_group(action_id: &str) -> &'static [&'static str] {
    extension_action_index()
        .get(action_id)
        .map(|route| route.row_controls)
        .unwrap_or(EMPTY_CONTROLS)
}

pub(super) fn workbench_extension_panel_command_control_id(
    action_id: &str,
) -> Option<&'static str> {
    extension_action_index()
        .get(action_id)
        .and_then(|route| route.command_control_id)
}

pub(super) fn workbench_extension_panel_field_action(action_id: &str) -> bool {
    extension_action_index()
        .get(action_id)
        .is_some_and(|route| route.field_action)
}

#[derive(Clone, Copy, Default)]
struct ExtensionActionRoute {
    workspace_control_id: Option<&'static str>,
    row_control_id: Option<&'static str>,
    row_controls: &'static [&'static str],
    command_control_id: Option<&'static str>,
    field_action: bool,
}

fn extension_action_index() -> &'static HashMap<&'static str, ExtensionActionRoute> {
    static INDEX: LazyLock<HashMap<&'static str, ExtensionActionRoute>> =
        LazyLock::new(build_extension_action_index);
    &INDEX
}

fn build_extension_action_index() -> HashMap<&'static str, ExtensionActionRoute> {
    let mut index = HashMap::<&'static str, ExtensionActionRoute>::new();
    for spec in EXTENSION_MODULE_NAVIGATION_SPECS {
        let _ = index
            .entry(spec.open_action_id)
            .or_default()
            .workspace_control_id
            .get_or_insert(spec.workspace_control_id);
        for action in spec.row_actions {
            let route = index.entry(action.action_id).or_default();
            let _ = route.row_control_id.get_or_insert(action.control_id);
            if route.row_controls.is_empty() {
                route.row_controls = spec.row_controls;
            }
        }
        for action in spec.command_actions {
            debug_assert!(spec.command_controls.contains(&action.control_id));
            let route = index.entry(action.action_id).or_default();
            let _ = route.command_control_id.get_or_insert(action.control_id);
        }
        for action_id in spec.field_actions {
            index.entry(action_id).or_default().field_action = true;
        }
    }
    index
}

#[cfg(test)]
#[path = "../../tests/extension_module_navigation_perf.rs"]
mod performance_tests;

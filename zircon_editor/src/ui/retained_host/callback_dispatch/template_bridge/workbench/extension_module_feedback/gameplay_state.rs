use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.spawn_rules.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Spawn rules opened",
            output_text: "Native extension workspace opened for SpawnRules_Enemy",
        },
        "workbench.extension.spawn_rules.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Spawn simulation queued",
            output_text: "Simulation queued   Zone_A   96 spawns",
        },
        "workbench.extension.spawn_rules.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Spawn validation queued",
            output_text: "Validation queued   18 rules   1 conflict",
        },
        "workbench.extension.spawn_rules.condition_night_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
                status_text: "Spawn condition selected",
                output_text: "Selected Condition_Night   server authority",
            }
        }
        "workbench.extension.spawn_rules.conflict_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Spawn conflict selected",
            output_text: "Selected Conflict   tag filter warning",
        },
        "workbench.extension.world_state.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "World state opened",
            output_text: "Native extension workspace opened for Scenario_NightRaid",
        },
        "workbench.extension.world_state.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "World state simulation queued",
            output_text: "Simulation queued   Night Raid   42 events",
        },
        "workbench.extension.world_state.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "World state validation queued",
            output_text: "Validation queued   84 keys   1 conflict",
        },
        "workbench.extension.world_state.alarm_active_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionWorldStateOutputRow",
                status_text: "World state key selected",
                output_text: "Selected Alarm.Active   Global   true",
            }
        }
        "workbench.extension.world_state.quest_flag_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "World state conflict selected",
            output_text: "Selected Quest.Flag   Scenario conflict",
        },
        "workbench.extension.navmesh_ai.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Navmesh AI opened",
            output_text: "Native extension workspace opened for NavMesh_Main",
        },
        "workbench.extension.navmesh_ai.rebuild.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Navmesh rebuild queued",
            output_text: "Rebuild queued   18 tiles   Humanoid agent",
        },
        "workbench.extension.navmesh_ai.query_path.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Navmesh query queued",
            output_text: "Path query queued   Query_Patrol   4 agents",
        },
        "workbench.extension.navmesh_ai.tile_1208_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Navmesh tile selected",
            output_text: "Selected Tile 12_08   green   96 polys",
        },
        "workbench.extension.navmesh_ai.blocked_link_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Navmesh blocked link selected",
            output_text: "Selected Blocked Link   Door_A03 disabled",
        },
        _ => return None,
    };
    Some(feedback)
}

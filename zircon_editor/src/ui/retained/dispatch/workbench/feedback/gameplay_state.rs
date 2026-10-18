use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.spawn_rules.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.spawn_rules.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.spawn_rules.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.spawn_rules.condition_night_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.spawn_rules.conflict_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSpawnRulesOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.world_state.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.world_state.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.world_state.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.world_state.alarm_active_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionWorldStateOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.world_state.quest_flag_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWorldStateOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.navmesh_ai.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.navmesh_ai.rebuild.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.navmesh_ai.query_path.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.navmesh_ai.tile_1208_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.navmesh_ai.blocked_link_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionNavmeshAiOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        _ => return None,
    };
    Some(feedback)
}

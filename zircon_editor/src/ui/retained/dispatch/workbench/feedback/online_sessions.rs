use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.lobby_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.lobby_editor.simulate_lobby.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.lobby_editor.validate_lobby.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.lobby_editor.leader_slot_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.lobby_editor.crossplay_rule_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.matchmaking_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.matchmaking_editor.simulate_match.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.matchmaking_editor.validate_rules.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.matchmaking_editor.gold_bucket_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.matchmaking_editor.backfill_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        _ => return None,
    };
    Some(feedback)
}

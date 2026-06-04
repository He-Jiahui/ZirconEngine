use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.lobby_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
            status_text: "Lobby editor opened",
            output_text: "Native extension workspace opened for Lobby_Default",
        },
        "workbench.extension.lobby_editor.simulate_lobby.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
            status_text: "Lobby simulation queued",
            output_text: "Simulation queued   8 slots   4 players",
        },
        "workbench.extension.lobby_editor.validate_lobby.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
            status_text: "Lobby validation queued",
            output_text: "Validation queued   1 warning   crossplay enabled",
        },
        "workbench.extension.lobby_editor.leader_slot_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
                status_text: "Lobby leader slot selected",
                output_text: "Selected Leader   Player_01   Host",
            }
        }
        "workbench.extension.lobby_editor.crossplay_rule_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLobbyEditorOutputRow",
                status_text: "Lobby crossplay rule selected",
                output_text: "Selected Crossplay   Windows/Console   Warning",
            }
        }
        "workbench.extension.matchmaking_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
            status_text: "Matchmaking editor opened",
            output_text: "Native extension workspace opened for Ranked playlist",
        },
        "workbench.extension.matchmaking_editor.simulate_match.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
            status_text: "Matchmaking simulation queued",
            output_text: "Simulation queued   6 queues   128 players",
        },
        "workbench.extension.matchmaking_editor.validate_rules.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
            status_text: "Matchmaking validation queued",
            output_text: "Validation queued   2 warnings   latency rule active",
        },
        "workbench.extension.matchmaking_editor.gold_bucket_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
                status_text: "Matchmaking bucket selected",
                output_text: "Selected Gold bucket   64 players   48 ms",
            }
        }
        "workbench.extension.matchmaking_editor.backfill_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMatchmakingEditorOutputRow",
                status_text: "Backfill queue selected",
                output_text: "Selected Backfill queue   18 players   queued",
            }
        }
        _ => return None,
    };
    Some(feedback)
}

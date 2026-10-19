use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.save_data.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.save_data.save_slot.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.save_data.load_slot.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.save_data.player_state_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.save_data.debug_slot_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        _ => return None,
    };
    Some(feedback)
}

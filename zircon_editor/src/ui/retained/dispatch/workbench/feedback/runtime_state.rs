use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.save_data.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Save data opened",
            output_text: "Native extension workspace opened for AutoSave_01",
        },
        "workbench.extension.save_data.save_slot.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Save slot queued",
            output_text: "Save queued   AutoSave_01   local sample only",
        },
        "workbench.extension.save_data.load_slot.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Load slot queued",
            output_text: "Load queued   AutoSave_01   schema v4",
        },
        "workbench.extension.save_data.player_state_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Save object selected",
            output_text: "Selected PlayerState   Level 12   health changed",
        },
        "workbench.extension.save_data.debug_slot_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSaveDataOutputRow",
            status_text: "Save warning selected",
            output_text: "Selected DebugSlot   old schema   repair suggested",
        },
        _ => return None,
    };
    Some(feedback)
}

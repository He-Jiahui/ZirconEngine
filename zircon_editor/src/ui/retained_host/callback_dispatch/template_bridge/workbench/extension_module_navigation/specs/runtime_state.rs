use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const SAVE_DATA_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionSaveDataAutoSaveRow",
    "WorkbenchExtensionSaveDataManualRow",
    "WorkbenchExtensionSaveDataCloudRow",
    "WorkbenchExtensionSaveDataPlayerStateRow",
    "WorkbenchExtensionSaveDataInventoryRow",
    "WorkbenchExtensionSaveDataQuestLogRow",
    "WorkbenchExtensionSaveDataDebugSlotRow",
];
const SAVE_DATA_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.save_data.auto_save_row.select",
        "WorkbenchExtensionSaveDataAutoSaveRow",
    ),
    action(
        "workbench.extension.save_data.manual_row.select",
        "WorkbenchExtensionSaveDataManualRow",
    ),
    action(
        "workbench.extension.save_data.cloud_row.select",
        "WorkbenchExtensionSaveDataCloudRow",
    ),
    action(
        "workbench.extension.save_data.player_state_row.select",
        "WorkbenchExtensionSaveDataPlayerStateRow",
    ),
    action(
        "workbench.extension.save_data.inventory_row.select",
        "WorkbenchExtensionSaveDataInventoryRow",
    ),
    action(
        "workbench.extension.save_data.quest_log_row.select",
        "WorkbenchExtensionSaveDataQuestLogRow",
    ),
    action(
        "workbench.extension.save_data.debug_slot_row.select",
        "WorkbenchExtensionSaveDataDebugSlotRow",
    ),
];
const SAVE_DATA_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsProductionToolsMenu",
    "WorkbenchExtensionSaveDataSaveSlotButton",
    "WorkbenchExtensionSaveDataLoadSlotButton",
];
const SAVE_DATA_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.save_data.open",
        "WorkbenchAssetsProductionToolsMenu",
    ),
    action(
        "workbench.extension.save_data.save_slot.invoke",
        "WorkbenchExtensionSaveDataSaveSlotButton",
    ),
    action(
        "workbench.extension.save_data.load_slot.invoke",
        "WorkbenchExtensionSaveDataLoadSlotButton",
    ),
];
const SAVE_DATA_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.save_data.schema.edit",
    "workbench.extension.save_data.schema.commit",
    "workbench.extension.save_data.slot.edit",
    "workbench.extension.save_data.slot.commit",
    "workbench.extension.save_data.compression.edit",
    "workbench.extension.save_data.compression.commit",
];

pub(super) const SAVE_DATA_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.save_data.open",
    "WorkbenchExtensionSaveDataWorkspace",
    SAVE_DATA_ROW_CONTROLS,
    SAVE_DATA_ROW_ACTIONS,
    SAVE_DATA_COMMAND_CONTROLS,
    SAVE_DATA_COMMAND_ACTIONS,
    SAVE_DATA_FIELD_ACTIONS,
);

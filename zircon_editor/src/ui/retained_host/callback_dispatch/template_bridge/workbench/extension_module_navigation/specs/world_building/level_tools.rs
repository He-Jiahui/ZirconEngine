use super::{action, spec, ActionControl, ExtensionNavigationSpec};

const LEVEL_STREAMING_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLevelStreamingCellsTab",
    "WorkbenchExtensionLevelStreamingRulesTab",
    "WorkbenchExtensionLevelStreamingEventsTab",
];
const LEVEL_STREAMING_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.level_streaming.cells_tab.select",
        "WorkbenchExtensionLevelStreamingCellsTab",
    ),
    action(
        "workbench.extension.level_streaming.rules_tab.select",
        "WorkbenchExtensionLevelStreamingRulesTab",
    ),
    action(
        "workbench.extension.level_streaming.events_tab.select",
        "WorkbenchExtensionLevelStreamingEventsTab",
    ),
];
const LEVEL_STREAMING_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLevelStreamingWorldRow",
    "WorkbenchExtensionLevelStreamingCellA12Row",
    "WorkbenchExtensionLevelStreamingRulePlayerDistanceRow",
    "WorkbenchExtensionLevelStreamingCellA12TableRow",
    "WorkbenchExtensionLevelStreamingCellA13TableRow",
    "WorkbenchExtensionLevelStreamingHlod04TableRow",
    "WorkbenchExtensionLevelStreamingCellB12TableRow",
    "WorkbenchExtensionLevelStreamingOutputRow",
];
const LEVEL_STREAMING_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.level_streaming.world_row.select",
        "WorkbenchExtensionLevelStreamingWorldRow",
    ),
    action(
        "workbench.extension.level_streaming.cell_a_12_row.select",
        "WorkbenchExtensionLevelStreamingCellA12Row",
    ),
    action(
        "workbench.extension.level_streaming.rule_player_distance_row.select",
        "WorkbenchExtensionLevelStreamingRulePlayerDistanceRow",
    ),
    action(
        "workbench.extension.level_streaming.cell_a_12_table_row.select",
        "WorkbenchExtensionLevelStreamingCellA12TableRow",
    ),
    action(
        "workbench.extension.level_streaming.cell_a_13_table_row.select",
        "WorkbenchExtensionLevelStreamingCellA13TableRow",
    ),
    action(
        "workbench.extension.level_streaming.hlod_04_table_row.select",
        "WorkbenchExtensionLevelStreamingHlod04TableRow",
    ),
    action(
        "workbench.extension.level_streaming.cell_b_12_table_row.select",
        "WorkbenchExtensionLevelStreamingCellB12TableRow",
    ),
    action(
        "workbench.extension.level_streaming.output.select",
        "WorkbenchExtensionLevelStreamingOutputRow",
    ),
];
const LEVEL_STREAMING_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsLevelStreamingButton",
    "WorkbenchExtensionLevelStreamingPreviewButton",
    "WorkbenchExtensionLevelStreamingLoadButton",
];
const LEVEL_STREAMING_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.level_streaming.open",
        "WorkbenchAssetsLevelStreamingButton",
    ),
    action(
        "workbench.extension.level_streaming.preview.invoke",
        "WorkbenchExtensionLevelStreamingPreviewButton",
    ),
    action(
        "workbench.extension.level_streaming.load.invoke",
        "WorkbenchExtensionLevelStreamingLoadButton",
    ),
];
const LEVEL_STREAMING_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.level_streaming.cell.edit",
    "workbench.extension.level_streaming.cell.commit",
    "workbench.extension.level_streaming.rule.edit",
    "workbench.extension.level_streaming.rule.commit",
    "workbench.extension.level_streaming.distance.edit",
    "workbench.extension.level_streaming.distance.commit",
];

pub(in super::super) const LEVEL_STREAMING_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.level_streaming.open",
    "WorkbenchExtensionLevelStreamingWorkspace",
    LEVEL_STREAMING_TAB_CONTROLS,
    LEVEL_STREAMING_TAB_ACTIONS,
    LEVEL_STREAMING_ROW_CONTROLS,
    LEVEL_STREAMING_ROW_ACTIONS,
    LEVEL_STREAMING_COMMAND_CONTROLS,
    LEVEL_STREAMING_COMMAND_ACTIONS,
    LEVEL_STREAMING_FIELD_ACTIONS,
);

const LEVEL_VARIANT_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLevelVariantVariantTab",
    "WorkbenchExtensionLevelVariantOverridesTab",
    "WorkbenchExtensionLevelVariantDiffTab",
];
const LEVEL_VARIANT_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.level_variant.variant_tab.select",
        "WorkbenchExtensionLevelVariantVariantTab",
    ),
    action(
        "workbench.extension.level_variant.overrides_tab.select",
        "WorkbenchExtensionLevelVariantOverridesTab",
    ),
    action(
        "workbench.extension.level_variant.diff_tab.select",
        "WorkbenchExtensionLevelVariantDiffTab",
    ),
];
const LEVEL_VARIANT_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLevelVariantSetRow",
    "WorkbenchExtensionLevelVariantRedRow",
    "WorkbenchExtensionLevelVariantActorOverrideRow",
    "WorkbenchExtensionLevelVariantCarBodyRow",
    "WorkbenchExtensionLevelVariantWheelFlRow",
    "WorkbenchExtensionLevelVariantLightRigRow",
    "WorkbenchExtensionLevelVariantDoorLRow",
    "WorkbenchExtensionLevelVariantOutputRow",
];
const LEVEL_VARIANT_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.level_variant.set_row.select",
        "WorkbenchExtensionLevelVariantSetRow",
    ),
    action(
        "workbench.extension.level_variant.red_row.select",
        "WorkbenchExtensionLevelVariantRedRow",
    ),
    action(
        "workbench.extension.level_variant.actor_override_row.select",
        "WorkbenchExtensionLevelVariantActorOverrideRow",
    ),
    action(
        "workbench.extension.level_variant.car_body_table_row.select",
        "WorkbenchExtensionLevelVariantCarBodyRow",
    ),
    action(
        "workbench.extension.level_variant.wheel_fl_table_row.select",
        "WorkbenchExtensionLevelVariantWheelFlRow",
    ),
    action(
        "workbench.extension.level_variant.light_rig_table_row.select",
        "WorkbenchExtensionLevelVariantLightRigRow",
    ),
    action(
        "workbench.extension.level_variant.door_l_table_row.select",
        "WorkbenchExtensionLevelVariantDoorLRow",
    ),
    action(
        "workbench.extension.level_variant.output.select",
        "WorkbenchExtensionLevelVariantOutputRow",
    ),
];
const LEVEL_VARIANT_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsLevelVariantButton",
    "WorkbenchExtensionLevelVariantPreviewButton",
    "WorkbenchExtensionLevelVariantApplyButton",
];
const LEVEL_VARIANT_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.level_variant.open",
        "WorkbenchAssetsLevelVariantButton",
    ),
    action(
        "workbench.extension.level_variant.preview.invoke",
        "WorkbenchExtensionLevelVariantPreviewButton",
    ),
    action(
        "workbench.extension.level_variant.apply.invoke",
        "WorkbenchExtensionLevelVariantApplyButton",
    ),
];
const LEVEL_VARIANT_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.level_variant.variant.edit",
    "workbench.extension.level_variant.variant.commit",
    "workbench.extension.level_variant.set.edit",
    "workbench.extension.level_variant.set.commit",
    "workbench.extension.level_variant.capture.edit",
    "workbench.extension.level_variant.capture.commit",
];

pub(in super::super) const LEVEL_VARIANT_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.level_variant.open",
    "WorkbenchExtensionLevelVariantWorkspace",
    LEVEL_VARIANT_TAB_CONTROLS,
    LEVEL_VARIANT_TAB_ACTIONS,
    LEVEL_VARIANT_ROW_CONTROLS,
    LEVEL_VARIANT_ROW_ACTIONS,
    LEVEL_VARIANT_COMMAND_CONTROLS,
    LEVEL_VARIANT_COMMAND_ACTIONS,
    LEVEL_VARIANT_FIELD_ACTIONS,
);

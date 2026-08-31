use super::{action, spec, ActionControl, ExtensionNavigationSpec};

const VOLUME_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionVolumeEditorDamageZoneRow",
    "WorkbenchExtensionVolumeEditorProfileDefaultRow",
    "WorkbenchExtensionVolumeEditorOverlapPlayerRow",
    "WorkbenchExtensionVolumeEditorBoundsTableRow",
    "WorkbenchExtensionVolumeEditorPlayerOverlapTableRow",
    "WorkbenchExtensionVolumeEditorDamageRuleTableRow",
    "WorkbenchExtensionVolumeEditorOnEnterEventTableRow",
];
const VOLUME_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.volume_editor.damage_zone_row.select",
        "WorkbenchExtensionVolumeEditorDamageZoneRow",
    ),
    action(
        "workbench.extension.volume_editor.profile_default_row.select",
        "WorkbenchExtensionVolumeEditorProfileDefaultRow",
    ),
    action(
        "workbench.extension.volume_editor.overlap_player_row.select",
        "WorkbenchExtensionVolumeEditorOverlapPlayerRow",
    ),
    action(
        "workbench.extension.volume_editor.bounds_table_row.select",
        "WorkbenchExtensionVolumeEditorBoundsTableRow",
    ),
    action(
        "workbench.extension.volume_editor.player_overlap_table_row.select",
        "WorkbenchExtensionVolumeEditorPlayerOverlapTableRow",
    ),
    action(
        "workbench.extension.volume_editor.damage_rule_table_row.select",
        "WorkbenchExtensionVolumeEditorDamageRuleTableRow",
    ),
    action(
        "workbench.extension.volume_editor.on_enter_event_table_row.select",
        "WorkbenchExtensionVolumeEditorOnEnterEventTableRow",
    ),
];
const VOLUME_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsWorldToolsMenu",
    "WorkbenchExtensionVolumeEditorInspectButton",
    "WorkbenchExtensionVolumeEditorValidateButton",
];
const VOLUME_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.volume_editor.open",
        "WorkbenchAssetsWorldToolsMenu",
    ),
    action(
        "workbench.extension.volume_editor.inspect.invoke",
        "WorkbenchExtensionVolumeEditorInspectButton",
    ),
    action(
        "workbench.extension.volume_editor.validate.invoke",
        "WorkbenchExtensionVolumeEditorValidateButton",
    ),
];
const VOLUME_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.volume_editor.volume.edit",
    "workbench.extension.volume_editor.volume.commit",
    "workbench.extension.volume_editor.profile.edit",
    "workbench.extension.volume_editor.profile.commit",
    "workbench.extension.volume_editor.priority.edit",
    "workbench.extension.volume_editor.priority.commit",
];

pub(in super::super) const VOLUME_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.volume_editor.open",
    "WorkbenchExtensionVolumeEditorWorkspace",
    VOLUME_EDITOR_ROW_CONTROLS,
    VOLUME_EDITOR_ROW_ACTIONS,
    VOLUME_EDITOR_COMMAND_CONTROLS,
    VOLUME_EDITOR_COMMAND_ACTIONS,
    VOLUME_EDITOR_FIELD_ACTIONS,
);

const WEATHER_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionWeatherEditorStormRow",
    "WorkbenchExtensionWeatherEditorMountainsRegionRow",
    "WorkbenchExtensionWeatherEditorCloudLayerRow",
    "WorkbenchExtensionWeatherEditorCloudBuildTimelineRow",
    "WorkbenchExtensionWeatherEditorRainBurstTimelineRow",
    "WorkbenchExtensionWeatherEditorWindGustTimelineRow",
    "WorkbenchExtensionWeatherEditorLightningTimelineRow",
];
const WEATHER_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.weather_editor.storm_row.select",
        "WorkbenchExtensionWeatherEditorStormRow",
    ),
    action(
        "workbench.extension.weather_editor.mountains_region_row.select",
        "WorkbenchExtensionWeatherEditorMountainsRegionRow",
    ),
    action(
        "workbench.extension.weather_editor.cloud_layer_row.select",
        "WorkbenchExtensionWeatherEditorCloudLayerRow",
    ),
    action(
        "workbench.extension.weather_editor.cloud_build_timeline_row.select",
        "WorkbenchExtensionWeatherEditorCloudBuildTimelineRow",
    ),
    action(
        "workbench.extension.weather_editor.rain_burst_timeline_row.select",
        "WorkbenchExtensionWeatherEditorRainBurstTimelineRow",
    ),
    action(
        "workbench.extension.weather_editor.wind_gust_timeline_row.select",
        "WorkbenchExtensionWeatherEditorWindGustTimelineRow",
    ),
    action(
        "workbench.extension.weather_editor.lightning_timeline_row.select",
        "WorkbenchExtensionWeatherEditorLightningTimelineRow",
    ),
];
const WEATHER_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsWorldToolsMenu",
    "WorkbenchExtensionWeatherEditorPreviewButton",
    "WorkbenchExtensionWeatherEditorBuildButton",
];
const WEATHER_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.weather_editor.open",
        "WorkbenchAssetsWorldToolsMenu",
    ),
    action(
        "workbench.extension.weather_editor.preview.invoke",
        "WorkbenchExtensionWeatherEditorPreviewButton",
    ),
    action(
        "workbench.extension.weather_editor.build.invoke",
        "WorkbenchExtensionWeatherEditorBuildButton",
    ),
];
const WEATHER_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.weather_editor.preset.edit",
    "workbench.extension.weather_editor.preset.commit",
    "workbench.extension.weather_editor.region.edit",
    "workbench.extension.weather_editor.region.commit",
    "workbench.extension.weather_editor.blend_time.edit",
    "workbench.extension.weather_editor.blend_time.commit",
];

pub(in super::super) const WEATHER_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.weather_editor.open",
    "WorkbenchExtensionWeatherEditorWorkspace",
    WEATHER_EDITOR_ROW_CONTROLS,
    WEATHER_EDITOR_ROW_ACTIONS,
    WEATHER_EDITOR_COMMAND_CONTROLS,
    WEATHER_EDITOR_COMMAND_ACTIONS,
    WEATHER_EDITOR_FIELD_ACTIONS,
);

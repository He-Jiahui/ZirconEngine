use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const DATA_TABLE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionDataTableItemsRow",
    "WorkbenchExtensionDataTableSchemaRow",
    "WorkbenchExtensionDataTableLocalizationRow",
    "WorkbenchExtensionDataTablePotionRow",
    "WorkbenchExtensionDataTableSwordRow",
    "WorkbenchExtensionDataTableArmorRow",
    "WorkbenchExtensionDataTableDebugRow",
];
const DATA_TABLE_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.data_table.items_row.select",
        "WorkbenchExtensionDataTableItemsRow",
    ),
    action(
        "workbench.extension.data_table.schema_row.select",
        "WorkbenchExtensionDataTableSchemaRow",
    ),
    action(
        "workbench.extension.data_table.localization_row.select",
        "WorkbenchExtensionDataTableLocalizationRow",
    ),
    action(
        "workbench.extension.data_table.potion_row.select",
        "WorkbenchExtensionDataTablePotionRow",
    ),
    action(
        "workbench.extension.data_table.sword_row.select",
        "WorkbenchExtensionDataTableSwordRow",
    ),
    action(
        "workbench.extension.data_table.armor_row.select",
        "WorkbenchExtensionDataTableArmorRow",
    ),
    action(
        "workbench.extension.data_table.debug_row.select",
        "WorkbenchExtensionDataTableDebugRow",
    ),
];
const DATA_TABLE_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsProductionToolsMenu",
    "WorkbenchExtensionDataTableValidateButton",
    "WorkbenchExtensionDataTableSaveButton",
];
const DATA_TABLE_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.data_table.open",
        "WorkbenchAssetsProductionToolsMenu",
    ),
    action(
        "workbench.extension.data_table.validate.invoke",
        "WorkbenchExtensionDataTableValidateButton",
    ),
    action(
        "workbench.extension.data_table.save.invoke",
        "WorkbenchExtensionDataTableSaveButton",
    ),
];
const DATA_TABLE_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.data_table.row.edit",
    "workbench.extension.data_table.row.commit",
    "workbench.extension.data_table.type.edit",
    "workbench.extension.data_table.type.commit",
    "workbench.extension.data_table.version.edit",
    "workbench.extension.data_table.version.commit",
];

pub(super) const DATA_TABLE_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.data_table.open",
    "WorkbenchExtensionDataTableWorkspace",
    DATA_TABLE_ROW_CONTROLS,
    DATA_TABLE_ROW_ACTIONS,
    DATA_TABLE_COMMAND_CONTROLS,
    DATA_TABLE_COMMAND_ACTIONS,
    DATA_TABLE_FIELD_ACTIONS,
);
const SOURCE_CONTROL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionSourceControlChangelistRow",
    "WorkbenchExtensionSourceControlRuntimeRow",
    "WorkbenchExtensionSourceControlEditorRow",
    "WorkbenchExtensionSourceControlRuntimeFileRow",
    "WorkbenchExtensionSourceControlEditorFileRow",
    "WorkbenchExtensionSourceControlDocsFileRow",
    "WorkbenchExtensionSourceControlConflictFileRow",
];
const SOURCE_CONTROL_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.source_control.changelist_row.select",
        "WorkbenchExtensionSourceControlChangelistRow",
    ),
    action(
        "workbench.extension.source_control.runtime_row.select",
        "WorkbenchExtensionSourceControlRuntimeRow",
    ),
    action(
        "workbench.extension.source_control.editor_row.select",
        "WorkbenchExtensionSourceControlEditorRow",
    ),
    action(
        "workbench.extension.source_control.runtime_file_row.select",
        "WorkbenchExtensionSourceControlRuntimeFileRow",
    ),
    action(
        "workbench.extension.source_control.editor_file_row.select",
        "WorkbenchExtensionSourceControlEditorFileRow",
    ),
    action(
        "workbench.extension.source_control.docs_file_row.select",
        "WorkbenchExtensionSourceControlDocsFileRow",
    ),
    action(
        "workbench.extension.source_control.conflict_file_row.select",
        "WorkbenchExtensionSourceControlConflictFileRow",
    ),
];
const SOURCE_CONTROL_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsProductionToolsMenu",
    "WorkbenchExtensionSourceControlValidateButton",
    "WorkbenchExtensionSourceControlSubmitButton",
];
const SOURCE_CONTROL_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.source_control.open",
        "WorkbenchAssetsProductionToolsMenu",
    ),
    action(
        "workbench.extension.source_control.validate.invoke",
        "WorkbenchExtensionSourceControlValidateButton",
    ),
    action(
        "workbench.extension.source_control.submit.invoke",
        "WorkbenchExtensionSourceControlSubmitButton",
    ),
];
const SOURCE_CONTROL_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.source_control.changelist.edit",
    "workbench.extension.source_control.changelist.commit",
    "workbench.extension.source_control.owner.edit",
    "workbench.extension.source_control.owner.commit",
    "workbench.extension.source_control.gate.edit",
    "workbench.extension.source_control.gate.commit",
];

pub(super) const SOURCE_CONTROL_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.source_control.open",
    "WorkbenchExtensionSourceControlWorkspace",
    SOURCE_CONTROL_ROW_CONTROLS,
    SOURCE_CONTROL_ROW_ACTIONS,
    SOURCE_CONTROL_COMMAND_CONTROLS,
    SOURCE_CONTROL_COMMAND_ACTIONS,
    SOURCE_CONTROL_FIELD_ACTIONS,
);
const BUILD_EXPORT_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionBuildExportShippingRow",
    "WorkbenchExtensionBuildExportDevelopmentRow",
    "WorkbenchExtensionBuildExportPatchRow",
    "WorkbenchExtensionBuildExportCookStepRow",
    "WorkbenchExtensionBuildExportPakStepRow",
    "WorkbenchExtensionBuildExportSignStepRow",
    "WorkbenchExtensionBuildExportPublishStepRow",
];
const BUILD_EXPORT_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.build_export.shipping_row.select",
        "WorkbenchExtensionBuildExportShippingRow",
    ),
    action(
        "workbench.extension.build_export.development_row.select",
        "WorkbenchExtensionBuildExportDevelopmentRow",
    ),
    action(
        "workbench.extension.build_export.patch_row.select",
        "WorkbenchExtensionBuildExportPatchRow",
    ),
    action(
        "workbench.extension.build_export.cook_step_row.select",
        "WorkbenchExtensionBuildExportCookStepRow",
    ),
    action(
        "workbench.extension.build_export.pak_step_row.select",
        "WorkbenchExtensionBuildExportPakStepRow",
    ),
    action(
        "workbench.extension.build_export.sign_step_row.select",
        "WorkbenchExtensionBuildExportSignStepRow",
    ),
    action(
        "workbench.extension.build_export.publish_step_row.select",
        "WorkbenchExtensionBuildExportPublishStepRow",
    ),
];
const BUILD_EXPORT_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsProductionToolsMenu",
    "WorkbenchExtensionBuildExportValidateButton",
    "WorkbenchExtensionBuildExportPackageButton",
];
const BUILD_EXPORT_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.build_export.open",
        "WorkbenchAssetsProductionToolsMenu",
    ),
    action(
        "workbench.extension.build_export.validate.invoke",
        "WorkbenchExtensionBuildExportValidateButton",
    ),
    action(
        "workbench.extension.build_export.package.invoke",
        "WorkbenchExtensionBuildExportPackageButton",
    ),
];
const BUILD_EXPORT_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.build_export.platform.edit",
    "workbench.extension.build_export.platform.commit",
    "workbench.extension.build_export.channel.edit",
    "workbench.extension.build_export.channel.commit",
    "workbench.extension.build_export.compression.edit",
    "workbench.extension.build_export.compression.commit",
];

pub(super) const BUILD_EXPORT_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.build_export.open",
    "WorkbenchExtensionBuildExportWorkspace",
    BUILD_EXPORT_ROW_CONTROLS,
    BUILD_EXPORT_ROW_ACTIONS,
    BUILD_EXPORT_COMMAND_CONTROLS,
    BUILD_EXPORT_COMMAND_ACTIONS,
    BUILD_EXPORT_FIELD_ACTIONS,
);
const AUTOMATION_REPORT_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionAutomationReportSmokeRow",
    "WorkbenchExtensionAutomationReportRenderingRow",
    "WorkbenchExtensionAutomationReportGameplayRow",
    "WorkbenchExtensionAutomationReportRendererTestRow",
    "WorkbenchExtensionAutomationReportGameplayTestRow",
    "WorkbenchExtensionAutomationReportAssetTestRow",
    "WorkbenchExtensionAutomationReportUiFailureRow",
];
const AUTOMATION_REPORT_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.automation_report.smoke_row.select",
        "WorkbenchExtensionAutomationReportSmokeRow",
    ),
    action(
        "workbench.extension.automation_report.rendering_row.select",
        "WorkbenchExtensionAutomationReportRenderingRow",
    ),
    action(
        "workbench.extension.automation_report.gameplay_row.select",
        "WorkbenchExtensionAutomationReportGameplayRow",
    ),
    action(
        "workbench.extension.automation_report.renderer_test_row.select",
        "WorkbenchExtensionAutomationReportRendererTestRow",
    ),
    action(
        "workbench.extension.automation_report.gameplay_test_row.select",
        "WorkbenchExtensionAutomationReportGameplayTestRow",
    ),
    action(
        "workbench.extension.automation_report.asset_test_row.select",
        "WorkbenchExtensionAutomationReportAssetTestRow",
    ),
    action(
        "workbench.extension.automation_report.ui_failure_row.select",
        "WorkbenchExtensionAutomationReportUiFailureRow",
    ),
];
const AUTOMATION_REPORT_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsProductionToolsMenu",
    "WorkbenchExtensionAutomationReportValidateButton",
    "WorkbenchExtensionAutomationReportPublishButton",
];
const AUTOMATION_REPORT_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.automation_report.open",
        "WorkbenchAssetsProductionToolsMenu",
    ),
    action(
        "workbench.extension.automation_report.validate.invoke",
        "WorkbenchExtensionAutomationReportValidateButton",
    ),
    action(
        "workbench.extension.automation_report.publish.invoke",
        "WorkbenchExtensionAutomationReportPublishButton",
    ),
];
const AUTOMATION_REPORT_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.automation_report.suite.edit",
    "workbench.extension.automation_report.suite.commit",
    "workbench.extension.automation_report.platform.edit",
    "workbench.extension.automation_report.platform.commit",
    "workbench.extension.automation_report.retry.edit",
    "workbench.extension.automation_report.retry.commit",
];

pub(super) const AUTOMATION_REPORT_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.automation_report.open",
    "WorkbenchExtensionAutomationReportWorkspace",
    AUTOMATION_REPORT_ROW_CONTROLS,
    AUTOMATION_REPORT_ROW_ACTIONS,
    AUTOMATION_REPORT_COMMAND_CONTROLS,
    AUTOMATION_REPORT_COMMAND_ACTIONS,
    AUTOMATION_REPORT_FIELD_ACTIONS,
);
const PROJECT_OVERVIEW_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionProjectOverviewProjectRow",
    "WorkbenchExtensionProjectOverviewMilestoneRow",
    "WorkbenchExtensionProjectOverviewRiskRow",
    "WorkbenchExtensionProjectOverviewHealthRow",
    "WorkbenchExtensionProjectOverviewBuildRow",
    "WorkbenchExtensionProjectOverviewCoverageRow",
    "WorkbenchExtensionProjectOverviewDependencyRow",
];
const PROJECT_OVERVIEW_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.project_overview.project_row.select",
        "WorkbenchExtensionProjectOverviewProjectRow",
    ),
    action(
        "workbench.extension.project_overview.milestone_row.select",
        "WorkbenchExtensionProjectOverviewMilestoneRow",
    ),
    action(
        "workbench.extension.project_overview.risk_row.select",
        "WorkbenchExtensionProjectOverviewRiskRow",
    ),
    action(
        "workbench.extension.project_overview.health_row.select",
        "WorkbenchExtensionProjectOverviewHealthRow",
    ),
    action(
        "workbench.extension.project_overview.build_row.select",
        "WorkbenchExtensionProjectOverviewBuildRow",
    ),
    action(
        "workbench.extension.project_overview.coverage_row.select",
        "WorkbenchExtensionProjectOverviewCoverageRow",
    ),
    action(
        "workbench.extension.project_overview.dependency_row.select",
        "WorkbenchExtensionProjectOverviewDependencyRow",
    ),
];
const PROJECT_OVERVIEW_COMMAND_CONTROLS: &[&str] = &["WorkbenchAssetsProductionToolsMenu"];
const PROJECT_OVERVIEW_COMMAND_ACTIONS: &[ActionControl] = &[action(
    "workbench.extension.project_overview.open",
    "WorkbenchAssetsProductionToolsMenu",
)];
const PROJECT_OVERVIEW_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.project_overview.owner.edit",
    "workbench.extension.project_overview.owner.commit",
    "workbench.extension.project_overview.channel.edit",
    "workbench.extension.project_overview.channel.commit",
    "workbench.extension.project_overview.health.edit",
    "workbench.extension.project_overview.health.commit",
];

pub(super) const PROJECT_OVERVIEW_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.project_overview.open",
    "WorkbenchExtensionProjectOverviewWorkspace",
    PROJECT_OVERVIEW_ROW_CONTROLS,
    PROJECT_OVERVIEW_ROW_ACTIONS,
    PROJECT_OVERVIEW_COMMAND_CONTROLS,
    PROJECT_OVERVIEW_COMMAND_ACTIONS,
    PROJECT_OVERVIEW_FIELD_ACTIONS,
);

const PLUGIN_MANAGER_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPluginManagerAudioRow",
    "WorkbenchExtensionPluginManagerRenderdocRow",
    "WorkbenchExtensionPluginManagerGameplayRow",
    "WorkbenchExtensionPluginManagerAudioRuntimeTableRow",
    "WorkbenchExtensionPluginManagerRenderdocBridgeTableRow",
    "WorkbenchExtensionPluginManagerGameplayPackTableRow",
    "WorkbenchExtensionPluginManagerVersionWarningTableRow",
];
const PLUGIN_MANAGER_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.plugin_manager.audio_row.select",
        "WorkbenchExtensionPluginManagerAudioRow",
    ),
    action(
        "workbench.extension.plugin_manager.renderdoc_row.select",
        "WorkbenchExtensionPluginManagerRenderdocRow",
    ),
    action(
        "workbench.extension.plugin_manager.gameplay_row.select",
        "WorkbenchExtensionPluginManagerGameplayRow",
    ),
    action(
        "workbench.extension.plugin_manager.audio_runtime_table_row.select",
        "WorkbenchExtensionPluginManagerAudioRuntimeTableRow",
    ),
    action(
        "workbench.extension.plugin_manager.renderdoc_bridge_table_row.select",
        "WorkbenchExtensionPluginManagerRenderdocBridgeTableRow",
    ),
    action(
        "workbench.extension.plugin_manager.gameplay_pack_table_row.select",
        "WorkbenchExtensionPluginManagerGameplayPackTableRow",
    ),
    action(
        "workbench.extension.plugin_manager.version_warning_table_row.select",
        "WorkbenchExtensionPluginManagerVersionWarningTableRow",
    ),
];
const PLUGIN_MANAGER_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsProductionToolsMenu",
    "WorkbenchExtensionPluginManagerHotReloadButton",
    "WorkbenchExtensionPluginManagerValidateButton",
];
const PLUGIN_MANAGER_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.plugin_manager.open",
        "WorkbenchAssetsProductionToolsMenu",
    ),
    action(
        "workbench.extension.plugin_manager.hot_reload.invoke",
        "WorkbenchExtensionPluginManagerHotReloadButton",
    ),
    action(
        "workbench.extension.plugin_manager.validate.invoke",
        "WorkbenchExtensionPluginManagerValidateButton",
    ),
];
const PLUGIN_MANAGER_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.plugin_manager.plugin.edit",
    "workbench.extension.plugin_manager.plugin.commit",
    "workbench.extension.plugin_manager.channel.edit",
    "workbench.extension.plugin_manager.channel.commit",
    "workbench.extension.plugin_manager.version.edit",
    "workbench.extension.plugin_manager.version.commit",
];

pub(super) const PLUGIN_MANAGER_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.plugin_manager.open",
    "WorkbenchExtensionPluginManagerWorkspace",
    PLUGIN_MANAGER_ROW_CONTROLS,
    PLUGIN_MANAGER_ROW_ACTIONS,
    PLUGIN_MANAGER_COMMAND_CONTROLS,
    PLUGIN_MANAGER_COMMAND_ACTIONS,
    PLUGIN_MANAGER_FIELD_ACTIONS,
);

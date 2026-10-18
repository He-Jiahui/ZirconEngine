use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.data_table.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.data_table.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.data_table.save.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.data_table.potion_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.data_table.debug_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.source_control.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.source_control.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.source_control.submit.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.source_control.runtime_file_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.source_control.conflict_file_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.build_export.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.build_export.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.build_export.package.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.build_export.cook_step_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.build_export.publish_step_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.automation_report.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.automation_report.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.automation_report.publish.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.automation_report.renderer_test_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.automation_report.ui_failure_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.project_overview.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.project_overview.health_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.project_overview.dependency_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.plugin_manager.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.plugin_manager.hot_reload.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.plugin_manager.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.plugin_manager.audio_runtime_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.plugin_manager.version_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        _ => return None,
    };
    Some(feedback)
}

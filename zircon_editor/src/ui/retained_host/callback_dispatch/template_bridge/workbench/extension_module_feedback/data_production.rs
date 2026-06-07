use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.data_table.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Data table opened",
            output_text: "Native extension workspace opened for DT_Items",
        },
        "workbench.extension.data_table.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Data table validation queued",
            output_text: "Validation queued   128 rows   2 warnings",
        },
        "workbench.extension.data_table.save.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Data table save queued",
            output_text: "Save queued   DT_Items   version 12",
        },
        "workbench.extension.data_table.potion_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Data table row selected",
            output_text: "Selected Potion_Health   Consumable   +50 HP",
        },
        "workbench.extension.data_table.debug_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionDataTableOutputRow",
            status_text: "Data table warning row selected",
            output_text: "Selected Debug_Item   Missing Icon   Warning",
        },
        "workbench.extension.source_control.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Source control opened",
            output_text: "Native extension workspace opened for CL_2048",
        },
        "workbench.extension.source_control.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Source control validation queued",
            output_text: "Validation queued   18 files   6 checks",
        },
        "workbench.extension.source_control.submit.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Source control submit queued",
            output_text: "Submit queued   CL_2048   2 conflicts",
        },
        "workbench.extension.source_control.runtime_file_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Source control file selected",
            output_text: "Selected runtime/ui/render.rs   Modified   Alice",
        },
        "workbench.extension.source_control.conflict_file_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSourceControlOutputRow",
            status_text: "Source control conflict selected",
            output_text: "Selected asset/import.rs   Conflict   Chen",
        },
        "workbench.extension.build_export.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Build export opened",
            output_text: "Native extension workspace opened for Win64 Shipping",
        },
        "workbench.extension.build_export.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Build export validation queued",
            output_text: "Validation queued   Win64 Shipping   release channel",
        },
        "workbench.extension.build_export.package.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Build export package queued",
            output_text: "Package queued   Cook Content   62 percent",
        },
        "workbench.extension.build_export.cook_step_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Build export cook step selected",
            output_text: "Selected Cook Content   Running   62 percent",
        },
        "workbench.extension.build_export.publish_step_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBuildExportOutputRow",
            status_text: "Build export publish step selected",
            output_text: "Selected Publish Build   CDN   Pending",
        },
        "workbench.extension.automation_report.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Automation report opened",
            output_text: "Native extension workspace opened for Rendering suite",
        },
        "workbench.extension.automation_report.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Automation validation queued",
            output_text: "Validation queued   Rendering suite   642 tests",
        },
        "workbench.extension.automation_report.publish.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Automation report publish queued",
            output_text: "Publish queued   7 failures   3 flakes",
        },
        "workbench.extension.automation_report.renderer_test_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
                status_text: "Automation renderer test selected",
                output_text: "Selected Renderer.Smoke   Running   Worker_03",
            }
        }
        "workbench.extension.automation_report.ui_failure_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAutomationReportOutputRow",
            status_text: "Automation failure selected",
            output_text: "Selected UI.Layout   Failed   Screenshot diff",
        },
        "workbench.extension.project_overview.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Project overview opened",
            output_text: "Native extension workspace opened for NebulaGame",
        },
        "workbench.extension.project_overview.refresh.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Project overview refreshed",
            output_text: "Refresh queued   health board   7 tasks",
        },
        "workbench.extension.project_overview.publish.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Project overview publish queued",
            output_text: "Publish queued   Development channel   Healthy",
        },
        "workbench.extension.project_overview.health_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Project health selected",
            output_text: "Selected Project Health   Healthy   All systems nominal",
        },
        "workbench.extension.project_overview.dependency_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionProjectOverviewOutputRow",
            status_text: "Project dependencies selected",
            output_text: "Selected Module Dependencies   2 warnings",
        },
        "workbench.extension.plugin_manager.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
            status_text: "Plugin manager opened",
            output_text: "Native extension workspace opened for Audio Runtime",
        },
        "workbench.extension.plugin_manager.hot_reload.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
            status_text: "Plugin hot reload queued",
            output_text: "Hot reload queued   Audio Runtime   editor + game",
        },
        "workbench.extension.plugin_manager.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
            status_text: "Plugin validation queued",
            output_text: "Validation queued   18 installed   1 warning",
        },
        "workbench.extension.plugin_manager.audio_runtime_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
                status_text: "Plugin runtime selected",
                output_text: "Selected Audio Runtime   enabled   v1.8.2",
            }
        }
        "workbench.extension.plugin_manager.version_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPluginManagerOutputRow",
                status_text: "Plugin warning selected",
                output_text: "Selected Version Warning   engine api 0.12",
            }
        }
        _ => return None,
    };
    Some(feedback)
}

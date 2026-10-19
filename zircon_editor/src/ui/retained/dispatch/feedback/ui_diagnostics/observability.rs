use super::super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.runtime_diagnostics.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.runtime_diagnostics.capture_snapshot.invoke" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.runtime_diagnostics.export_report.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.runtime_diagnostics.actor_health_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.runtime_diagnostics.weapon_target_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.performance.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPerformanceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.performance.capture_frame.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPerformanceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.performance.filter_samples.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPerformanceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.performance.render_thread_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPerformanceOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.performance.gpu_lighting_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPerformanceOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.telemetry_dashboard.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.telemetry_dashboard.filter_telemetry.invoke" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.telemetry_dashboard.run_query.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.telemetry_dashboard.dau_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.telemetry_dashboard.fps_p95_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        _ => return None,
    };
    Some(feedback)
}

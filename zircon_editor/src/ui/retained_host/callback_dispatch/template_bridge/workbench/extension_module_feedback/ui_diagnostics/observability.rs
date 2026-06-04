use super::super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.runtime_diagnostics.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
            status_text: "Runtime diagnostics workspace opened",
            output_text: "Native extension workspace opened for Session_Player_01",
        },
        "workbench.extension.runtime_diagnostics.capture_snapshot.invoke" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
                status_text: "Runtime snapshot queued",
                output_text: "Capture Snapshot   Session_Player_01   Actors 420   Events 1.2K",
            }
        }
        "workbench.extension.runtime_diagnostics.export_report.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
            status_text: "Runtime report export queued",
            output_text: "Export Report   subsystem World   filter health",
        },
        "workbench.extension.runtime_diagnostics.actor_health_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
                status_text: "Runtime watch value selected",
                output_text: "Selected Player.Health   Float   82",
            }
        }
        "workbench.extension.runtime_diagnostics.weapon_target_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
                status_text: "Runtime warning selected",
                output_text: "Selected Weapon.Target   Object   Null   Warning",
            }
        }
        "workbench.extension.performance.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPerformanceOutputRow",
            status_text: "Performance workspace opened",
            output_text: "Native extension workspace opened for Capture_1234",
        },
        "workbench.extension.performance.capture_frame.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPerformanceOutputRow",
            status_text: "Performance capture queued",
            output_text: "Capture Frame   Frame 1234   CPU 7.1 ms   GPU 9.2 ms",
        },
        "workbench.extension.performance.filter_samples.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPerformanceOutputRow",
            status_text: "Performance samples filtered",
            output_text: "Filter Samples   lane GPU   threshold 1.0 ms",
        },
        "workbench.extension.performance.render_thread_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPerformanceOutputRow",
                status_text: "Performance sample selected",
                output_text: "Selected Render Thread   4.8 ms   frame 1234",
            }
        }
        "workbench.extension.performance.gpu_lighting_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPerformanceOutputRow",
                status_text: "Performance hotspot selected",
                output_text: "Selected GPU Lighting   3.2 ms   hotspot",
            }
        }
        "workbench.extension.telemetry_dashboard.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
            status_text: "Telemetry dashboard opened",
            output_text: "Native extension workspace opened for Query_Retention",
        },
        "workbench.extension.telemetry_dashboard.filter_telemetry.invoke" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
                status_text: "Telemetry filter applied",
                output_text: "Filtered Query_Retention   24h   New Users",
            }
        }
        "workbench.extension.telemetry_dashboard.run_query.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
            status_text: "Telemetry query queued",
            output_text: "Query queued   2.4M events   latency 120 ms",
        },
        "workbench.extension.telemetry_dashboard.dau_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
            status_text: "Telemetry metric selected",
            output_text: "Selected DAU   128K   +4.2 percent",
        },
        "workbench.extension.telemetry_dashboard.fps_p95_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTelemetryDashboardOutputRow",
                status_text: "Telemetry warning selected",
                output_text: "Selected FPS P95   58 FPS   Warning",
            }
        }
        _ => return None,
    };
    Some(feedback)
}

use super::super::types::{ActionControl, ExtensionNavigationSpec, action, spec};

const CONSOLE_DIAGNOSTICS_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionConsoleDiagnosticsLiveLogTab",
    "WorkbenchExtensionConsoleDiagnosticsCountersTab",
    "WorkbenchExtensionConsoleDiagnosticsReportTab",
];
const CONSOLE_DIAGNOSTICS_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.console_diagnostics.live_log_tab.select",
        "WorkbenchExtensionConsoleDiagnosticsLiveLogTab",
    ),
    action(
        "workbench.extension.console_diagnostics.counters_tab.select",
        "WorkbenchExtensionConsoleDiagnosticsCountersTab",
    ),
    action(
        "workbench.extension.console_diagnostics.report_tab.select",
        "WorkbenchExtensionConsoleDiagnosticsReportTab",
    ),
];
const CONSOLE_DIAGNOSTICS_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionConsoleDiagnosticsSessionRow",
    "WorkbenchExtensionConsoleDiagnosticsRendererRow",
    "WorkbenchExtensionConsoleDiagnosticsAssetRow",
    "WorkbenchExtensionConsoleDiagnosticsWarningTableRow",
    "WorkbenchExtensionConsoleDiagnosticsInfoTableRow",
    "WorkbenchExtensionConsoleDiagnosticsGameplayTableRow",
    "WorkbenchExtensionConsoleDiagnosticsErrorTableRow",
    "WorkbenchExtensionConsoleDiagnosticsOutputRow",
];
const CONSOLE_DIAGNOSTICS_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.console_diagnostics.session_row.select",
        "WorkbenchExtensionConsoleDiagnosticsSessionRow",
    ),
    action(
        "workbench.extension.console_diagnostics.renderer_row.select",
        "WorkbenchExtensionConsoleDiagnosticsRendererRow",
    ),
    action(
        "workbench.extension.console_diagnostics.asset_row.select",
        "WorkbenchExtensionConsoleDiagnosticsAssetRow",
    ),
    action(
        "workbench.extension.console_diagnostics.warning_table_row.select",
        "WorkbenchExtensionConsoleDiagnosticsWarningTableRow",
    ),
    action(
        "workbench.extension.console_diagnostics.info_table_row.select",
        "WorkbenchExtensionConsoleDiagnosticsInfoTableRow",
    ),
    action(
        "workbench.extension.console_diagnostics.gameplay_table_row.select",
        "WorkbenchExtensionConsoleDiagnosticsGameplayTableRow",
    ),
    action(
        "workbench.extension.console_diagnostics.error_table_row.select",
        "WorkbenchExtensionConsoleDiagnosticsErrorTableRow",
    ),
    action(
        "workbench.extension.console_diagnostics.output.select",
        "WorkbenchExtensionConsoleDiagnosticsOutputRow",
    ),
];
const CONSOLE_DIAGNOSTICS_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudConsoleDiagnosticsButton",
    "WorkbenchExtensionConsoleDiagnosticsFilterConsoleButton",
    "WorkbenchExtensionConsoleDiagnosticsClearConsoleButton",
];
const CONSOLE_DIAGNOSTICS_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.console_diagnostics.open",
        "WorkbenchHudConsoleDiagnosticsButton",
    ),
    action(
        "workbench.extension.console_diagnostics.filter_console.invoke",
        "WorkbenchExtensionConsoleDiagnosticsFilterConsoleButton",
    ),
    action(
        "workbench.extension.console_diagnostics.clear_console.invoke",
        "WorkbenchExtensionConsoleDiagnosticsClearConsoleButton",
    ),
];
const CONSOLE_DIAGNOSTICS_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.console_diagnostics.subsystem.edit",
    "workbench.extension.console_diagnostics.subsystem.commit",
    "workbench.extension.console_diagnostics.severity.edit",
    "workbench.extension.console_diagnostics.severity.commit",
    "workbench.extension.console_diagnostics.regex.edit",
    "workbench.extension.console_diagnostics.regex.commit",
];

pub(in super::super) const CONSOLE_DIAGNOSTICS_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.console_diagnostics.open",
    "WorkbenchExtensionConsoleDiagnosticsWorkspace",
    CONSOLE_DIAGNOSTICS_TAB_CONTROLS,
    CONSOLE_DIAGNOSTICS_TAB_ACTIONS,
    CONSOLE_DIAGNOSTICS_ROW_CONTROLS,
    CONSOLE_DIAGNOSTICS_ROW_ACTIONS,
    CONSOLE_DIAGNOSTICS_COMMAND_CONTROLS,
    CONSOLE_DIAGNOSTICS_COMMAND_ACTIONS,
    CONSOLE_DIAGNOSTICS_FIELD_ACTIONS,
);

const RUNTIME_DIAGNOSTICS_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionRuntimeDiagnosticsWatchTab",
    "WorkbenchExtensionRuntimeDiagnosticsEventsTab",
    "WorkbenchExtensionRuntimeDiagnosticsConsoleTab",
];
const RUNTIME_DIAGNOSTICS_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.runtime_diagnostics.watch_tab.select",
        "WorkbenchExtensionRuntimeDiagnosticsWatchTab",
    ),
    action(
        "workbench.extension.runtime_diagnostics.events_tab.select",
        "WorkbenchExtensionRuntimeDiagnosticsEventsTab",
    ),
    action(
        "workbench.extension.runtime_diagnostics.console_tab.select",
        "WorkbenchExtensionRuntimeDiagnosticsConsoleTab",
    ),
];
const RUNTIME_DIAGNOSTICS_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionRuntimeDiagnosticsSessionPlayerRow",
    "WorkbenchExtensionRuntimeDiagnosticsWorldRuntimeRow",
    "WorkbenchExtensionRuntimeDiagnosticsEventStreamRow",
    "WorkbenchExtensionRuntimeDiagnosticsActorHealthTableRow",
    "WorkbenchExtensionRuntimeDiagnosticsAiGuardStateTableRow",
    "WorkbenchExtensionRuntimeDiagnosticsWorldTimeTableRow",
    "WorkbenchExtensionRuntimeDiagnosticsWeaponTargetTableRow",
    "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
];
const RUNTIME_DIAGNOSTICS_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.runtime_diagnostics.session_player_row.select",
        "WorkbenchExtensionRuntimeDiagnosticsSessionPlayerRow",
    ),
    action(
        "workbench.extension.runtime_diagnostics.world_runtime_row.select",
        "WorkbenchExtensionRuntimeDiagnosticsWorldRuntimeRow",
    ),
    action(
        "workbench.extension.runtime_diagnostics.event_stream_row.select",
        "WorkbenchExtensionRuntimeDiagnosticsEventStreamRow",
    ),
    action(
        "workbench.extension.runtime_diagnostics.actor_health_table_row.select",
        "WorkbenchExtensionRuntimeDiagnosticsActorHealthTableRow",
    ),
    action(
        "workbench.extension.runtime_diagnostics.ai_guard_state_table_row.select",
        "WorkbenchExtensionRuntimeDiagnosticsAiGuardStateTableRow",
    ),
    action(
        "workbench.extension.runtime_diagnostics.world_time_table_row.select",
        "WorkbenchExtensionRuntimeDiagnosticsWorldTimeTableRow",
    ),
    action(
        "workbench.extension.runtime_diagnostics.weapon_target_table_row.select",
        "WorkbenchExtensionRuntimeDiagnosticsWeaponTargetTableRow",
    ),
    action(
        "workbench.extension.runtime_diagnostics.output.select",
        "WorkbenchExtensionRuntimeDiagnosticsOutputRow",
    ),
];
const RUNTIME_DIAGNOSTICS_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudRuntimeDiagnosticsButton",
    "WorkbenchExtensionRuntimeDiagnosticsCaptureSnapshotButton",
    "WorkbenchExtensionRuntimeDiagnosticsExportReportButton",
];
const RUNTIME_DIAGNOSTICS_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.runtime_diagnostics.open",
        "WorkbenchHudRuntimeDiagnosticsButton",
    ),
    action(
        "workbench.extension.runtime_diagnostics.capture_snapshot.invoke",
        "WorkbenchExtensionRuntimeDiagnosticsCaptureSnapshotButton",
    ),
    action(
        "workbench.extension.runtime_diagnostics.export_report.invoke",
        "WorkbenchExtensionRuntimeDiagnosticsExportReportButton",
    ),
];
const RUNTIME_DIAGNOSTICS_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.runtime_diagnostics.session.edit",
    "workbench.extension.runtime_diagnostics.session.commit",
    "workbench.extension.runtime_diagnostics.subsystem.edit",
    "workbench.extension.runtime_diagnostics.subsystem.commit",
    "workbench.extension.runtime_diagnostics.filter.edit",
    "workbench.extension.runtime_diagnostics.filter.commit",
];

pub(in super::super) const RUNTIME_DIAGNOSTICS_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.runtime_diagnostics.open",
    "WorkbenchExtensionRuntimeDiagnosticsWorkspace",
    RUNTIME_DIAGNOSTICS_TAB_CONTROLS,
    RUNTIME_DIAGNOSTICS_TAB_ACTIONS,
    RUNTIME_DIAGNOSTICS_ROW_CONTROLS,
    RUNTIME_DIAGNOSTICS_ROW_ACTIONS,
    RUNTIME_DIAGNOSTICS_COMMAND_CONTROLS,
    RUNTIME_DIAGNOSTICS_COMMAND_ACTIONS,
    RUNTIME_DIAGNOSTICS_FIELD_ACTIONS,
);

const PERFORMANCE_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPerformanceFrameCaptureTab",
    "WorkbenchExtensionPerformanceCpuLaneTab",
    "WorkbenchExtensionPerformanceGpuLaneTab",
];
const PERFORMANCE_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.performance.frame_capture_tab.select",
        "WorkbenchExtensionPerformanceFrameCaptureTab",
    ),
    action(
        "workbench.extension.performance.cpu_lane_tab.select",
        "WorkbenchExtensionPerformanceCpuLaneTab",
    ),
    action(
        "workbench.extension.performance.gpu_lane_tab.select",
        "WorkbenchExtensionPerformanceGpuLaneTab",
    ),
];
const PERFORMANCE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPerformanceCaptureRow",
    "WorkbenchExtensionPerformanceCpuGameThreadRow",
    "WorkbenchExtensionPerformanceGpuLightingRow",
    "WorkbenchExtensionPerformanceGameThreadTableRow",
    "WorkbenchExtensionPerformanceRenderThreadTableRow",
    "WorkbenchExtensionPerformanceGpuLightingTableRow",
    "WorkbenchExtensionPerformanceTextureUploadTableRow",
    "WorkbenchExtensionPerformanceOutputRow",
];
const PERFORMANCE_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.performance.capture_row.select",
        "WorkbenchExtensionPerformanceCaptureRow",
    ),
    action(
        "workbench.extension.performance.cpu_game_thread_row.select",
        "WorkbenchExtensionPerformanceCpuGameThreadRow",
    ),
    action(
        "workbench.extension.performance.gpu_lighting_row.select",
        "WorkbenchExtensionPerformanceGpuLightingRow",
    ),
    action(
        "workbench.extension.performance.game_thread_table_row.select",
        "WorkbenchExtensionPerformanceGameThreadTableRow",
    ),
    action(
        "workbench.extension.performance.render_thread_table_row.select",
        "WorkbenchExtensionPerformanceRenderThreadTableRow",
    ),
    action(
        "workbench.extension.performance.gpu_lighting_table_row.select",
        "WorkbenchExtensionPerformanceGpuLightingTableRow",
    ),
    action(
        "workbench.extension.performance.texture_upload_table_row.select",
        "WorkbenchExtensionPerformanceTextureUploadTableRow",
    ),
    action(
        "workbench.extension.performance.output.select",
        "WorkbenchExtensionPerformanceOutputRow",
    ),
];
const PERFORMANCE_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudPerformanceButton",
    "WorkbenchExtensionPerformanceCaptureFrameButton",
    "WorkbenchExtensionPerformanceFilterSamplesButton",
];
const PERFORMANCE_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.performance.open",
        "WorkbenchHudPerformanceButton",
    ),
    action(
        "workbench.extension.performance.capture_frame.invoke",
        "WorkbenchExtensionPerformanceCaptureFrameButton",
    ),
    action(
        "workbench.extension.performance.filter_samples.invoke",
        "WorkbenchExtensionPerformanceFilterSamplesButton",
    ),
];
const PERFORMANCE_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.performance.capture.edit",
    "workbench.extension.performance.capture.commit",
    "workbench.extension.performance.lane.edit",
    "workbench.extension.performance.lane.commit",
    "workbench.extension.performance.threshold.edit",
    "workbench.extension.performance.threshold.commit",
];

pub(in super::super) const PERFORMANCE_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.performance.open",
    "WorkbenchExtensionPerformanceWorkspace",
    PERFORMANCE_TAB_CONTROLS,
    PERFORMANCE_TAB_ACTIONS,
    PERFORMANCE_ROW_CONTROLS,
    PERFORMANCE_ROW_ACTIONS,
    PERFORMANCE_COMMAND_CONTROLS,
    PERFORMANCE_COMMAND_ACTIONS,
    PERFORMANCE_FIELD_ACTIONS,
);

const TELEMETRY_DASHBOARD_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionTelemetryDashboardMetricsTab",
    "WorkbenchExtensionTelemetryDashboardSegmentsTab",
    "WorkbenchExtensionTelemetryDashboardRawEventsTab",
];
const TELEMETRY_DASHBOARD_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.telemetry_dashboard.metrics_tab.select",
        "WorkbenchExtensionTelemetryDashboardMetricsTab",
    ),
    action(
        "workbench.extension.telemetry_dashboard.segments_tab.select",
        "WorkbenchExtensionTelemetryDashboardSegmentsTab",
    ),
    action(
        "workbench.extension.telemetry_dashboard.raw_events_tab.select",
        "WorkbenchExtensionTelemetryDashboardRawEventsTab",
    ),
];
const TELEMETRY_DASHBOARD_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionTelemetryDashboardRetentionQueryRow",
    "WorkbenchExtensionTelemetryDashboardNewUsersSegmentRow",
    "WorkbenchExtensionTelemetryDashboardMetricFpsRow",
    "WorkbenchExtensionTelemetryDashboardDauTableRow",
    "WorkbenchExtensionTelemetryDashboardFpsP95TableRow",
    "WorkbenchExtensionTelemetryDashboardCrashRateTableRow",
    "WorkbenchExtensionTelemetryDashboardQueueWaitTableRow",
    "WorkbenchExtensionTelemetryDashboardOutputRow",
];
const TELEMETRY_DASHBOARD_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.telemetry_dashboard.retention_query_row.select",
        "WorkbenchExtensionTelemetryDashboardRetentionQueryRow",
    ),
    action(
        "workbench.extension.telemetry_dashboard.new_users_segment_row.select",
        "WorkbenchExtensionTelemetryDashboardNewUsersSegmentRow",
    ),
    action(
        "workbench.extension.telemetry_dashboard.metric_fps_row.select",
        "WorkbenchExtensionTelemetryDashboardMetricFpsRow",
    ),
    action(
        "workbench.extension.telemetry_dashboard.dau_table_row.select",
        "WorkbenchExtensionTelemetryDashboardDauTableRow",
    ),
    action(
        "workbench.extension.telemetry_dashboard.fps_p95_table_row.select",
        "WorkbenchExtensionTelemetryDashboardFpsP95TableRow",
    ),
    action(
        "workbench.extension.telemetry_dashboard.crash_rate_table_row.select",
        "WorkbenchExtensionTelemetryDashboardCrashRateTableRow",
    ),
    action(
        "workbench.extension.telemetry_dashboard.queue_wait_table_row.select",
        "WorkbenchExtensionTelemetryDashboardQueueWaitTableRow",
    ),
    action(
        "workbench.extension.telemetry_dashboard.output.select",
        "WorkbenchExtensionTelemetryDashboardOutputRow",
    ),
];
const TELEMETRY_DASHBOARD_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchHudTelemetryDashboardButton",
    "WorkbenchExtensionTelemetryDashboardFilterTelemetryButton",
    "WorkbenchExtensionTelemetryDashboardRunQueryButton",
];
const TELEMETRY_DASHBOARD_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.telemetry_dashboard.open",
        "WorkbenchHudTelemetryDashboardButton",
    ),
    action(
        "workbench.extension.telemetry_dashboard.filter_telemetry.invoke",
        "WorkbenchExtensionTelemetryDashboardFilterTelemetryButton",
    ),
    action(
        "workbench.extension.telemetry_dashboard.run_query.invoke",
        "WorkbenchExtensionTelemetryDashboardRunQueryButton",
    ),
];
const TELEMETRY_DASHBOARD_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.telemetry_dashboard.query.edit",
    "workbench.extension.telemetry_dashboard.query.commit",
    "workbench.extension.telemetry_dashboard.range.edit",
    "workbench.extension.telemetry_dashboard.range.commit",
    "workbench.extension.telemetry_dashboard.segment.edit",
    "workbench.extension.telemetry_dashboard.segment.commit",
];

pub(in super::super) const TELEMETRY_DASHBOARD_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.telemetry_dashboard.open",
    "WorkbenchExtensionTelemetryDashboardWorkspace",
    TELEMETRY_DASHBOARD_TAB_CONTROLS,
    TELEMETRY_DASHBOARD_TAB_ACTIONS,
    TELEMETRY_DASHBOARD_ROW_CONTROLS,
    TELEMETRY_DASHBOARD_ROW_ACTIONS,
    TELEMETRY_DASHBOARD_COMMAND_CONTROLS,
    TELEMETRY_DASHBOARD_COMMAND_ACTIONS,
    TELEMETRY_DASHBOARD_FIELD_ACTIONS,
);

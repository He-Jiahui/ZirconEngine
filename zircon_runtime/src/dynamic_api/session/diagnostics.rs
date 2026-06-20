use zircon_runtime_interface::{
    ProfileControlResponse, RuntimeDiagnosticMeasurement, RuntimeDiagnosticSeriesSnapshot,
    RuntimeDiagnosticsSnapshot, RuntimeSceneAssetReloadDiagnostics,
};

use crate::core::diagnostics::collect_runtime_diagnostics;
use crate::scene::{DynamicSceneAssetReloadFrameApplyReport, DynamicSceneAssetReloadSkipReason};

use super::RuntimeDynamicSession;

pub(super) fn runtime_diagnostics_response(
    session: &RuntimeDynamicSession,
) -> ProfileControlResponse {
    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let mut response = ProfileControlResponse::ok("runtime diagnostics snapshot captured");
    response.runtime_diagnostics = Some(RuntimeDiagnosticsSnapshot {
        frame_index: session.runtime.real_time().frame_index(),
        diagnostic_series: diagnostics
            .store
            .series
            .into_iter()
            .map(|series| RuntimeDiagnosticSeriesSnapshot {
                path: series.path.as_str().to_string(),
                unit: series.unit,
                subsystem_tags: series.subsystem_tags,
                current: series.current,
                smoothed: series.smoothed,
                min: series.min,
                max: series.max,
                history: series
                    .history
                    .into_iter()
                    .map(|measurement| RuntimeDiagnosticMeasurement {
                        frame_index: measurement.frame_index,
                        value: measurement.value,
                    })
                    .collect(),
            })
            .collect(),
        scene_asset_reload: Some(scene_asset_reload_diagnostics(
            session.scene_asset_reload_queue.is_some(),
            session.last_scene_asset_reload_report.as_ref(),
        )),
        profile: diagnostics.profile,
    });
    response
}

fn scene_asset_reload_diagnostics(
    enabled: bool,
    report: Option<&DynamicSceneAssetReloadFrameApplyReport>,
) -> RuntimeSceneAssetReloadDiagnostics {
    let Some(report) = report else {
        return RuntimeSceneAssetReloadDiagnostics {
            enabled,
            ..RuntimeSceneAssetReloadDiagnostics::default()
        };
    };

    RuntimeSceneAssetReloadDiagnostics {
        enabled,
        events_drained: report.events_drained(),
        scheduled: report.scheduled_count(),
        skipped: report.skipped_count(),
        skipped_removed: report.skipped_count_for(DynamicSceneAssetReloadSkipReason::Removed),
        skipped_reload_failed: report
            .skipped_count_for(DynamicSceneAssetReloadSkipReason::ReloadFailed),
        skipped_missing_locator: report
            .skipped_count_for(DynamicSceneAssetReloadSkipReason::MissingLocator),
        skipped_stale_revision: report
            .skipped_count_for(DynamicSceneAssetReloadSkipReason::StaleRevision),
        superseded_pending: report.superseded_pending_count(),
        applied: report.applied_count(),
        failed: report.failed_count(),
        stale: report.stale_count(),
        pending: report.pending_count(),
        receiver_disconnected: report.receiver_disconnected(),
    }
}

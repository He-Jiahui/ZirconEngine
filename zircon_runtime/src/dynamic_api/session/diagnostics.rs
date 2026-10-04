use zircon_runtime_interface::{
    ProfileControlResponse, RuntimeDiagnosticMeasurement, RuntimeDiagnosticSeriesSnapshot,
    RuntimeDiagnosticsSnapshot, RuntimeRenderDeviceDiagnosticsSnapshot,
    RuntimeSceneAssetReloadDiagnostics,
};

use crate::core::framework::render::RenderStats;
use crate::runtime_diagnostics::collect_runtime_diagnostics;
use crate::scene::{DynamicSceneAssetReloadFrameApplyReport, DynamicSceneAssetReloadSkipReason};

use super::RuntimeDynamicSession;

pub(super) fn runtime_diagnostics_response(
    session: &RuntimeDynamicSession,
) -> ProfileControlResponse {
    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let (render_backend_name, render_device) =
        take_runtime_render_diagnostics(diagnostics.render.stats);
    let mut response = ProfileControlResponse::ok("runtime diagnostics snapshot captured");
    response.runtime_diagnostics = Some(RuntimeDiagnosticsSnapshot {
        frame_index: session.runtime.real_time().frame_index(),
        project_identity: session.project_identity.clone(),
        scene_uri: session.scene_uri.clone(),
        selected_model_resource_id: session.selected_model_resource_id.clone(),
        selected_material_resource_id: session.selected_material_resource_id.clone(),
        render_backend_name,
        render_device,
        input: session.input_diagnostics.snapshot(),
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

// 消费拥有的 RenderStats 并移动字符串到 ABI 快照，避免诊断采样再次复制设备名称。
fn take_runtime_render_diagnostics(
    stats: Option<RenderStats>,
) -> (
    Option<String>,
    Option<RuntimeRenderDeviceDiagnosticsSnapshot>,
) {
    let Some(stats) = stats else {
        return (None, None);
    };
    let render_backend_name =
        Some(stats.capabilities.backend_name).filter(|name| !name.trim().is_empty());
    let render_device = stats.device_diagnostics.map(|diagnostics| {
        let limits = diagnostics.limits;
        RuntimeRenderDeviceDiagnosticsSnapshot {
            adapter_name: diagnostics.adapter_name,
            adapter_device_type: diagnostics.adapter_device_type,
            max_bind_groups: limits.max_bind_groups,
            max_texture_dimension_2d: limits.max_texture_dimension_2d,
            max_texture_array_layers: limits.max_texture_array_layers,
            max_sampled_textures_per_shader_stage: limits.max_sampled_textures_per_shader_stage,
            max_binding_array_elements_per_shader_stage: limits
                .max_binding_array_elements_per_shader_stage,
            max_binding_array_sampler_elements_per_shader_stage: limits
                .max_binding_array_sampler_elements_per_shader_stage,
            max_storage_buffers_per_shader_stage: limits.max_storage_buffers_per_shader_stage,
            max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
        }
    });
    (render_backend_name, render_device)
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

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;

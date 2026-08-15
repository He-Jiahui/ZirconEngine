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
    let render_backend_name = diagnostics
        .render
        .stats
        .as_ref()
        .map(|stats| stats.capabilities.backend_name.clone())
        .filter(|name| !name.trim().is_empty());
    let render_device = runtime_render_device_snapshot(diagnostics.render.stats.as_ref());
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

fn runtime_render_device_snapshot(
    stats: Option<&RenderStats>,
) -> Option<RuntimeRenderDeviceDiagnosticsSnapshot> {
    let diagnostics = stats?.device_diagnostics.as_ref()?;
    Some(RuntimeRenderDeviceDiagnosticsSnapshot {
        adapter_name: diagnostics.adapter_name.clone(),
        adapter_device_type: diagnostics.adapter_device_type.clone(),
        max_bind_groups: diagnostics.limits.max_bind_groups,
        max_texture_dimension_2d: diagnostics.limits.max_texture_dimension_2d,
        max_texture_array_layers: diagnostics.limits.max_texture_array_layers,
        max_sampled_textures_per_shader_stage: diagnostics
            .limits
            .max_sampled_textures_per_shader_stage,
        max_binding_array_elements_per_shader_stage: diagnostics
            .limits
            .max_binding_array_elements_per_shader_stage,
        max_binding_array_sampler_elements_per_shader_stage: diagnostics
            .limits
            .max_binding_array_sampler_elements_per_shader_stage,
        max_storage_buffers_per_shader_stage: diagnostics
            .limits
            .max_storage_buffers_per_shader_stage,
        max_storage_buffer_binding_size: diagnostics.limits.max_storage_buffer_binding_size,
    })
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
mod tests {
    use crate::core::framework::render::{
        RenderDeviceDiagnostics, RenderDeviceLimitDiagnostics, RenderStats,
    };

    use super::runtime_render_device_snapshot;

    #[test]
    fn runtime_render_device_snapshot_copies_the_framework_diagnostic_contract() {
        let stats = RenderStats {
            device_diagnostics: Some(RenderDeviceDiagnostics {
                adapter_name: "Zircon Test Adapter".to_owned(),
                adapter_device_type: "discrete_gpu".to_owned(),
                limits: RenderDeviceLimitDiagnostics {
                    max_bind_groups: 5,
                    max_texture_dimension_2d: 16_384,
                    max_texture_array_layers: 256,
                    max_sampled_textures_per_shader_stage: 16,
                    max_binding_array_elements_per_shader_stage: 500_000,
                    max_binding_array_sampler_elements_per_shader_stage: 1_000,
                    max_storage_buffers_per_shader_stage: 8,
                    max_storage_buffer_binding_size: 134_217_728,
                },
            }),
            ..RenderStats::default()
        };

        let snapshot = runtime_render_device_snapshot(Some(&stats)).expect("device snapshot");

        assert_eq!(snapshot.adapter_name, "Zircon Test Adapter");
        assert_eq!(snapshot.adapter_device_type, "discrete_gpu");
        assert_eq!(snapshot.max_bind_groups, 5);
        assert_eq!(snapshot.max_texture_dimension_2d, 16_384);
        assert_eq!(snapshot.max_texture_array_layers, 256);
        assert_eq!(snapshot.max_sampled_textures_per_shader_stage, 16);
        assert_eq!(
            snapshot.max_binding_array_elements_per_shader_stage,
            500_000
        );
        assert_eq!(
            snapshot.max_binding_array_sampler_elements_per_shader_stage,
            1_000
        );
        assert_eq!(snapshot.max_storage_buffers_per_shader_stage, 8);
        assert_eq!(snapshot.max_storage_buffer_binding_size, 134_217_728);
    }

    #[test]
    fn runtime_render_device_snapshot_omits_unavailable_renderer_facts() {
        assert!(runtime_render_device_snapshot(None).is_none());
        assert!(runtime_render_device_snapshot(Some(&RenderStats::default())).is_none());
    }
}

use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::viewport_generation_guard::{
    validate_viewport_generation, viewport_record_mut_after_generation_check_in,
};
use crate::core::framework::render::{RenderFrameworkError, RenderPluginRendererOutputs};
use crate::graphics::{HybridGiRuntimePrepareInput, VirtualGeometryRuntimePrepareInput};

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn prepare_runtime_submission(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Result<PreparedRuntimeSubmission, RenderFrameworkError> {
    validate_viewport_generation(state, viewport, context)?;
    let (hybrid_gi_evictable_probe_ids, hybrid_gi_renderer_outputs, hybrid_gi_prepared_frame) =
        prepare_hybrid_gi_runtime(state, viewport, context)?
            .map(crate::graphics::HybridGiRuntimePrepareOutput::into_parts)
            .unwrap_or_default();
    let (virtual_geometry_evictable_page_ids, virtual_geometry_renderer_outputs) =
        prepare_virtual_geometry_runtime(state, viewport, context)?
            .map(crate::graphics::VirtualGeometryRuntimePrepareOutput::into_parts)
            .unwrap_or_default();
    let plugin_renderer_outputs = merge_prepare_plugin_renderer_outputs(
        hybrid_gi_renderer_outputs,
        virtual_geometry_renderer_outputs,
    );

    Ok(PreparedRuntimeSubmission::new(
        hybrid_gi_evictable_probe_ids,
        hybrid_gi_prepared_frame,
        virtual_geometry_evictable_page_ids,
        plugin_renderer_outputs,
    ))
}

fn merge_prepare_plugin_renderer_outputs(
    hybrid_gi_outputs: RenderPluginRendererOutputs,
    virtual_geometry_outputs: RenderPluginRendererOutputs,
) -> RenderPluginRendererOutputs {
    RenderPluginRendererOutputs {
        hybrid_gi: hybrid_gi_outputs.hybrid_gi,
        virtual_geometry: virtual_geometry_outputs.virtual_geometry,
        ..RenderPluginRendererOutputs::default()
    }
}

fn prepare_hybrid_gi_runtime(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Result<Option<crate::graphics::HybridGiRuntimePrepareOutput>, RenderFrameworkError> {
    if !context.hybrid_gi_enabled() {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_hybrid_gi_runtimes();
        }
        return Ok(None);
    }

    let Some(provider) = state
        .hybrid_gi_runtime_provider
        .as_ref()
        .map(crate::graphics::HybridGiRuntimeProviderRegistration::provider)
    else {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_hybrid_gi_runtimes();
        }
        return Err(missing_runtime_provider("hybrid global illumination"));
    };
    let record =
        viewport_record_mut_after_generation_check_in(&mut state.viewports, viewport, context)?;
    let input = HybridGiRuntimePrepareInput::new(
        context.hybrid_gi_extract(),
        context.scene_meshes(),
        context.scene_directional_lights(),
        context.scene_point_lights(),
        context.scene_spot_lights(),
        context.scene_baked_lighting(),
        context.scene_has_baked_probe_grid(),
        context.hybrid_gi_update_plan(),
        context.predicted_generation(),
    )
    .with_view_state(
        Some(context.scene_camera_position()),
        context.hybrid_gi_history_invalidated(),
    );
    Ok(Some(
        record
            .ensure_hybrid_gi_runtime(context.camera_history_key(), provider)
            .prepare_frame(input),
    ))
}

fn prepare_virtual_geometry_runtime(
    state: &mut RenderFrameworkState,
    viewport: crate::core::framework::render::RenderViewportHandle,
    context: &FrameSubmissionContext,
) -> Result<Option<crate::graphics::VirtualGeometryRuntimePrepareOutput>, RenderFrameworkError> {
    if !context.virtual_geometry_enabled() {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_virtual_geometry_runtimes();
        }
        return Ok(None);
    }

    let Some(provider) = state
        .virtual_geometry_runtime_provider
        .as_ref()
        .map(crate::graphics::VirtualGeometryRuntimeProviderRegistration::provider)
    else {
        if let Some(record) = state.viewports.get_mut(&viewport) {
            record.clear_virtual_geometry_runtimes();
        }
        return Err(missing_runtime_provider("virtual geometry"));
    };
    let record =
        viewport_record_mut_after_generation_check_in(&mut state.viewports, viewport, context)?;
    let visibility_context = context.visibility_context();
    let input = VirtualGeometryRuntimePrepareInput::new(
        context.virtual_geometry_extract(),
        context.virtual_geometry_page_upload_plan(),
        &visibility_context.virtual_geometry_visible_clusters,
        &visibility_context.virtual_geometry_draw_segments,
        context.predicted_generation(),
    );
    Ok(Some(
        record
            .ensure_virtual_geometry_runtime(context.camera_history_key(), provider)
            .prepare_frame(input),
    ))
}

fn missing_runtime_provider(feature: &str) -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: format!("{feature} runtime provider"),
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use super::merge_prepare_plugin_renderer_outputs;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometryReadbackOutputs,
    };

    const SAMPLE_PAIRS: usize = 17;
    const FRAMES_PER_SAMPLE: usize = 262_144;

    #[test]
    fn prepare_merge_keeps_particle_sideband_empty_until_particle_prepare_exists() {
        let merged = merge_prepare_plugin_renderer_outputs(
            RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![11],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 5,
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 7,
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        assert_eq!(merged.hybrid_gi.completed_probe_ids, vec![11]);
        assert_eq!(
            merged.virtual_geometry.node_cluster_cull.page_request_ids,
            vec![300]
        );
        assert!(merged.particles.is_empty());
    }

    #[test]
    fn optimization_batch_fj_runtime466_runtime_prepare_borrows_provider_arcs() {
        let source = include_str!("prepare.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");

        assert_eq!(production.matches("Registration::provider)").count(), 2);
        assert!(!production.contains("provider_arc"));
        assert!(!production.contains(concat!("runtime_provider", ".clone()")));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fj_runtime466_borrowed_prepare_providers_benchmark() {
        let provider = Arc::new(41_u64);
        for _ in 0..4 {
            black_box(measure_legacy_provider_clones(&provider));
            black_box(measure_borrowed_providers(&provider));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy_provider_clones(&provider));
                optimized_samples.push(measure_borrowed_providers(&provider));
            } else {
                optimized_samples.push(measure_borrowed_providers(&provider));
                legacy_samples.push(measure_legacy_provider_clones(&provider));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy_provider_clones(provider: &Arc<u64>) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..FRAMES_PER_SAMPLE {
            let hybrid_gi = black_box(Arc::clone(black_box(provider)));
            let virtual_geometry = black_box(Arc::clone(black_box(provider)));
            checksum = checksum
                .wrapping_add(*hybrid_gi)
                .wrapping_add(*virtual_geometry);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed_providers(provider: &Arc<u64>) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..FRAMES_PER_SAMPLE {
            let hybrid_gi = black_box(black_box(provider).as_ref());
            let virtual_geometry = black_box(black_box(provider).as_ref());
            checksum = checksum
                .wrapping_add(*hybrid_gi)
                .wrapping_add(*virtual_geometry);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME466_BORROWED_PREPARE_PROVIDERS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} frames_per_sample={FRAMES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=80",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(20),
            "optimized p95 {optimized_p95}ns must be at most 20% of legacy p95 {legacy_p95}ns"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

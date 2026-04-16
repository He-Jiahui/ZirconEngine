use zircon_render_graph::QueueLane;
use zircon_render_server::{
    CapturedFrame, FrameHistoryHandle, RenderServerError, RenderViewportHandle,
};
use zircon_scene::RenderFrameExtract;

use crate::{
    runtime::{HybridGiRuntimeState, ViewportFrameHistory},
    BuiltinRenderFeature, EditorOrRuntimeFrame, RenderPipelineAsset, ViewportState,
    VisibilityContext,
};

use super::compile_options_for_profile::compile_options_for_profile;
use super::compiled_feature_names::compiled_feature_names;
use super::render_server_backend_error::render_server_backend_error;
use super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn submit_frame_extract(
    server: &WgpuRenderServer,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> Result<(), RenderServerError> {
    let mut state = server.state.lock().unwrap();
    let (
        size,
        pipeline_handle,
        quality_profile,
        previous_visibility,
        previous_hybrid_gi_runtime,
        previous_virtual_geometry_runtime,
        compile_options,
    ) = {
        let record = state
            .viewports
            .get(&viewport)
            .ok_or(RenderServerError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
        (
            record.descriptor.size,
            record
                .pipeline
                .or_else(|| {
                    record
                        .quality_profile
                        .as_ref()
                        .and_then(|profile| profile.pipeline_override)
                })
                .unwrap_or(RenderPipelineAsset::default_forward_plus().handle),
            record
                .quality_profile
                .as_ref()
                .map(|profile| profile.name.clone()),
            record
                .history
                .as_ref()
                .map(|history| history.visibility.clone()),
            record.hybrid_gi_runtime.clone(),
            record.virtual_geometry_runtime.clone(),
            compile_options_for_profile(record.quality_profile.as_ref(), &state.stats.capabilities),
        )
    };
    let pipeline_asset = state.pipelines.get(&pipeline_handle).cloned().ok_or(
        RenderServerError::UnknownPipeline {
            pipeline: pipeline_handle.raw(),
        },
    )?;
    let compiled_pipeline = pipeline_asset
        .compile_with_options(&extract, &compile_options)
        .map_err(RenderServerError::Backend)?;
    let hybrid_gi_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.feature == BuiltinRenderFeature::GlobalIllumination);
    let virtual_geometry_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.feature == BuiltinRenderFeature::VirtualGeometry);
    let visibility_context =
        VisibilityContext::from_extract_with_history(&extract, previous_visibility.as_ref());
    let hybrid_gi_extract = hybrid_gi_enabled
        .then(|| extract.lighting.hybrid_global_illumination.clone())
        .flatten();
    let hybrid_gi_update_plan =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_update_plan.clone());
    let hybrid_gi_feedback =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_feedback.clone());
    let virtual_geometry_extract = virtual_geometry_enabled
        .then(|| extract.geometry.virtual_geometry.clone())
        .flatten();
    let virtual_geometry_page_upload_plan = virtual_geometry_enabled
        .then(|| visibility_context.virtual_geometry_page_upload_plan.clone());
    let virtual_geometry_feedback =
        virtual_geometry_enabled.then(|| visibility_context.virtual_geometry_feedback.clone());
    let predicted_generation = state.stats.last_generation.unwrap_or(0) + 1;
    let mut hybrid_gi_runtime = if let Some(extract) = hybrid_gi_extract.as_ref() {
        let mut runtime = previous_hybrid_gi_runtime.unwrap_or_default();
        runtime.register_extract(Some(extract));
        if let Some(plan) = hybrid_gi_update_plan.as_ref() {
            runtime.ingest_plan(predicted_generation, plan);
        }
        Some(runtime)
    } else {
        None
    };
    let mut virtual_geometry_runtime = if let Some(extract) = virtual_geometry_extract.as_ref() {
        let mut runtime = previous_virtual_geometry_runtime.unwrap_or_default();
        runtime.register_extract(Some(extract));
        if let Some(plan) = virtual_geometry_page_upload_plan.as_ref() {
            runtime.ingest_plan(predicted_generation, plan);
        }
        Some(runtime)
    } else {
        None
    };
    let hybrid_gi_prepare = hybrid_gi_runtime
        .as_ref()
        .map(HybridGiRuntimeState::build_prepare_frame);
    let hybrid_gi_evictable_probe_ids = hybrid_gi_prepare
        .as_ref()
        .map(|prepare| prepare.evictable_probe_ids.clone())
        .unwrap_or_default();
    let virtual_geometry_prepare = virtual_geometry_runtime.as_ref().map(|runtime| {
        runtime.build_prepare_frame(&visibility_context.virtual_geometry_visible_clusters)
    });
    let virtual_geometry_evictable_page_ids = virtual_geometry_prepare
        .as_ref()
        .map(|prepare| {
            prepare
                .evictable_pages
                .iter()
                .map(|page| page.page_id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let history_needs_rotation = state
        .viewports
        .get(&viewport)
        .and_then(|record| record.history.as_ref())
        .is_none_or(|history| {
            !history.is_compatible(size, pipeline_handle, &compiled_pipeline.history_bindings)
        });
    let allocated_history = if history_needs_rotation {
        let handle = FrameHistoryHandle::new(state.next_history_id);
        state.next_history_id += 1;
        Some(handle)
    } else {
        None
    };
    let current_history_handle = allocated_history.or_else(|| {
        state
            .viewports
            .get(&viewport)
            .and_then(|record| record.history.as_ref().map(|history| history.handle))
    });
    let frame = state
        .renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(size))
                .with_hybrid_gi_prepare(hybrid_gi_prepare)
                .with_virtual_geometry_prepare(virtual_geometry_prepare),
            &compiled_pipeline,
            current_history_handle,
        )
        .map_err(render_server_backend_error)?;
    let hybrid_gi_gpu_readback = state.renderer.take_last_hybrid_gi_gpu_readback();
    let virtual_geometry_gpu_readback = state.renderer.take_last_virtual_geometry_gpu_readback();

    let (
        history_handle,
        previous_handle,
        hybrid_gi_runtime_snapshot,
        virtual_geometry_runtime_snapshot,
    ) = {
        let record = state
            .viewports
            .get_mut(&viewport)
            .expect("viewport checked above");
        let previous_handle = record.history.as_ref().map(|history| history.handle);
        let history_handle = match (record.history.as_mut(), allocated_history) {
            (Some(history), None) => {
                history.update(
                    frame.generation,
                    compiled_pipeline.history_bindings.clone(),
                    visibility_context.history_snapshot.clone(),
                );
                history.handle
            }
            (_, Some(handle)) => {
                record.history = Some(ViewportFrameHistory::new(
                    handle,
                    size,
                    pipeline_handle,
                    frame.generation,
                    compiled_pipeline.history_bindings.clone(),
                    visibility_context.history_snapshot.clone(),
                ));
                handle
            }
            (None, None) => unreachable!("rotation is required when no history exists"),
        };
        record.compiled_pipeline = Some(compiled_pipeline);
        record.last_capture = Some(CapturedFrame::new(
            frame.width,
            frame.height,
            frame.rgba,
            frame.generation,
        ));
        let hybrid_gi_runtime_snapshot = if let Some(runtime) = hybrid_gi_runtime.as_mut() {
            if let Some(readback) = hybrid_gi_gpu_readback.as_ref() {
                runtime.complete_gpu_updates(
                    readback.completed_probe_ids.iter().copied(),
                    readback.completed_trace_region_ids.iter().copied(),
                    &readback.probe_irradiance_rgb,
                    &hybrid_gi_evictable_probe_ids,
                );
            } else if let Some(feedback) = hybrid_gi_feedback.as_ref() {
                runtime.consume_feedback(feedback);
            }
            Some(runtime.snapshot())
        } else {
            None
        };
        let virtual_geometry_runtime_snapshot =
            if let Some(runtime) = virtual_geometry_runtime.as_mut() {
                if let Some(readback) = virtual_geometry_gpu_readback.as_ref() {
                    runtime.complete_gpu_uploads(
                        readback.completed_page_ids.iter().copied(),
                        &virtual_geometry_evictable_page_ids,
                    );
                } else if let Some(feedback) = virtual_geometry_feedback.as_ref() {
                    runtime.consume_feedback(feedback);
                }
                Some(runtime.snapshot())
            } else {
                None
            };
        record.hybrid_gi_runtime = hybrid_gi_runtime;
        record.virtual_geometry_runtime = virtual_geometry_runtime;
        (
            history_handle,
            previous_handle,
            hybrid_gi_runtime_snapshot,
            virtual_geometry_runtime_snapshot,
        )
    };
    if let Some(previous_handle) = previous_handle {
        if previous_handle != history_handle {
            state.renderer.release_history(previous_handle);
        }
    }
    state.stats.submitted_frames += 1;
    state.stats.last_generation = Some(frame.generation);
    state.stats.last_pipeline = Some(pipeline_handle);
    state.stats.last_frame_history = Some(history_handle);
    state.stats.last_effective_features = compiled_feature_names(
        state
            .viewports
            .get(&viewport)
            .and_then(|record| record.compiled_pipeline.as_ref())
            .expect("compiled pipeline recorded"),
    );
    state.stats.last_async_compute_pass_count = state
        .viewports
        .get(&viewport)
        .and_then(|record| record.compiled_pipeline.as_ref())
        .expect("compiled pipeline recorded")
        .graph
        .passes()
        .iter()
        .filter(|pass| pass.queue == QueueLane::AsyncCompute)
        .count();
    if hybrid_gi_enabled {
        state.stats.last_hybrid_gi_active_probe_count =
            visibility_context.hybrid_gi_active_probes.len();
        state.stats.last_hybrid_gi_requested_probe_count = visibility_context
            .hybrid_gi_update_plan
            .requested_probe_ids
            .len();
        state.stats.last_hybrid_gi_dirty_probe_count = visibility_context
            .hybrid_gi_update_plan
            .dirty_requested_probe_ids
            .len();
        let runtime_snapshot = hybrid_gi_runtime_snapshot.unwrap_or_default();
        state.stats.last_hybrid_gi_cache_entry_count = runtime_snapshot.cache_entry_count;
        state.stats.last_hybrid_gi_resident_probe_count = runtime_snapshot.resident_probe_count;
        state.stats.last_hybrid_gi_pending_update_count = runtime_snapshot.pending_update_count;
        state.stats.last_hybrid_gi_scheduled_trace_region_count =
            runtime_snapshot.scheduled_trace_region_count;
    } else {
        state.stats.last_hybrid_gi_active_probe_count = 0;
        state.stats.last_hybrid_gi_requested_probe_count = 0;
        state.stats.last_hybrid_gi_dirty_probe_count = 0;
        state.stats.last_hybrid_gi_cache_entry_count = 0;
        state.stats.last_hybrid_gi_resident_probe_count = 0;
        state.stats.last_hybrid_gi_pending_update_count = 0;
        state.stats.last_hybrid_gi_scheduled_trace_region_count = 0;
    }
    if virtual_geometry_enabled {
        state.stats.last_virtual_geometry_visible_cluster_count =
            visibility_context.virtual_geometry_visible_clusters.len();
        state.stats.last_virtual_geometry_requested_page_count = visibility_context
            .virtual_geometry_page_upload_plan
            .requested_pages
            .len();
        state.stats.last_virtual_geometry_dirty_page_count = visibility_context
            .virtual_geometry_page_upload_plan
            .dirty_requested_pages
            .len();
        let runtime_snapshot = virtual_geometry_runtime_snapshot.unwrap_or_default();
        state.stats.last_virtual_geometry_page_table_entry_count =
            runtime_snapshot.page_table_entry_count;
        state.stats.last_virtual_geometry_resident_page_count =
            runtime_snapshot.resident_page_count;
        state.stats.last_virtual_geometry_pending_request_count =
            runtime_snapshot.pending_request_count;
    } else {
        state.stats.last_virtual_geometry_visible_cluster_count = 0;
        state.stats.last_virtual_geometry_requested_page_count = 0;
        state.stats.last_virtual_geometry_dirty_page_count = 0;
        state.stats.last_virtual_geometry_page_table_entry_count = 0;
        state.stats.last_virtual_geometry_resident_page_count = 0;
        state.stats.last_virtual_geometry_pending_request_count = 0;
    }
    if let Some(profile) = quality_profile {
        state.stats.last_quality_profile = Some(profile);
    }
    Ok(())
}

use crate::core::framework::render::FrameHistoryHandle;

use crate::graphics::ViewportFrame;

use super::super::super::viewport_record::{ViewportCameraHistoryKey, ViewportRecord};
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;
use super::super::submission_record_update::{
    HybridGiStatSnapshot, ParticleStatSnapshot, SubmissionRecordUpdate, VirtualGeometryStatSnapshot,
};
use super::record_capture::record_capture;
use super::record_history::record_history;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn record_submission(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    allocated_history: Option<FrameHistoryHandle>,
    frame: ViewportFrame,
    runtime_feedback: RuntimeFeedbackBatch,
) -> SubmissionRecordUpdate {
    let (hybrid_gi_feedback, particle_feedback, virtual_geometry_feedback) =
        runtime_feedback.into_parts();
    let virtual_geometry_indirect_segment_count = 0;
    let (previous_handle, history_handle, history_status) =
        record_history(record, context, frame.generation, allocated_history);
    record_capture(record, context, frame);
    let hybrid_gi_stats =
        update_hybrid_gi_runtime(record, context.camera_history_key(), hybrid_gi_feedback);
    let particle_stats = particle_feedback_stat_snapshot(particle_feedback);
    let virtual_geometry_stats = update_virtual_geometry_runtime(
        record,
        context.camera_history_key(),
        virtual_geometry_feedback,
        virtual_geometry_indirect_segment_count,
    );

    SubmissionRecordUpdate::new(
        history_handle,
        previous_handle,
        history_status,
        record
            .last_capture()
            .map(|capture| capture.capture_report)
            .unwrap_or_else(|| {
                crate::core::framework::render::RenderCaptureReport::not_captured(
                    context.output_target().kind(),
                )
            }),
        hybrid_gi_stats,
        particle_stats,
        virtual_geometry_stats,
    )
}

pub(super) fn particle_feedback_stat_snapshot(
    feedback: crate::graphics::ParticleRuntimeFeedback,
) -> ParticleStatSnapshot {
    feedback
        .into_gpu_feedback()
        .map(|feedback| {
            let outputs = feedback.into_readback_outputs();
            ParticleStatSnapshot::new(
                outputs.alive_count as usize,
                outputs.spawned_total as usize,
                outputs.per_emitter_spawned.len(),
                outputs.indirect_draw_args,
            )
        })
        .unwrap_or_default()
}

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn update_hybrid_gi_runtime(
    record: &mut ViewportRecord,
    camera_history_key: &ViewportCameraHistoryKey,
    feedback: crate::graphics::HybridGiRuntimeFeedback,
) -> HybridGiStatSnapshot {
    let Some(runtime) = record.hybrid_gi_runtime_mut(camera_history_key) else {
        return HybridGiStatSnapshot::default();
    };
    let update = runtime.update_after_render(feedback);
    let stats = update.stats();
    HybridGiStatSnapshot::new(
        stats.cache_entry_count(),
        stats.resident_probe_count(),
        stats.pending_update_count(),
        stats.scheduled_trace_region_count(),
        stats.scene_card_count(),
        stats.scene_screen_probe_count(),
        stats.scene_radiance_cache_entry_count(),
        stats.surface_cache_resident_page_count(),
        stats.surface_cache_dirty_page_count(),
        stats.surface_cache_feedback_card_count(),
        stats.surface_cache_capture_slot_count(),
        stats.surface_cache_invalidated_page_count(),
        stats.voxel_resident_clipmap_count(),
        stats.voxel_dirty_clipmap_count(),
        stats.voxel_invalidated_clipmap_count(),
    )
}

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn update_virtual_geometry_runtime(
    record: &mut ViewportRecord,
    camera_history_key: &ViewportCameraHistoryKey,
    feedback: crate::graphics::VirtualGeometryRuntimeFeedback,
    indirect_segment_count: usize,
) -> VirtualGeometryStatSnapshot {
    let Some(runtime) = record.virtual_geometry_runtime_mut(camera_history_key) else {
        return VirtualGeometryStatSnapshot::default();
    };
    let update = runtime.update_after_render(feedback);
    let stats = update.stats();
    VirtualGeometryStatSnapshot::new(
        stats.page_table_entry_count(),
        stats.resident_page_count(),
        stats.pending_request_count(),
        stats.page_dependency_count(),
        stats.completed_page_count(),
        stats.replaced_page_count(),
        indirect_segment_count,
    )
}

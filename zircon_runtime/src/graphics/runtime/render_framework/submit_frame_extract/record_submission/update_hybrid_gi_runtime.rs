use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::submission_record_update::HybridGiStatSnapshot;
use crate::graphics::runtime::hybrid_gi::HybridGiRuntimeFeedback;

pub(super) fn update_hybrid_gi_runtime(
    prepared: &mut PreparedRuntimeSubmission,
    hybrid_gi_feedback: &HybridGiRuntimeFeedback,
) -> HybridGiStatSnapshot {
    if let Some(runtime) = prepared.hybrid_gi_runtime_mut() {
        if let Some(completion) = hybrid_gi_feedback.gpu_completion() {
            runtime.apply_gpu_cache_entries(completion.cache_entries());
            if let Some(scene_prepare_resources) = completion.scene_prepare_resources() {
                runtime.apply_scene_prepare_resources(scene_prepare_resources);
            }
            runtime.complete_gpu_updates(
                completion.completed_probe_ids().iter().copied(),
                completion.completed_trace_region_ids().iter().copied(),
                completion.probe_irradiance_rgb(),
                completion.probe_trace_lighting_rgb(),
                hybrid_gi_feedback.evictable_probe_ids(),
            );
        } else if let Some(feedback) = hybrid_gi_feedback.visibility_feedback() {
            runtime.consume_feedback(feedback);
        }
        let snapshot = runtime.snapshot();
        HybridGiStatSnapshot::new(
            snapshot.cache_entry_count(),
            snapshot.resident_probe_count(),
            snapshot.pending_update_count(),
            snapshot.scheduled_trace_region_count(),
            snapshot.scene_card_count(),
            snapshot.surface_cache_resident_page_count(),
            snapshot.surface_cache_dirty_page_count(),
            snapshot.surface_cache_feedback_card_count(),
            snapshot.surface_cache_capture_slot_count(),
            snapshot.surface_cache_invalidated_page_count(),
            snapshot.voxel_resident_clipmap_count(),
            snapshot.voxel_dirty_clipmap_count(),
            snapshot.voxel_invalidated_clipmap_count(),
        )
    } else {
        HybridGiStatSnapshot::default()
    }
}

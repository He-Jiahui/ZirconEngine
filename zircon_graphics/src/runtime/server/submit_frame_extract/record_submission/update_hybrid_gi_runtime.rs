use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::gpu_completion::HybridGiGpuCompletion;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::submission_record_update::HybridGiStatSnapshot;

pub(super) fn update_hybrid_gi_runtime(
    context: &FrameSubmissionContext,
    prepared: &mut PreparedRuntimeSubmission,
    hybrid_gi_gpu_completion: Option<&HybridGiGpuCompletion>,
) -> HybridGiStatSnapshot {
    if let Some(runtime) = prepared.hybrid_gi_runtime.as_mut() {
        if let Some(completion) = hybrid_gi_gpu_completion {
            runtime.apply_gpu_cache_entries(&completion.cache_entries);
            runtime.complete_gpu_updates(
                completion.completed_probe_ids.iter().copied(),
                completion.completed_trace_region_ids.iter().copied(),
                &completion.probe_irradiance_rgb,
                &prepared.hybrid_gi_evictable_probe_ids,
            );
        } else if let Some(feedback) = context.hybrid_gi_feedback.as_ref() {
            runtime.consume_feedback(feedback);
        }
        let snapshot = runtime.snapshot();
        HybridGiStatSnapshot {
            cache_entry_count: snapshot.cache_entry_count,
            resident_probe_count: snapshot.resident_probe_count,
            pending_update_count: snapshot.pending_update_count,
            scheduled_trace_region_count: snapshot.scheduled_trace_region_count,
        }
    } else {
        HybridGiStatSnapshot::default()
    }
}

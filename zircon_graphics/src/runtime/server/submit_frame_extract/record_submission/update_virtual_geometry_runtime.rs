use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::gpu_completion::VirtualGeometryGpuCompletion;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::submission_record_update::VirtualGeometryStatSnapshot;
use crate::types::VirtualGeometryPrepareClusterState;

pub(super) fn update_virtual_geometry_runtime(
    context: &FrameSubmissionContext,
    prepared: &mut PreparedRuntimeSubmission,
    virtual_geometry_gpu_completion: Option<&VirtualGeometryGpuCompletion>,
) -> VirtualGeometryStatSnapshot {
    let (indirect_draw_count, indirect_segment_count) = prepared
        .virtual_geometry_prepare
        .as_ref()
        .map(|prepare| {
            (
                prepare.unified_indirect_draws().len(),
                prepare
                    .cluster_draw_segments
                    .iter()
                    .filter(|segment| {
                        !matches!(segment.state, VirtualGeometryPrepareClusterState::Missing)
                    })
                    .count(),
            )
        })
        .unwrap_or_default();
    let completed_page_count = virtual_geometry_gpu_completion
        .map(|completion| completion.completed_page_assignments.len())
        .unwrap_or(0);

    if let Some(runtime) = prepared.virtual_geometry_runtime.as_mut() {
        if let Some(completion) = virtual_geometry_gpu_completion {
            runtime.apply_gpu_page_table_entries(&completion.page_table_entries);
            runtime.complete_gpu_uploads_with_slots(
                completion.completed_page_assignments.iter().copied(),
                &prepared.virtual_geometry_evictable_page_ids,
            );
        } else if let Some(feedback) = context.virtual_geometry_feedback.as_ref() {
            runtime.consume_feedback(feedback);
        }
        let snapshot = runtime.snapshot();
        VirtualGeometryStatSnapshot {
            page_table_entry_count: snapshot.page_table_entry_count,
            resident_page_count: snapshot.resident_page_count,
            pending_request_count: snapshot.pending_request_count,
            completed_page_count,
            indirect_draw_count,
            indirect_segment_count,
        }
    } else {
        VirtualGeometryStatSnapshot {
            completed_page_count,
            indirect_draw_count,
            indirect_segment_count,
            ..VirtualGeometryStatSnapshot::default()
        }
    }
}

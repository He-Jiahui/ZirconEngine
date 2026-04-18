use zircon_framework::render::FrameHistoryHandle;

use crate::ViewportFrame;

use super::super::super::viewport_record::ViewportRecord;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::gpu_completion::{HybridGiGpuCompletion, VirtualGeometryGpuCompletion};
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::record_capture::record_capture;
use super::record_history::record_history;
use super::update_hybrid_gi_runtime::update_hybrid_gi_runtime;
use super::update_virtual_geometry_runtime::update_virtual_geometry_runtime;

pub(in crate::runtime::render_framework::submit_frame_extract) fn record_submission(
    record: &mut ViewportRecord,
    context: &FrameSubmissionContext,
    prepared: PreparedRuntimeSubmission,
    allocated_history: Option<FrameHistoryHandle>,
    frame: ViewportFrame,
    hybrid_gi_gpu_completion: Option<HybridGiGpuCompletion>,
    virtual_geometry_gpu_completion: Option<VirtualGeometryGpuCompletion>,
) -> SubmissionRecordUpdate {
    let mut prepared = prepared;
    let (previous_handle, history_handle) =
        record_history(record, context, &frame, allocated_history);
    record_capture(record, context, frame);
    let hybrid_gi_stats =
        update_hybrid_gi_runtime(context, &mut prepared, hybrid_gi_gpu_completion.as_ref());
    let virtual_geometry_stats = update_virtual_geometry_runtime(
        context,
        &mut prepared,
        virtual_geometry_gpu_completion.as_ref(),
    );

    record.hybrid_gi_runtime = prepared.hybrid_gi_runtime;
    record.virtual_geometry_runtime = prepared.virtual_geometry_runtime;

    SubmissionRecordUpdate {
        history_handle,
        previous_handle,
        hybrid_gi_stats,
        virtual_geometry_stats,
    }
}

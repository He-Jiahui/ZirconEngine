use zr_rhi::{
    DeviceGeneration, DeviceId, RenderQueueClass, SubmissionPollReceipt, SubmissionTicket,
};

use super::{
    ensure_retirement_capacity, RenderAssetGpuResidencyLimits, RenderAssetGpuResidencyState,
    RenderAssetGpuRetirementBackpressure,
};
use crate::graphics::scene::resources::render_asset_residency::{
    RenderAssetDeviceEpoch, RenderAssetResidencyTransitionError,
};

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(31),
        DeviceGeneration::new(4),
        RenderQueueClass::Copy,
        sequence,
    )
}

#[test]
fn gpu_tracking_rejects_duplicates_and_capacity_before_state_mutation() {
    let mut state = RenderAssetGpuResidencyState::new(RenderAssetGpuResidencyLimits::new(1, 1));
    assert_eq!(state.ensure_can_track(ticket(1)), Ok(()));
    assert!(state.tracked_submissions.insert(ticket(1)));

    assert_eq!(
        state.ensure_can_track(ticket(1)),
        Err(
            RenderAssetResidencyTransitionError::SubmissionAlreadyTracked {
                submission: ticket(1),
            }
        )
    );
    assert_eq!(
        state.ensure_can_track(ticket(2)),
        Err(
            RenderAssetResidencyTransitionError::GpuTrackingBackpressure {
                tracked_submissions: 1,
                limit: 1,
            }
        )
    );
    assert_eq!(state.tracked_submissions.len(), 1);
}

#[test]
fn retirement_capacity_rejects_full_and_overflowing_batches_without_saturation() {
    assert_eq!(ensure_retirement_capacity(1, 2, 3), Ok(()));
    let full = RenderAssetGpuRetirementBackpressure {
        ready_retirements: 2,
        requested_retirements: 2,
        limit: 3,
    };
    assert_eq!(ensure_retirement_capacity(2, 2, 3), Err(full));

    let overflow = RenderAssetGpuRetirementBackpressure {
        ready_retirements: usize::MAX,
        requested_retirements: 1,
        limit: usize::MAX,
    };
    assert_eq!(
        ensure_retirement_capacity(usize::MAX, 1, usize::MAX),
        Err(overflow)
    );
}

#[test]
fn device_recovery_resets_completion_stream_and_rejects_implicit_epoch_changes() {
    let old = RenderAssetDeviceEpoch::new(DeviceId::new(31), DeviceGeneration::new(4));
    let replacement = RenderAssetDeviceEpoch::new(DeviceId::new(31), DeviceGeneration::new(5));
    let mut state = RenderAssetGpuResidencyState::default();
    let old_poll = SubmissionPollReceipt::new(old.device_id(), old.generation(), 7);
    state.record_poll_receipt(old_poll);

    let replacement_submission = SubmissionTicket::new(
        replacement.device_id(),
        replacement.generation(),
        RenderQueueClass::Copy,
        1,
    );
    assert!(matches!(
        state.ensure_can_track(replacement_submission),
        Err(RenderAssetResidencyTransitionError::SubmissionDeviceMismatch { .. })
    ));

    let abandoned = state.abandon_for_device_recovery(replacement);
    assert_eq!(abandoned, super::RenderAssetGpuAbandonReport::default());
    assert_eq!(state.bound_device_epoch(), Some(replacement));
    assert_eq!(state.last_poll_receipt(), None);
    assert_eq!(state.ensure_can_track(replacement_submission), Ok(()));
}

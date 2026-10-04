use zr_rhi::{
    DeviceGeneration, DeviceId, RenderQueueClass, SubmissionPollReceipt, SubmissionTicket,
};

use super::{
    validate_poll_receipt, RenderAssetGpuPollReceiptError, RenderAssetGpuSubmissionFrontier,
};
use crate::graphics::scene::resources::render_asset_residency::RenderAssetDeviceEpoch;

fn ticket(device: u64, generation: u64, sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(device),
        DeviceGeneration::new(generation),
        RenderQueueClass::Copy,
        sequence,
    )
}

#[test]
fn submission_observation_budget_rotates_fairly_without_full_frontier_scans() {
    let mut frontier = RenderAssetGpuSubmissionFrontier::default();
    for sequence in 1..=4 {
        assert!(frontier.insert(ticket(7, 3, sequence)));
    }
    let mut observed = Vec::with_capacity(2);

    assert_eq!(frontier.append_next_batch(2, &mut observed), 2);
    assert_eq!(observed, vec![ticket(7, 3, 1), ticket(7, 3, 2)]);
    observed.clear();
    assert_eq!(frontier.append_next_batch(2, &mut observed), 2);
    assert_eq!(observed, vec![ticket(7, 3, 3), ticket(7, 3, 4)]);
    observed.clear();
    assert_eq!(frontier.append_next_batch(2, &mut observed), 2);
    assert_eq!(observed, vec![ticket(7, 3, 1), ticket(7, 3, 2)]);
    assert_eq!(frontier.len(), 4);
}

#[test]
fn submission_observation_identity_includes_device_generation_and_removal_is_exact() {
    let mut frontier = RenderAssetGpuSubmissionFrontier::default();
    let old = ticket(9, 1, 1);
    let recreated = ticket(9, 2, 1);
    assert!(frontier.insert(old));
    assert!(frontier.insert(recreated));
    assert!(!frontier.insert(old));

    assert!(frontier.remove(old));
    assert!(!frontier.remove(old));
    let mut observed = Vec::new();
    assert_eq!(frontier.append_next_batch(8, &mut observed), 1);
    assert_eq!(observed, vec![recreated]);
    assert_eq!(frontier.len(), 1);
}

#[test]
fn asset_maintenance_consumes_owner_poll_results_without_scanning_residency_entries() {
    let source = include_str!("../gpu_maintenance.rs");
    let artifact_source = include_str!("../gpu_upload/submit.rs");

    assert!(source.contains("budget.max_submission_status_checks()"));
    assert!(source.contains("budget.max_artifact_retirements()"));
    assert!(source.contains("device.append_submission_statuses(&observations, &mut statuses)"));
    // BUG: [CR-R02-runtime_wave12_graphics_resource_residency-0002] 扫描完整源码时，左侧首次匹配命中测试自身，右侧先命中生产调用，顺序断言恒失败；生产校验调用的参数跨行。
    assert!(
        source
            .find("validate_poll_receipt(expected")
            .unwrap_or(usize::MAX)
            < source
                .find("device.append_submission_statuses(")
                .unwrap_or(usize::MAX)
    );
    assert!(source.contains(".min(self.ready_gpu_retirement_count())"));
    assert!(source.contains("self.gpu.enqueue_retirement(artifact)"));
    assert!(source.contains("report.deferred_terminal_uploads"));
    assert!(artifact_source.contains("retirement_progress: u8"));
    assert!(artifact_source.contains("Result<(), (Self, RhiError)>"));
    for forbidden in [
        ["device", ".poll_submissions("].concat(),
        ["self.entries", ".iter("].concat(),
        ["self.entries", ".values("].concat(),
    ] {
        assert!(!source.contains(&forbidden));
    }
}

#[test]
fn poll_receipt_requires_matching_device_generation_and_strict_progress() {
    let expected = RenderAssetDeviceEpoch::new(DeviceId::new(5), DeviceGeneration::new(7));
    let first = SubmissionPollReceipt::new(expected.device_id(), expected.generation(), 11);
    let next = SubmissionPollReceipt::new(expected.device_id(), expected.generation(), 12);
    assert_eq!(validate_poll_receipt(expected, None, None, first), Ok(()));
    assert_eq!(
        validate_poll_receipt(expected, Some(expected), Some(first), next),
        Ok(())
    );
    assert!(matches!(
        validate_poll_receipt(expected, Some(expected), Some(first), first),
        Err(RenderAssetGpuPollReceiptError::NotAdvanced { .. })
    ));

    let foreign = SubmissionPollReceipt::new(DeviceId::new(6), DeviceGeneration::new(7), 13);
    assert!(matches!(
        validate_poll_receipt(expected, Some(expected), Some(first), foreign),
        Err(RenderAssetGpuPollReceiptError::DeviceMismatch { .. })
    ));

    let replacement = RenderAssetDeviceEpoch::new(DeviceId::new(5), DeviceGeneration::new(8));
    let replacement_poll =
        SubmissionPollReceipt::new(replacement.device_id(), replacement.generation(), 1);
    assert!(matches!(
        validate_poll_receipt(replacement, Some(expected), None, replacement_poll),
        Err(RenderAssetGpuPollReceiptError::BoundEpochMismatch { .. })
    ));
}

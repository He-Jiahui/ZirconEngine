use crate::{
    DeviceGeneration, DeviceId, RenderQueueClass, RhiGraphAccessId, RhiGraphAccessRange,
    RhiGraphExecutionAccess, RhiGraphExecutionPass, RhiGraphExecutionReceipt,
    RhiGraphExecutionTransition, RhiGraphPhysicalResourceLease, RhiGraphQueueLane,
    RhiGraphResourceAccessKind, RhiGraphResourceBounds, RhiGraphResourceId, RhiGraphResourceKind,
    RhiGraphResourceState, SubmissionHistory, SubmissionLimits, SubmissionStatus, SubmissionTicket,
};
use std::sync::Arc;

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(41),
        DeviceGeneration::initial(),
        RenderQueueClass::Copy,
        sequence,
    )
}

#[test]
fn submission_ticket_binds_device_generation_queue_and_sequence() {
    let ticket = SubmissionTicket::new(
        DeviceId::new(41),
        DeviceGeneration::initial(),
        RenderQueueClass::Compute,
        17,
    );

    assert_eq!(ticket.device_id(), DeviceId::new(41));
    assert_eq!(ticket.generation(), DeviceGeneration::initial());
    assert_eq!(ticket.queue_class(), RenderQueueClass::Compute);
    assert_eq!(ticket.sequence(), 17);
}

#[test]
fn only_submission_terminal_states_allow_completion_consumers_to_advance() {
    assert!(!SubmissionStatus::Accepted.is_terminal());
    assert!(!SubmissionStatus::Submitted.is_terminal());
    assert!(SubmissionStatus::Completed.is_terminal());
    assert!(SubmissionStatus::Failed.is_terminal());
    assert!(SubmissionStatus::Cancelled.is_terminal());
    assert!(SubmissionStatus::DeviceLost.is_terminal());
}

#[test]
fn submission_history_bounds_observable_statuses_without_losing_retirement_safety() {
    let mut history = SubmissionHistory::new(SubmissionLimits::new(2, 1));
    let first = ticket(1);
    let second = ticket(2);
    let third = ticket(3);

    assert!(history.record_accepted(first));
    assert!(history.record_accepted(second));
    assert!(!history.can_accept());
    history.transition(second, SubmissionStatus::Cancelled);
    assert!(history.record_accepted(third));
    history.transition(third, SubmissionStatus::Cancelled);

    assert_eq!(history.status(second), None);
    assert!(history.is_terminal(second));
    assert!(history.is_terminal(third));
    assert!(!history.is_terminal(first));
    assert_eq!(history.unresolved_count(), 1);

    history.transition(first, SubmissionStatus::Completed);
    assert!(history.is_terminal(first));
    assert_eq!(history.unresolved_count(), 0);
}

#[test]
fn submission_history_terminal_ranges_stay_bounded_by_unresolved_gaps() {
    let mut history = SubmissionHistory::new(SubmissionLimits::new(3, 0));
    let first = ticket(1);
    assert!(history.record_accepted(first));

    for sequence in 2..32 {
        let current = ticket(sequence);
        assert!(history.record_accepted(current));
        history.transition(current, SubmissionStatus::Cancelled);
    }

    assert_eq!(history.unresolved_count(), 1);
    assert_eq!(history.terminal_range_count(), 1);
    assert!(history.is_terminal(ticket(31)));
    assert!(!history.is_terminal(first));
}

#[test]
fn graph_submission_receipt_binds_exact_ordered_access_range_and_queue_transition() {
    let device_id = DeviceId::new(41);
    let generation = DeviceGeneration::initial();
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::Buffer, 9, 77);
    let range = RhiGraphAccessRange::buffer(32, 64);
    let writer_id = RhiGraphAccessId::new(0, 77, 0);
    let reader_id = RhiGraphAccessId::new(1, 77, 0);
    let writer = RhiGraphExecutionAccess::new(
        writer_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Write,
        range,
        RhiGraphResourceState::StorageBufferReadWrite,
        RhiGraphQueueLane::AsyncCompute,
        Some(3),
        true,
    );
    let reader = RhiGraphExecutionAccess::new(
        reader_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Read,
        range,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::Graphics,
        Some(3),
        true,
    );
    let transition = RhiGraphExecutionTransition::new(
        resource,
        range,
        writer_id,
        reader_id,
        RhiGraphResourceState::StorageBufferReadWrite,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::AsyncCompute,
        RhiGraphQueueLane::Graphics,
    );
    let receipt = RhiGraphExecutionReceipt::new(
        device_id,
        generation,
        12,
        77,
        RenderQueueClass::Graphics,
        vec![
            RhiGraphExecutionPass::new(
                0,
                0,
                77,
                RhiGraphQueueLane::AsyncCompute,
                vec![writer],
                vec![],
            ),
            RhiGraphExecutionPass::new(
                1,
                1,
                77,
                RhiGraphQueueLane::Graphics,
                vec![reader],
                vec![transition.clone()],
            ),
        ],
        vec![
            RhiGraphPhysicalResourceLease::new(
                writer_id,
                resource,
                Some(3),
                device_id,
                generation,
                RhiGraphResourceBounds::buffer(128).unwrap(),
                Arc::new(()),
            ),
            RhiGraphPhysicalResourceLease::new(
                reader_id,
                resource,
                Some(3),
                device_id,
                generation,
                RhiGraphResourceBounds::buffer(128).unwrap(),
                Arc::new(()),
            ),
        ],
    )
    .expect("receipt must preserve the exact producer-to-consumer device proof");

    assert_eq!(receipt.frame_generation(), 12);
    assert_eq!(receipt.device_id(), device_id);
    assert_eq!(receipt.generation(), generation);
    assert_eq!(receipt.passes().len(), 2);
    assert_eq!(receipt.passes()[0].accesses()[0].range(), range);
    assert_eq!(receipt.passes()[1].transitions_before(), &[transition]);
    assert_eq!(receipt.physical_lease_count(), 2);
}

#[test]
fn graph_submission_receipt_rejects_access_outside_physical_buffer_lease() {
    let device_id = DeviceId::new(41);
    let generation = DeviceGeneration::initial();
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::Buffer, 9, 77);
    let access_id = RhiGraphAccessId::new(0, 77, 0);
    let access = RhiGraphExecutionAccess::new(
        access_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Write,
        RhiGraphAccessRange::buffer(96, 64),
        RhiGraphResourceState::StorageBufferReadWrite,
        RhiGraphQueueLane::Graphics,
        Some(3),
        true,
    );

    let error = RhiGraphExecutionReceipt::new(
        device_id,
        generation,
        12,
        77,
        RenderQueueClass::Graphics,
        vec![RhiGraphExecutionPass::new(
            0,
            0,
            77,
            RhiGraphQueueLane::Graphics,
            vec![access],
            vec![],
        )],
        vec![RhiGraphPhysicalResourceLease::new(
            access_id,
            resource,
            Some(3),
            device_id,
            generation,
            RhiGraphResourceBounds::buffer(128).unwrap(),
            Arc::new(()),
        )],
    )
    .expect_err("an out-of-range access must fail before device ticket admission");

    assert!(error.to_string().contains("exceeds physical buffer lease"));
}

#[test]
fn graph_submission_receipt_rejects_a_transition_whose_source_was_not_recorded_first() {
    let device_id = DeviceId::new(41);
    let generation = DeviceGeneration::initial();
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::Buffer, 9, 77);
    let range = RhiGraphAccessRange::buffer(32, 64);
    let writer_id = RhiGraphAccessId::new(0, 77, 0);
    let reader_id = RhiGraphAccessId::new(1, 77, 0);
    let writer = RhiGraphExecutionAccess::new(
        writer_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Write,
        range,
        RhiGraphResourceState::StorageBufferReadWrite,
        RhiGraphQueueLane::AsyncCompute,
        Some(3),
        true,
    );
    let reader = RhiGraphExecutionAccess::new(
        reader_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Read,
        range,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::Graphics,
        Some(3),
        true,
    );
    let transition = RhiGraphExecutionTransition::new(
        resource,
        range,
        writer_id,
        reader_id,
        RhiGraphResourceState::StorageBufferReadWrite,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::AsyncCompute,
        RhiGraphQueueLane::Graphics,
    );

    let error = RhiGraphExecutionReceipt::new(
        device_id,
        generation,
        12,
        77,
        RenderQueueClass::Graphics,
        vec![
            RhiGraphExecutionPass::new(
                0,
                1,
                77,
                RhiGraphQueueLane::Graphics,
                vec![reader],
                vec![transition],
            ),
            RhiGraphExecutionPass::new(
                1,
                0,
                77,
                RhiGraphQueueLane::AsyncCompute,
                vec![writer],
                vec![],
            ),
        ],
        vec![
            RhiGraphPhysicalResourceLease::new(
                writer_id,
                resource,
                Some(3),
                device_id,
                generation,
                RhiGraphResourceBounds::buffer(128).unwrap(),
                Arc::new(()),
            ),
            RhiGraphPhysicalResourceLease::new(
                reader_id,
                resource,
                Some(3),
                device_id,
                generation,
                RhiGraphResourceBounds::buffer(128).unwrap(),
                Arc::new(()),
            ),
        ],
    )
    .expect_err("queue lowering must not reorder a producer after its consumer");

    assert!(error
        .to_string()
        .contains("source access must precede destination"));
}

#[test]
fn graph_submission_receipt_rejects_stale_graph_generation() {
    let device_id = DeviceId::new(41);
    let generation = DeviceGeneration::initial();
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::Buffer, 9, 77);
    let access_id = RhiGraphAccessId::new(0, 78, 0);
    let access = RhiGraphExecutionAccess::new(
        access_id,
        resource,
        1,
        RhiGraphResourceAccessKind::Write,
        RhiGraphAccessRange::buffer(0, 16),
        RhiGraphResourceState::StorageBufferReadWrite,
        RhiGraphQueueLane::Graphics,
        Some(3),
        true,
    );
    let error = RhiGraphExecutionReceipt::new(
        device_id,
        generation,
        12,
        78,
        RenderQueueClass::Graphics,
        vec![RhiGraphExecutionPass::new(
            0,
            0,
            78,
            RhiGraphQueueLane::Graphics,
            vec![access],
            vec![],
        )],
        vec![RhiGraphPhysicalResourceLease::new(
            access_id,
            resource,
            Some(3),
            device_id,
            generation,
            RhiGraphResourceBounds::buffer(32).unwrap(),
            Arc::new(()),
        )],
    )
    .expect_err("a stale graph generation must fail before ticket admission");

    assert!(error
        .to_string()
        .contains("graph resource generation does not match receipt"));
}

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

fn optional_absence_receipt(
    accesses: Vec<RhiGraphExecutionAccess>,
    leases: Vec<RhiGraphPhysicalResourceLease>,
) -> Result<RhiGraphExecutionReceipt, crate::RhiGraphExecutionReceiptError> {
    RhiGraphExecutionReceipt::new(
        DeviceId::new(41),
        DeviceGeneration::initial(),
        12,
        77,
        RenderQueueClass::Graphics,
        vec![RhiGraphExecutionPass::new(
            0,
            0,
            77,
            RhiGraphQueueLane::Graphics,
            accesses,
            vec![],
        )],
        leases,
    )
}

fn optional_absence_access(
    id: RhiGraphAccessId,
    resource: RhiGraphResourceId,
    range: RhiGraphAccessRange,
    allocation: Option<u64>,
    physical: bool,
) -> RhiGraphExecutionAccess {
    RhiGraphExecutionAccess::new(
        id,
        resource,
        1,
        RhiGraphResourceAccessKind::Read,
        range,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::Graphics,
        allocation,
        physical,
    )
}

#[test]
fn graph_submission_optional_external_absence_retains_typed_buffer_and_texture_ranges() {
    let buffer_range = RhiGraphAccessRange::buffer(32, 64);
    let texture_range = RhiGraphAccessRange::texture(1, 3, 0, 1, 1);
    let buffer_bounds = RhiGraphResourceBounds::buffer(128).unwrap();
    let texture_bounds = RhiGraphResourceBounds::texture(4, 1, 1).unwrap();
    let buffer = optional_absence_access(
        RhiGraphAccessId::new(0, 77, 0),
        RhiGraphResourceId::new(RhiGraphResourceKind::External, 9, 77),
        buffer_range,
        None,
        false,
    );
    assert!(
        optional_absence_receipt(vec![buffer.clone()], vec![]).is_err(),
        "default typed accesses still require a physical lease"
    );
    let texture = optional_absence_access(
        RhiGraphAccessId::new(0, 77, 1),
        RhiGraphResourceId::new(RhiGraphResourceKind::External, 10, 77),
        texture_range,
        None,
        false,
    );
    let receipt = optional_absence_receipt(
        vec![
            buffer
                .with_optional_external_absence(buffer_bounds)
                .unwrap(),
            texture
                .with_optional_external_absence(texture_bounds)
                .unwrap(),
        ],
        vec![],
    )
    .expect("explicit optional absence retains both typed scopes without fake leases");
    let accesses = receipt.passes()[0].accesses();
    assert_eq!(accesses[0].range(), buffer_range);
    assert_eq!(accesses[1].range(), texture_range);
    assert_eq!(
        accesses[0].optional_external_absence_bounds(),
        Some(buffer_bounds)
    );
    assert_eq!(
        accesses[1].optional_external_absence_bounds(),
        Some(texture_bounds)
    );
    assert!(!accesses[0].has_physical_binding());
    assert!(!accesses[1].has_physical_binding());
    assert_eq!(receipt.physical_lease_count(), 0);
}

#[test]
fn graph_submission_optional_external_absence_rejects_origin_bounds_and_physical_leases() {
    let id = RhiGraphAccessId::new(0, 77, 0);
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::External, 9, 77);
    let range = RhiGraphAccessRange::buffer(32, 64);
    let bounds = RhiGraphResourceBounds::buffer(128).unwrap();
    for (resource_kind, allocation, physical, candidate_range, candidate_bounds) in [
        (RhiGraphResourceKind::Buffer, None, false, range, bounds),
        (RhiGraphResourceKind::Texture, None, false, range, bounds),
        (
            RhiGraphResourceKind::External,
            Some(3),
            false,
            range,
            bounds,
        ),
        (RhiGraphResourceKind::External, None, true, range, bounds),
        (
            RhiGraphResourceKind::External,
            None,
            false,
            RhiGraphAccessRange::UnresolvedExternal,
            bounds,
        ),
        (
            RhiGraphResourceKind::External,
            None,
            false,
            range,
            RhiGraphResourceBounds::buffer(64).unwrap(),
        ),
        (
            RhiGraphResourceKind::External,
            None,
            false,
            range,
            RhiGraphResourceBounds::texture(4, 1, 1).unwrap(),
        ),
        (
            RhiGraphResourceKind::External,
            None,
            false,
            RhiGraphAccessRange::buffer(u64::MAX, 2),
            bounds,
        ),
        (
            RhiGraphResourceKind::External,
            None,
            false,
            RhiGraphAccessRange::texture(0, 2, 0, 1, 2),
            RhiGraphResourceBounds::texture(2, 1, 1).unwrap(),
        ),
    ] {
        let invalid = optional_absence_access(
            id,
            RhiGraphResourceId::new(resource_kind, 9, 77),
            candidate_range,
            allocation,
            physical,
        );
        assert!(
            invalid
                .with_optional_external_absence(candidate_bounds)
                .is_err(),
            "absence cannot bypass origin, allocation, presence, range kind, overflow, or aspects"
        );
    }
    let access = optional_absence_access(id, resource, range, None, false)
        .with_optional_external_absence(bounds)
        .unwrap();
    let lease = RhiGraphPhysicalResourceLease::new(
        id,
        resource,
        None,
        DeviceId::new(41),
        DeviceGeneration::initial(),
        bounds,
        Arc::new(()),
    );
    let error = optional_absence_receipt(vec![access.clone()], vec![lease])
        .expect_err("an absent external must never carry a physical lease");
    assert!(error
        .to_string()
        .contains("logical-only graph access has a physical lease"));
    let stale_lease = RhiGraphPhysicalResourceLease::new(
        id,
        resource,
        None,
        DeviceId::new(41),
        DeviceGeneration::new(2),
        bounds,
        Arc::new(()),
    );
    let error = optional_absence_receipt(vec![access], vec![stale_lease])
        .expect_err("absence must not bypass device generation checks");
    assert!(error
        .to_string()
        .contains("physical lease device generation"));
}

#[test]
fn graph_submission_optional_external_absence_preserves_identity_generation_and_transitions() {
    let resource = RhiGraphResourceId::new(RhiGraphResourceKind::External, 9, 77);
    let range = RhiGraphAccessRange::buffer(0, 16);
    let bounds = RhiGraphResourceBounds::buffer(32).unwrap();
    for (id, resource) in [
        (RhiGraphAccessId::new(0, 77, 1), resource),
        (
            RhiGraphAccessId::new(0, 77, 0),
            RhiGraphResourceId::new(RhiGraphResourceKind::External, 9, 78),
        ),
    ] {
        let access = optional_absence_access(id, resource, range, None, false)
            .with_optional_external_absence(bounds)
            .unwrap();
        assert!(
            optional_absence_receipt(vec![access], vec![]).is_err(),
            "typed absence does not waive access ordinal or resource generation"
        );
    }
    let from_id = RhiGraphAccessId::new(0, 77, 0);
    let to_id = RhiGraphAccessId::new(1, 77, 0);
    let from = optional_absence_access(from_id, resource, range, None, false)
        .with_optional_external_absence(bounds)
        .unwrap();
    let to = optional_absence_access(to_id, resource, range, None, false)
        .with_optional_external_absence(bounds)
        .unwrap();
    let transition = RhiGraphExecutionTransition::new(
        resource,
        range,
        from_id,
        to_id,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::Graphics,
        RhiGraphQueueLane::Graphics,
    );
    let receipt_for = |transition| {
        RhiGraphExecutionReceipt::new(
            DeviceId::new(41),
            DeviceGeneration::initial(),
            12,
            77,
            RenderQueueClass::Graphics,
            vec![
                RhiGraphExecutionPass::new(
                    0,
                    0,
                    77,
                    RhiGraphQueueLane::Graphics,
                    vec![from.clone()],
                    vec![],
                ),
                RhiGraphExecutionPass::new(
                    1,
                    1,
                    77,
                    RhiGraphQueueLane::Graphics,
                    vec![to.clone()],
                    vec![transition],
                ),
            ],
            vec![],
        )
    };
    assert!(
        receipt_for(transition).is_ok(),
        "valid optional scopes retain their transition proof"
    );
    let wrong_transition = RhiGraphExecutionTransition::new(
        resource,
        RhiGraphAccessRange::buffer(16, 16),
        from_id,
        to_id,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphResourceState::StorageBufferRead,
        RhiGraphQueueLane::Graphics,
        RhiGraphQueueLane::Graphics,
    );
    assert!(
        receipt_for(wrong_transition).is_err(),
        "absence cannot widen transition ranges"
    );
}

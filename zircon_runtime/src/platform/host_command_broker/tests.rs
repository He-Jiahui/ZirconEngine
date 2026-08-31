use std::num::{NonZeroU32, NonZeroUsize};
use std::time::{Duration, Instant};

use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayTopologyGeneration, WindowCommandTerminal, WindowCreateSpec,
    WindowDisplayTarget, WindowEffectiveMode, WindowEffectivePlacement, WindowEffectiveState,
    WindowFocusState, WindowId, WindowLogicalExtent, WindowLogicalPosition, WindowObservedMode,
    WindowObservedState, WindowOcclusionState, WindowPhysicalExtent, WindowPlacementRequest,
    WindowRegistryId, WindowRequestedMode, WindowRequestedState, WindowStateResizeConstraints,
    WindowStateSnapshot, WindowVisibilityState,
};
use crate::platform::WindowStateRegistry;

use super::{HostCommandBroker, HostCommandBrokerError, HostCommandDispatch};

fn window() -> WindowId {
    WindowId::new(
        WindowRegistryId::new(73).expect("fixture registry identity is nonzero"),
        4,
        NonZeroU32::new(2).expect("fixture window generation is nonzero"),
    )
}

fn display() -> DisplayId {
    DisplayId::new(DisplayKind::PhysicalOutput, "edid:panel-a")
        .expect("fixture display identity is valid")
}

fn topology_generation() -> DisplayTopologyGeneration {
    DisplayTopologyGeneration::new(11).expect("fixture topology generation is nonzero")
}

fn physical_extent() -> WindowPhysicalExtent {
    WindowPhysicalExtent::new(1920, 1080).expect("fixture physical extent is valid")
}

fn logical_extent() -> WindowLogicalExtent {
    WindowLogicalExtent::new(960.0, 540.0).expect("fixture logical extent is valid")
}

fn constraints() -> WindowStateResizeConstraints {
    WindowStateResizeConstraints::new(
        WindowLogicalExtent::new(320.0, 180.0).expect("fixture minimum is valid"),
        Some(logical_extent()),
    )
    .expect("fixture constraints are valid")
}

fn requested(title: &str) -> WindowRequestedState {
    WindowRequestedState::new(
        title,
        WindowPlacementRequest::CenteredOn(WindowDisplayTarget::Display(display())),
        WindowRequestedMode::Windowed,
        physical_extent(),
        constraints(),
        true,
        true,
        true,
    )
}

fn current_state() -> WindowStateSnapshot {
    let observed = WindowObservedState::new(
        display(),
        topology_generation(),
        physical_extent(),
        logical_extent(),
        WindowLogicalPosition::new(80.0, 40.0).expect("fixture position is valid"),
        2.0,
        WindowObservedMode::Windowed,
        WindowFocusState::Focused,
        WindowVisibilityState::Visible,
        WindowOcclusionState::Unoccluded,
    )
    .expect("fixture observed state is valid");
    let effective = WindowEffectiveState::new(
        "Zircon Runtime",
        WindowEffectivePlacement::new(
            display(),
            WindowLogicalPosition::new(80.0, 40.0).expect("fixture position is valid"),
        ),
        WindowEffectiveMode::Windowed,
        physical_extent(),
        constraints(),
        true,
        true,
        true,
        topology_generation(),
    )
    .expect("fixture effective state is valid");
    let mut states = WindowStateRegistry::default();
    states
        .register(
            window(),
            WindowCreateSpec::new(requested("Zircon Runtime"), topology_generation()),
            observed,
            effective,
        )
        .expect("fixture state registers")
}

fn broker(limit: usize) -> HostCommandBroker {
    HostCommandBroker::new(NonZeroUsize::new(limit).expect("fixture broker limit is nonzero"))
}

#[test]
fn admission_dispatch_and_completion_preserve_the_target_generation_and_terminal_receipt() {
    let mut broker = broker(4);
    let state = current_state();
    let submitted_at = Instant::now();
    let accepted = broker
        .submit(
            window(),
            submitted_at + Duration::from_secs(1),
            requested("Updated Title"),
            &state,
            submitted_at,
        )
        .expect("live matching snapshot is admitted");

    assert_eq!(accepted.header().target(), window());
    assert_eq!(accepted.header().request_id().raw(), 1);
    assert_eq!(broker.pending_len(), 1);
    assert_eq!(broker.next_target(), Some(window()));

    let command = match broker
        .dispatch_next(submitted_at, &state)
        .expect("matching snapshot dispatches the pending command")
    {
        Some(HostCommandDispatch::Execute(command)) => command,
        other => panic!("expected executable command, received {other:?}"),
    };
    assert_eq!(command.target(), window());
    assert_eq!(command.desired().title(), "Updated Title");
    assert_eq!(
        command.requested_generation(),
        state.requested().generation(),
        "a direct broker admission preserves the state generation it was validated against"
    );
    assert_eq!(broker.in_flight_len(), 1);

    let receipt = broker
        .complete(
            accepted.header().request_id(),
            &state,
            WindowCommandTerminal::Applied,
        )
        .expect("dispatched command completes exactly once");
    assert_eq!(receipt.header(), accepted.header());
    assert_eq!(receipt.observed_generation(), state.observed().generation());
    assert_eq!(receipt.effective(), state.effective());
    assert!(matches!(receipt.terminal(), WindowCommandTerminal::Applied));
    assert_eq!(broker.in_flight_len(), 0);
    assert_eq!(broker.terminal_len(), 1);
    assert_eq!(
        broker.take_terminal_receipt(accepted.header().request_id()),
        Some(receipt)
    );
}

#[test]
fn expired_admission_emits_a_canceled_terminal_receipt_without_dispatching() {
    let mut broker = broker(2);
    let state = current_state();
    let submitted_at = Instant::now();
    let accepted = broker
        .submit(
            window(),
            submitted_at,
            requested("Expired Title"),
            &state,
            submitted_at,
        )
        .expect("expired commands still receive a terminal receipt");

    assert_eq!(broker.pending_len(), 0);
    assert_eq!(broker.in_flight_len(), 0);
    let receipt = broker
        .take_terminal_receipt(accepted.header().request_id())
        .expect("expired command retained one terminal receipt");
    assert!(matches!(
        receipt.terminal(),
        WindowCommandTerminal::Canceled
    ));
    assert_eq!(receipt.effective(), state.effective());
}

#[test]
fn expired_admission_does_not_publish_requested_state() {
    let mut broker = broker(2);
    let state = current_state();
    let submitted_at = Instant::now();
    let mut requested_state_published = false;

    broker
        .submit_after_requested_state(
            window(),
            submitted_at,
            requested("Expired Title"),
            &state,
            submitted_at,
            || {
                requested_state_published = true;
                Ok::<(), ()>(())
            },
        )
        .expect("expired command still returns its canceled terminal receipt");

    assert!(!requested_state_published);
    assert_eq!(broker.pending_len(), 0);
    assert_eq!(broker.terminal_len(), 1);
}

#[test]
fn admission_rejects_a_snapshot_for_another_window_without_consuming_capacity() {
    let mut broker = broker(1);
    let state = current_state();
    let submitted_at = Instant::now();
    let wrong_target = WindowId::new(
        WindowRegistryId::new(73).expect("fixture registry identity is nonzero"),
        5,
        NonZeroU32::new(2).expect("fixture window generation is nonzero"),
    );

    assert_eq!(
        broker.submit(
            wrong_target,
            submitted_at + Duration::from_secs(1),
            requested("Wrong Target"),
            &state,
            submitted_at,
        ),
        Err(HostCommandBrokerError::SnapshotTargetMismatch {
            expected: wrong_target,
            actual: window(),
        })
    );
    assert_eq!(broker.pending_len(), 0);
    assert_eq!(broker.in_flight_len(), 0);
    assert_eq!(broker.terminal_len(), 0);
}

#[test]
fn broker_bounds_outstanding_work_and_cancels_queued_and_dispatched_commands_after_quiesce() {
    let mut broker = broker(2);
    let state = current_state();
    let submitted_at = Instant::now();
    let first = broker
        .submit(
            window(),
            submitted_at + Duration::from_secs(1),
            requested("First"),
            &state,
            submitted_at,
        )
        .expect("first command is admitted");
    let second = broker
        .submit(
            window(),
            submitted_at + Duration::from_secs(1),
            requested("Second"),
            &state,
            submitted_at,
        )
        .expect("second command is admitted");
    assert_eq!(
        broker.submit(
            window(),
            submitted_at + Duration::from_secs(1),
            requested("Third"),
            &state,
            submitted_at,
        ),
        Err(HostCommandBrokerError::OutstandingLimitReached { limit: 2 })
    );

    assert!(matches!(
        broker
            .dispatch_next(submitted_at, &state)
            .expect("first command dispatches"),
        Some(HostCommandDispatch::Execute(_))
    ));
    assert_eq!(
        broker
            .cancel_window_after_quiesce(window(), &state)
            .expect("quiesced host cancels every outstanding command for its window"),
        2
    );
    assert_eq!(broker.pending_len(), 0);
    assert_eq!(broker.in_flight_len(), 0);
    for request_id in [first.header().request_id(), second.header().request_id()] {
        assert!(matches!(
            broker
                .take_terminal_receipt(request_id)
                .expect("quiesce cancellation retains every receipt")
                .terminal(),
            WindowCommandTerminal::Canceled
        ));
    }
}

#[test]
fn quiesce_cancellation_keeps_one_terminal_receipt_for_every_full_queue_entry() {
    let mut broker = broker(4);
    let state = current_state();
    let submitted_at = Instant::now();
    let accepted = ["First", "Second", "Third", "Fourth"].map(|title| {
        broker
            .submit(
                window(),
                submitted_at + Duration::from_secs(1),
                requested(title),
                &state,
                submitted_at,
            )
            .expect("queue entry is admitted within the broker limit")
    });

    assert_eq!(
        broker
            .cancel_window_after_quiesce(window(), &state)
            .expect("quiesced host terminalizes the complete bounded queue"),
        accepted.len()
    );
    assert_eq!(broker.pending_len(), 0);
    assert_eq!(broker.in_flight_len(), 0);
    assert_eq!(broker.terminal_len(), accepted.len());
    for accepted in accepted {
        assert!(matches!(
            broker
                .take_terminal_receipt(accepted.header().request_id())
                .expect("every accepted command retains its terminal receipt")
                .terminal(),
            WindowCommandTerminal::Canceled
        ));
    }
}

#[test]
fn broker_serializes_platform_execution_until_the_prior_command_has_a_terminal_receipt() {
    let mut broker = broker(2);
    let state = current_state();
    let submitted_at = Instant::now();
    let first = broker
        .submit(
            window(),
            submitted_at + Duration::from_secs(1),
            requested("First"),
            &state,
            submitted_at,
        )
        .expect("first command is admitted");
    let second = broker
        .submit(
            window(),
            submitted_at + Duration::from_secs(1),
            requested("Second"),
            &state,
            submitted_at,
        )
        .expect("second command is admitted");

    assert!(matches!(
        broker
            .dispatch_next(submitted_at, &state)
            .expect("first command dispatches"),
        Some(HostCommandDispatch::Execute(_))
    ));
    assert_eq!(
        broker
            .dispatch_next(submitted_at, &state)
            .expect("broker observes the active platform operation"),
        None
    );
    broker
        .complete(
            first.header().request_id(),
            &state,
            WindowCommandTerminal::Applied,
        )
        .expect("first operation terminalizes");
    assert!(matches!(
        broker
            .dispatch_next(submitted_at, &state)
            .expect("second command dispatches after the terminal receipt"),
        Some(HostCommandDispatch::Execute(execution))
            if execution.request_id() == second.header().request_id()
    ));
}

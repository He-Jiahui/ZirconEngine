use std::sync::{mpsc, Arc, Barrier};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{EditorMessagePayload, ToolMessage};
use crate::core::tools::{
    AcquireOutcome, ToolAuthorityState, ToolDefinitionId, ToolInputCaptureDisposition,
    ToolInputCaptureEvent, ToolInputCaptureOutcome, ToolInputCaptureOwner,
    ToolInputCapturePriority, ToolInputCaptureRequest, ToolInputScope, ToolInputSource,
    ToolInstanceId, ToolLifecycleEvent, ToolOwnerGeneration, ToolOwnerRevokeOutcome,
    ToolQueueLimits, ToolResourceCatalogError, ToolResourceChannelPolicy, ToolResourceKey,
    ToolResourceKindId, ToolResourceKindRegistration, ToolResourceSet, ToolScope, ToolScopeKind,
    ToolTransitionRevision,
};
use zircon_runtime_interface::ui::dispatch::{
    UiPointerId, UiPointerSource, UiSurfaceId, UiWindowId,
};

use super::{
    ToolSchedulerLimits, ToolSchedulerLimitsError, ToolSchedulerService, ToolSchedulerServiceError,
    ToolTransitionCursor, ToolTransitionRead, ToolTransitionReadError,
};

mod owner_lifecycle;

#[test]
fn revisioned_outbox_preserves_commit_order_when_callers_dispatch_in_reverse() {
    let bus = crate::core::editor_message::SharedEditorMessageBus::default();
    let topic = crate::core::editor_message::EditorTopic::tool();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let scheduler = ToolSchedulerService::new(bus.clone());
    let tool = ToolInstanceId::for_test("scene.viewport.concurrent", ToolOwnerGeneration::BUILTIN)
        .unwrap();
    let first_committed = Arc::new(Barrier::new(2));
    let second_dispatched = Arc::new(Barrier::new(2));
    let (lease_tx, lease_rx) = mpsc::channel();

    let first = {
        let scheduler = scheduler.clone();
        let tool = tool.clone();
        let first_committed = Arc::clone(&first_committed);
        let second_dispatched = Arc::clone(&second_dispatched);
        std::thread::spawn(move || {
            let report = scheduler
                .commit_transition(|authority| {
                    authority.acquire(tool, ToolResourceSet::single(viewport_resource()))
                })
                .unwrap();
            let AcquireOutcome::Acquired { lease } = report.outcome() else {
                panic!("first transition should acquire");
            };
            lease_tx.send(lease.id()).unwrap();
            first_committed.wait();
            second_dispatched.wait();
            scheduler.dispatch_outbox();
        })
    };
    let second = {
        let scheduler = scheduler.clone();
        std::thread::spawn(move || {
            first_committed.wait();
            let lease_id = lease_rx.recv().unwrap();
            scheduler
                .commit_transition(|authority| authority.release(lease_id))
                .unwrap();
            scheduler.dispatch_outbox();
            second_dispatched.wait();
        })
    };

    first.join().unwrap();
    second.join().unwrap();

    let deliveries = bus.drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 2);
    let batches = deliveries
        .iter()
        .map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Tool(ToolMessage::Transition(batch)) => batch,
            payload => panic!("expected a tool transition batch, got {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(batches[0].revision().value(), 1);
    assert_eq!(batches[1].revision().value(), 2);
    let ToolLifecycleEvent::Activated { lease: activated } = &batches[0].events()[0] else {
        panic!("first event should activate the lease");
    };
    let ToolLifecycleEvent::Deactivated { lease: deactivated } = &batches[1].events()[0] else {
        panic!("second event should deactivate the lease");
    };
    assert_eq!(activated.id(), deactivated.id());
    assert_eq!(activated.instance(), &tool);
    assert_eq!(scheduler.holder(&viewport_resource()), None);
    let health = scheduler.delivery_health();
    assert_eq!(health.committed_revision().value(), 2);
    assert_eq!(health.dispatched_revision().value(), 2);
    assert_eq!(health.delivered_batches(), 2);
    assert!(!health.requires_resync());
}

#[test]
fn delivery_health_marks_a_transition_unobserved_without_subscribers() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    scheduler
        .acquire(
            ToolInstanceId::for_test("scene.viewport.unobserved", ToolOwnerGeneration::BUILTIN)
                .unwrap(),
            ToolResourceSet::single(viewport_resource()),
        )
        .unwrap();

    let health = scheduler.delivery_health();
    assert_eq!(health.committed_revision().value(), 1);
    assert_eq!(health.dispatched_revision().value(), 1);
    assert_eq!(health.delivered_batches(), 0);
    assert_eq!(health.unobserved_batches(), 1);
    assert!(health.requires_resync());
}

#[test]
fn limits_reject_an_empty_transition_journal() {
    assert_eq!(
        ToolSchedulerLimits::new(ToolQueueLimits::new(1, 1), 0),
        Err(ToolSchedulerLimitsError::EmptyTransitionJournal)
    );
}

#[test]
fn snapshot_keeps_state_and_revision_from_one_authority_lock() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    let tool = ToolInstanceId::for_test(
        "scene.viewport.snapshot.atomic",
        ToolOwnerGeneration::BUILTIN,
    )
    .unwrap();
    let (reader_started_tx, reader_started_rx) = mpsc::channel();
    let mut authority = scheduler.lock_authority();
    let report = authority
        .scheduler
        .acquire(tool.clone(), ToolResourceSet::single(viewport_resource()));
    let reader = {
        let scheduler = scheduler.clone();
        std::thread::spawn(move || {
            reader_started_tx.send(()).unwrap();
            scheduler.snapshot()
        })
    };

    reader_started_rx.recv().unwrap();
    authority.commit(
        ToolTransitionRevision::INITIAL.checked_next().unwrap(),
        report.events(),
    );
    drop(authority);

    let snapshot = reader.join().unwrap();
    assert_eq!(snapshot.cursor().revision().value(), 1);
    assert_eq!(
        snapshot
            .state()
            .resource(&viewport_resource())
            .and_then(|resource| resource.holder())
            .map(|lease| lease.instance()),
        Some(&tool)
    );
}

#[test]
fn transition_read_returns_contiguous_batches_after_the_cursor() {
    let scheduler = ToolSchedulerService::with_limits(
        crate::core::editor_message::SharedEditorMessageBus::default(),
        ToolSchedulerLimits::new(ToolQueueLimits::new(2, 2), 3).unwrap(),
    );
    let tool = ToolInstanceId::for_test(
        "scene.viewport.cursor.available",
        ToolOwnerGeneration::BUILTIN,
    )
    .unwrap();
    let lease = acquired_lease(&scheduler, tool.clone());
    scheduler.release(lease.id()).unwrap();
    acquired_lease(&scheduler, tool);

    let read = scheduler
        .read_transitions(ToolTransitionCursor::INITIAL)
        .unwrap();
    let ToolTransitionRead::Available {
        from_exclusive,
        through,
        batches,
    } = read
    else {
        panic!("expected available transition batches");
    };
    assert_eq!(from_exclusive, ToolTransitionCursor::INITIAL);
    assert_eq!(through.revision().value(), 3);
    assert_eq!(
        batches
            .iter()
            .map(|batch| batch.revision().value())
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(
        scheduler.read_transitions(through),
        Ok(ToolTransitionRead::Current { cursor: through })
    );
}

#[test]
fn stale_cursor_receives_an_atomic_current_snapshot() {
    let scheduler = ToolSchedulerService::with_limits(
        crate::core::editor_message::SharedEditorMessageBus::default(),
        ToolSchedulerLimits::new(ToolQueueLimits::new(2, 2), 2).unwrap(),
    );
    let tool =
        ToolInstanceId::for_test("scene.viewport.cursor.resync", ToolOwnerGeneration::BUILTIN)
            .unwrap();
    let lease = acquired_lease(&scheduler, tool.clone());
    scheduler.release(lease.id()).unwrap();
    acquired_lease(&scheduler, tool.clone());

    let first_revision = ToolTransitionCursor::from_revision(
        ToolTransitionRevision::INITIAL.checked_next().unwrap(),
    );
    let ToolTransitionRead::Available { batches, .. } =
        scheduler.read_transitions(first_revision).unwrap()
    else {
        panic!("expected retained transition batches at the journal boundary");
    };
    assert_eq!(
        batches
            .iter()
            .map(|batch| batch.revision().value())
            .collect::<Vec<_>>(),
        [2, 3]
    );

    let ToolTransitionRead::ResyncRequired {
        requested,
        oldest_available_revision,
        snapshot,
    } = scheduler
        .read_transitions(ToolTransitionCursor::INITIAL)
        .unwrap()
    else {
        panic!("expected a resync snapshot for the stale cursor");
    };
    assert_eq!(requested, ToolTransitionCursor::INITIAL);
    assert_eq!(oldest_available_revision.value(), 2);
    assert_eq!(snapshot.cursor().revision().value(), 3);
    assert_eq!(
        snapshot
            .state()
            .resource(&viewport_resource())
            .and_then(|resource| resource.holder())
            .map(|lease| lease.instance()),
        Some(&tool)
    );
}

#[test]
fn future_cursor_is_rejected_with_current_revision() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    let future = ToolTransitionCursor::from_revision(
        ToolTransitionRevision::INITIAL.checked_next().unwrap(),
    );

    assert_eq!(
        scheduler.read_transitions(future),
        Err(ToolTransitionReadError::FutureCursor {
            requested: future,
            current: ToolTransitionCursor::INITIAL,
        })
    );
}

#[test]
fn revision_exhaustion_rejects_a_transition_before_scheduler_mutation() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    {
        let mut authority = scheduler.lock_authority();
        authority.revision = ToolTransitionRevision::for_test(u64::MAX);
    }
    let tool = ToolInstanceId::for_test(
        "scene.viewport.revision-exhaustion",
        ToolOwnerGeneration::BUILTIN,
    )
    .unwrap();

    assert_eq!(
        scheduler.acquire(tool, ToolResourceSet::single(viewport_resource()),),
        Err(ToolSchedulerServiceError::TransitionRevisionExhausted)
    );
    assert_eq!(scheduler.holder(&viewport_resource()), None);
    assert_eq!(scheduler.snapshot().cursor().revision().value(), u64::MAX);
}

#[test]
fn definition_validation_precedes_instance_ordinal_allocation() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    let invalid_definition = "x".repeat(crate::core::tools::MAX_TOOL_DEFINITION_ID_BYTES + 1);

    assert!(matches!(
        ToolDefinitionId::parse(invalid_definition),
        Err(crate::core::tools::ToolDefinitionIdError::TooLong { .. })
    ));
    let definition = ToolDefinitionId::parse("editor.scene.viewport").unwrap();
    assert_eq!(
        scheduler
            .allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN)
            .unwrap()
            .as_str(),
        "editor.scene.viewport@1.1"
    );
}

#[test]
fn quiesce_rejects_new_work_and_close_drains_to_an_idempotent_terminal_state() {
    let scheduler = ToolSchedulerService::with_queue_limits(
        crate::core::editor_message::SharedEditorMessageBus::default(),
        ToolQueueLimits::new(1, 1),
    );
    let definition = ToolDefinitionId::parse("editor.scene.viewport").unwrap();
    let active = scheduler
        .allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN)
        .unwrap();
    let rejected = scheduler
        .allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN)
        .unwrap();
    acquired_lease(&scheduler, active);

    assert_eq!(scheduler.quiesce(), Ok(ToolAuthorityState::Quiescing));
    let quiesced = scheduler.snapshot();
    assert_eq!(quiesced.authority_state(), ToolAuthorityState::Quiescing);
    assert_eq!(quiesced.cursor().revision().value(), 2);
    assert_eq!(
        scheduler.acquire(rejected, ToolResourceSet::single(viewport_resource()),),
        Err(ToolSchedulerServiceError::AuthorityUnavailable {
            state: ToolAuthorityState::Quiescing,
        })
    );
    assert_eq!(
        scheduler.allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN),
        Err(ToolSchedulerServiceError::AuthorityUnavailable {
            state: ToolAuthorityState::Quiescing,
        })
    );
    assert_eq!(scheduler.snapshot(), quiesced);

    let closed = scheduler.close().unwrap();
    assert_eq!(closed.outcome().released_single_leases(), 1);
    assert_eq!(closed.outcome().released_set_leases(), 0);
    let terminal = scheduler.snapshot();
    assert_eq!(terminal.authority_state(), ToolAuthorityState::Closed);
    assert_eq!(terminal.cursor().revision().value(), 3);
    assert!(terminal.state().active_leases().is_empty());
    assert!(terminal.state().queued_requests().is_empty());
    assert!(terminal.active_owner_generations().is_empty());

    let repeated = scheduler.close().unwrap();
    assert!(repeated.events().is_empty());
    assert_eq!(scheduler.snapshot(), terminal);
}

#[test]
fn poisoned_authority_enters_fail_stop_and_can_only_drain_closed() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    let definition = ToolDefinitionId::parse("editor.scene.viewport").unwrap();
    let tool = scheduler
        .allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN)
        .unwrap();
    let lease = acquired_lease(&scheduler, tool.clone());
    let poisoner = {
        let scheduler = scheduler.clone();
        std::thread::spawn(move || {
            let _authority = scheduler.lock_authority();
            panic!("poison the tool authority for fail-stop coverage");
        })
    };
    assert!(poisoner.join().is_err());

    assert_eq!(scheduler.authority_state(), ToolAuthorityState::Faulted);
    assert_eq!(
        scheduler.acquire(tool, ToolResourceSet::single(modal_resource()),),
        Err(ToolSchedulerServiceError::AuthorityUnavailable {
            state: ToolAuthorityState::Faulted,
        })
    );
    assert_eq!(
        scheduler.release(lease.id()),
        Err(ToolSchedulerServiceError::AuthorityUnavailable {
            state: ToolAuthorityState::Faulted,
        })
    );
    let faulted = scheduler.snapshot();
    assert_eq!(faulted.authority_state(), ToolAuthorityState::Faulted);
    assert_eq!(faulted.cursor().revision().value(), 2);
    assert!(scheduler.delivery_health().requires_resync());
    assert_eq!(
        faulted
            .state()
            .resource(&viewport_resource())
            .and_then(|resource| resource.holder())
            .map(|lease| lease.instance().definition()),
        Some(&definition)
    );

    let close = scheduler.close().unwrap();
    assert_eq!(close.outcome().released_single_leases(), 1);
    assert_eq!(scheduler.authority_state(), ToolAuthorityState::Closed);
    assert!(scheduler.snapshot().state().active_leases().is_empty());
}

#[test]
fn service_commits_capture_and_owner_loss_in_the_ordered_tool_journal() {
    let bus = crate::core::editor_message::SharedEditorMessageBus::default();
    let scheduler = ToolSchedulerService::new(bus);
    let lease = acquired_lease(
        &scheduler,
        ToolInstanceId::for_test("scene.capture.journal", ToolOwnerGeneration::BUILTIN).unwrap(),
    );
    let owner = ToolInputCaptureOwner::from_lease(&lease);
    let source = ToolInputSource::Pointer {
        scope: ToolInputScope::new(
            UiWindowId::new("test.window"),
            UiSurfaceId::new("test.viewport"),
        ),
        user_id: None,
        device_id: None,
        pointer_id: Some(UiPointerId::new(1)),
        pointer_source: UiPointerSource::Mouse,
    };

    let captured = scheduler
        .begin_input_capture(ToolInputCaptureRequest::new(
            owner,
            source.clone(),
            viewport_resource(),
            ToolInputCapturePriority::new(10),
        ))
        .unwrap();
    assert!(matches!(
        captured.outcome(),
        ToolInputCaptureOutcome::Captured { .. }
    ));
    assert_eq!(
        scheduler.snapshot().state().active_input_captures().len(),
        1
    );

    let released = scheduler.release(lease.id()).unwrap();
    assert!(matches!(
        released.events().first(),
        Some(ToolLifecycleEvent::InputCapture {
            event: ToolInputCaptureEvent::Ended {
                disposition: ToolInputCaptureDisposition::OwnerLost,
                ..
            }
        })
    ));
    assert!(matches!(
        released.events().get(1),
        Some(ToolLifecycleEvent::Deactivated { .. })
    ));
    assert!(scheduler.active_input_capture(&source).is_none());
    assert!(scheduler
        .snapshot()
        .state()
        .active_input_captures()
        .is_empty());
}

#[test]
fn focus_loss_releases_every_capture_in_the_exact_window() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    let lease = acquired_lease(
        &scheduler,
        ToolInstanceId::for_test("scene.capture.focus-loss", ToolOwnerGeneration::BUILTIN).unwrap(),
    );
    let owner = ToolInputCaptureOwner::from_lease(&lease);
    let window_id = UiWindowId::new("test.window");
    let source = |window: &UiWindowId, surface: &str, device| ToolInputSource::Pointer {
        scope: ToolInputScope::new(window.clone(), UiSurfaceId::new(surface)),
        user_id: None,
        device_id: Some(zircon_runtime_interface::ui::dispatch::UiDeviceId::new(
            device,
        )),
        pointer_id: None,
        pointer_source: UiPointerSource::Mouse,
    };
    let first = source(&window_id, "test.viewport.first", 7);
    let second = source(&window_id, "test.viewport.second", 8);
    let other_window = source(&UiWindowId::new("other.window"), "test.viewport", 9);
    for source in [&first, &second, &other_window] {
        let _ = scheduler
            .begin_input_capture(ToolInputCaptureRequest::new(
                owner.clone(),
                source.clone(),
                viewport_resource(),
                ToolInputCapturePriority::new(10),
            ))
            .unwrap();
    }

    let released = scheduler
        .release_input_window_on_focus_loss(&window_id)
        .unwrap();

    assert_eq!(released.outcome().len(), 2);
    assert!(released.events().iter().all(|event| matches!(
        event,
        ToolLifecycleEvent::InputCapture {
            event: ToolInputCaptureEvent::Ended {
                disposition: ToolInputCaptureDisposition::FocusLost,
                ..
            }
        }
    )));
    assert!(scheduler.active_input_capture(&first).is_none());
    assert!(scheduler.active_input_capture(&second).is_none());
    assert!(scheduler.active_input_capture(&other_window).is_some());
}

fn acquired_lease(
    scheduler: &ToolSchedulerService,
    tool: ToolInstanceId,
) -> crate::core::tools::ToolLeaseHandle {
    let report = scheduler
        .acquire(tool, ToolResourceSet::single(viewport_resource()))
        .unwrap();
    match report.into_parts().0 {
        AcquireOutcome::Acquired { lease } | AcquireOutcome::AlreadyHeld { lease } => lease,
        outcome => panic!("expected an acquired lease, got {outcome:?}"),
    }
}

fn viewport_resource() -> ToolResourceKey {
    ToolResourceKey::viewport_input(ViewInstanceId::new("editor.scene#1"))
}

fn modal_resource() -> ToolResourceKey {
    ToolResourceKey::modal_surface(UiWindowId::new("editor.main"))
}

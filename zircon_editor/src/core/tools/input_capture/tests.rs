use super::*;
use crate::core::editor_event::ViewInstanceId;
use std::num::NonZeroUsize;

fn owner(lease: u64, generation: u64) -> ToolInputCaptureOwner {
    ToolInputCaptureOwner::new(
        ToolLeaseId::from_ordinal(NonZeroU64::new(lease).expect("non-zero test lease")),
        ToolInstanceId::from_parts("test.tool", generation, lease).expect("valid test instance"),
    )
}

fn pointer_source(pointer: u64) -> ToolInputSource {
    ToolInputSource::Pointer {
        scope: ToolInputScope::new(UiWindowId::new("window"), UiSurfaceId::new("viewport")),
        user_id: None,
        device_id: None,
        pointer_id: Some(UiPointerId::new(pointer)),
        pointer_source: UiPointerSource::Mouse,
    }
}

fn mouse_source_without_pointer_id(device: u64) -> ToolInputSource {
    ToolInputSource::Pointer {
        scope: ToolInputScope::new(UiWindowId::new("window"), UiSurfaceId::new("viewport")),
        user_id: None,
        device_id: Some(UiDeviceId::new(device)),
        pointer_id: None,
        pointer_source: UiPointerSource::Mouse,
    }
}

fn request(
    owner: ToolInputCaptureOwner,
    source: ToolInputSource,
    priority: u16,
) -> ToolInputCaptureRequest {
    ToolInputCaptureRequest::new(
        owner,
        source,
        ToolResourceKey::viewport_input(ViewInstanceId::new("editor.scene#1")),
        ToolInputCapturePriority::new(priority),
    )
}

#[test]
fn same_owner_reuses_existing_capture_without_new_identity() {
    let mut authority = ToolInputCaptureAuthority::new();
    let request = request(owner(1, 1), pointer_source(1), 10);
    let first = authority.begin(request.clone());
    let second = authority.begin(request);
    let ToolInputCaptureOutcome::Captured { handle, .. } = first.outcome().clone() else {
        panic!("first capture should start");
    };
    assert!(matches!(
        second.outcome(),
        ToolInputCaptureOutcome::AlreadyHeld { handle: existing } if existing.id() == handle.id()
    ));
    assert_eq!(authority.active().count(), 1);
}

#[test]
fn mouse_capture_preserves_missing_platform_pointer_identity() {
    let mut authority = ToolInputCaptureAuthority::new();
    let source = mouse_source_without_pointer_id(7);

    let started = authority.begin(request(owner(1, 1), source.clone(), 10));

    assert!(matches!(
        started.outcome(),
        ToolInputCaptureOutcome::Captured { handle, .. } if handle.source() == &source
    ));
    assert_eq!(
        authority
            .active_for_source(&source)
            .map(|capture| capture.source()),
        Some(&source)
    );
}

#[test]
fn higher_priority_capture_steals_and_orders_end_before_start() {
    let mut authority = ToolInputCaptureAuthority::new();
    let first = authority.begin(request(owner(1, 1), pointer_source(1), 1));
    let second = authority.begin(request(owner(2, 1), pointer_source(1), 2));
    assert!(matches!(
        second.outcome(),
        ToolInputCaptureOutcome::Captured {
            preempted: Some(_),
            ..
        }
    ));
    assert!(matches!(
        second.events().first(),
        Some(ToolInputCaptureEvent::Ended {
            disposition: ToolInputCaptureDisposition::Stolen,
            ..
        })
    ));
    assert!(matches!(
        second.events().get(1),
        Some(ToolInputCaptureEvent::Started { .. })
    ));
    assert_eq!(authority.active().count(), 1);
    assert!(matches!(
        first.outcome(),
        ToolInputCaptureOutcome::Captured { .. }
    ));
}

#[test]
fn lower_priority_capture_is_denied_without_mutating_holder() {
    let mut authority = ToolInputCaptureAuthority::new();
    let _ = authority.begin(request(owner(1, 1), pointer_source(1), 3));
    let denied = authority.begin(request(owner(2, 1), pointer_source(1), 2));
    assert!(matches!(
        denied.outcome(),
        ToolInputCaptureOutcome::Denied {
            reason: ToolInputCaptureDenial::LowerPriority {
                holder_priority,
            },
            ..
        } if *holder_priority == ToolInputCapturePriority::new(3)
    ));
    assert_eq!(
        authority
            .active()
            .next()
            .map(|capture| capture.owner().lease_id().value()),
        Some(ToolLeaseId::from_ordinal(NonZeroU64::new(1).expect("non-zero")).value())
    );
}

#[test]
fn active_capture_capacity_is_bounded_per_distinct_source() {
    let mut authority = ToolInputCaptureAuthority::with_max_active_captures(
        NonZeroUsize::new(1).expect("non-zero test capacity"),
    );
    let _ = authority.begin(request(owner(1, 1), pointer_source(1), 1));
    let denied = authority.begin(request(owner(2, 1), pointer_source(2), 1));

    assert!(matches!(
        denied.outcome(),
        ToolInputCaptureOutcome::Denied {
            holder: None,
            reason: ToolInputCaptureDenial::CapacityReached {
                max_active_captures: 1,
            },
        }
    ));
    assert_eq!(authority.active().count(), 1);
}

#[test]
fn stale_owner_cannot_end_a_newer_generation_capture() {
    let mut authority = ToolInputCaptureAuthority::new();
    let current_owner = owner(1, 2);
    let started = authority.begin(request(current_owner, pointer_source(1), 1));
    let ToolInputCaptureOutcome::Captured { handle, .. } = started.outcome().clone() else {
        panic!("capture should start");
    };

    let stale = authority.end(
        handle.id(),
        &owner(1, 1),
        ToolInputCaptureDisposition::Cancelled,
    );

    assert_eq!(stale.outcome(), &ToolInputCaptureEndOutcome::OwnerMismatch);
    assert_eq!(authority.active().count(), 1);
}

#[test]
fn aborted_capture_preserves_its_distinct_terminal_disposition() {
    let mut authority = ToolInputCaptureAuthority::new();
    let capture_owner = owner(1, 1);
    let started = authority.begin(request(capture_owner.clone(), pointer_source(1), 1));
    let ToolInputCaptureOutcome::Captured { handle, .. } = started.outcome().clone() else {
        panic!("capture should start");
    };

    let ended = authority.end(
        handle.id(),
        &capture_owner,
        ToolInputCaptureDisposition::Aborted,
    );

    assert!(matches!(
        ended.outcome(),
        ToolInputCaptureEndOutcome::Ended {
            disposition: ToolInputCaptureDisposition::Aborted,
            ..
        }
    ));
}

#[test]
fn owner_loss_ends_only_matching_generation_and_keeps_other_owner() {
    let mut authority = ToolInputCaptureAuthority::new();
    let first_owner = owner(1, 1);
    let second_owner = owner(2, 1);
    let _ = authority.begin(request(first_owner.clone(), pointer_source(1), 1));
    let _ = authority.begin(request(second_owner.clone(), pointer_source(2), 1));
    let report = authority.release_owner(&first_owner, ToolInputCaptureDisposition::OwnerLost);
    assert_eq!(report.outcome().len(), 1);
    assert_eq!(authority.active().count(), 1);
    assert_eq!(
        authority.active().next().map(|capture| capture.owner()),
        Some(&second_owner)
    );
}

#[test]
fn shutdown_is_deterministic_and_clears_source_index() {
    let mut authority = ToolInputCaptureAuthority::new();
    let source = pointer_source(1);
    let _ = authority.begin(request(owner(1, 1), source.clone(), 1));
    let report = authority.shutdown();
    assert_eq!(report.outcome().len(), 1);
    assert!(authority.active_for_source(&source).is_none());
    assert_eq!(authority.active().count(), 0);
    assert!(report.events().iter().all(|event| matches!(
        event,
        ToolInputCaptureEvent::Ended {
            disposition: ToolInputCaptureDisposition::Shutdown,
            ..
        }
    )));
}

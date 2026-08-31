use super::{acquired, modal_surface, queued, resources, scene_mode_slot, tool, viewport_input};
use crate::core::tools::{
    AcquireDenial, AcquireOutcome, ReleaseOutcome, ToolLeaseHandle, ToolQueueLimits,
    ToolResourceSet, ToolScheduler,
};

#[test]
fn single_and_set_queue_limits_are_independent() {
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(0, 1));
    acquired(scheduler.acquire(
        tool("tool.modal-holder"),
        ToolResourceSet::single(modal_surface("window.main")),
    ));
    assert!(matches!(
        scheduler
            .acquire(
                tool("tool.queued-set"),
                resources([
                    modal_surface("window.main"),
                    scene_mode_slot("viewport.main"),
                ]),
            )
            .outcome(),
        AcquireOutcome::Queued { position: 1, .. }
    ));
    assert!(matches!(
        scheduler
            .acquire(
                tool("tool.denied-single"),
                ToolResourceSet::single(modal_surface("window.main")),
            )
            .outcome(),
        AcquireOutcome::Denied {
            reason: AcquireDenial::QueueFull { max_queued: 0 },
            ..
        }
    ));
}

#[test]
fn pending_set_only_reserves_its_overlapping_resources() {
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(4, 4));
    acquired(scheduler.acquire(
        tool("tool.modal-holder"),
        ToolResourceSet::single(modal_surface("window.main")),
    ));
    queued(scheduler.acquire(
        tool("tool.pending-set"),
        resources([
            modal_surface("window.main"),
            scene_mode_slot("viewport.main"),
        ]),
    ));

    let viewport_resource = viewport_input("viewport.main");
    let viewport = acquired(scheduler.acquire(
        tool("tool.viewport"),
        ToolResourceSet::single(viewport_resource.clone()),
    ));
    assert_eq!(
        scheduler
            .holder(&viewport_resource)
            .map(ToolLeaseHandle::id),
        Some(viewport.id())
    );
}

#[test]
fn unrelated_single_queue_promotes_while_set_head_remains_blocked() {
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(4, 4));
    let modal_resource = modal_surface("window.main");
    let modal_holder = acquired(scheduler.acquire(
        tool("tool.modal-holder"),
        ToolResourceSet::single(modal_resource.clone()),
    ));
    let viewport_holder = acquired(scheduler.acquire(
        tool("tool.viewport-holder"),
        ToolResourceSet::single(viewport_input("viewport.main")),
    ));
    queued(scheduler.acquire(
        tool("tool.pending-set"),
        resources([modal_resource.clone(), scene_mode_slot("viewport.main")]),
    ));
    let viewport_waiter = queued(scheduler.acquire(
        tool("tool.viewport-waiter"),
        ToolResourceSet::single(viewport_input("viewport.main")),
    ));

    let released = scheduler.release(viewport_holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = released.outcome()
    else {
        panic!("viewport holder should release");
    };
    assert_eq!(activated_leases[0].request_id(), viewport_waiter.id());
    assert_eq!(
        scheduler.holder(&modal_resource).map(ToolLeaseHandle::id),
        Some(modal_holder.id())
    );
}

#[test]
fn overlapping_single_queue_yields_to_the_pending_set_head() {
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(4, 4));
    let modal_resource = modal_surface("window.main");
    let holder = acquired(scheduler.acquire(
        tool("tool.modal-holder"),
        ToolResourceSet::single(modal_resource.clone()),
    ));
    let pending_set = queued(scheduler.acquire(
        tool("tool.pending-set"),
        resources([modal_resource.clone(), scene_mode_slot("viewport.main")]),
    ));
    let modal_waiter = queued(scheduler.acquire(
        tool("tool.modal-waiter"),
        ToolResourceSet::single(modal_resource.clone()),
    ));

    let released = scheduler.release(holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = released.outcome()
    else {
        panic!("modal holder should release");
    };
    assert_eq!(activated_leases.len(), 1);
    assert_eq!(activated_leases[0].request_id(), pending_set.id());
    assert_eq!(
        scheduler
            .queued_requests(&modal_resource)
            .map(|request| request.id())
            .collect::<Vec<_>>(),
        [modal_waiter.id()]
    );
}

#[test]
fn later_available_set_cannot_bypass_a_blocked_set_head() {
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(4, 4));
    let modal_resource = modal_surface("window.main");
    let viewport_resource = viewport_input("viewport.main");
    let scene_mode_resource = scene_mode_slot("viewport.main");
    let holder = acquired(scheduler.acquire(
        tool("tool.modal-holder"),
        ToolResourceSet::single(modal_resource.clone()),
    ));
    let first = queued(scheduler.acquire(
        tool("tool.first-set"),
        resources([modal_resource, scene_mode_resource.clone()]),
    ));
    let second = queued(scheduler.acquire(
        tool("tool.second-set"),
        resources([viewport_resource.clone(), scene_mode_resource]),
    ));

    assert!(scheduler.holder(&viewport_resource).is_none());
    let released = scheduler.release(holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = released.outcome()
    else {
        panic!("modal holder should release");
    };
    assert_eq!(activated_leases[0].request_id(), first.id());
    assert_eq!(scheduler.pending_request(second.instance()), Some(&second));
}

use super::{acquired, modal_surface, queued, resources, scene_mode_slot, tool, viewport_input};
use crate::core::tools::{ToolLeaseHandle, ToolQueueLimits, ToolResourceSet, ToolScheduler};

#[test]
fn snapshot_captures_resource_views_and_canonical_claim_handles() {
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(4, 4));
    let viewport_resource = viewport_input("viewport.main");
    let viewport_holder = acquired(scheduler.acquire(
        tool("tool.viewport-holder"),
        ToolResourceSet::single(viewport_resource.clone()),
    ));
    let viewport_waiter = queued(scheduler.acquire(
        tool("tool.viewport-waiter"),
        ToolResourceSet::single(viewport_resource.clone()),
    ));
    let active_set = acquired(scheduler.acquire(
        tool("tool.active-set"),
        resources([
            modal_surface("window.main"),
            scene_mode_slot("viewport.main"),
        ]),
    ));

    let snapshot = scheduler.snapshot();

    assert_eq!(snapshot.resources().len(), 3);
    assert_eq!(
        snapshot.active_leases(),
        [viewport_holder.clone(), active_set]
    );
    assert_eq!(snapshot.queued_requests(), [viewport_waiter.clone()]);
    assert_eq!(
        snapshot
            .resource(&viewport_resource)
            .and_then(|state| state.holder())
            .map(ToolLeaseHandle::id),
        Some(viewport_holder.id())
    );
    assert_eq!(
        snapshot.resource(&viewport_resource).unwrap().queued(),
        [viewport_waiter]
    );
}

#[test]
fn snapshots_are_immutable_receipts_of_capture_time() {
    let viewport_resource = viewport_input("viewport.main");
    let resources = ToolResourceSet::single(viewport_resource.clone());
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(2, 2));
    let holder = acquired(scheduler.acquire(tool("tool.holder"), resources.clone()));
    let next = queued(scheduler.acquire(tool("tool.next"), resources));
    let before_release = scheduler.snapshot();

    scheduler.release(holder.id());
    let after_release = scheduler.snapshot();

    assert_eq!(
        before_release
            .resource(&viewport_resource)
            .and_then(|state| state.holder())
            .map(ToolLeaseHandle::id),
        Some(holder.id())
    );
    assert_eq!(
        before_release
            .resource(&viewport_resource)
            .unwrap()
            .queued(),
        [next.clone()]
    );
    assert_eq!(
        after_release
            .resource(&viewport_resource)
            .and_then(|state| state.holder())
            .map(ToolLeaseHandle::request_id),
        Some(next.id())
    );
}

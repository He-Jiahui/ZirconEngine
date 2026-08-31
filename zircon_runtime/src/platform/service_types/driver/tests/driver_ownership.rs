use std::num::NonZeroUsize;
use std::time::Instant;

use crate::core::framework::platform::{
    EventLoopBackgroundPolicy, EventLoopControlFlow, EventLoopHostWakeReason, EventLoopWakeRequest,
    EventLoopWakeSource,
};
use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayTopologyGeneration, NativeWindowId, SurfaceLeaseRequest,
};
use crate::platform::test_support::platform_driver;

use super::fixtures::display_topology;
use crate::platform::{
    HostCommandBrokerAccessError, PlatformSurfaceLeaseError, WindowRegistryError,
};

#[test]
fn each_platform_driver_owns_an_isolated_window_registry() {
    let first_driver = platform_driver();
    let second_driver = platform_driver();
    let native_window = NativeWindowId::new(1).expect("test native window is nonzero");

    let first_window = first_driver
        .with_window_registry(|registry| registry.register(native_window))
        .expect("first platform driver registers its native window");

    assert!(second_driver
        .with_window_registry(|registry| Ok(registry.is_empty()))
        .expect("second platform driver retains its window registry"));
    assert_ne!(
        first_window.registry(),
        second_driver
            .with_window_registry(|registry| Ok(registry.registry_id()))
            .expect("second platform driver retains its window registry")
    );
}

#[test]
fn each_platform_driver_owns_an_isolated_surface_lease_registry() {
    let first_driver = platform_driver();
    let second_driver = platform_driver();

    assert_eq!(
        first_driver
            .with_surface_leases(|registry| Ok(registry.active_count()))
            .expect("first platform driver retains its surface lease registry"),
        0
    );
    assert_eq!(
        second_driver
            .with_surface_leases(|registry| Ok(registry.preparing_count()))
            .expect("second platform driver retains its surface lease registry"),
        0
    );
}

#[test]
fn surface_lease_driver_rejects_publication_after_the_native_window_starts_closing() {
    let driver = platform_driver();
    let display = "edid:surface-lease-panel";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface preparation");
    let window = driver
        .with_window_registry(|registry| {
            let window = registry.register(
                NativeWindowId::new(76).expect("surface fixture native window is nonzero"),
            )?;
            registry.bind_viewport(
                window,
                zircon_runtime_interface::ZrRuntimeViewportHandle::new(9),
            )?;
            Ok(window)
        })
        .expect("surface fixture registers native window");
    let prepared = driver
        .prepare_surface_lease(SurfaceLeaseRequest::new(
            window,
            zircon_runtime_interface::ZrRuntimeViewportHandle::new(9),
            DisplayId::new(DisplayKind::PhysicalOutput, display)
                .expect("surface fixture display identity is valid"),
            generation,
        ))
        .expect("live native window prepares a surface lease");
    driver
        .with_window_registry(|registry| registry.begin_close(window))
        .expect("window starts its closing transaction");

    assert_eq!(
        driver.publish_surface_lease(&prepared),
        Err(PlatformSurfaceLeaseError::Registry(
            WindowRegistryError::ClosingWindow { window }
        ))
    );
    driver
        .cancel_surface_lease_preparation(&prepared)
        .expect("closing window still cancels an unpublished surface candidate");
}

#[test]
fn each_platform_driver_owns_an_isolated_event_loop_scheduler() {
    let first_driver = platform_driver();
    let second_driver = platform_driver();
    let now = Instant::now();

    first_driver.observe_event_loop_backlog(EventLoopWakeSource::HostCommand, 2);
    first_driver.schedule_event_loop_wake(EventLoopWakeRequest::immediate(
        EventLoopWakeSource::HostCommand,
        now,
    ));

    assert_eq!(
        first_driver.event_loop_control_flow(now),
        EventLoopControlFlow::Poll
    );
    assert!(first_driver
        .take_due_event_loop_wakes(now)
        .contains(EventLoopWakeSource::HostCommand));
    let first_snapshot = first_driver.event_loop_scheduler_snapshot();
    assert_eq!(first_snapshot.backlog(), 2);
    assert_eq!(first_snapshot.backlog_high_watermark(), 2);
    assert_eq!(first_snapshot.pending_sources(), 0);
    assert_eq!(first_snapshot.dispatched_wakes(), 1);
    first_driver.observe_event_loop_background_policy(EventLoopBackgroundPolicy::Throttled);
    assert_eq!(
        first_driver
            .event_loop_scheduler_snapshot()
            .background_policy(),
        Some(EventLoopBackgroundPolicy::Throttled)
    );
    first_driver.observe_event_loop_host_wake(EventLoopHostWakeReason::ProxyWake, now);
    assert_eq!(
        first_driver
            .event_loop_scheduler_snapshot()
            .host_wake_count(EventLoopHostWakeReason::ProxyWake),
        1
    );
    assert_eq!(
        second_driver.event_loop_control_flow(now),
        EventLoopControlFlow::Wait
    );
    assert_eq!(second_driver.event_loop_scheduler_snapshot().backlog(), 0);
    assert_eq!(
        second_driver
            .event_loop_scheduler_snapshot()
            .background_policy(),
        None
    );
    assert_eq!(
        second_driver
            .event_loop_scheduler_snapshot()
            .host_wake_count(EventLoopHostWakeReason::ProxyWake),
        0
    );
}

#[test]
fn each_platform_driver_owns_an_isolated_window_state_registry() {
    let first_driver = platform_driver();
    let second_driver = platform_driver();

    assert_eq!(
        first_driver
            .with_window_states(|registry| Ok(registry.len()))
            .expect("first platform driver retains its window state registry"),
        0
    );
    assert_eq!(
        second_driver
            .with_window_states(|registry| Ok(registry.len()))
            .expect("second platform driver retains its window state registry"),
        0
    );
}

#[test]
fn host_command_broker_requires_explicit_host_installation_and_remains_driver_owned() {
    let driver = platform_driver();

    assert_eq!(
        driver.with_host_command_broker(|broker| Ok(broker.pending_len())),
        Err(HostCommandBrokerAccessError::Uninstalled)
    );
    driver
        .install_host_command_broker(
            NonZeroUsize::new(2).expect("fixture command broker limit is nonzero"),
        )
        .expect("host installs a bounded command broker");
    assert_eq!(
        driver
            .with_host_command_broker(|broker| Ok(broker.pending_len()))
            .expect("installed host retains command broker ownership"),
        0
    );
    assert_eq!(
        driver.install_host_command_broker(
            NonZeroUsize::new(2).expect("fixture command broker limit is nonzero"),
        ),
        Err(HostCommandBrokerAccessError::AlreadyInstalled)
    );
}

use zircon_runtime_interface::ZrRuntimeViewportHandle;

use crate::core::framework::platform::{ApplicationLifecycleState, ApplicationSurfaceAvailability};
use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayTopologyGeneration, NativeWindowId, SurfaceLeaseError,
    SurfaceLeaseRequest,
};
use crate::platform::test_support::platform_driver;
use crate::platform::{
    ApplicationLifecycleServiceError, PlatformApplicationSuspendError, PlatformSurfaceLeaseError,
    PlatformWindowCloseError, WindowParentKind, WindowRegistryError,
};

use super::display_topology;

fn surface_request(
    window: crate::core::framework::window::WindowId,
    viewport: ZrRuntimeViewportHandle,
    display: &str,
    generation: DisplayTopologyGeneration,
) -> SurfaceLeaseRequest {
    SurfaceLeaseRequest::new(
        window,
        viewport,
        DisplayId::new(DisplayKind::PhysicalOutput, display)
            .expect("surface fixture display identity is valid"),
        generation,
    )
}

#[test]
fn surface_lease_preparation_requires_a_window_registry_viewport_binding() {
    let driver = platform_driver();
    let display = "edid:surface-binding-required";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let viewport = ZrRuntimeViewportHandle::new(17);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface preparation");
    let window = driver
        .with_window_registry(|registry| {
            registry.register(
                NativeWindowId::new(77).expect("surface fixture native window is nonzero"),
            )
        })
        .expect("surface fixture registers native window");

    assert_eq!(
        driver.prepare_surface_lease(surface_request(window, viewport, display, generation)),
        Err(PlatformSurfaceLeaseError::Registry(
            WindowRegistryError::UnknownViewportBinding { viewport }
        ))
    );
}

#[test]
fn surface_lease_preparation_rejects_a_viewport_owned_by_another_window() {
    let driver = platform_driver();
    let display = "edid:surface-binding-owner";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let viewport = ZrRuntimeViewportHandle::new(18);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface preparation");
    let (bound_window, requested_window) = driver
        .with_window_registry(|registry| {
            let bound_window = registry.register(
                NativeWindowId::new(78).expect("first surface fixture native window is nonzero"),
            )?;
            let requested_window = registry.register(
                NativeWindowId::new(79).expect("second surface fixture native window is nonzero"),
            )?;
            registry.bind_viewport(bound_window, viewport)?;
            Ok((bound_window, requested_window))
        })
        .expect("surface fixtures register and bind their viewport");

    assert_eq!(
        driver.prepare_surface_lease(surface_request(
            requested_window,
            viewport,
            display,
            generation,
        )),
        Err(PlatformSurfaceLeaseError::Registry(
            WindowRegistryError::ViewportBoundToDifferentWindow {
                viewport,
                expected_window: requested_window,
                observed_window: bound_window,
            }
        ))
    );
}

#[test]
fn surface_lease_publication_rechecks_the_window_registry_viewport_binding() {
    let driver = platform_driver();
    let display = "edid:surface-binding-publication";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let viewport = ZrRuntimeViewportHandle::new(19);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface preparation");
    let (window, rebound_window) = driver
        .with_window_registry(|registry| {
            let window = registry.register(
                NativeWindowId::new(80).expect("first surface fixture native window is nonzero"),
            )?;
            let rebound_window = registry.register(
                NativeWindowId::new(81).expect("second surface fixture native window is nonzero"),
            )?;
            registry.bind_viewport(window, viewport)?;
            Ok((window, rebound_window))
        })
        .expect("surface fixtures register and bind their viewport");
    let prepared = driver
        .prepare_surface_lease(surface_request(window, viewport, display, generation))
        .expect("bound live viewport prepares a surface lease");
    driver
        .with_window_registry(|registry| {
            registry.unbind_viewport(window, viewport)?;
            registry.bind_viewport(rebound_window, viewport)
        })
        .expect("test moves the viewport binding before publication");

    assert_eq!(
        driver.publish_surface_lease(&prepared),
        Err(PlatformSurfaceLeaseError::Registry(
            WindowRegistryError::ViewportBoundToDifferentWindow {
                viewport,
                expected_window: window,
                observed_window: rebound_window,
            }
        ))
    );
    driver
        .cancel_surface_lease_preparation(&prepared)
        .expect("rejected publication remains cancelable");
}

#[test]
fn window_close_tree_rejects_a_prepared_surface_without_closing_any_window() {
    let driver = platform_driver();
    let display = "edid:surface-close-preflight";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let parent_viewport = ZrRuntimeViewportHandle::new(20);
    let child_viewport = ZrRuntimeViewportHandle::new(21);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface preparation");
    let (parent, child) = driver
        .with_window_registry(|registry| {
            let parent = registry
                .register(NativeWindowId::new(82).expect("parent native window is nonzero"))?;
            let child = registry
                .register(NativeWindowId::new(83).expect("child native window is nonzero"))?;
            registry.set_parent(child, parent, WindowParentKind::Transient)?;
            registry.bind_viewport(parent, parent_viewport)?;
            registry.bind_viewport(child, child_viewport)?;
            Ok((parent, child))
        })
        .expect("tree fixtures register and bind their viewports");
    let prepared = driver
        .prepare_surface_lease(surface_request(child, child_viewport, display, generation))
        .expect("child surface prepares before close preflight");

    assert_eq!(
        driver.begin_window_close_tree_after_quiesce(parent),
        Err(PlatformWindowCloseError::Lease(
            SurfaceLeaseError::WindowHasPreparedLease { window: child }
        ))
    );
    assert_eq!(
        driver.with_window_registry(|registry| registry.native_for(parent)),
        Ok(NativeWindowId::new(82).expect("parent native window remains live"))
    );
    assert_eq!(
        driver.with_window_registry(|registry| registry.native_for(child)),
        Ok(NativeWindowId::new(83).expect("child native window remains live"))
    );
    driver
        .cancel_surface_lease_preparation(&prepared)
        .expect("rejected close leaves the prepared lease cancelable");
}

#[test]
fn window_close_tree_moves_child_first_windows_and_leases_into_retirement() {
    let driver = platform_driver();
    let display = "edid:surface-close-transaction";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let parent_viewport = ZrRuntimeViewportHandle::new(22);
    let child_viewport = ZrRuntimeViewportHandle::new(23);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface publication");
    let (parent, child) = driver
        .with_window_registry(|registry| {
            let parent = registry
                .register(NativeWindowId::new(84).expect("parent native window is nonzero"))?;
            let child = registry
                .register(NativeWindowId::new(85).expect("child native window is nonzero"))?;
            registry.set_parent(child, parent, WindowParentKind::Transient)?;
            registry.bind_viewport(parent, parent_viewport)?;
            registry.bind_viewport(child, child_viewport)?;
            Ok((parent, child))
        })
        .expect("tree fixtures register and bind their viewports");
    let parent_lease = driver
        .prepare_surface_lease(surface_request(
            parent,
            parent_viewport,
            display,
            generation,
        ))
        .and_then(|prepared| driver.publish_surface_lease(&prepared))
        .expect("parent surface publishes")
        .current()
        .clone();
    let child_lease = driver
        .prepare_surface_lease(surface_request(child, child_viewport, display, generation))
        .and_then(|prepared| driver.publish_surface_lease(&prepared))
        .expect("child surface publishes")
        .current()
        .clone();

    let close = driver
        .begin_window_close_tree_after_quiesce(parent)
        .expect("fully published subtree begins one close transaction");

    assert_eq!(
        close
            .windows()
            .iter()
            .map(|entry| entry.window())
            .collect::<Vec<_>>(),
        vec![child, parent]
    );
    assert_eq!(
        close
            .retiring_leases()
            .iter()
            .map(|lease| lease.window())
            .collect::<Vec<_>>(),
        vec![child, parent]
    );
    assert_eq!(
        driver.with_window_registry(|registry| registry.native_for(parent)),
        Err(WindowRegistryError::ClosingWindow { window: parent })
    );
    assert_eq!(
        driver.with_window_registry(|registry| registry.native_for(child)),
        Err(WindowRegistryError::ClosingWindow { window: child })
    );
    assert_eq!(
        driver.with_surface_leases(|registry| registry.active(&parent_lease)),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: parent_lease.clone()
        })
    );
    assert_eq!(
        driver.with_surface_leases(|registry| registry.active(&child_lease)),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: child_lease.clone()
        })
    );
}

#[test]
fn all_surface_retirement_revokes_every_active_lease_without_closing_windows() {
    let driver = platform_driver();
    let display = "edid:surface-suspend-transaction";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let first_viewport = ZrRuntimeViewportHandle::new(24);
    let second_viewport = ZrRuntimeViewportHandle::new(25);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface publication");
    let (first_window, second_window) = driver
        .with_window_registry(|registry| {
            let first_window = registry
                .register(NativeWindowId::new(86).expect("first native window is nonzero"))?;
            let second_window = registry
                .register(NativeWindowId::new(87).expect("second native window is nonzero"))?;
            registry.bind_viewport(first_window, first_viewport)?;
            registry.bind_viewport(second_window, second_viewport)?;
            Ok((first_window, second_window))
        })
        .expect("surface fixtures register and bind their viewports");
    let first_lease = driver
        .prepare_surface_lease(surface_request(
            first_window,
            first_viewport,
            display,
            generation,
        ))
        .and_then(|prepared| driver.publish_surface_lease(&prepared))
        .expect("first surface publishes")
        .current()
        .clone();
    let second_lease = driver
        .prepare_surface_lease(surface_request(
            second_window,
            second_viewport,
            display,
            generation,
        ))
        .and_then(|prepared| driver.publish_surface_lease(&prepared))
        .expect("second surface publishes")
        .current()
        .clone();

    let retiring = driver
        .begin_all_surface_lease_retirement_after_quiesce()
        .expect("published surfaces retire as one transaction");

    assert_eq!(retiring, vec![first_lease.clone(), second_lease.clone()]);
    assert_eq!(
        driver.with_window_registry(|registry| registry.native_for(first_window)),
        Ok(NativeWindowId::new(86).expect("first native window remains live"))
    );
    assert_eq!(
        driver.with_window_registry(|registry| registry.native_for(second_window)),
        Ok(NativeWindowId::new(87).expect("second native window remains live"))
    );
    assert_eq!(
        driver.with_surface_leases(|registry| registry.active(&first_lease)),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: first_lease.clone()
        })
    );
    assert_eq!(
        driver.with_surface_leases(|registry| registry.active(&second_lease)),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: second_lease.clone()
        })
    );
}

#[test]
fn all_surface_retirement_rejects_preparation_without_revoking_an_active_lease() {
    let driver = platform_driver();
    let display = "edid:surface-suspend-preflight";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let active_viewport = ZrRuntimeViewportHandle::new(26);
    let prepared_viewport = ZrRuntimeViewportHandle::new(27);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface preparation");
    let (active_window, prepared_window) = driver
        .with_window_registry(|registry| {
            let active_window = registry
                .register(NativeWindowId::new(88).expect("active native window is nonzero"))?;
            let prepared_window = registry
                .register(NativeWindowId::new(89).expect("prepared native window is nonzero"))?;
            registry.bind_viewport(active_window, active_viewport)?;
            registry.bind_viewport(prepared_window, prepared_viewport)?;
            Ok((active_window, prepared_window))
        })
        .expect("surface fixtures register and bind their viewports");
    let active = driver
        .prepare_surface_lease(surface_request(
            active_window,
            active_viewport,
            display,
            generation,
        ))
        .and_then(|prepared| driver.publish_surface_lease(&prepared))
        .expect("active surface publishes")
        .current()
        .clone();
    let prepared = driver
        .prepare_surface_lease(surface_request(
            prepared_window,
            prepared_viewport,
            display,
            generation,
        ))
        .expect("second surface prepares");

    assert_eq!(
        driver.begin_all_surface_lease_retirement_after_quiesce(),
        Err(PlatformSurfaceLeaseError::Lease(
            SurfaceLeaseError::WindowHasPreparedLease {
                window: prepared_window
            }
        ))
    );
    assert_eq!(
        driver.with_surface_leases(|registry| registry.active(&active)),
        Ok(())
    );
    driver
        .cancel_surface_lease_preparation(&prepared)
        .expect("rejected retirement leaves the pending surface cancelable");
}

#[test]
fn window_surface_retirement_rejects_viewport_rebinding_without_mutating_lease_state() {
    let driver = platform_driver();
    let display = "edid:surface-window-retirement-binding";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let viewport = ZrRuntimeViewportHandle::new(28);
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface publication");
    let (lease_window, rebound_window) = driver
        .with_window_registry(|registry| {
            let lease_window = registry
                .register(NativeWindowId::new(90).expect("lease native window is nonzero"))?;
            let rebound_window = registry
                .register(NativeWindowId::new(91).expect("rebound native window is nonzero"))?;
            registry.bind_viewport(lease_window, viewport)?;
            Ok((lease_window, rebound_window))
        })
        .expect("surface fixtures register and bind their viewport");
    let lease = driver
        .prepare_surface_lease(surface_request(lease_window, viewport, display, generation))
        .and_then(|prepared| driver.publish_surface_lease(&prepared))
        .expect("surface publishes before the binding divergence")
        .current()
        .clone();
    driver
        .with_window_registry(|registry| {
            registry.unbind_viewport(lease_window, viewport)?;
            registry.bind_viewport(rebound_window, viewport)
        })
        .expect("test moves the registry binding without touching the lease registry");

    assert_eq!(
        driver.begin_window_surface_retirement(lease_window),
        Err(PlatformSurfaceLeaseError::Registry(
            WindowRegistryError::ViewportBoundToDifferentWindow {
                viewport,
                expected_window: lease_window,
                observed_window: rebound_window,
            }
        ))
    );
    assert_eq!(
        driver.with_surface_leases(|registry| registry.active(&lease)),
        Ok(())
    );
}

#[test]
fn application_suspend_waits_for_every_surface_retirement_receipt() {
    let driver = platform_driver();
    let display = "edid:surface-suspend-terminal";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let viewport = ZrRuntimeViewportHandle::new(29);
    let resume = driver
        .request_application_resume()
        .expect("cold application requests resume");
    driver
        .publish_application_running(resume)
        .expect("matching resume enters running");
    driver
        .publish_application_surface_availability(ApplicationSurfaceAvailability::Available)
        .expect("running application observes an available surface");
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface publication");
    let window = driver
        .with_window_registry(|registry| {
            let window = registry
                .register(NativeWindowId::new(92).expect("suspend native window is nonzero"))?;
            registry.bind_viewport(window, viewport)?;
            Ok(window)
        })
        .expect("suspend fixture registers and binds its viewport");
    let lease = driver
        .prepare_surface_lease(surface_request(window, viewport, display, generation))
        .and_then(|prepared| driver.publish_surface_lease(&prepared))
        .expect("surface publishes before suspension")
        .current()
        .clone();

    let suspend = driver
        .begin_application_suspend_after_quiesce()
        .expect("all active leases enter retirement with the suspend operation");
    assert_eq!(suspend.retiring_leases(), &[lease.clone()]);
    assert_eq!(
        driver.application_lifecycle_snapshot().state(),
        ApplicationLifecycleState::WillSuspend
    );
    assert_eq!(
        driver.publish_application_suspended(suspend.operation()),
        Err(PlatformApplicationSuspendError::SurfaceLeasesPending {
            active_count: 0,
            preparing_count: 0,
            retiring_count: 1,
        })
    );
    assert_eq!(
        driver.with_surface_leases(|registry| registry.active(&lease)),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: lease.clone(),
        })
    );

    driver
        .complete_surface_lease_retirement(&lease)
        .expect("graphics teardown completes the surface receipt");
    let suspended = driver
        .publish_application_suspended(suspend.operation())
        .expect("all completed receipts permit the lifecycle terminal transition");
    assert_eq!(suspended.state(), ApplicationLifecycleState::Suspended);
    assert_eq!(
        suspended.surface_availability(),
        ApplicationSurfaceAvailability::Unavailable
    );
}

#[test]
fn application_suspend_preflight_failure_leaves_lifecycle_and_leases_unchanged() {
    let driver = platform_driver();
    let display = "edid:surface-suspend-rollback";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let viewport = ZrRuntimeViewportHandle::new(30);
    let resume = driver
        .request_application_resume()
        .expect("cold application requests resume");
    driver
        .publish_application_running(resume)
        .expect("matching resume enters running");
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before surface preparation");
    let window = driver
        .with_window_registry(|registry| {
            let window = registry
                .register(NativeWindowId::new(93).expect("rollback native window is nonzero"))?;
            registry.bind_viewport(window, viewport)?;
            Ok(window)
        })
        .expect("rollback fixture registers and binds its viewport");
    let prepared = driver
        .prepare_surface_lease(surface_request(window, viewport, display, generation))
        .expect("surface preparation remains pending");

    assert_eq!(
        driver.begin_application_suspend_after_quiesce(),
        Err(PlatformApplicationSuspendError::Surface(
            PlatformSurfaceLeaseError::Lease(SurfaceLeaseError::WindowHasPreparedLease { window })
        ))
    );
    assert_eq!(
        driver.application_lifecycle_snapshot().state(),
        ApplicationLifecycleState::Running
    );
    driver
        .cancel_surface_lease_preparation(&prepared)
        .expect("rejected suspend leaves the prepared lease cancelable");
}

#[test]
fn will_suspend_rejects_new_surface_lease_preparation() {
    let driver = platform_driver();
    let display = "edid:surface-suspend-admission";
    let generation = DisplayTopologyGeneration::new(2).expect("generation is nonzero");
    let viewport = ZrRuntimeViewportHandle::new(31);
    let resume = driver
        .request_application_resume()
        .expect("cold application requests resume");
    driver
        .publish_application_running(resume)
        .expect("matching resume enters running");
    driver
        .publish_display_topology(display_topology(generation.get(), display))
        .expect("display topology publishes before suspension");
    let window = driver
        .with_window_registry(|registry| {
            let window = registry
                .register(NativeWindowId::new(94).expect("admission native window is nonzero"))?;
            registry.bind_viewport(window, viewport)?;
            Ok(window)
        })
        .expect("admission fixture registers and binds its viewport");

    let suspend = driver
        .begin_application_suspend_after_quiesce()
        .expect("empty surface set begins suspension");
    assert_eq!(suspend.retiring_leases(), &[]);
    assert_eq!(
        driver.prepare_surface_lease(surface_request(window, viewport, display, generation)),
        Err(PlatformSurfaceLeaseError::Lifecycle(
            ApplicationLifecycleServiceError::InvalidState {
                operation: "prepare surface lease",
                state: ApplicationLifecycleState::WillSuspend,
            }
        ))
    );
    driver
        .publish_application_suspended(suspend.operation())
        .expect("blocked preparation leaves suspension free of retirement receipts");
}

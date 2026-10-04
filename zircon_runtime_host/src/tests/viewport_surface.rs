use std::sync::Arc;

use super::ViewportSurfaceBindings;
use zircon_runtime_interface::ZrRuntimeViewportHandle;

#[test]
fn failed_rebind_restores_the_previous_viewport_surface_binding() {
    let viewport = ZrRuntimeViewportHandle::new(7);
    let bindings = ViewportSurfaceBindings::default();

    let initial_bind = bindings
        .begin_binding(viewport)
        .expect("first viewport binding begins");
    assert!(initial_bind.finish(true));

    let rebind = bindings
        .begin_binding(viewport)
        .expect("existing viewport rebind begins");
    assert!(rebind.finish(false));
    assert_eq!(bindings.bound_viewports(), vec![viewport]);
}

#[test]
fn bound_viewports_are_released_in_stable_handle_order() {
    let bindings = ViewportSurfaceBindings::default();
    let first = ZrRuntimeViewportHandle::new(21);
    let second = ZrRuntimeViewportHandle::new(3);

    let first_bind = bindings
        .begin_binding(first)
        .expect("first viewport binding begins");
    let second_bind = bindings
        .begin_binding(second)
        .expect("second viewport binding begins");
    assert!(first_bind.finish(true));
    assert!(second_bind.finish(true));

    assert_eq!(bindings.bound_viewports(), vec![second, first]);
}

#[test]
fn in_flight_binding_rejects_a_concurrent_release() {
    let viewport = ZrRuntimeViewportHandle::new(4);
    let bindings = ViewportSurfaceBindings::default();
    let binding = bindings
        .begin_binding(viewport)
        .expect("viewport binding begins");

    assert_eq!(
        bindings
            .begin_release(viewport)
            .expect_err("binding viewport rejects a concurrent release")
            .viewport(),
        viewport
    );
    assert_eq!(
        bindings
            .begin_binding(viewport)
            .expect_err("binding viewport rejects a concurrent rebind")
            .viewport(),
        viewport
    );
    assert!(!binding.finish(false));
}

#[test]
fn failed_release_restores_the_viewport_surface_binding_for_retry() {
    let viewport = ZrRuntimeViewportHandle::new(12);
    let bindings = ViewportSurfaceBindings::default();
    let binding = bindings
        .begin_binding(viewport)
        .expect("viewport binding begins");
    assert!(binding.finish(true));

    let first_release = bindings
        .begin_release(viewport)
        .expect("bound viewport begins release")
        .expect("bound viewport has a release operation");
    assert!(first_release.finish(false));

    let retry = bindings
        .begin_release(viewport)
        .expect("failed release restores a retryable binding")
        .expect("restored binding has a release operation");
    assert!(!retry.finish(true));
}

#[test]
fn abandoned_binding_reservation_is_rolled_back() {
    let viewport = ZrRuntimeViewportHandle::new(9);
    let bindings = ViewportSurfaceBindings::default();

    let binding = bindings
        .begin_binding(viewport)
        .expect("viewport binding begins");
    drop(binding);

    assert!(bindings.bound_viewports().is_empty());
    assert!(bindings.begin_binding(viewport).is_ok());
}

#[test]
fn abandoned_release_reservation_restores_the_published_binding() {
    let viewport = ZrRuntimeViewportHandle::new(10);
    let bindings = ViewportSurfaceBindings::default();
    let binding = bindings
        .begin_binding(viewport)
        .expect("viewport binding begins");
    assert!(binding.finish(true));

    let release = bindings
        .begin_release(viewport)
        .expect("bound viewport begins release")
        .expect("bound viewport has a release operation");
    drop(release);

    assert_eq!(bindings.bound_viewports(), vec![viewport]);
}

#[test]
fn in_flight_release_rejects_a_concurrent_rebind() {
    let viewport = ZrRuntimeViewportHandle::new(13);
    let bindings = ViewportSurfaceBindings::default();
    let binding = bindings
        .begin_binding(viewport)
        .expect("viewport binding begins");
    assert!(binding.finish(true));

    let release = bindings
        .begin_release(viewport)
        .expect("bound viewport begins release")
        .expect("bound viewport has a release operation");
    assert_eq!(
        bindings
            .begin_binding(viewport)
            .expect_err("releasing viewport rejects a concurrent rebind")
            .viewport(),
        viewport
    );
    assert!(release.finish(false));
}

#[test]
fn shared_owners_observe_the_same_viewport_lifecycle() {
    let viewport = ZrRuntimeViewportHandle::new(14);
    let session_bindings = Arc::new(ViewportSurfaceBindings::default());
    let gateway_bindings = Arc::clone(&session_bindings);

    let binding = gateway_bindings
        .begin_binding(viewport)
        .expect("gateway binding begins");
    assert!(binding.finish(true));
    assert_eq!(session_bindings.bound_viewports(), vec![viewport]);

    let release = session_bindings
        .begin_release(viewport)
        .expect("session release begins")
        .expect("published binding has a release operation");
    assert!(!release.finish(true));
    assert!(gateway_bindings.bound_viewports().is_empty());
}

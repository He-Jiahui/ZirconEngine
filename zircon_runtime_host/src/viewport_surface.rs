use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use zircon_runtime_interface::ZrRuntimeViewportHandle;

/// Shared host-side ownership for viewport-to-native-surface bindings.
///
/// A caller reserves a transition, makes the ABI call without holding this
/// registry's mutex, then completes the transition with the call result. If a
/// caller exits before completion, the transition token restores the last
/// published state when it is dropped.
#[derive(Default)]
pub struct ViewportSurfaceBindings {
    registry: Mutex<ViewportSurfaceBindingRegistry>,
}

#[derive(Debug, Default)]
struct ViewportSurfaceBindingRegistry {
    states: HashMap<ZrRuntimeViewportHandle, ViewportSurfaceBindingState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewportSurfaceBindingState {
    Bound,
    Binding,
    Releasing,
}

/// A reserved bind or rebind operation.
///
/// Dropping an unfinished operation restores its preceding published state.
#[must_use = "an unfinished binding operation is rolled back when dropped"]
#[derive(Debug)]
pub struct ViewportSurfaceBindingOperation<'a> {
    registry: &'a Mutex<ViewportSurfaceBindingRegistry>,
    viewport: ZrRuntimeViewportHandle,
    was_bound: bool,
    completed: bool,
}

/// A reserved unbind operation.
///
/// Dropping an unfinished operation restores the published binding.
#[must_use = "an unfinished release operation is rolled back when dropped"]
#[derive(Debug)]
pub struct ViewportSurfaceReleaseOperation<'a> {
    registry: &'a Mutex<ViewportSurfaceBindingRegistry>,
    viewport: ZrRuntimeViewportHandle,
    completed: bool,
}

/// Reports that a viewport already has a transition in progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportSurfaceOperationInFlight {
    viewport: ZrRuntimeViewportHandle,
}

impl ViewportSurfaceOperationInFlight {
    pub const fn viewport(self) -> ZrRuntimeViewportHandle {
        self.viewport
    }
}

impl ViewportSurfaceBindings {
    /// Reserves a bind or rebind for `viewport`.
    pub fn begin_binding(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<ViewportSurfaceBindingOperation<'_>, ViewportSurfaceOperationInFlight> {
        let mut registry = self.lock_registry();
        let was_bound = match registry.states.get(&viewport) {
            None => false,
            Some(ViewportSurfaceBindingState::Bound) => true,
            Some(ViewportSurfaceBindingState::Binding | ViewportSurfaceBindingState::Releasing) => {
                return Err(ViewportSurfaceOperationInFlight { viewport });
            }
        };
        registry
            .states
            .insert(viewport, ViewportSurfaceBindingState::Binding);
        Ok(ViewportSurfaceBindingOperation {
            registry: &self.registry,
            viewport,
            was_bound,
            completed: false,
        })
    }

    /// Reserves an unbind for `viewport`.
    ///
    /// `Ok(None)` means the viewport has no published binding and no ABI call
    /// is required.
    pub fn begin_release(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<Option<ViewportSurfaceReleaseOperation<'_>>, ViewportSurfaceOperationInFlight> {
        let mut registry = self.lock_registry();
        match registry.states.get(&viewport) {
            None => Ok(None),
            Some(ViewportSurfaceBindingState::Bound) => {
                registry
                    .states
                    .insert(viewport, ViewportSurfaceBindingState::Releasing);
                Ok(Some(ViewportSurfaceReleaseOperation {
                    registry: &self.registry,
                    viewport,
                    completed: false,
                }))
            }
            Some(ViewportSurfaceBindingState::Binding | ViewportSurfaceBindingState::Releasing) => {
                Err(ViewportSurfaceOperationInFlight { viewport })
            }
        }
    }

    /// Returns all currently bound viewports in deterministic teardown order.
    pub fn bound_viewports(&self) -> Vec<ZrRuntimeViewportHandle> {
        self.lock_registry().bound_viewports()
    }

    fn lock_registry(&self) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
        lock_registry(&self.registry)
    }
}

impl ViewportSurfaceBindingOperation<'_> {
    /// Commits or rolls back the reserved bind after the ABI call returns.
    ///
    /// Returns whether at least one viewport remains bound.
    pub fn finish(mut self, succeeded: bool) -> bool {
        let any_bound = {
            let mut registry = self.lock_registry();
            if !matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Binding)
            ) {
                registry.has_bound_viewports()
            } else {
                if succeeded || self.was_bound {
                    registry
                        .states
                        .insert(self.viewport, ViewportSurfaceBindingState::Bound);
                } else {
                    registry.states.remove(&self.viewport);
                }
                registry.has_bound_viewports()
            }
        };
        self.completed = true;
        any_bound
    }

    fn lock_registry(&self) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
        lock_registry(self.registry)
    }

    fn rollback(&mut self) {
        if self.completed {
            return;
        }
        {
            let mut registry = self.lock_registry();
            if matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Binding)
            ) {
                if self.was_bound {
                    registry
                        .states
                        .insert(self.viewport, ViewportSurfaceBindingState::Bound);
                } else {
                    registry.states.remove(&self.viewport);
                }
            }
        }
        self.completed = true;
    }
}

impl Drop for ViewportSurfaceBindingOperation<'_> {
    fn drop(&mut self) {
        self.rollback();
    }
}

impl ViewportSurfaceReleaseOperation<'_> {
    /// Commits or rolls back the reserved unbind after the ABI call returns.
    ///
    /// Returns whether at least one viewport remains bound.
    pub fn finish(mut self, succeeded: bool) -> bool {
        let any_bound = {
            let mut registry = self.lock_registry();
            if !matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Releasing)
            ) {
                registry.has_bound_viewports()
            } else {
                if succeeded {
                    registry.states.remove(&self.viewport);
                } else {
                    registry
                        .states
                        .insert(self.viewport, ViewportSurfaceBindingState::Bound);
                }
                registry.has_bound_viewports()
            }
        };
        self.completed = true;
        any_bound
    }

    fn lock_registry(&self) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
        lock_registry(self.registry)
    }

    fn rollback(&mut self) {
        if self.completed {
            return;
        }
        {
            let mut registry = self.lock_registry();
            if matches!(
                registry.states.get(&self.viewport),
                Some(ViewportSurfaceBindingState::Releasing)
            ) {
                registry
                    .states
                    .insert(self.viewport, ViewportSurfaceBindingState::Bound);
            }
        }
        self.completed = true;
    }
}

impl Drop for ViewportSurfaceReleaseOperation<'_> {
    fn drop(&mut self) {
        self.rollback();
    }
}

impl ViewportSurfaceBindingRegistry {
    fn has_bound_viewports(&self) -> bool {
        self.states
            .values()
            .any(|state| *state == ViewportSurfaceBindingState::Bound)
    }

    fn bound_viewports(&self) -> Vec<ZrRuntimeViewportHandle> {
        let mut viewports = self
            .states
            .iter()
            .filter_map(|(viewport, state)| {
                (*state == ViewportSurfaceBindingState::Bound).then_some(*viewport)
            })
            .collect::<Vec<_>>();
        viewports.sort_unstable_by_key(|viewport| viewport.raw());
        viewports
    }
}

fn lock_registry(
    registry: &Mutex<ViewportSurfaceBindingRegistry>,
) -> MutexGuard<'_, ViewportSurfaceBindingRegistry> {
    registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
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
}

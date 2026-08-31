use std::sync::MutexGuard;

use zircon_runtime_interface::ZrRuntimeViewportHandle;

use crate::core::framework::platform::{
    ApplicationActivationState, ApplicationLifecycleOperation, ApplicationLifecycleSnapshot,
    ApplicationLifecycleState, ApplicationSurfaceAvailability,
};
use crate::core::framework::window::{
    PreparedSurfaceLease, SurfaceLease, SurfaceLeaseError, SurfaceLeasePublication,
    SurfaceLeaseRegistry, SurfaceLeaseRequest, WindowId,
};
use crate::platform::{
    ApplicationLifecycleServiceError, PlatformApplicationSuspendError,
    PlatformApplicationSuspendTransaction, PlatformSurfaceLeaseError, PlatformWindowCloseError,
    PlatformWindowCloseTransaction, WindowRegistry, WindowRegistryError,
};

use super::{PlatformDriver, WindowRegistryState};

impl PlatformDriver {
    /// Returns application lifecycle facts without conflating them with a
    /// focused window, visibility, or any native platform object. The shared
    /// gate prevents a reader from observing a partially committed suspend.
    pub fn application_lifecycle_snapshot(&self) -> ApplicationLifecycleSnapshot {
        let _gate = self.lock_surface_lifecycle_gate();
        self.application_lifecycle.snapshot()
    }

    pub(crate) fn publish_application_activation(
        &self,
        activation: ApplicationActivationState,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let _gate = self.lock_surface_lifecycle_gate();
        self.application_lifecycle.publish_activation(activation)
    }

    pub(crate) fn publish_application_surface_availability(
        &self,
        surface_availability: ApplicationSurfaceAvailability,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let _gate = self.lock_surface_lifecycle_gate();
        self.application_lifecycle
            .publish_surface_availability(surface_availability)
    }

    pub(crate) fn request_application_resume(
        &self,
    ) -> Result<ApplicationLifecycleOperation, ApplicationLifecycleServiceError> {
        let _gate = self.lock_surface_lifecycle_gate();
        self.application_lifecycle.request_resume()
    }

    pub(crate) fn publish_application_running(
        &self,
        operation: ApplicationLifecycleOperation,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let _gate = self.lock_surface_lifecycle_gate();
        self.application_lifecycle.publish_running(operation)
    }

    pub(crate) fn publish_application_suspended(
        &self,
        operation: ApplicationLifecycleOperation,
    ) -> Result<ApplicationLifecycleSnapshot, PlatformApplicationSuspendError> {
        let _gate = self.lock_surface_lifecycle_gate();
        let surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let active_count = surface_leases.active_count();
        let preparing_count = surface_leases.preparing_count();
        let retiring_count = surface_leases.retiring_count();
        if active_count != 0 || preparing_count != 0 || retiring_count != 0 {
            return Err(PlatformApplicationSuspendError::SurfaceLeasesPending {
                active_count,
                preparing_count,
                retiring_count,
            });
        }
        self.application_lifecycle
            .publish_suspended(operation)
            .map_err(PlatformApplicationSuspendError::Lifecycle)
    }

    pub(crate) fn begin_application_exit(
        &self,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let _gate = self.lock_surface_lifecycle_gate();
        self.application_lifecycle.begin_exit()
    }

    /// Prepares a surface only for a live, routable native window generation.
    /// Graphics creation remains outside the lock and must publish or cancel
    /// this candidate through the paired driver entry points.
    pub(crate) fn prepare_surface_lease(
        &self,
        request: SurfaceLeaseRequest,
    ) -> Result<PreparedSurfaceLease, PlatformSurfaceLeaseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        self.ensure_surface_lease_admission("prepare surface lease")?;
        let topology = self.read_display_topology();
        let registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &*registry_state else {
            return Err(PlatformSurfaceLeaseError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        registry
            .native_for(request.window())
            .map_err(PlatformSurfaceLeaseError::Registry)?;
        ensure_surface_lease_viewport_owner(registry, request.window(), request.viewport())
            .map_err(PlatformSurfaceLeaseError::Registry)?;
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        surface_leases
            .prepare(request, topology.as_ref())
            .map_err(PlatformSurfaceLeaseError::Lease)
    }

    /// Publishes a candidate only while its native window generation remains
    /// routable and its display-topology facts are still current.
    pub(crate) fn publish_surface_lease(
        &self,
        prepared: &PreparedSurfaceLease,
    ) -> Result<SurfaceLeasePublication, PlatformSurfaceLeaseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        self.ensure_surface_lease_admission("publish surface lease")?;
        let topology = self.read_display_topology();
        let registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &*registry_state else {
            return Err(PlatformSurfaceLeaseError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        registry
            .native_for(prepared.candidate().window())
            .map_err(PlatformSurfaceLeaseError::Registry)?;
        ensure_surface_lease_viewport_owner(
            registry,
            prepared.candidate().window(),
            prepared.candidate().viewport(),
        )
        .map_err(PlatformSurfaceLeaseError::Registry)?;
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        surface_leases
            .publish(prepared, topology.as_ref())
            .map_err(PlatformSurfaceLeaseError::Lease)
    }

    /// Revokes all active routes for a live window generation before graphics
    /// fences and native surface destruction begin.
    pub(crate) fn begin_window_surface_retirement(
        &self,
        window: WindowId,
    ) -> Result<Vec<SurfaceLease>, PlatformSurfaceLeaseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        let registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &*registry_state else {
            return Err(PlatformSurfaceLeaseError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        registry
            .native_for(window)
            .map_err(PlatformSurfaceLeaseError::Registry)?;
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let retirement_plan = surface_leases
            .plan_window_retirement(&[window])
            .map_err(PlatformSurfaceLeaseError::Lease)?;
        for lease in retirement_plan.leases() {
            ensure_surface_lease_viewport_owner(registry, lease.window(), lease.viewport())
                .map_err(PlatformSurfaceLeaseError::Registry)?;
        }
        Ok(surface_leases.commit_retirement(retirement_plan))
    }

    /// Commits the platform-owned half of a close only after host commands
    /// and graphics submission work have quiesced. Graphics and native owners
    /// must consume the returned receipts child-first before `finish_destroy`.
    pub(crate) fn begin_window_close_tree_after_quiesce(
        &self,
        root: WindowId,
    ) -> Result<PlatformWindowCloseTransaction, PlatformWindowCloseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        let mut registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &mut *registry_state else {
            return Err(PlatformWindowCloseError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        let close_order = registry
            .preflight_close_tree(root)
            .map_err(PlatformWindowCloseError::Registry)?;
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let retirement_plan = surface_leases
            .plan_window_retirement(&close_order)
            .map_err(PlatformWindowCloseError::Lease)?;
        for lease in retirement_plan.leases() {
            ensure_surface_lease_viewport_owner(registry, lease.window(), lease.viewport())
                .map_err(PlatformWindowCloseError::Registry)?;
        }
        let windows = registry
            .begin_close_order_after_preflight(close_order)
            .map_err(PlatformWindowCloseError::Registry)?;
        let retiring_leases = surface_leases.commit_retirement(retirement_plan);
        Ok(PlatformWindowCloseTransaction::new(
            windows,
            retiring_leases,
        ))
    }

    /// Makes every currently active surface lease non-routable only after the
    /// host and graphics owners quiesced their submit work. Native windows
    /// remain live, so `destroy_surfaces` and suspend can release graphics
    /// resources without implicitly closing the application window tree.
    pub(crate) fn begin_all_surface_lease_retirement_after_quiesce(
        &self,
    ) -> Result<Vec<SurfaceLease>, PlatformSurfaceLeaseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        let registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &*registry_state else {
            return Err(PlatformSurfaceLeaseError::Registry(
                WindowRegistryError::RegistryIdentityExhausted,
            ));
        };
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let retirement_plan = surface_leases
            .plan_all_retirement()
            .map_err(PlatformSurfaceLeaseError::Lease)?;
        for lease in retirement_plan.leases() {
            ensure_surface_lease_viewport_owner(registry, lease.window(), lease.viewport())
                .map_err(PlatformSurfaceLeaseError::Registry)?;
        }
        Ok(surface_leases.commit_retirement(retirement_plan))
    }

    /// Enters `WillSuspend` only after all active surface routes have a
    /// retirement receipt. The caller must complete every receipt before it
    /// publishes `Suspended`, so native windows cannot outlive their surfaces.
    pub(crate) fn begin_application_suspend_after_quiesce(
        &self,
    ) -> Result<PlatformApplicationSuspendTransaction, PlatformApplicationSuspendError> {
        let _gate = self.lock_surface_lifecycle_gate();
        if self.application_lifecycle.snapshot().state() == ApplicationLifecycleState::WillSuspend {
            return Err(PlatformApplicationSuspendError::Lifecycle(
                ApplicationLifecycleServiceError::InvalidState {
                    operation: "begin application suspend",
                    state: ApplicationLifecycleState::WillSuspend,
                },
            ));
        }
        let registry_state = self
            .window_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let WindowRegistryState::Available(registry) = &*registry_state else {
            return Err(PlatformApplicationSuspendError::Surface(
                PlatformSurfaceLeaseError::Registry(WindowRegistryError::RegistryIdentityExhausted),
            ));
        };
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let retirement_plan = surface_leases.plan_all_retirement().map_err(|error| {
            PlatformApplicationSuspendError::Surface(PlatformSurfaceLeaseError::Lease(error))
        })?;
        for lease in retirement_plan.leases() {
            ensure_surface_lease_viewport_owner(registry, lease.window(), lease.viewport())
                .map_err(|error| {
                    PlatformApplicationSuspendError::Surface(PlatformSurfaceLeaseError::Registry(
                        error,
                    ))
                })?;
        }
        let operation = self
            .application_lifecycle
            .request_suspend()
            .map_err(PlatformApplicationSuspendError::Lifecycle)?;
        let retiring_leases = surface_leases.commit_retirement(retirement_plan);
        Ok(PlatformApplicationSuspendTransaction::new(
            operation,
            retiring_leases,
        ))
    }

    /// Cleanup remains valid after `WindowRegistry` has marked the generation
    /// closing, so cancellation intentionally does not revalidate native state.
    pub(crate) fn cancel_surface_lease_preparation(
        &self,
        prepared: &PreparedSurfaceLease,
    ) -> Result<(), SurfaceLeaseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        surface_leases.cancel(prepared)
    }

    /// Completes graphics cleanup for a retiring lease after the native window
    /// may already be in Closing state.
    pub(crate) fn complete_surface_lease_retirement(
        &self,
        lease: &SurfaceLease,
    ) -> Result<(), SurfaceLeaseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        surface_leases.complete_retirement(lease)
    }

    /// Test-only access to registry diagnostics. Production callers must use
    /// the generation-validating driver operations above.
    #[cfg(test)]
    pub(crate) fn with_surface_leases<T>(
        &self,
        operation: impl FnOnce(&mut SurfaceLeaseRegistry) -> Result<T, SurfaceLeaseError>,
    ) -> Result<T, SurfaceLeaseError> {
        let _gate = self.lock_surface_lifecycle_gate();
        let mut surface_leases = self
            .surface_leases
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        operation(&mut surface_leases)
    }

    fn lock_surface_lifecycle_gate(&self) -> MutexGuard<'_, ()> {
        self.surface_lifecycle_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The gate is held by every lifecycle transition and lease mutation, so
    /// this snapshot cannot race a transition into a surface-forbidden state.
    fn ensure_surface_lease_admission(
        &self,
        operation: &'static str,
    ) -> Result<(), PlatformSurfaceLeaseError> {
        let state = self.application_lifecycle.snapshot().state();
        if matches!(
            state,
            ApplicationLifecycleState::WillSuspend
                | ApplicationLifecycleState::Suspended
                | ApplicationLifecycleState::Exiting
        ) {
            return Err(PlatformSurfaceLeaseError::Lifecycle(
                ApplicationLifecycleServiceError::InvalidState { operation, state },
            ));
        }
        Ok(())
    }
}

/// `WindowRegistry` owns the routable viewport route, while
/// `SurfaceLeaseRegistry` owns graphics-surface generations. Every driver
/// transition must prove that both owners still name the same window generation.
fn ensure_surface_lease_viewport_owner(
    registry: &WindowRegistry,
    expected_window: WindowId,
    viewport: ZrRuntimeViewportHandle,
) -> Result<(), WindowRegistryError> {
    let observed_window = registry.window_for_viewport(viewport)?;
    if observed_window != expected_window {
        return Err(WindowRegistryError::ViewportBoundToDifferentWindow {
            viewport,
            expected_window,
            observed_window,
        });
    }
    Ok(())
}

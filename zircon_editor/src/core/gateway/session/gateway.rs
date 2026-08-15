use std::sync::{atomic::AtomicBool, Arc};

use zircon_runtime_interface::{
    ZrRuntimeApiV6, ZrRuntimeSessionHandle, ZIRCON_RUNTIME_API_VERSION_V6,
};

use super::super::{GatewayError, RuntimeCapabilities};

/// Editor-owned facade over a validated runtime session API table.
pub struct SessionGateway {
    pub(super) _runtime_owner: Arc<dyn Send + Sync>,
    pub(super) api: ZrRuntimeApiV6,
    pub(super) session: ZrRuntimeSessionHandle,
    pub(super) capabilities: Arc<RuntimeCapabilities>,
    pub(super) viewport_surface_bound: Arc<AtomicBool>,
}

impl SessionGateway {
    /// Creates a gateway over a validated runtime API table.
    ///
    /// # Safety
    ///
    /// `runtime_owner` must keep the library or linked provider that supplied every
    /// function pointer in `api` loaded until the gateway is dropped.
    pub unsafe fn new(
        runtime_owner: Arc<dyn Send + Sync>,
        api: ZrRuntimeApiV6,
        session: ZrRuntimeSessionHandle,
        capabilities: RuntimeCapabilities,
    ) -> Result<Self, GatewayError> {
        if !session.is_valid() {
            return Err(GatewayError::SessionLost);
        }
        if api.abi_version != ZIRCON_RUNTIME_API_VERSION_V6 {
            return Err(GatewayError::Protocol {
                message: format!(
                    "session gateway requires runtime API V6, received version {}",
                    api.abi_version
                ),
            });
        }
        Ok(Self {
            _runtime_owner: runtime_owner,
            api,
            session,
            capabilities: Arc::new(capabilities),
            viewport_surface_bound: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Shares the surface lifecycle marker owned by the runtime session.
    ///
    /// The editor gateway invokes the normalized V6 functions directly, while
    /// `RuntimeSession` retains destruction authority. Both sides therefore
    /// need the same marker so session teardown can unbind a surface that the
    /// gateway successfully bound.
    pub fn with_viewport_surface_lifecycle_state(
        mut self,
        viewport_surface_bound: Arc<AtomicBool>,
    ) -> Self {
        self.viewport_surface_bound = viewport_surface_bound;
        self
    }

    pub(super) fn required<T: Copy>(
        entry: Option<T>,
        capability: &'static str,
    ) -> Result<T, GatewayError> {
        entry.ok_or(GatewayError::CapabilityMissing { capability })
    }
}

impl std::fmt::Debug for SessionGateway {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionGateway")
            .field("session", &self.session)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

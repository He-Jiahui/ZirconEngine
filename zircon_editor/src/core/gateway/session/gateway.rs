use std::sync::{atomic::AtomicBool, Arc};

use serde::de::DeserializeOwned;
use zircon_runtime_host::foreign_output::{
    RuntimeForeignOutputBudget, RuntimeForeignOutputKind, RuntimeForeignOutputState,
};
use zircon_runtime_interface::{
    ZrOwnedByteBuffer, ZrRuntimeApiV6, ZrRuntimeSessionHandle, ZrStatus,
    ZIRCON_RUNTIME_API_VERSION_V6,
};

use super::super::{GatewayError, RuntimeCapabilities};

/// Editor-owned facade over a validated runtime session API table.
pub struct SessionGateway {
    pub(super) _runtime_owner: Arc<dyn Send + Sync>,
    pub(super) api: ZrRuntimeApiV6,
    pub(super) session: ZrRuntimeSessionHandle,
    pub(super) capabilities: Arc<RuntimeCapabilities>,
    pub(super) viewport_surface_bound: Arc<AtomicBool>,
    pub(super) foreign_output: Arc<RuntimeForeignOutputState>,
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
        foreign_output: Arc<RuntimeForeignOutputState>,
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
            foreign_output,
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

    pub(super) fn ensure_session_available(
        &self,
        operation: &'static str,
    ) -> Result<(), GatewayError> {
        self.foreign_output
            .ensure_session_available(operation)
            .map_err(Into::into)
    }

    pub(super) fn ensure_output_available(
        &self,
        kind: RuntimeForeignOutputKind,
    ) -> Result<(), GatewayError> {
        self.foreign_output
            .ensure_available(kind)
            .map_err(Into::into)
    }

    pub(super) fn reject_protocol<T>(
        &self,
        kind: RuntimeForeignOutputKind,
        error: impl std::fmt::Display,
    ) -> Result<T, GatewayError> {
        self.foreign_output
            .reject_protocol(kind, error)
            .map_err(Into::into)
    }

    pub(super) fn decode_output<T, E>(
        &self,
        status: ZrStatus,
        output: ZrOwnedByteBuffer,
        kind: RuntimeForeignOutputKind,
        budget: RuntimeForeignOutputBudget,
        operation: &'static str,
        release_operation: &'static str,
        validate: impl FnOnce(&T) -> Result<usize, E>,
    ) -> Result<Option<T>, GatewayError>
    where
        T: DeserializeOwned,
        E: std::fmt::Display,
    {
        self.foreign_output.ensure_call_succeeded(
            status,
            output,
            kind,
            operation,
            release_operation,
        )?;
        self.foreign_output
            .decode_json(output, kind, budget, operation, release_operation, validate)
            .map_err(Into::into)
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

use std::sync::Arc;

use serde::de::DeserializeOwned;
use zircon_runtime_host::foreign_output::{
    RuntimeForeignOutputBudget, RuntimeForeignOutputKind, RuntimeForeignOutputState,
    RuntimeOwnedOutputReleaser,
};
use zircon_runtime_host::viewport_surface::ViewportSurfaceBindings;
use zircon_runtime_interface::runtime_build_set::{
    ZrRuntimeModuleCompositionReceiptV1, ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
};
use zircon_runtime_interface::{
    validate_runtime_api_v8_shape, ZrOwnedResultV2, ZrRuntimeApiV8, ZrRuntimeSessionHandle,
    ZrStatus,
};

use super::super::{GatewayError, GatewaySessionIdentity, RuntimeCapabilities};

/// Editor-owned facade over a validated runtime session API table.
pub struct SessionGateway {
    pub(super) _runtime_owner: Arc<dyn Send + Sync>,
    pub(super) api: ZrRuntimeApiV8,
    pub(super) output_releaser: RuntimeOwnedOutputReleaser,
    pub(super) session: ZrRuntimeSessionHandle,
    pub(super) identity: GatewaySessionIdentity,
    pub(super) capabilities: Arc<RuntimeCapabilities>,
    pub(super) module_composition_receipt: Option<Arc<ZrRuntimeModuleCompositionReceiptV1>>,
    pub(super) viewport_surface_bindings: Arc<ViewportSurfaceBindings>,
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
        api: ZrRuntimeApiV8,
        session: ZrRuntimeSessionHandle,
        capabilities: RuntimeCapabilities,
        foreign_output: Arc<RuntimeForeignOutputState>,
    ) -> Result<Self, GatewayError> {
        Self::new_with_identity(
            runtime_owner,
            api,
            session,
            GatewaySessionIdentity::new(0, session, 0, None),
            capabilities,
            foreign_output,
        )
    }

    /// Creates a gateway with the App-issued identity for its owning runtime session.
    ///
    /// The descriptor is validated against the ABI session handle before the gateway is exposed.
    pub unsafe fn new_with_identity(
        runtime_owner: Arc<dyn Send + Sync>,
        api: ZrRuntimeApiV8,
        session: ZrRuntimeSessionHandle,
        identity: GatewaySessionIdentity,
        capabilities: RuntimeCapabilities,
        foreign_output: Arc<RuntimeForeignOutputState>,
    ) -> Result<Self, GatewayError> {
        if !session.is_valid() {
            return Err(GatewayError::SessionLost);
        }
        if identity.runtime_session() != session {
            return Err(GatewayError::Protocol {
                message: "gateway session identity does not match the ABI session handle"
                    .to_owned(),
            });
        }
        // V8 is a frozen exact table, not a prefix-compatible ABI family.
        validate_runtime_api_v8_shape(&api).map_err(|error| GatewayError::Protocol {
            message: error.to_string(),
        })?;
        let release_allocation =
            Self::required(api.release_allocation, "runtime.allocation.release")?;
        Self::required(api.request_viewport_pick, "runtime.viewport.pick.request")?;
        Self::required(api.poll_viewport_pick, "runtime.viewport.pick.poll")?;
        Self::required(api.cancel_viewport_pick, "runtime.viewport.pick.cancel")?;
        Ok(Self {
            _runtime_owner: runtime_owner,
            api,
            output_releaser: RuntimeOwnedOutputReleaser::new(session, release_allocation),
            session,
            identity,
            capabilities: Arc::new(capabilities),
            module_composition_receipt: None,
            viewport_surface_bindings: Arc::new(ViewportSurfaceBindings::default()),
            foreign_output,
        })
    }

    pub fn with_module_composition_receipt(
        mut self,
        receipt: ZrRuntimeModuleCompositionReceiptV1,
    ) -> Result<Self, GatewayError> {
        if receipt.schema_version != ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1 {
            return Err(GatewayError::Protocol {
                message: format!(
                    "runtime module composition receipt requires schema {}; received {}",
                    ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1, receipt.schema_version
                ),
            });
        }
        self.module_composition_receipt = Some(Arc::new(receipt));
        Ok(self)
    }

    /// Shares viewport-surface ownership with the runtime session.
    ///
    /// The editor gateway invokes the normalized V8 functions directly, while
    /// `RuntimeSession` retains destruction authority. Both sides therefore
    /// need the same registry so session teardown can unbind every surface
    /// that the gateway successfully bound.
    pub fn with_viewport_surface_bindings(
        mut self,
        viewport_surface_bindings: Arc<ViewportSurfaceBindings>,
    ) -> Self {
        self.viewport_surface_bindings = viewport_surface_bindings;
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
        output: ZrOwnedResultV2,
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
        let output = self.foreign_output.ensure_call_succeeded(
            status,
            output,
            self.output_releaser,
            kind,
            operation,
            release_operation,
        )?;
        self.foreign_output
            .decode_json(
                output,
                self.output_releaser,
                kind,
                budget,
                operation,
                release_operation,
                validate,
            )
            .map_err(Into::into)
    }
}

impl std::fmt::Debug for SessionGateway {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionGateway")
            .field("session", &self.session)
            .field("capabilities", &self.capabilities)
            .field(
                "module_composition_receipt",
                &self.module_composition_receipt,
            )
            .finish_non_exhaustive()
    }
}

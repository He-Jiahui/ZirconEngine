use std::marker::PhantomData;
use std::sync::Arc;

use zircon_runtime_interface::{
    ZrHostApiV4, ZrHostAssetApiV1, ZrHostBridgeApiV1, ZrHostDiagnosticsApiV1, ZrHostEcsApiV2,
    ZrHostEventApiV1, ZrRuntimePluginHandle,
};

use crate::plugin::{PluginModuleId, RuntimeExtensionRegistry};

use super::abi_decode::{NativeHostApiAdapterError, NativeHostApiAdapterResult};
use super::bridge_scope::native_host_bridge_call_v1;
use super::context_handles::{
    insert_context, remove_context, NativeHostApiV3Context, NativeHostRegistrationScopeState,
};
use super::ecs_registration::{
    native_host_asset_request_v1, native_host_diagnostics_emit_v1,
    native_host_diagnostics_metric_v1, native_host_event_drain_v1, native_host_event_emit_v1,
    native_host_register_component_v1, native_host_register_system_v2,
    native_host_spawn_command_v1, plugin_id_from_runtime_module_name,
    NativeHostApiV3RegistrationContext,
};

/// Host-owned authority inputs for a V4 native registration scope.
///
/// Component IDs are read from the runtime registry at each registration so components declared
/// earlier in the same entry callback become immediately usable. Resource IDs and capability
/// grants are host policy, never plugin-provided inputs.
#[derive(Clone, Debug, Default)]
pub struct NativeHostApiV4RegistrationPolicy {
    pub(super) granted_capabilities: Vec<String>,
    pub(super) known_resource_ids: Vec<String>,
}

impl NativeHostApiV4RegistrationPolicy {
    pub fn new<C, R, CS, RS>(granted_capabilities: C, known_resource_ids: R) -> Self
    where
        C: IntoIterator<Item = CS>,
        R: IntoIterator<Item = RS>,
        CS: Into<String>,
        RS: Into<String>,
    {
        let mut granted_capabilities = granted_capabilities
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        let mut known_resource_ids = known_resource_ids
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        granted_capabilities.sort();
        granted_capabilities.dedup();
        known_resource_ids.sort();
        known_resource_ids.dedup();
        Self {
            granted_capabilities,
            known_resource_ids,
        }
    }
}

pub struct NativeHostApiV4RegistrationScope<'registry> {
    handle: ZrRuntimePluginHandle,
    lifetime: Arc<NativeHostRegistrationScopeState>,
    _registry: PhantomData<&'registry mut RuntimeExtensionRegistry>,
}

impl<'registry> NativeHostApiV4RegistrationScope<'registry> {
    pub fn new(
        registry: &'registry mut RuntimeExtensionRegistry,
        module_name: impl Into<String>,
        policy: NativeHostApiV4RegistrationPolicy,
    ) -> Result<Self, String> {
        Self::new_result(registry, module_name, policy).map_err(|error| error.to_string())
    }

    fn new_result(
        registry: &'registry mut RuntimeExtensionRegistry,
        module_name: impl Into<String>,
        policy: NativeHostApiV4RegistrationPolicy,
    ) -> NativeHostApiAdapterResult<Self> {
        let module_name = module_name.into();
        let plugin_id = plugin_id_from_runtime_module_name(&module_name)
            .ok_or_else(|| NativeHostApiAdapterError::InvalidV4RuntimeModuleName {
                module_name: module_name.clone(),
            })?
            .to_string();
        let owner = registry
            .intern_plugin_module(module_name)
            .map_err(|source| NativeHostApiAdapterError::InvalidPluginModuleOwner { source })?;
        let lifetime = Arc::new(NativeHostRegistrationScopeState::default());
        let handle = ZrRuntimePluginHandle::new(insert_context(
            NativeHostApiV3Context::RegistrationV4(NativeHostApiV4RegistrationContext {
                registry: registry as *mut RuntimeExtensionRegistry as usize,
                owner,
                plugin_id,
                policy,
                lifetime: Arc::clone(&lifetime),
            }),
        ));
        Ok(Self {
            handle,
            lifetime,
            _registry: PhantomData,
        })
    }

    pub const fn handle(&self) -> ZrRuntimePluginHandle {
        self.handle
    }

    pub fn api(&self) -> ZrHostApiV4 {
        ZrHostApiV4 {
            abi_version: 4,
            size_bytes: std::mem::size_of::<ZrHostApiV4>(),
            ecs: ZrHostEcsApiV2 {
                register_system: Some(native_host_register_system_v2),
                register_component: Some(native_host_register_component_v1),
                spawn_command: Some(native_host_spawn_command_v1),
            },
            asset: ZrHostAssetApiV1 {
                request: Some(native_host_asset_request_v1),
            },
            event: ZrHostEventApiV1 {
                emit: Some(native_host_event_emit_v1),
                drain: Some(native_host_event_drain_v1),
            },
            bridge: ZrHostBridgeApiV1 {
                call: Some(native_host_bridge_call_v1),
            },
            diagnostics: ZrHostDiagnosticsApiV1 {
                emit: Some(native_host_diagnostics_emit_v1),
                metric: Some(native_host_diagnostics_metric_v1),
            },
        }
    }
}

impl Drop for NativeHostApiV4RegistrationScope<'_> {
    fn drop(&mut self) {
        self.lifetime.close_and_wait();
        remove_context(self.handle.raw());
    }
}

#[derive(Clone)]
pub(super) struct NativeHostApiV4RegistrationContext {
    pub(super) registry: usize,
    pub(super) owner: PluginModuleId,
    pub(super) plugin_id: String,
    pub(super) policy: NativeHostApiV4RegistrationPolicy,
    pub(super) lifetime: Arc<NativeHostRegistrationScopeState>,
}

impl NativeHostApiV4RegistrationContext {
    pub(super) fn v3_context(&self) -> NativeHostApiV3RegistrationContext {
        NativeHostApiV3RegistrationContext {
            registry: self.registry,
            owner: self.owner,
            lifetime: Arc::clone(&self.lifetime),
        }
    }
}

#[cfg(test)]
mod tests;

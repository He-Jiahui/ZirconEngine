use std::sync::Arc;

use crate::core::framework::bridge::PluginInterface;
use crate::plugin::bridge::{FrozenBridgeTable, InterfaceExport};
use crate::plugin::RuntimeExtensionRegistryError;

use super::{PluginModuleId, RuntimeExtensionRegistry};

impl RuntimeExtensionRegistry {
    pub fn export_interface<T>(
        &mut self,
        owner: PluginModuleId,
        implementation: Arc<T>,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        let interface_id = T::INTERFACE_ID.to_string();
        if self.plugin_interfaces.contains_key(&interface_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginInterface(
                interface_id,
            ));
        }

        self.plugin_interfaces
            .register(
                owner,
                interface_id.clone(),
                InterfaceExport::new(implementation),
            )
            .expect("plugin interface duplicate was prechecked");
        Ok(())
    }

    pub(in crate::plugin) fn register_interface_export(
        &mut self,
        owner: PluginModuleId,
        export: InterfaceExport,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let interface_id = export.interface_id().to_string();
        if self.plugin_interfaces.contains_key(&interface_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginInterface(
                interface_id,
            ));
        }

        self.plugin_interfaces
            .register(owner, interface_id, export)
            .expect("plugin interface duplicate was prechecked");
        Ok(())
    }

    pub fn frozen_bridge_table(&self) -> FrozenBridgeTable {
        FrozenBridgeTable::from_exports(self.plugin_interfaces.values().iter().map(|export| {
            let slot = self
                .plugin_interfaces
                .resolve(&export.interface_id)
                .expect("plugin interface export has slot");
            let owner = self
                .plugin_interfaces
                .owner_for_slot(slot)
                .expect("plugin interface export has owner");
            (owner, export.interface_id.clone(), export.clone())
        }))
    }
}

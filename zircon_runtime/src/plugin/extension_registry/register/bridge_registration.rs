use std::sync::Arc;

use crate::core::framework::bridge::PluginInterface;
use crate::plugin::bridge::{BridgeImport, FrozenBridgeTable, InterfaceExport, InterfaceImport};
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

        self.invalidate_bridge_table();
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

        self.invalidate_bridge_table();
        self.plugin_interfaces
            .register(owner, interface_id, export)
            .expect("plugin interface duplicate was prechecked");
        Ok(())
    }

    /// Declares an owner-scoped dependency on a plugin interface. The returned
    /// handle is bound only after the catalog has merged and finalized all
    /// plugin registrations, so consumers cannot accidentally capture a
    /// per-plugin staging table.
    pub fn import_interface<T>(
        &mut self,
        owner: PluginModuleId,
    ) -> Result<BridgeImport<T>, RuntimeExtensionRegistryError>
    where
        T: PluginInterface + ?Sized,
    {
        let (imported, registration) = BridgeImport::<T>::new();
        self.register_interface_import(owner, registration)?;
        Ok(imported)
    }

    pub(in crate::plugin) fn register_interface_import(
        &mut self,
        owner: PluginModuleId,
        import: InterfaceImport,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let module_name = self.plugin_module_name(owner).ok_or_else(|| {
            RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                "unknown plugin module owner {}",
                owner.raw()
            ))
        })?;
        let key = format!("{module_name}=>{}", import.interface_id());
        if self.plugin_interface_imports.contains_key(&key) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginInterfaceImport(key));
        }

        self.plugin_interface_imports
            .register(owner, key, import.clone())
            .expect("plugin interface import duplicate was prechecked");
        if let Some(table) = self.bridge_table.as_ref() {
            import.bind(table);
        }
        Ok(())
    }

    pub(in crate::plugin) fn plugin_interface_imports(
        &self,
    ) -> impl Iterator<Item = (PluginModuleId, &InterfaceImport)> {
        self.plugin_interface_imports
            .iter()
            .map(|(owner, _, import)| (owner, import))
    }

    pub fn frozen_bridge_table(&self) -> FrozenBridgeTable {
        if let Some(table) = self.bridge_table.as_ref() {
            return table.clone();
        }
        self.build_bridge_table()
    }

    pub(crate) fn finalize_bridge_imports(&mut self) {
        if self.bridge_table.is_some() {
            return;
        }
        let table = self.build_bridge_table();
        for import in self.plugin_interface_imports.values() {
            import.bind(&table);
        }
        self.bridge_table = Some(table);
    }

    pub(crate) fn invalidate_bridge_table(&mut self) {
        self.bridge_table = None;
    }

    pub(crate) fn unbind_interface_imports_owned_by(&self, owner: PluginModuleId) {
        for slot in self.plugin_interface_imports.entries_owned_by(owner) {
            if let Some(import) = self.plugin_interface_imports.get(slot) {
                import.unbind();
            }
        }
    }

    fn build_bridge_table(&self) -> FrozenBridgeTable {
        FrozenBridgeTable::from_exports(
            self.plugin_interfaces
                .iter()
                .map(|(owner, interface_id, export)| (owner, interface_id.clone(), export.clone())),
        )
    }
}

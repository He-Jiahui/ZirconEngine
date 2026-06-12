use std::collections::HashMap;

use crate::plugin::RuntimeExtensionRegistryError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PluginModuleId(u32);

impl PluginModuleId {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::plugin::extension_registry) struct PluginModuleInterner {
    names: Vec<String>,
    ids_by_name: HashMap<String, PluginModuleId>,
}

impl PluginModuleInterner {
    pub(in crate::plugin::extension_registry) fn intern(
        &mut self,
        name: impl Into<String>,
    ) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
        let name = name.into();
        validate_plugin_module_name(&name)?;
        if let Some(id) = self.ids_by_name.get(&name).copied() {
            return Ok(id);
        }

        let id = PluginModuleId::from_raw(self.names.len() as u32);
        self.names.push(name.clone());
        self.ids_by_name.insert(name, id);
        Ok(id)
    }

    pub(in crate::plugin::extension_registry) fn name(&self, id: PluginModuleId) -> Option<&str> {
        self.names.get(id.index()).map(String::as_str)
    }
}

fn validate_plugin_module_name(name: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if name.trim().is_empty() || name.trim() != name || !name.contains('.') {
        return Err(RuntimeExtensionRegistryError::InvalidPluginModule(
            name.to_string(),
        ));
    }
    Ok(())
}

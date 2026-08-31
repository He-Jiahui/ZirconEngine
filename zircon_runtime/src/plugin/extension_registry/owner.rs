use std::collections::HashMap;
use std::sync::Arc;

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
    names: Vec<Arc<str>>,
    ids_by_name: HashMap<Arc<str>, PluginModuleId>,
}

impl PluginModuleInterner {
    pub(in crate::plugin::extension_registry) fn intern(
        &mut self,
        name: impl Into<String>,
    ) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
        let name = name.into();
        validate_plugin_module_name(&name)?;
        if let Some(id) = self.ids_by_name.get(name.as_str()).copied() {
            return Ok(id);
        }

        let id = PluginModuleId::from_raw(self.names.len() as u32);
        let name: Arc<str> = name.into();
        self.names.push(Arc::clone(&name));
        self.ids_by_name.insert(name, id);
        Ok(id)
    }

    pub(in crate::plugin::extension_registry) fn name(&self, id: PluginModuleId) -> Option<&str> {
        self.names.get(id.index()).map(AsRef::as_ref)
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::PluginModuleInterner;

    #[test]
    fn interner_indexes_and_clones_share_name_storage() {
        let mut interner = PluginModuleInterner::default();
        let id = interner
            .intern("weather.runtime")
            .expect("valid module name should intern");
        let (indexed_name, indexed_id) = interner
            .ids_by_name
            .get_key_value("weather.runtime")
            .expect("interned module should be indexed");

        assert_eq!(*indexed_id, id);
        assert!(Arc::ptr_eq(&interner.names[id.index()], indexed_name));

        let cloned = interner.clone();
        let (cloned_indexed_name, _) = cloned
            .ids_by_name
            .get_key_value("weather.runtime")
            .expect("cloned interner should preserve the index");
        assert!(Arc::ptr_eq(
            &interner.names[id.index()],
            &cloned.names[id.index()],
        ));
        assert!(Arc::ptr_eq(&cloned.names[id.index()], cloned_indexed_name,));
        assert_eq!(cloned.name(id), Some("weather.runtime"));
    }
}

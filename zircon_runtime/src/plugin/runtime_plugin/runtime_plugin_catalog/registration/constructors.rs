use crate::plugin::{RuntimePlugin, RuntimePluginDescriptor};

use super::super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn from_plugins<'a>(plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>) -> Self {
        let mut catalog = Self::default();
        for plugin in plugins {
            catalog.register(plugin);
        }
        catalog
    }

    pub fn from_descriptors(
        descriptors: impl IntoIterator<Item = RuntimePluginDescriptor>,
    ) -> Self {
        let mut catalog = Self::default();
        for descriptor in descriptors {
            catalog.register(&descriptor);
        }
        catalog
    }

    pub fn builtin() -> Self {
        Self::from_descriptors(RuntimePluginDescriptor::builtin_catalog())
    }
}

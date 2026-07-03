use crate::plugin::{RuntimePlugin, RuntimePluginDescriptor};

use super::super::RuntimePluginCatalog;
use super::order::{order_runtime_plugin_descriptors, order_runtime_plugins};

impl RuntimePluginCatalog {
    pub fn from_plugins<'a>(plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>) -> Self {
        let mut catalog = Self::default();
        let (plugins, order_diagnostics) =
            order_runtime_plugins(plugins.into_iter().collect::<Vec<_>>());
        for plugin in plugins {
            catalog.register(plugin);
        }
        catalog.diagnostics.extend(order_diagnostics);
        catalog
    }

    pub fn from_descriptors(
        descriptors: impl IntoIterator<Item = RuntimePluginDescriptor>,
    ) -> Self {
        let mut catalog = Self::default();
        let (descriptors, order_diagnostics) =
            order_runtime_plugin_descriptors(descriptors.into_iter().collect::<Vec<_>>());
        for descriptor in descriptors {
            catalog.register(&descriptor);
        }
        catalog.diagnostics.extend(order_diagnostics);
        catalog
    }

    pub fn builtin() -> Self {
        Self::from_descriptors(RuntimePluginDescriptor::builtin_catalog())
    }
}

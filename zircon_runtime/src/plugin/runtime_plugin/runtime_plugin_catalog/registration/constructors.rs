use crate::plugin::{RuntimePlugin, RuntimePluginDescriptor};

use super::super::RuntimePluginCatalog;
use super::order::{order_runtime_plugin_descriptors, order_runtime_plugins};

impl RuntimePluginCatalog {
    pub fn from_plugins<'a>(plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>) -> Self {
        let mut catalog = Self::default();
        let plugins = match order_runtime_plugins(plugins.into_iter().collect::<Vec<_>>()) {
            Ok(plugins) => plugins,
            Err(error) => {
                catalog.reject_module_order(error);
                return catalog;
            }
        };
        for plugin in plugins {
            catalog.register(plugin);
        }
        catalog
    }

    pub fn from_descriptors(
        descriptors: impl IntoIterator<Item = RuntimePluginDescriptor>,
    ) -> Self {
        let mut catalog = Self::default();
        let descriptors =
            match order_runtime_plugin_descriptors(descriptors.into_iter().collect::<Vec<_>>()) {
                Ok(descriptors) => descriptors,
                Err(error) => {
                    catalog.reject_module_order(error);
                    return catalog;
                }
            };
        for descriptor in descriptors {
            catalog.register(&descriptor);
        }
        catalog
    }

    pub fn builtin() -> Self {
        Self::from_descriptors(RuntimePluginDescriptor::builtin_catalog())
    }
}

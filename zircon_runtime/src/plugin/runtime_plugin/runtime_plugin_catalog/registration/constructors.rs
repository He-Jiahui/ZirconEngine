use std::sync::OnceLock;

use crate::plugin::{
    RuntimePlugin, RuntimePluginDescriptor, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

use super::super::RuntimePluginCatalog;
use super::order::{order_runtime_plugin_descriptors, order_runtime_plugins};

impl RuntimePluginCatalog {
    /// 先按模块依赖排序插件，再一次性构造注册报告与目录投影。
    pub fn from_plugins<'a>(plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>) -> Self {
        let mut catalog = Self::default();
        let plugins = match order_runtime_plugins(plugins.into_iter().collect::<Vec<_>>()) {
            Ok(plugins) => plugins,
            Err(error) => {
                catalog.reject_module_order(error);
                return catalog;
            }
        };
        let registrations = plugins
            .into_iter()
            .map(RuntimePluginRegistrationReport::from_plugin);
        Self::from_registration_reports(
            registrations,
            std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
        )
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
        let registrations = descriptors
            .iter()
            .map(|descriptor| RuntimePluginRegistrationReport::from_plugin(descriptor));
        Self::from_registration_reports(
            registrations,
            std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
        )
    }

    /// Returns the process-wide immutable builtin generation without cloning its rows.
    pub fn builtin() -> &'static Self {
        static BUILTIN_CATALOG: OnceLock<RuntimePluginCatalog> = OnceLock::new();
        BUILTIN_CATALOG
            .get_or_init(|| Self::from_descriptors(RuntimePluginDescriptor::builtin_catalog()))
    }

    pub(crate) fn builtin_shared() -> &'static Self {
        Self::builtin()
    }
}

#[cfg(test)]
#[path = "tests/constructors.rs"]
mod tests;

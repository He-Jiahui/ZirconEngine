use crate::plugin::{
    RuntimePlugin, RuntimePluginDescriptor, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

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

    pub fn builtin() -> Self {
        Self::from_descriptors(RuntimePluginDescriptor::builtin_catalog())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn catalog_constructors_do_not_rebuild_after_each_registration() {
        let source = include_str!("constructors.rs");
        let incremental_register = ["catalog", ".register("].concat();
        assert!(!source.contains(&incremental_register));
    }
}

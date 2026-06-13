use crate::plugin::{
    CapabilityView, PluginFinishContext, RuntimePlugin, RuntimePluginFeature,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn from_lifecycle_plugins<'a>(
        plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>,
        features: impl IntoIterator<Item = &'a dyn RuntimePluginFeature>,
    ) -> Self {
        let plugins = plugins.into_iter().collect::<Vec<_>>();
        let features = features.into_iter().collect::<Vec<_>>();
        let mut catalog = Self::default();

        for plugin in &plugins {
            catalog
                .registrations
                .push(RuntimePluginRegistrationReport::from_plugin(*plugin));
        }
        for feature in &features {
            catalog.feature_registrations.push(
                RuntimePluginFeatureRegistrationReport::from_feature(*feature),
            );
        }

        catalog.finish_registered_plugins(&plugins, &features);
        catalog.rebuild_diagnostics();
        catalog
    }

    fn finish_registered_plugins(
        &mut self,
        plugins: &[&dyn RuntimePlugin],
        features: &[&dyn RuntimePluginFeature],
    ) {
        let capabilities = CapabilityView::from_registration_reports(
            self.registrations.iter(),
            self.feature_registrations.iter(),
        );

        for (plugin, registration) in plugins.iter().zip(self.registrations.iter_mut()) {
            let mut context = PluginFinishContext::new(&mut registration.extensions, &capabilities);
            if let Err(error) = plugin.finish(&mut context) {
                registration.diagnostics.push(error.to_string());
            }
        }
        for (feature, registration) in features.iter().zip(self.feature_registrations.iter_mut()) {
            let mut context = PluginFinishContext::new(&mut registration.extensions, &capabilities);
            if let Err(error) = feature.finish(&mut context) {
                registration.diagnostics.push(error.to_string());
            }
        }
    }
}

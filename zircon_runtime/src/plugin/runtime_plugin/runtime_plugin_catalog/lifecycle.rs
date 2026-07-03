use crate::plugin::{
    CapabilityView, PluginFinishContext, PluginReadyContext, PluginRuntimeContext, RuntimePlugin,
    RuntimePluginFeature, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

use super::registration::order::order_runtime_plugins;
use super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn from_lifecycle_plugins<'a>(
        plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>,
        features: impl IntoIterator<Item = &'a dyn RuntimePluginFeature>,
    ) -> Self {
        let (plugins, order_diagnostics) =
            order_runtime_plugins(plugins.into_iter().collect::<Vec<_>>());
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

        catalog.ready_and_finish_registered_plugins(&plugins, &features);
        catalog.rebuild_diagnostics();
        catalog.diagnostics.extend(order_diagnostics);
        catalog
    }

    fn ready_and_finish_registered_plugins(
        &mut self,
        plugins: &[&dyn RuntimePlugin],
        features: &[&dyn RuntimePluginFeature],
    ) {
        let capabilities = CapabilityView::from_registration_reports(
            self.registrations.iter(),
            self.feature_registrations.iter(),
        );
        let plugin_ready = self.ready_registered_plugins(plugins, &capabilities);
        let feature_ready = self.ready_registered_features(features, &capabilities);
        let all_ready = plugin_ready
            .iter()
            .chain(feature_ready.iter())
            .all(|ready| *ready);
        if !all_ready {
            return;
        }

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

    fn ready_registered_plugins(
        &mut self,
        plugins: &[&dyn RuntimePlugin],
        capabilities: &CapabilityView,
    ) -> Vec<bool> {
        plugins
            .iter()
            .zip(self.registrations.iter_mut())
            .map(|(plugin, registration)| {
                let context = PluginReadyContext::new(&registration.extensions, capabilities);
                match plugin.ready(&context) {
                    Ok(true) => true,
                    Ok(false) => {
                        registration.diagnostics.push(format!(
                            "runtime plugin `{}` is not ready",
                            plugin.descriptor().package_id()
                        ));
                        false
                    }
                    Err(error) => {
                        registration.diagnostics.push(error.to_string());
                        false
                    }
                }
            })
            .collect()
    }

    fn ready_registered_features(
        &mut self,
        features: &[&dyn RuntimePluginFeature],
        capabilities: &CapabilityView,
    ) -> Vec<bool> {
        features
            .iter()
            .zip(self.feature_registrations.iter_mut())
            .map(|(feature, registration)| {
                let context = PluginReadyContext::new(&registration.extensions, capabilities);
                match feature.ready(&context) {
                    Ok(true) => true,
                    Ok(false) => {
                        registration.diagnostics.push(format!(
                            "runtime plugin feature `{}` is not ready",
                            registration.manifest.id
                        ));
                        false
                    }
                    Err(error) => {
                        registration.diagnostics.push(error.to_string());
                        false
                    }
                }
            })
            .collect()
    }

    pub fn activate_lifecycle_plugins<'a>(
        &mut self,
        plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>,
        features: impl IntoIterator<Item = &'a dyn RuntimePluginFeature>,
        context: &mut PluginRuntimeContext<'_>,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        let (plugins, order_diagnostics) =
            order_runtime_plugins(plugins.into_iter().collect::<Vec<_>>());
        self.diagnostics.extend(order_diagnostics);
        let features = features.into_iter().collect::<Vec<_>>();

        for plugin in plugins {
            if let Err(error) = plugin.activate(context) {
                self.diagnostics.push(error.to_string());
                return Err(error);
            }
        }
        for feature in features {
            if let Err(error) = feature.activate(context) {
                self.diagnostics.push(error.to_string());
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn deactivate_lifecycle_plugins<'a>(
        &mut self,
        plugins: impl IntoIterator<Item = &'a dyn RuntimePlugin>,
        features: impl IntoIterator<Item = &'a dyn RuntimePluginFeature>,
        context: &mut PluginRuntimeContext<'_>,
    ) {
        let (plugins, order_diagnostics) =
            order_runtime_plugins(plugins.into_iter().collect::<Vec<_>>());
        self.diagnostics.extend(order_diagnostics);
        let features = features.into_iter().collect::<Vec<_>>();

        for feature in features.iter().rev() {
            feature.deactivate(context);
        }
        for plugin in plugins.iter().rev() {
            plugin.deactivate(context);
        }
    }
}

use crate::plugin::{
    RuntimePlugin, RuntimePluginFeature, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

use super::super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn register(&mut self, plugin: &dyn RuntimePlugin) {
        let report = RuntimePluginRegistrationReport::from_plugin(plugin);
        self.diagnostics.extend(report.diagnostics.iter().cloned());
        self.registrations.push(report);
    }

    pub fn register_feature(&mut self, feature: &dyn RuntimePluginFeature) {
        let report = RuntimePluginFeatureRegistrationReport::from_feature(feature);
        self.diagnostics.extend(report.diagnostics.iter().cloned());
        self.feature_registrations.push(report);
    }
}

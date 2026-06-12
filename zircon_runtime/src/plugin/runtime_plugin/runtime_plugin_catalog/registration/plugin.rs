use crate::plugin::{
    RuntimePlugin, RuntimePluginFeature, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

use super::super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn register(&mut self, plugin: &dyn RuntimePlugin) {
        let report = RuntimePluginRegistrationReport::from_plugin(plugin);
        self.registrations.push(report);
        self.rebuild_diagnostics();
    }

    pub fn register_feature(&mut self, feature: &dyn RuntimePluginFeature) {
        let report = RuntimePluginFeatureRegistrationReport::from_feature(feature);
        self.feature_registrations.push(report);
        self.rebuild_diagnostics();
    }
}

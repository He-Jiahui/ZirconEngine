use crate::plugin::{
    RuntimePlugin, RuntimePluginFeature, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

use super::super::{RuntimePluginCatalog, RuntimePluginCatalogUpdateOutcome};

impl RuntimePluginCatalog {
    pub fn register(&mut self, plugin: &dyn RuntimePlugin) -> RuntimePluginCatalogUpdateOutcome {
        self.register_reports_batch(
            [RuntimePluginRegistrationReport::from_plugin(plugin)],
            std::iter::empty(),
        )
    }

    pub fn register_feature(
        &mut self,
        feature: &dyn RuntimePluginFeature,
    ) -> RuntimePluginCatalogUpdateOutcome {
        self.register_reports_batch(
            std::iter::empty(),
            [RuntimePluginFeatureRegistrationReport::from_feature(
                feature,
            )],
        )
    }

    /// Applies a batch through one candidate build and one atomic generation decision.
    pub fn register_reports_batch(
        &mut self,
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> RuntimePluginCatalogUpdateOutcome {
        let mut update = self.update();
        for registration in registrations {
            update.append_registration(registration);
        }
        for registration in feature_registrations {
            update.append_feature_registration(registration);
        }
        update.publish()
    }
}

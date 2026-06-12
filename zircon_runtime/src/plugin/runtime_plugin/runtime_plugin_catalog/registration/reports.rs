use crate::plugin::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

use super::super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn from_registration_reports(
        registrations: impl IntoIterator<Item = RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        let mut catalog = Self::default();
        for registration in registrations {
            catalog.registrations.push(registration);
        }
        for registration in feature_registrations {
            catalog.feature_registrations.push(registration);
        }
        catalog.rebuild_diagnostics();
        catalog
    }
}

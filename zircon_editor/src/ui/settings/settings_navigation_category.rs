use std::sync::Arc;

use super::SettingsLocalizationDomain;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsNavigationCategory {
    pub(super) localization_domain: SettingsLocalizationDomain,
    pub(super) keys: Arc<[Arc<str>]>,
    pub(super) labels: Arc<[Arc<str>]>,
}

impl SettingsNavigationCategory {
    pub fn localization_domain(&self) -> &SettingsLocalizationDomain {
        &self.localization_domain
    }

    pub fn keys(&self) -> &[Arc<str>] {
        &self.keys
    }

    pub fn labels(&self) -> &[Arc<str>] {
        &self.labels
    }
}

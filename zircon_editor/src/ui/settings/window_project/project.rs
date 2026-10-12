use std::sync::Arc;

use crate::core::extension::{CapabilitySet, LocalizedSettingsPage, SettingsPageProjection};
use crate::core::i18n::EditorLocale;
use crate::core::settings::{SettingsCatalog, SettingsSnapshot};

use super::super::{LocalizedSetting, SettingsNavigationCategory};

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsWindowProjection {
    pub(super) settings_generation: u64,
    pub(super) settings_catalog: Arc<SettingsCatalog>,
    pub(super) locale: EditorLocale,
    pub(super) enabled_capabilities: CapabilitySet,
    pub(super) title: Arc<str>,
    pub(super) categories: Arc<[SettingsNavigationCategory]>,
    pub(super) settings: Arc<[LocalizedSetting]>,
    pub(super) plugin_pages: SettingsPageProjection,
}

impl SettingsWindowProjection {
    pub const fn settings_generation(&self) -> u64 {
        self.settings_generation
    }

    pub fn locale(&self) -> &EditorLocale {
        &self.locale
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn contribution_generation(&self) -> u64 {
        self.plugin_pages.contribution_generation()
    }

    pub fn enabled_capabilities(&self) -> impl Iterator<Item = &str> {
        self.enabled_capabilities.iter()
    }

    pub fn categories(&self) -> &[SettingsNavigationCategory] {
        &self.categories
    }

    pub fn settings(&self) -> &[LocalizedSetting] {
        &self.settings
    }

    pub fn plugin_pages(&self) -> &[LocalizedSettingsPage] {
        self.plugin_pages.pages()
    }

    pub(super) fn settings_snapshot_is_current(&self, snapshot: &SettingsSnapshot) -> bool {
        snapshot.shares_catalog_handle_with(&self.settings_catalog)
    }
}

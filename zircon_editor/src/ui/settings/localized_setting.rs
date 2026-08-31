use std::sync::Arc;

use crate::core::settings::{SettingSchema, SettingsScope};

#[derive(Clone, Debug, PartialEq)]
pub struct LocalizedSetting {
    pub(super) key: Arc<str>,
    pub(super) label: Arc<str>,
    pub(super) description: Arc<str>,
    pub(super) category_keys: Arc<[Arc<str>]>,
    pub(super) category_labels: Arc<[Arc<str>]>,
    pub(super) scope: SettingsScope,
    pub(super) schema: SettingSchema,
    pub(super) requires_restart: bool,
}

impl LocalizedSetting {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn category_keys(&self) -> &[Arc<str>] {
        &self.category_keys
    }

    pub fn category_labels(&self) -> &[Arc<str>] {
        &self.category_labels
    }

    pub const fn scope(&self) -> SettingsScope {
        self.scope
    }

    pub fn schema(&self) -> &SettingSchema {
        &self.schema
    }

    pub const fn requires_restart(&self) -> bool {
        self.requires_restart
    }
}

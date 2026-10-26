use std::sync::Arc;

use super::super::{SettingValue, SettingsError, SettingsKey, SettingsRegistry, SettingsScope};
use super::SettingValueSource;

/// One effective setting value captured as part of a generation-consistent batch.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSettingValue {
    key: SettingsKey,
    value: SettingValue,
    source: SettingValueSource,
    user_override: Option<SettingValue>,
    project_override: Option<SettingValue>,
    session_override: Option<SettingValue>,
}

impl ResolvedSettingValue {
    pub fn key(&self) -> &SettingsKey {
        &self.key
    }

    pub fn value(&self) -> &SettingValue {
        &self.value
    }

    pub const fn source(&self) -> SettingValueSource {
        self.source
    }

    /// The raw override in one layer at this batch's exact generation.
    /// Absence means inheritance, even when another layer supplies the effective value.
    pub fn override_in_scope(&self, scope: SettingsScope) -> Option<&SettingValue> {
        match scope {
            SettingsScope::User => self.user_override.as_ref(),
            SettingsScope::Project => self.project_override.as_ref(),
            SettingsScope::Session => self.session_override.as_ref(),
        }
    }
}

/// Effective values cloned under one authority lock at one exact settings generation.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSettingsBatch {
    generation: u64,
    values: Arc<[ResolvedSettingValue]>,
}

impl ResolvedSettingsBatch {
    pub(in crate::core::settings) fn from_registry<'a>(
        registry: &SettingsRegistry,
        keys: impl IntoIterator<Item = &'a SettingsKey>,
    ) -> Result<Self, SettingsError> {
        let values = keys
            .into_iter()
            .map(|key| {
                let (value, source) = registry.resolve_with_source(key)?;
                Ok(ResolvedSettingValue {
                    key: key.clone(),
                    value: value.clone(),
                    source,
                    user_override: registry
                        .override_in_scope(SettingsScope::User, key)
                        .cloned(),
                    project_override: registry
                        .override_in_scope(SettingsScope::Project, key)
                        .cloned(),
                    session_override: registry
                        .override_in_scope(SettingsScope::Session, key)
                        .cloned(),
                })
            })
            .collect::<Result<Vec<_>, SettingsError>>()?;
        Ok(Self {
            generation: registry.revision,
            values: values.into(),
        })
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn values(&self) -> &[ResolvedSettingValue] {
        &self.values
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
#[path = "tests/resolved_settings_batch.rs"]
mod tests;

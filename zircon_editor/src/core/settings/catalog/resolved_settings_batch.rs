use std::sync::Arc;

use super::super::{SettingValue, SettingsError, SettingsKey, SettingsRegistry};
use super::SettingValueSource;

/// One effective setting value captured as part of a generation-consistent batch.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSettingValue {
    key: SettingsKey,
    value: SettingValue,
    source: SettingValueSource,
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
}

/// Effective values cloned under one authority lock at one exact settings generation.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSettingsBatch {
    generation: u64,
    values: Arc<[ResolvedSettingValue]>,
}

impl ResolvedSettingsBatch {
    pub(in crate::core::settings) fn from_registry(
        registry: &SettingsRegistry,
        keys: &[SettingsKey],
    ) -> Result<Self, SettingsError> {
        let values = keys
            .iter()
            .map(|key| {
                let (value, source) = registry.resolve_with_source(key)?;
                Ok(ResolvedSettingValue {
                    key: key.clone(),
                    value: value.clone(),
                    source,
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

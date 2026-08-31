use super::super::SettingValue;
use super::SettingValueSource;

/// One cloned value read under the settings authority lock at an exact generation.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSettingSnapshot {
    generation: u64,
    value: SettingValue,
    source: SettingValueSource,
}

impl ResolvedSettingSnapshot {
    pub(in crate::core::settings) fn new(
        generation: u64,
        value: SettingValue,
        source: SettingValueSource,
    ) -> Self {
        Self {
            generation,
            value,
            source,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn value(&self) -> &SettingValue {
        &self.value
    }

    pub const fn source(&self) -> SettingValueSource {
        self.source
    }
}

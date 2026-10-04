use std::sync::OnceLock;

use crate::core::settings::{
    SettingDefinition, SettingSchema, SettingValue, SettingsAuthority, SettingsError, SettingsKey,
    SettingsPresentation, SettingsRegistry, SettingsScope,
};

pub const SCRIPT_BUILD_BATCH_WINDOW_MS_KEY: &str = "editor.script_build.batch_window_ms";
pub const DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS: u64 = 300;
pub const DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS: u64 = 1_000;
pub const MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS: i64 = 50;
pub const MAXIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS: i64 = 1_000;
pub const SCRIPT_BUILD_BATCH_WINDOW_STEP_MS: i64 = 50;

/// Resolved watch batching policy consumed by the script-build state machine.
///
/// The debounce value comes only from the registered User setting in production. The hard
/// first-event limit remains owned by Editor13 and cannot be weakened by that setting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScriptBuildBatchPolicy {
    debounce_ms: u64,
    max_latency_ms: u64,
}

impl ScriptBuildBatchPolicy {
    pub const fn debounce_ms(self) -> u64 {
        self.debounce_ms
    }

    pub const fn max_latency_ms(self) -> u64 {
        self.max_latency_ms
    }

    pub(super) fn from_settings(settings: &SettingsAuthority) -> Result<Self, SettingsError> {
        let resolved = settings.resolved_setting(batch_window_key())?;
        let SettingValue::Int(debounce_ms) = resolved.value() else {
            unreachable!("the script-build batch-window key has an integer schema")
        };
        Ok(Self {
            debounce_ms: u64::try_from(*debounce_ms)
                .expect("the script-build batch-window schema permits only positive values"),
            max_latency_ms: DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS,
        })
    }

    #[cfg(test)]
    pub(super) const fn from_test_limits(debounce_ms: u64, max_latency_ms: u64) -> Self {
        Self {
            debounce_ms,
            max_latency_ms: if max_latency_ms < debounce_ms {
                debounce_ms
            } else {
                max_latency_ms
            },
        }
    }
}

pub(crate) fn register_script_build_settings(
    registry: &mut SettingsRegistry,
) -> Result<(), SettingsError> {
    registry.register(
        SettingDefinition::new(
            batch_window_key().clone(),
            SettingsScope::User,
            SettingSchema::Int {
                minimum: MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS,
                maximum: MAXIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS,
                step: SCRIPT_BUILD_BATCH_WINDOW_STEP_MS,
            },
            SettingValue::Int(
                i64::try_from(DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS)
                    .expect("the default script-build batch window fits the integer schema"),
            ),
            false,
            SettingsPresentation::new(
                "settings.editor.script_build.batch_window_ms.label",
                "settings.editor.script_build.batch_window_ms.description",
                ["settings.category.editor", "settings.category.script_build"],
            )
            .expect("the built-in script-build presentation keys are valid"),
        )
        .expect("the built-in script-build batch-window definition is valid"),
    )
}

fn batch_window_key() -> &'static SettingsKey {
    static KEY: OnceLock<SettingsKey> = OnceLock::new();
    KEY.get_or_init(|| {
        SettingsKey::parse(SCRIPT_BUILD_BATCH_WINDOW_MS_KEY)
            .expect("the built-in script-build batch-window key is valid")
    })
}

#[cfg(test)]
#[path = "settings/tests/cases.rs"]
mod tests;

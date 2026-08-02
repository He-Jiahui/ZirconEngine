use crate::core::settings::{
    SettingDefinition, SettingSchema, SettingValue, SettingsError, SettingsKey, SettingsRegistry,
    SettingsScope,
};
use thiserror::Error;

use super::{EditorJobLimits, JobCategory};

pub const EDITOR_JOB_THUMBNAIL_QUOTA_KEY: &str = "editor.jobs.thumbnail_quota";
pub const EDITOR_JOB_EXPORT_QUOTA_KEY: &str = "editor.jobs.export_quota";
pub const EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY: &str = "editor.jobs.interactive_save_quota";
pub const EDITOR_JOB_PLAY_QUOTA_KEY: &str = "editor.jobs.play_quota";

const MAXIMUM_USER_JOB_CATEGORY_QUOTA: i64 = 64;
const QUOTA_CATEGORY_PATH: &str = "Editor/Job Scheduling";

const USER_CONFIGURABLE_QUOTAS: [(JobCategory, &str); 4] = [
    (JobCategory::Thumbnail, EDITOR_JOB_THUMBNAIL_QUOTA_KEY),
    (JobCategory::Export, EDITOR_JOB_EXPORT_QUOTA_KEY),
    (
        JobCategory::InteractiveSave,
        EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY,
    ),
    (JobCategory::Play, EDITOR_JOB_PLAY_QUOTA_KEY),
];

#[derive(Debug, Error)]
pub enum EditorJobQuotaSettingsError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error("job quota setting `{key}` must resolve to a positive integer, received {value:?}")]
    InvalidQuota { key: String, value: SettingValue },
}

/// Registers the User-scoped, restart-only quotas owned by the job admission
/// authority. Runtime-width categories remain derived from the JobScheduler.
pub fn register_editor_job_quota_settings(
    registry: &mut SettingsRegistry,
) -> Result<(), SettingsError> {
    for (category, key) in USER_CONFIGURABLE_QUOTAS {
        let default = i64::try_from(EditorJobLimits::default().limit(category))
            .expect("editor job defaults fit the settings integer range");
        registry.register(
            SettingDefinition::new(
                SettingsKey::parse(key).expect("built-in job quota key is valid"),
                SettingsScope::User,
                SettingSchema::Int {
                    minimum: 1,
                    maximum: MAXIMUM_USER_JOB_CATEGORY_QUOTA,
                },
                SettingValue::Int(default),
                true,
                QUOTA_CATEGORY_PATH,
            )
            .expect("built-in job quota definition is valid"),
        )?;
    }
    Ok(())
}

/// Resolves the limits consumed by EditorJobSystem construction.
///
/// Definitions must be registered before User settings are loaded. This makes
/// a missing startup registration explicit instead of silently reviving a
/// hard-coded admission path. Validation makes a zero or negative quota
/// unrepresentable, and a setting change advertises that restart is required.
pub fn resolve_editor_job_limits(
    registry: &SettingsRegistry,
    worker_parallelism: usize,
) -> Result<EditorJobLimits, EditorJobQuotaSettingsError> {
    let mut limits = EditorJobLimits::default().with_runtime_defaults(worker_parallelism);
    for (category, key) in USER_CONFIGURABLE_QUOTAS {
        let key = SettingsKey::parse(key).expect("built-in job quota key is valid");
        let value = registry.resolve(&key)?.clone();
        let SettingValue::Int(value) = value else {
            return Err(EditorJobQuotaSettingsError::InvalidQuota {
                key: key.as_str().to_string(),
                value,
            });
        };
        let Ok(limit) = usize::try_from(value) else {
            return Err(EditorJobQuotaSettingsError::InvalidQuota {
                key: key.as_str().to_string(),
                value: SettingValue::Int(value),
            });
        };
        if limit == 0 {
            return Err(EditorJobQuotaSettingsError::InvalidQuota {
                key: key.as_str().to_string(),
                value: SettingValue::Int(value),
            });
        }
        limits = limits.with_limit(category, limit);
    }
    Ok(limits)
}

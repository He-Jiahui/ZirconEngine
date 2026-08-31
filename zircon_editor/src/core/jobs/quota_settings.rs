use crate::core::settings::{
    SettingDefinition, SettingSchema, SettingValue, SettingsError, SettingsKey,
    SettingsPresentation, SettingsRegistry, SettingsScope,
};
use thiserror::Error;

use super::limits::user_configurable_default_limit;
use super::{EditorJobLimits, JobCategory};

pub const EDITOR_JOB_THUMBNAIL_QUOTA_KEY: &str = "editor.jobs.thumbnail_quota";
pub const EDITOR_JOB_EXPORT_QUOTA_KEY: &str = "editor.jobs.export_quota";
pub const EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY: &str = "editor.jobs.interactive_save_quota";
pub const EDITOR_JOB_PLAY_QUOTA_KEY: &str = "editor.jobs.play_quota";

const MAXIMUM_USER_JOB_CATEGORY_QUOTA: i64 = 64;
const JOB_QUOTA_CATEGORY_PATH: [&str; 2] = [
    "settings.category.editor",
    "settings.category.job_scheduling",
];

#[derive(Clone, Copy)]
struct UserConfigurableQuota {
    category: JobCategory,
    key: &'static str,
    label_key: &'static str,
    description_key: &'static str,
}

const USER_CONFIGURABLE_QUOTAS: [UserConfigurableQuota; 4] = [
    UserConfigurableQuota {
        category: JobCategory::Thumbnail,
        key: EDITOR_JOB_THUMBNAIL_QUOTA_KEY,
        label_key: "settings.editor.jobs.thumbnail_quota.label",
        description_key: "settings.editor.jobs.thumbnail_quota.description",
    },
    UserConfigurableQuota {
        category: JobCategory::Export,
        key: EDITOR_JOB_EXPORT_QUOTA_KEY,
        label_key: "settings.editor.jobs.export_quota.label",
        description_key: "settings.editor.jobs.export_quota.description",
    },
    UserConfigurableQuota {
        category: JobCategory::InteractiveSave,
        key: EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY,
        label_key: "settings.editor.jobs.interactive_save_quota.label",
        description_key: "settings.editor.jobs.interactive_save_quota.description",
    },
    UserConfigurableQuota {
        category: JobCategory::Play,
        key: EDITOR_JOB_PLAY_QUOTA_KEY,
        label_key: "settings.editor.jobs.play_quota.label",
        description_key: "settings.editor.jobs.play_quota.description",
    },
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
    for quota in USER_CONFIGURABLE_QUOTAS {
        let default = i64::try_from(
            user_configurable_default_limit(quota.category)
                .expect("every registered User quota has one canonical default"),
        )
        .expect("editor job defaults fit the settings integer range");
        registry.register(
            SettingDefinition::new(
                SettingsKey::parse(quota.key).expect("built-in job quota key is valid"),
                SettingsScope::User,
                SettingSchema::Int {
                    minimum: 1,
                    maximum: MAXIMUM_USER_JOB_CATEGORY_QUOTA,
                    step: 1,
                },
                SettingValue::Int(default),
                true,
                job_quota_presentation(quota),
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
    let mut configured_limits = Vec::with_capacity(USER_CONFIGURABLE_QUOTAS.len());
    for quota in USER_CONFIGURABLE_QUOTAS {
        let key = SettingsKey::parse(quota.key).expect("built-in job quota key is valid");
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
        configured_limits.push((quota.category, limit));
    }
    Ok(EditorJobLimits::resolved(
        worker_parallelism,
        configured_limits,
    ))
}

fn job_quota_presentation(quota: UserConfigurableQuota) -> SettingsPresentation {
    SettingsPresentation::new(
        quota.label_key,
        quota.description_key,
        JOB_QUOTA_CATEGORY_PATH,
    )
    .expect("built-in job quota presentation keys are valid")
}

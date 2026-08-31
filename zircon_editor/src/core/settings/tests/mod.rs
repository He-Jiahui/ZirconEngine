use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use zircon_runtime_interface::serialization::write_versioned_text;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use crate::core::commands::EditorKeyChord;

use super::defaults::{
    EDITOR_COMMAND_PALETTE_MRU_KEY, EDITOR_DESIGN_TOKENS_KEY, EDITOR_KEYMAP_OVERRIDES_KEY,
    EDITOR_LOCALE_KEY, VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_STEP_KEY,
    VIEWPORT_TRANSLATE_STEP_KEY,
};
use super::io::SettingsDocument;
use super::{
    settings_registry_with_defaults, EditorCommandPaletteMru, SettingDefinition, SettingSchema,
    SettingValue, SettingsAuthority, SettingsChangeCursor, SettingsChangeLogPolicy,
    SettingsChangeSubscriber, SettingsDecodeError, SettingsError, SettingsKey, SettingsLoad,
    SettingsMutationCoordinator, SettingsMutationDisposition, SettingsMutationError, SettingsPaths,
    SettingsPersistenceHealthSnapshot, SettingsPersistenceHealthStatus,
    SettingsPersistenceHealthSubscriber, SettingsPersistenceLimits,
    SettingsPersistenceRetryDisposition, SettingsPersistenceService,
    SettingsPersistenceShutdownError, SettingsPersistenceSubmitError, SettingsPresentation,
    SettingsProjectLayerLoad, SettingsRegistry, SettingsScope, SettingsStore, SettingsStoreError,
};
use crate::core::editor_operation::EditorOperationPath;

fn key(value: &str) -> SettingsKey {
    SettingsKey::parse(value).unwrap()
}

fn presentation(
    label_key: &str,
    description_key: &str,
    category_path: &[&str],
) -> SettingsPresentation {
    SettingsPresentation::new(label_key, description_key, category_path.iter().copied()).unwrap()
}

fn embedded_bundle_contains_translation(
    locale: &crate::core::i18n::EditorLocale,
    key: &str,
) -> bool {
    let bundle = match locale.as_str() {
        "en" => include_str!("../../../../assets/i18n/en.toml"),
        "zh-CN" => include_str!("../../../../assets/i18n/zh-CN.toml"),
        _ => return false,
    };
    toml::from_str::<toml::Table>(bundle)
        .ok()
        .and_then(|document| document.get("translations").cloned())
        .and_then(|translations| translations.as_table().cloned())
        .is_some_and(|translations| translations.contains_key(key))
}

fn project_grid_setting() -> SettingDefinition {
    SettingDefinition::new(
        key("editor.scene.grid_step"),
        SettingsScope::Project,
        SettingSchema::Int {
            minimum: 1,
            maximum: 100,
            step: 1,
        },
        SettingValue::Int(10),
        false,
        presentation(
            "settings.editor.scene.grid_step.label",
            "settings.editor.scene.grid_step.description",
            &["settings.category.scene", "settings.category.grid"],
        ),
    )
    .unwrap()
}

struct ReentrantSettingsChangeSubscriber {
    authority: Arc<SettingsAuthority>,
    observed: Sender<(usize, u64, usize)>,
}

impl SettingsChangeSubscriber for ReentrantSettingsChangeSubscriber {
    fn settings_changed(
        &self,
        changes: &[super::SettingChange],
        snapshot: &super::SettingsSnapshot,
    ) {
        let delta = self.authority.changes_since(SettingsChangeCursor::origin());
        let _ = self
            .observed
            .send((changes.len(), snapshot.generation(), delta.changes.len()));
    }
}

struct ProjectLayerPrepareSubscriber {
    authority: Arc<SettingsAuthority>,
    store: SettingsStore,
    observed: Sender<bool>,
}

impl SettingsChangeSubscriber for ProjectLayerPrepareSubscriber {
    fn settings_changed(
        &self,
        _changes: &[super::SettingChange],
        _snapshot: &super::SettingsSnapshot,
    ) {
        let prepared = self
            .authority
            .prepare_persistent_layer_for_write(SettingsScope::Project, &self.store);
        let _ = self.observed.send(matches!(prepared, Ok(None)));
    }
}

mod autosave;
mod mutation;
mod persistence;
mod registry;
mod value_batch;

fn temporary_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-settings-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos()
    ))
}

fn remove_temporary_root(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

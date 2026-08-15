use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use super::editor_ui_host::EditorUiHost;
use super::runtime_services::EditorHostRuntimeServices;
use crate::core::commands::{EditorCommandPaletteMru, EditorKeymap};
use crate::core::context::{EditorContext, EditorContextBuilder};
use crate::core::document::DocumentLifecycleAuthority;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::plugin::{EditorPluginLifecycleMessageBridge, EditorPluginManager};
use crate::core::recovery::SessionGuard;
use crate::core::settings::{EditorKeymapOverrides, SettingsSnapshot};
use crate::ui::host::editor_manager_plugins_export::{
    EditorPluginStatusReport, ProjectPluginStatusSnapshot,
};
use zircon_runtime::asset::{project::ProjectManifest, AssetUri};
use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::plugin::RuntimePluginCatalog;
use zircon_runtime_interface::hub_protocol::HubSessionToken;
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

/// A manager-local derived cache, keyed by the authority's immutable override payload.
struct EditorKeymapProjection {
    overrides: Arc<EditorKeymapOverrides>,
    keymap: EditorKeymap,
}

impl EditorKeymapProjection {
    fn from_snapshot(snapshot: &SettingsSnapshot) -> Self {
        let overrides = snapshot.keymap_overrides_handle();
        Self {
            keymap: default_workbench_keymap().with_overrides(&overrides),
            overrides,
        }
    }

    fn refresh_if_changed(&mut self, snapshot: &SettingsSnapshot) -> bool {
        let overrides = snapshot.keymap_overrides_handle();
        if Arc::ptr_eq(&self.overrides, &overrides) {
            return false;
        }
        self.keymap = default_workbench_keymap().with_overrides(&overrides);
        self.overrides = overrides;
        true
    }
}

fn default_workbench_keymap() -> &'static EditorKeymap {
    static DEFAULT: OnceLock<EditorKeymap> = OnceLock::new();
    DEFAULT.get_or_init(EditorKeymap::default_workbench)
}

/// The dynamic module service for the current authority-derived keymap.
///
/// It preserves the module's named manager boundary without retaining a startup snapshot after
/// a settings override changes.
pub struct EditorKeymapService {
    settings: Arc<crate::core::settings::SettingsAuthority>,
    projection: Mutex<EditorKeymapProjection>,
}

impl EditorKeymapService {
    fn new(settings: Arc<crate::core::settings::SettingsAuthority>) -> Self {
        let snapshot = settings.snapshot();
        Self {
            projection: Mutex::new(EditorKeymapProjection::from_snapshot(snapshot.as_ref())),
            settings,
        }
    }

    pub fn snapshot(&self) -> EditorKeymap {
        let snapshot = self.settings.snapshot();
        let mut projection = self
            .projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        projection.refresh_if_changed(snapshot.as_ref());
        projection.keymap.clone()
    }

    pub fn resolve_keyboard_input(&self, keyboard: &UiKeyboardInputEvent) -> Option<String> {
        let snapshot = self.settings.snapshot();
        let mut projection = self
            .projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        projection.refresh_if_changed(snapshot.as_ref());
        projection
            .keymap
            .resolve_keyboard_input(keyboard)
            .map(str::to_owned)
    }
}

pub struct EditorManager {
    pub(super) host: EditorUiHost,
    context: Arc<EditorContext>,
    pub(super) document_lifecycle: DocumentLifecycleAuthority,
    keymap: Arc<EditorKeymapService>,
    plugin_manager: EditorPluginManager,
    plugin_lifecycle_messages: EditorPluginLifecycleMessageBridge,
    builtin_plugin_status: Mutex<Arc<ProjectPluginStatusSnapshot>>,
    project_plugin_status: Mutex<Option<Arc<ProjectPluginStatusSnapshot>>>,
    /// The OS-backed admission lease for the active runtime project generation.
    pub(super) project_session_guard: Mutex<Option<SessionGuard>>,
    /// One Hub launch token consumed only by the first project admission attempt of this host.
    pub(super) hub_launch_session: Mutex<Option<HubSessionToken>>,
    capability_updates: Mutex<()>,
}

impl Drop for EditorManager {
    fn drop(&mut self) {
        // `run_editor_with_config` releases this lease explicitly. This covers construction and
        // other early-error paths where Rust cannot propagate a shutdown error from `Drop`.
        let guard_slot = self
            .project_session_guard
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(guard) = guard_slot.as_mut() {
            let _ = guard.release();
        }
    }
}

impl EditorManager {
    pub(in crate::ui::host) fn plugin_manager(&self) -> &EditorPluginManager {
        &self.plugin_manager
    }

    pub fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        let scheduler = core.scheduler().clone();
        let context = EditorContextBuilder::new(scheduler).build();
        let runtime_services = EditorHostRuntimeServices::new(core);
        let host = EditorUiHost::bootstrap(
            runtime_services,
            context.jobs().clone(),
            context.logs_handle(),
            context.dirty_documents().clone(),
            Arc::clone(context.settings()),
        )
        .map_err(|error| {
            CoreError::Initialization("EditorManager".to_string(), error.to_string())
        })?;
        let keymap = Arc::new(EditorKeymapService::new(Arc::clone(context.settings())));
        let plugin_manager = EditorPluginManager::builtin(
            RuntimePluginCatalog::builtin().package_manifests().cloned(),
        )
        .map_err(|error| {
            CoreError::Initialization("EditorPluginManager".to_string(), error.to_string())
        })?;
        let plugin_lifecycle_messages = EditorPluginLifecycleMessageBridge::new(context.bus())
            .map_err(|error| {
                CoreError::Initialization(
                    "EditorPluginLifecycleMessageBridge".to_string(),
                    error.to_string(),
                )
            })?;
        let manager = Self {
            host,
            context,
            document_lifecycle: DocumentLifecycleAuthority::default(),
            keymap,
            plugin_manager,
            plugin_lifecycle_messages,
            builtin_plugin_status: Mutex::new(Arc::new(ProjectPluginStatusSnapshot::new(
                EditorPluginStatusReport::default(),
            ))),
            project_plugin_status: Mutex::new(None),
            project_session_guard: Mutex::new(None),
            hub_launch_session: Mutex::new(None),
            capability_updates: Mutex::new(()),
        };
        manager.refresh_builtin_plugin_status();
        Ok(manager)
    }

    pub fn context(&self) -> &Arc<EditorContext> {
        &self.context
    }

    /// Configures the single Hub launch context before startup is allowed to admit a project.
    pub(crate) fn configure_hub_launch_session(&self, session: Option<HubSessionToken>) {
        *self
            .hub_launch_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = session;
    }

    /// Returns the current derived keymap for the legacy manager service boundary.
    pub(crate) fn keymap(&self) -> EditorKeymap {
        self.keymap.snapshot()
    }

    pub(crate) fn keymap_service(&self) -> Arc<EditorKeymapService> {
        Arc::clone(&self.keymap)
    }

    pub(crate) fn resolve_keyboard_input(&self, keyboard: &UiKeyboardInputEvent) -> Option<String> {
        self.keymap.resolve_keyboard_input(keyboard)
    }

    pub(crate) fn command_palette_mru(&self) -> EditorCommandPaletteMru {
        self.context
            .settings()
            .snapshot()
            .command_palette_mru()
            .clone()
    }

    pub(crate) fn record_command_palette_usage(&self, command: EditorOperationPath) {
        self.context
            .settings()
            .record_command_palette_usage(command)
            .expect("the command-palette MRU authority accepts built-in Session usage");
    }

    pub(crate) fn lock_editor_capability_updates(&self) -> MutexGuard<'_, ()> {
        self.capability_updates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn validate_editor_plugin_state(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        self.plugin_manager
            .validate_enablement(plugin_id, enabled)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn update_editor_plugin_state(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        self.update_editor_plugin_state_unpublished(plugin_id, enabled)?;
        self.refresh_builtin_plugin_status();
        Ok(())
    }

    pub(in crate::ui::host) fn update_editor_plugin_state_unpublished(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        self.plugin_manager
            .set_enabled(plugin_id, enabled)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    pub(crate) fn pump_plugin_lifecycle_messages(&self) -> Result<usize, String> {
        self.plugin_lifecycle_messages
            .pump(self.context.bus(), &self.plugin_manager)
            .map(|report| report.lifecycle_messages())
            .map_err(|error| error.to_string())
    }

    pub(crate) fn published_plugin_status_report(&self) -> Arc<EditorPluginStatusReport> {
        let project_snapshot = self
            .project_plugin_status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(snapshot) = project_snapshot.as_ref() {
            return Arc::clone(snapshot.report());
        }
        drop(project_snapshot);

        let builtin_snapshot = self
            .builtin_plugin_status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(builtin_snapshot.report())
    }

    pub(in crate::ui::host) fn refresh_builtin_plugin_status(&self) {
        let report = self.plugin_status_report(&builtin_plugin_status_manifest());
        let mut snapshot = self
            .builtin_plugin_status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *snapshot = Arc::new(ProjectPluginStatusSnapshot::new(report));
    }

    pub(in crate::ui::host) fn publish_project_plugin_status(
        &self,
        report: EditorPluginStatusReport,
    ) {
        let mut snapshot = self
            .project_plugin_status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *snapshot = Some(Arc::new(ProjectPluginStatusSnapshot::new(report)));
    }

    pub(in crate::ui::host) fn clear_project_plugin_status(&self) {
        let mut snapshot = self
            .project_plugin_status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        snapshot.take();
    }
}

fn builtin_plugin_status_manifest() -> ProjectManifest {
    ProjectManifest::new(
        "Unsaved",
        AssetUri::parse("res://scenes/main.scene.toml")
            .expect("builtin status fallback asset URI is valid"),
        1,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::core::commands::EditorKeyChord;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::settings::{
        EditorKeymapOverrides, SettingValue, SettingsAuthority, SettingsKey, SettingsScope,
        EDITOR_KEYMAP_OVERRIDES_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
    };

    use super::EditorKeymapProjection;

    #[test]
    fn keymap_projection_reuses_the_authority_payload_until_overrides_change() {
        let authority = SettingsAuthority::with_defaults();
        let initial = authority.snapshot();
        let mut projection = EditorKeymapProjection::from_snapshot(initial.as_ref());
        assert!(!projection.refresh_if_changed(initial.as_ref()));

        let viewport_key = SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        authority
            .set(
                SettingsScope::Project,
                &viewport_key,
                SettingValue::Float(2.0),
            )
            .unwrap();
        assert!(!projection.refresh_if_changed(authority.snapshot().as_ref()));

        let keymap_key = SettingsKey::parse(EDITOR_KEYMAP_OVERRIDES_KEY).unwrap();
        let overrides = EditorKeymapOverrides::new(BTreeMap::from([(
            EditorOperationPath::parse("file.project.open").unwrap(),
            Some("Alt+O".parse::<EditorKeyChord>().unwrap()),
        )]));
        authority
            .set(
                SettingsScope::User,
                &keymap_key,
                SettingValue::KeymapOverrides(overrides),
            )
            .unwrap();
        assert!(projection.refresh_if_changed(authority.snapshot().as_ref()));
        assert_eq!(
            projection
                .keymap
                .chord_for_command("file.project.open")
                .unwrap()
                .to_string(),
            "Alt+O"
        );
    }
}

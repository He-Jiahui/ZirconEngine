use std::sync::{Arc, Mutex, MutexGuard};

use super::editor_ui_host::EditorUiHost;
use crate::core::commands::{EditorCommandPaletteMru, EditorKeymap};
use crate::core::context::{EditorContext, EditorContextBuilder};
use crate::core::document::DocumentLifecycleAuthority;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::plugin::{EditorPluginLifecycleMessageBridge, EditorPluginManager};
use crate::core::settings::{
    editor_command_palette_mru, editor_keymap_overrides, record_editor_command_palette_usage,
    settings_registry_at_startup, SettingsRegistry,
};
use crate::ui::host::editor_manager_plugins_export::{
    EditorPluginStatusReport, ProjectPluginStatusSnapshot,
};
use zircon_runtime::asset::{project::ProjectManifest, AssetUri};
use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::plugin::RuntimePluginCatalog;

pub struct EditorManager {
    pub(super) host: EditorUiHost,
    context: Arc<EditorContext>,
    pub(super) document_lifecycle: DocumentLifecycleAuthority,
    keymap: EditorKeymap,
    plugin_manager: EditorPluginManager,
    plugin_lifecycle_messages: EditorPluginLifecycleMessageBridge,
    builtin_plugin_status: Mutex<Arc<ProjectPluginStatusSnapshot>>,
    project_plugin_status: Mutex<Option<Arc<ProjectPluginStatusSnapshot>>>,
    settings: Mutex<SettingsRegistry>,
    capability_updates: Mutex<()>,
}

impl EditorManager {
    pub fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        let scheduler = core.scheduler().clone();
        let context = EditorContextBuilder::new(scheduler).build();
        let host = EditorUiHost::bootstrap(
            core,
            context.jobs().clone(),
            context.dirty_documents().clone(),
        )
        .map_err(|error| {
            CoreError::Initialization("EditorManager".to_string(), error.to_string())
        })?;
        let settings = settings_registry_at_startup();
        let keymap =
            EditorKeymap::default_workbench().with_overrides(editor_keymap_overrides(&settings));
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
            settings: Mutex::new(settings),
            capability_updates: Mutex::new(()),
        };
        manager.refresh_builtin_plugin_status();
        Ok(manager)
    }

    pub fn context(&self) -> &Arc<EditorContext> {
        &self.context
    }

    pub(crate) fn keymap(&self) -> &EditorKeymap {
        &self.keymap
    }

    pub(crate) fn command_palette_mru(&self) -> EditorCommandPaletteMru {
        let settings = self
            .settings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        editor_command_palette_mru(&settings).clone()
    }

    pub(crate) fn record_command_palette_usage(&self, command: EditorOperationPath) {
        let mut settings = self
            .settings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        record_editor_command_palette_usage(&mut settings, command);
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

    fn refresh_builtin_plugin_status(&self) {
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

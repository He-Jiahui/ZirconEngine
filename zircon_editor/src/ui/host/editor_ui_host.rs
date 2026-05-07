use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use zircon_runtime::core::CoreHandle;

use crate::ui::workbench::layout::LayoutManager;
use crate::ui::workbench::view::{ViewInstanceId, ViewRegistry};
use crate::ui::workbench::window_registry::EditorWindowRegistry;

use super::animation_editor_sessions::AnimationEditorWorkspaceEntry;
use super::asset_editor_sessions::{UiAssetWorkspaceEntry, UiAssetWorkspaceWatcher};
use super::editor_capabilities::EditorCapabilitySnapshot;
use super::editor_error::EditorError;
use super::editor_session_state::EditorSessionState;
use super::editor_subsystems::{
    editor_runtime_sandbox_enabled, editor_subsystem_report_from_core, EditorSubsystemReport,
};
use super::host_capability_bridge::{register_vm_host_capabilities, EditorHostVmBridgeReport};
use super::minimal_host_contract::{editor_host_minimal_contract, EditorHostMinimalReport};
use super::window_host_manager::WindowHostManager;

pub(super) struct EditorUiHost {
    pub(super) core: CoreHandle,
    pub(super) view_registry: Mutex<ViewRegistry>,
    pub(super) layout_manager: LayoutManager,
    pub(super) window_host_manager: Mutex<WindowHostManager>,
    pub(super) window_registry: Mutex<EditorWindowRegistry>,
    pub(super) session: Mutex<EditorSessionState>,
    pub(super) animation_editor_sessions:
        Mutex<BTreeMap<ViewInstanceId, AnimationEditorWorkspaceEntry>>,
    pub(super) ui_asset_sessions: Mutex<BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>>,
    pub(super) ui_asset_workspace_watcher: Mutex<Option<UiAssetWorkspaceWatcher>>,
    pub(super) minimal_report: EditorHostMinimalReport,
    pub(super) subsystem_report: Mutex<EditorSubsystemReport>,
    pub(super) capability_snapshot: Mutex<EditorCapabilitySnapshot>,
    pub(super) vm_bridge_report: EditorHostVmBridgeReport,
}

impl EditorUiHost {
    fn recover_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_view_registry(&self) -> MutexGuard<'_, ViewRegistry> {
        Self::recover_lock(&self.view_registry)
    }

    pub(super) fn lock_window_host_manager(&self) -> MutexGuard<'_, WindowHostManager> {
        Self::recover_lock(&self.window_host_manager)
    }

    pub(super) fn lock_window_registry(&self) -> MutexGuard<'_, EditorWindowRegistry> {
        Self::recover_lock(&self.window_registry)
    }

    pub(super) fn lock_session(&self) -> MutexGuard<'_, EditorSessionState> {
        Self::recover_lock(&self.session)
    }

    pub(super) fn lock_animation_editor_sessions(
        &self,
    ) -> MutexGuard<'_, BTreeMap<ViewInstanceId, AnimationEditorWorkspaceEntry>> {
        Self::recover_lock(&self.animation_editor_sessions)
    }

    pub(super) fn lock_ui_asset_sessions(
        &self,
    ) -> MutexGuard<'_, BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>> {
        Self::recover_lock(&self.ui_asset_sessions)
    }

    pub(super) fn lock_ui_asset_workspace_watcher(
        &self,
    ) -> MutexGuard<'_, Option<UiAssetWorkspaceWatcher>> {
        Self::recover_lock(&self.ui_asset_workspace_watcher)
    }

    pub(super) fn lock_subsystem_report(&self) -> MutexGuard<'_, EditorSubsystemReport> {
        Self::recover_lock(&self.subsystem_report)
    }

    pub(super) fn lock_capability_snapshot(&self) -> MutexGuard<'_, EditorCapabilitySnapshot> {
        Self::recover_lock(&self.capability_snapshot)
    }

    pub(super) fn new(core: CoreHandle) -> Self {
        let minimal_report = editor_host_minimal_contract().self_check();
        let subsystem_report = editor_subsystem_report_from_core(&core);
        let capability_snapshot =
            EditorCapabilitySnapshot::from_reports(&minimal_report, &subsystem_report);
        let runtime_sandbox_enabled = editor_runtime_sandbox_enabled(&core);
        let vm_bridge_report = register_vm_host_capabilities(&core, runtime_sandbox_enabled);

        Self {
            core,
            view_registry: Mutex::new(ViewRegistry::default()),
            layout_manager: LayoutManager,
            window_host_manager: Mutex::new(WindowHostManager::default()),
            window_registry: Mutex::new(EditorWindowRegistry::default()),
            session: Mutex::new(EditorSessionState::default()),
            animation_editor_sessions: Mutex::new(BTreeMap::new()),
            ui_asset_sessions: Mutex::new(BTreeMap::new()),
            ui_asset_workspace_watcher: Mutex::new(None),
            minimal_report,
            subsystem_report: Mutex::new(subsystem_report),
            capability_snapshot: Mutex::new(capability_snapshot),
            vm_bridge_report,
        }
    }

    pub(super) fn bootstrap(core: CoreHandle) -> Result<Self, EditorError> {
        let host = Self::new(core);
        host.register_builtin_views()?;
        host.bootstrap_default_layout()?;
        Ok(host)
    }

    pub(super) fn refresh_capabilities(&self) -> Result<EditorCapabilitySnapshot, EditorError> {
        let subsystem_report = editor_subsystem_report_from_core(&self.core);
        let snapshot =
            EditorCapabilitySnapshot::from_reports(&self.minimal_report, &subsystem_report);
        *self.lock_subsystem_report() = subsystem_report;
        *self.lock_capability_snapshot() = snapshot.clone();
        self.register_builtin_views()?;
        Ok(snapshot)
    }
}

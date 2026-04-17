use crate::layout::{MainHostPageLayout, MainPageId, RestorePolicy, WorkbenchLayout};
use crate::project::ProjectEditorWorkspace;
use crate::view::{ViewDescriptor, ViewInstance};

use super::builtin_layout::ensure_builtin_shell_instances;
use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use super::editor_session_state::EditorSessionState;
use super::layout_hosts::{
    active_tab_from_document::active_tab_from_document,
    collect_instance_hosts::collect_instance_hosts,
    repair_builtin_shell_layout::repair_builtin_shell_layout,
};
use super::ui_asset_sessions::UI_ASSET_EDITOR_DESCRIPTOR_ID;

impl EditorManager {
    pub fn current_layout(&self) -> WorkbenchLayout {
        self.session.lock().unwrap().layout.clone()
    }

    pub fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.session
            .lock()
            .unwrap()
            .open_view_instances
            .values()
            .cloned()
            .collect()
    }

    pub fn native_window_hosts(&self) -> Vec<super::window_host_manager::NativeWindowHostState> {
        self.window_host_manager.lock().unwrap().states()
    }

    pub fn sync_native_window_projection_bounds(
        &self,
        window_id: &MainPageId,
        bounds: [f32; 4],
    ) {
        self.window_host_manager
            .lock()
            .unwrap()
            .sync_window_bounds(window_id, bounds);
    }

    pub fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.view_registry.lock().unwrap().list_descriptors()
    }

    pub fn restore_workspace(&self, policy: RestorePolicy) -> Result<WorkbenchLayout, EditorError> {
        let global = self.load_global_default_layout();
        let workspace = self.project_workspace();
        let restored = self
            .layout_manager
            .restore_workspace(policy, Some(workspace), global)
            .map_err(EditorError::Layout)?;
        let mut session = self.session.lock().unwrap();
        session.layout = restored.clone();
        self.recompute_session_metadata(&mut session);
        Ok(restored)
    }

    pub fn apply_project_workspace(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Result<(), EditorError> {
        if workspace.is_none() {
            return self.bootstrap_default_layout();
        }

        let mut session = self.session.lock().unwrap();
        let mut registry = self.view_registry.lock().unwrap();
        registry.clear_instances();
        self.ui_asset_sessions.lock().unwrap().clear();

        let workspace = workspace.expect("checked above");
        session.layout = workspace.workbench;
        session.open_view_instances.clear();
        for instance in workspace.open_view_instances {
            let restored = registry
                .restore_instance(instance)
                .map_err(EditorError::Registry)?;
            session
                .open_view_instances
                .insert(restored.instance_id.clone(), restored);
        }
        session.active_center_tab = workspace.active_center_tab;
        session.active_drawers = workspace.active_drawers;
        self.layout_manager
            .normalize(&mut session.layout, &registry);
        self.recompute_session_metadata(&mut session);
        let ui_asset_instances = session
            .open_view_instances
            .values()
            .filter(|instance| instance.descriptor_id.0 == UI_ASSET_EDITOR_DESCRIPTOR_ID)
            .cloned()
            .collect::<Vec<_>>();
        drop(registry);
        drop(session);
        for instance in ui_asset_instances {
            self.restore_ui_asset_editor_instance(&instance)?;
        }
        Ok(())
    }

    pub fn project_workspace(&self) -> ProjectEditorWorkspace {
        let session = self.session.lock().unwrap();
        ProjectEditorWorkspace {
            layout_version: 1,
            workbench: session.layout.clone(),
            open_view_instances: session.open_view_instances.values().cloned().collect(),
            active_center_tab: session.active_center_tab.clone(),
            active_drawers: session.active_drawers.clone(),
        }
    }

    pub(super) fn bootstrap_default_layout(&self) -> Result<(), EditorError> {
        let mut registry = self.view_registry.lock().unwrap();
        registry.clear_instances();
        self.ui_asset_sessions.lock().unwrap().clear();
        let mut session = EditorSessionState::default();
        ensure_builtin_shell_instances(&mut registry, &mut session)?;
        session.layout = self.layout_manager.default_layout();
        self.layout_manager
            .normalize(&mut session.layout, &registry);
        *self.session.lock().unwrap() = session;

        if let Some(layout) = self.load_global_default_layout() {
            let mut session = self.session.lock().unwrap();
            session.layout = layout;
            repair_builtin_shell_layout(&mut session.layout);
            self.layout_manager
                .normalize(&mut session.layout, &registry);
            self.recompute_session_metadata(&mut session);
        } else {
            let mut session = self.session.lock().unwrap();
            self.recompute_session_metadata(&mut session);
        }
        Ok(())
    }

    pub(super) fn recompute_session_metadata(&self, session: &mut EditorSessionState) {
        let placements = collect_instance_hosts(&session.layout);
        session
            .open_view_instances
            .retain(|instance_id, _| placements.contains_key(instance_id));
        for (instance_id, host) in placements {
            if let Some(instance) = session.open_view_instances.get_mut(&instance_id) {
                instance.host = host;
            }
        }

        session.active_drawers = session
            .layout
            .drawers
            .iter()
            .filter_map(|(slot, drawer)| drawer.visible.then_some(*slot))
            .collect();
        session.active_center_tab = session
            .layout
            .main_pages
            .iter()
            .find(|page| page.id() == &session.layout.active_main_page)
            .and_then(|page| match page {
                MainHostPageLayout::WorkbenchPage {
                    document_workspace, ..
                } => active_tab_from_document(document_workspace),
                MainHostPageLayout::ExclusiveActivityWindowPage {
                    window_instance, ..
                } => Some(window_instance.clone()),
            });
        self.ui_asset_sessions
            .lock()
            .unwrap()
            .retain(|instance_id, _| session.open_view_instances.contains_key(instance_id));
        self.window_host_manager
            .lock()
            .unwrap()
            .sync_layout_windows(&session.layout);
    }
}

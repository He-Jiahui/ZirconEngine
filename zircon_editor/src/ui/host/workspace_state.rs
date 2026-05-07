use serde_json::Value;

use crate::ui::workbench::layout::{
    MainHostPageLayout, MainPageId, RestorePolicy, WorkbenchLayout,
};
use crate::ui::workbench::project::ProjectEditorWorkspace;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance, ViewInstanceId};
use crate::ui::workbench::window_registry::EditorWindowRegistry;

use super::asset_editor_sessions::UI_ASSET_EDITOR_DESCRIPTOR_ID;
use super::builtin_layout::{builtin_hybrid_layout_for_subsystems, ensure_builtin_shell_instances};
use super::editor_error::EditorError;
use super::editor_session_state::EditorSessionState;
use super::editor_ui_host::EditorUiHost;
use super::layout_hosts::{
    active_tab_from_document::active_tab_from_document,
    collect_instance_hosts::collect_instance_hosts,
    repair_builtin_shell_layout::repair_builtin_shell_layout,
};

impl EditorUiHost {
    pub(super) fn current_layout(&self) -> WorkbenchLayout {
        self.lock_session().layout.clone()
    }

    pub(super) fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.lock_session()
            .open_view_instances
            .values()
            .cloned()
            .collect()
    }

    pub(super) fn update_view_instance_metadata(
        &self,
        instance_id: &ViewInstanceId,
        title: Option<String>,
        dirty: Option<bool>,
        payload: Option<Value>,
    ) -> Result<(), EditorError> {
        let mut session = self.lock_session();
        let instance = session
            .open_view_instances
            .get_mut(instance_id)
            .ok_or_else(|| {
                EditorError::Registry(format!("missing view instance {}", instance_id.0))
            })?;
        if let Some(title) = title {
            instance.title = title;
        }
        if let Some(dirty) = dirty {
            instance.dirty = dirty;
        }
        if let Some(payload) = payload {
            instance.serializable_payload = payload;
        }
        Ok(())
    }

    pub(super) fn native_window_hosts(
        &self,
    ) -> Vec<super::window_host_manager::NativeWindowHostState> {
        self.lock_window_host_manager().states()
    }

    pub(super) fn sync_native_window_projection_bounds(
        &self,
        window_id: &MainPageId,
        bounds: [f32; 4],
    ) {
        self.lock_window_host_manager()
            .sync_window_bounds(window_id, bounds);
    }

    pub(super) fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.lock_view_registry().list_descriptors()
    }

    pub(super) fn restore_workspace(
        &self,
        policy: RestorePolicy,
    ) -> Result<WorkbenchLayout, EditorError> {
        let global = self.load_global_default_layout();
        let workspace = self.project_workspace();
        let restored = self
            .layout_manager
            .restore_workspace(policy, Some(workspace), global)
            .map_err(EditorError::Layout)?;
        let mut session = self.lock_session();
        session.layout = restored.clone();
        self.recompute_session_metadata(&mut session);
        Ok(restored)
    }

    pub(super) fn apply_project_workspace_state(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Result<Vec<ViewInstance>, EditorError> {
        if workspace.is_none() {
            self.bootstrap_default_layout()?;
            return Ok(Vec::new());
        }

        let mut session = self.lock_session();
        let mut registry = self.lock_view_registry();
        registry.clear_instances();
        self.lock_animation_editor_sessions().clear();
        self.lock_ui_asset_sessions().clear();

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
        let snapshot = self.lock_capability_snapshot().clone();
        let subsystem_report = self.lock_subsystem_report().clone();
        ensure_builtin_shell_instances(&mut registry, &mut session, &snapshot)?;
        let open_instances = session
            .open_view_instances
            .values()
            .cloned()
            .collect::<Vec<_>>();
        repair_builtin_shell_layout(&mut session.layout, &open_instances, &subsystem_report);
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
        Ok(ui_asset_instances)
    }

    pub(super) fn apply_project_workspace(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Result<(), EditorError> {
        for instance in self.apply_project_workspace_state(workspace)? {
            self.restore_ui_asset_editor_instance(&instance)?;
        }
        Ok(())
    }

    pub(super) fn project_workspace(&self) -> ProjectEditorWorkspace {
        let session = self.lock_session();
        ProjectEditorWorkspace {
            layout_version: 1,
            workbench: session.layout.clone(),
            open_view_instances: session.open_view_instances.values().cloned().collect(),
            active_center_tab: session.active_center_tab.clone(),
            active_drawers: session.active_drawers.clone(),
        }
    }

    pub(super) fn bootstrap_default_layout(&self) -> Result<(), EditorError> {
        let mut registry = self.lock_view_registry();
        registry.clear_instances();
        self.lock_animation_editor_sessions().clear();
        self.lock_ui_asset_sessions().clear();
        let mut session = EditorSessionState::default();
        let snapshot = self.lock_capability_snapshot().clone();
        let subsystem_report = self.lock_subsystem_report().clone();
        ensure_builtin_shell_instances(&mut registry, &mut session, &snapshot)?;
        session.layout = builtin_hybrid_layout_for_subsystems(&subsystem_report);
        self.layout_manager
            .normalize(&mut session.layout, &registry);
        *self.lock_session() = session;

        if let Some(layout) = self.load_global_default_layout() {
            let mut session = self.lock_session();
            session.layout = layout;
            let open_instances = session
                .open_view_instances
                .values()
                .cloned()
                .collect::<Vec<_>>();
            repair_builtin_shell_layout(&mut session.layout, &open_instances, &subsystem_report);
            self.layout_manager
                .normalize(&mut session.layout, &registry);
            self.recompute_session_metadata(&mut session);
        } else {
            let mut session = self.lock_session();
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

        let open_instances = session
            .open_view_instances
            .values()
            .cloned()
            .collect::<Vec<_>>();
        *self.lock_window_registry() =
            EditorWindowRegistry::sync_from_layout(&session.layout, &open_instances);
        session.active_drawers = session
            .layout
            .active_activity_window_drawers()
            .into_iter()
            .filter_map(|(slot, drawer)| drawer.visible.then_some(slot))
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
        self.lock_animation_editor_sessions()
            .retain(|instance_id, _| session.open_view_instances.contains_key(instance_id));
        self.lock_ui_asset_sessions()
            .retain(|instance_id, _| session.open_view_instances.contains_key(instance_id));
        self.lock_window_host_manager()
            .sync_layout_windows(&session.layout);
    }
}

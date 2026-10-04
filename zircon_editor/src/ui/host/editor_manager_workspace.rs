use serde_json::Value;
use zircon_runtime::asset::AssetUri;

use crate::core::project::ProjectAuthority;
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, MainPageId, RestorePolicy, WorkbenchLayout,
};
use crate::ui::workbench::project::ProjectEditorWorkspace;
use crate::ui::workbench::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn current_layout(&self) -> WorkbenchLayout {
        self.host.current_layout()
    }

    pub fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.host.current_view_instances()
    }

    pub fn current_focused_view(&self) -> Option<ViewInstanceId> {
        self.host.current_focused_view()
    }

    pub fn current_focused_view_matches(&self, descriptor_id: &ViewDescriptorId) -> bool {
        self.host.current_focused_view_matches(descriptor_id)
    }

    pub fn active_activity_window_template_document_is(&self, document_id: &str) -> bool {
        self.host
            .active_activity_window_template_document_is(document_id)
    }

    pub fn floating_window_focus_target(&self, window_id: &MainPageId) -> Option<ViewInstanceId> {
        self.host.floating_window_focus_target(window_id)
    }

    pub fn floating_window_id_for_surface_key(&self, surface_key: &str) -> Option<MainPageId> {
        self.host.floating_window_id_for_surface_key(surface_key)
    }

    pub fn active_drawer_toggle_state(
        &self,
        slot: ActivityDrawerSlot,
        instance_id: &ViewInstanceId,
    ) -> Result<(ActivityDrawerMode, bool, bool), String> {
        self.host.active_drawer_toggle_state(slot, instance_id)
    }

    pub fn active_drawer_mode(&self, slot: ActivityDrawerSlot) -> Option<ActivityDrawerMode> {
        self.host.active_drawer_mode(slot)
    }

    pub fn floating_window_exists(&self, window_id: &MainPageId) -> bool {
        self.host.floating_window_exists(window_id)
    }

    pub fn floating_window_instance_ids(
        &self,
        window_id: &MainPageId,
    ) -> Option<Vec<ViewInstanceId>> {
        self.host.floating_window_instance_ids(window_id)
    }

    pub fn view_host_for_instance_key(&self, surface_key: &str) -> Option<ViewHost> {
        self.host.view_host_for_instance_key(surface_key)
    }

    pub fn view_instance_id_for_descriptor(
        &self,
        descriptor_id: &ViewDescriptorId,
    ) -> Option<ViewInstanceId> {
        self.host.view_instance_id_for_descriptor(descriptor_id)
    }

    pub fn current_view_instance_ids(&self) -> Vec<ViewInstanceId> {
        self.host.current_view_instance_ids()
    }

    pub fn view_instance_ids_for_descriptor_key(
        &self,
        descriptor_key: &str,
    ) -> Vec<ViewInstanceId> {
        self.host
            .view_instance_ids_for_descriptor_key(descriptor_key)
    }

    pub fn editor_pane_instance_ids(
        &self,
        collect_ui_asset_panes: bool,
        collect_animation_panes: bool,
    ) -> (Vec<ViewInstanceId>, Vec<ViewInstanceId>) {
        self.host
            .editor_pane_instance_ids(collect_ui_asset_panes, collect_animation_panes)
    }

    pub fn update_view_instance_metadata(
        &self,
        instance_id: &ViewInstanceId,
        title: Option<String>,
        dirty: Option<bool>,
        payload: Option<Value>,
    ) -> Result<(), EditorError> {
        self.host
            .update_view_instance_metadata(instance_id, title, dirty, payload)
    }

    pub fn native_window_hosts(&self) -> Vec<super::window_host_manager::NativeWindowHostState> {
        self.host.native_window_hosts()
    }

    pub fn sync_native_window_projection_bounds(&self, window_id: &MainPageId, bounds: [f32; 4]) {
        self.host
            .sync_native_window_projection_bounds(window_id, bounds)
    }

    pub fn descriptors(&self) -> Vec<ViewDescriptor> {
        self.host.descriptors()
    }

    pub(crate) fn retire_extension_views(
        &self,
        descriptor_ids: &[ViewDescriptorId],
    ) -> Result<(), EditorError> {
        self.host.retire_extension_views(descriptor_ids)
    }

    pub fn restore_workspace(&self, policy: RestorePolicy) -> Result<WorkbenchLayout, EditorError> {
        self.host.restore_workspace(policy)
    }

    pub fn apply_project_workspace(
        &self,
        workspace: Option<ProjectEditorWorkspace>,
    ) -> Result<(), EditorError> {
        self.host.apply_project_workspace(workspace)
    }

    pub fn project_workspace(&self) -> ProjectEditorWorkspace {
        self.host.project_workspace()
    }

    pub(crate) fn save_active_scene_with_workspace(
        &self,
        path: impl AsRef<std::path::Path>,
        world: &zircon_runtime::scene::Scene,
        workspace: &ProjectEditorWorkspace,
    ) -> Result<(), EditorError> {
        let project_root = ProjectAuthority::default().resolve_existing_project_root(&path)?;
        let active_scene = self
            .document_lifecycle
            .active_scene_identity(&project_root)
            .ok_or_else(|| {
                EditorError::Project(
                    "cannot save without an active project scene document".to_string(),
                )
            })?;
        let scene_uri = AssetUri::parse(active_scene.scene_uri()).map_err(|error| {
            EditorError::Project(format!(
                "active scene document {} has an invalid source URI: {error}",
                active_scene.document().value()
            ))
        })?;
        self.host
            .save_active_scene_with_workspace(&project_root, &scene_uri, world, workspace)?;
        self.publish_document_messages(
            self.document_lifecycle
                .save_scene_identity_if_active(&active_scene),
        );
        Ok(())
    }
}

use serde_json::Value;

use crate::ui::workbench::layout::{MainPageId, RestorePolicy, WorkbenchLayout};
use crate::ui::workbench::project::ProjectEditorWorkspace;
use crate::ui::workbench::view::{ViewDescriptor, ViewInstance, ViewInstanceId};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn current_layout(&self) -> WorkbenchLayout {
        self.host.current_layout()
    }

    pub fn current_view_instances(&self) -> Vec<ViewInstance> {
        self.host.current_view_instances()
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
}

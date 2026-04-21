use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn apply_layout_command(&self, cmd: LayoutCommand) -> Result<bool, EditorError> {
        self.host.apply_layout_command(cmd)
    }

    pub fn open_view(
        &self,
        descriptor_id: ViewDescriptorId,
        target_host: Option<ViewHost>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.host.open_view(descriptor_id, target_host)
    }

    pub fn close_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.host.close_view(instance_id)
    }

    pub fn focus_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.host.focus_view(instance_id)
    }

    pub fn detach_view_to_window(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.host.detach_view_to_window(instance_id)
    }

    pub fn attach_view_to_target(
        &self,
        instance_id: &ViewInstanceId,
        drop_target: ViewHost,
    ) -> Result<bool, EditorError> {
        self.host.attach_view_to_target(instance_id, drop_target)
    }

    pub fn save_global_default_layout(&self) -> Result<(), EditorError> {
        self.host.save_global_default_layout()
    }

    pub fn preset_names(&self) -> Result<Vec<String>, EditorError> {
        self.host.preset_names()
    }
}

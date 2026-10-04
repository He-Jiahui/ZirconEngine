use super::*;
use crate::ui::host::EditorError;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
    authored_panes, owns_ordinary_panes,
};
use crate::ui::workbench::autolayout::right_drawer_should_collapse_for_logical_width;
use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

impl RetainedEditorHost {
    pub(super) fn reveal_explicitly_opened_view(
        &mut self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        self.sync_shell_size();
        let resolution = ResolutionContext::from_physical_size_with_scale_mode(
            self.shell_size,
            self.shell_scale_factor,
            self.shell_scale_mode,
        );
        if !right_drawer_should_collapse_for_logical_width(resolution.logical_width()) {
            return Ok(());
        }
        let instance = self
            .editor_manager
            .current_view_instances()
            .into_iter()
            .find(|view| &view.instance_id == instance_id)
            .ok_or_else(|| {
                EditorError::Registry(format!(
                    "opened view {} is no longer present",
                    instance_id.0
                ))
            })?;
        // Existing floating/document/custom drawer placements remain user-owned.
        let (default_slot, authored_root) = match instance.descriptor_id.0.as_str() {
            "editor.hierarchy" => (
                ActivityDrawerSlot::LeftTop,
                "WorkbenchMainBandSceneTreePanel",
            ),
            "editor.inspector" => (
                ActivityDrawerSlot::RightTop,
                "WorkbenchMainBandInspectorPanel",
            ),
            _ => return Ok(()),
        };
        if instance.host != ViewHost::Drawer(default_slot)
            || self.editor_manager.current_layout().active_main_page != MainPageId::workbench()
        {
            return Ok(());
        }
        self.recompute_if_dirty();
        let presentation = self.ui.get_host_presentation();
        if !owns_ordinary_panes(&presentation)
            || authored_panes(&presentation)
                .iter()
                .any(|pane| pane.root.control_id.as_str() == authored_root)
        {
            return Ok(());
        }
        // Hierarchy's registry slot is historically LeftTop, while the current authored
        // Outliner owns a right-hand pane. Check the actual authored pane, then move this
        // exact instance through the existing floating/native callback owner.
        if self.editor_manager.detach_view_to_window(instance_id)? {
            self.invalidate_host(HostInvalidationMask::LAYOUT);
        }
        Ok(())
    }
}

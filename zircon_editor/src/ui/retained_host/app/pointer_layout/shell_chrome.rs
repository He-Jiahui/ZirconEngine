use super::super::*;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_activity_rail_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
    ) {
        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        self.activity_rail_pointer_bridge.sync(
            build_host_activity_rail_pointer_layout_with_workbench_layout_frames(
                model,
                &self.chrome_metrics,
                workbench_layout_frames,
            ),
        );
    }

    pub(in crate::ui::retained_host::app) fn sync_host_page_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
    ) {
        let outer_shell_frames = self.template_bridge.outer_shell_frames();
        self.host_page_pointer_bridge
            .sync(build_host_page_pointer_layout(
                model,
                &self.chrome_metrics,
                Some(&outer_shell_frames),
            ));
    }

    pub(in crate::ui::retained_host::app) fn sync_document_tab_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) {
        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        self.document_tab_pointer_bridge.sync(
            build_host_document_tab_pointer_layout_with_workbench_layout_frames(
                model,
                floating_window_projection_bundle,
                workbench_layout_frames,
            ),
        );
    }

    pub(in crate::ui::retained_host::app) fn sync_drawer_header_pointer_layout_with_workbench_layout_frames(
        &mut self,
        model: &WorkbenchViewModel,
        componentized_workbench_layout_frames: callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
    ) {
        self.drawer_header_pointer_bridge.sync(
            build_host_drawer_header_pointer_layout_with_workbench_layout_frames(
                model,
                &self.chrome_metrics,
                componentized_workbench_layout_frames,
            ),
        );
    }
}

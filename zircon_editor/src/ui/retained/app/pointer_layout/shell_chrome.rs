use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_activity_rail_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
    ) {
        self.sync_activity_rail_pointer_layout_for_target(model, false);
    }

    pub(in crate::ui::retained_host::app) fn sync_activity_rail_pointer_layout_for_target(
        &mut self,
        model: &WorkbenchViewModel,
        window_metrics_target: bool,
    ) {
        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        let current = self.activity_rail_pointer_bridge.layout();
        let next_layout = if window_metrics_target
            && (!current.left_tabs.is_empty() || !current.right_tabs.is_empty())
        {
            build_host_activity_rail_pointer_geometry_layout(
                current,
                &self.chrome_metrics,
                workbench_layout_frames,
            )
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.activity_rail.semantic_product_build_count",
                1
            );
            build_host_activity_rail_pointer_layout_with_workbench_layout_frames(
                model,
                &self.chrome_metrics,
                workbench_layout_frames,
            )
        };
        self.activity_rail_pointer_bridge.sync(next_layout);
    }

    pub(in crate::ui::retained_host::app) fn sync_host_page_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
    ) {
        self.host_page_pointer_bridge
            .sync(build_host_page_pointer_layout(model));
    }

    pub(in crate::ui::retained_host::app) fn sync_document_tab_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
    ) {
        self.document_tab_pointer_bridge
            .sync(build_host_document_tab_pointer_layout(model));
    }

    pub(in crate::ui::retained_host::app) fn sync_drawer_header_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
    ) {
        self.drawer_header_pointer_bridge
            .sync(build_host_drawer_header_pointer_layout(model));
    }
}

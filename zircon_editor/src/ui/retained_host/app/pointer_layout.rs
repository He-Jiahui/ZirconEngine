use super::*;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::menu_pointer::build_host_menu_pointer_layout;
use crate::ui::retained_host::{HostMenuStateData, PaneSurfaceHostContext, UiHostContext};

mod asset_surfaces;
mod detail_scrolls;
mod welcome_recent;

impl RetainedEditorHost {
    pub(super) fn sync_menu_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
        preset_names: &[String],
    ) {
        let outer_shell_frames = self.template_bridge.outer_shell_frames();
        self.menu_pointer_layout = build_host_menu_pointer_layout(
            &model.menu_bar,
            chrome,
            self.shell_size,
            preset_names,
            self.active_layout_preset.as_deref(),
            Some(&outer_shell_frames),
        );
        self.menu_pointer_bridge.sync(
            self.menu_pointer_layout.clone(),
            self.menu_pointer_state.clone(),
        );
        self.apply_menu_pointer_state_to_ui();
    }

    pub(super) fn apply_menu_pointer_state_to_ui(&self) {
        let host_shell = self.ui.global::<UiHostContext>();
        host_shell.set_menu_state(HostMenuStateData {
            open_menu_index: self
                .menu_pointer_state
                .open_menu_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            hovered_menu_index: self
                .menu_pointer_state
                .hovered_menu_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            hovered_menu_item_index: self
                .menu_pointer_state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            hovered_menu_item_path: self.menu_pointer_state.hovered_item_path.clone(),
            open_submenu_path: self.menu_pointer_state.open_submenu_path.clone(),
            menu_bar_scroll_px: self.menu_pointer_state.menu_bar_scroll_offset,
            window_menu_scroll_px: self.menu_pointer_state.popup_scroll_offset,
            window_menu_popup_height_px: self.menu_pointer_layout.window_popup_height,
        });
    }

    fn pane_surface_host(&self) -> PaneSurfaceHostContext<'_> {
        self.ui.global::<PaneSurfaceHostContext>()
    }

    pub(super) fn sync_activity_rail_pointer_layout(&mut self, model: &WorkbenchViewModel) {
        let workbench_layout_frames = self.workbench_window_bridge.layout_frames();
        self.activity_rail_pointer_bridge.sync(
            build_host_activity_rail_pointer_layout_with_workbench_layout_frames(
                model,
                &self.chrome_metrics,
                workbench_layout_frames,
            ),
        );
    }

    pub(super) fn sync_host_page_pointer_layout(&mut self, model: &WorkbenchViewModel) {
        let outer_shell_frames = self.template_bridge.outer_shell_frames();
        self.host_page_pointer_bridge
            .sync(build_host_page_pointer_layout(
                model,
                &self.chrome_metrics,
                Some(&outer_shell_frames),
            ));
    }

    pub(super) fn sync_document_tab_pointer_layout(
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

    pub(super) fn sync_drawer_header_pointer_layout_with_workbench_layout_frames(
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

    pub(super) fn sync_hierarchy_pointer_layout(
        &mut self,
        scene_entries: &[crate::ui::workbench::snapshot::SceneEntry],
    ) {
        if self.hierarchy_pointer_size.width <= 0.0 || self.hierarchy_pointer_size.height <= 0.0 {
            self.apply_hierarchy_pointer_state_to_ui();
            return;
        }

        self.hierarchy_pointer_bridge.sync(
            HierarchyPointerLayout {
                pane_width: self.hierarchy_pointer_size.width,
                pane_height: self.hierarchy_pointer_size.height,
                node_ids: scene_entries
                    .iter()
                    .map(|entry| entry.id.to_string())
                    .collect(),
            },
            self.hierarchy_pointer_state.clone(),
        );
        self.apply_hierarchy_pointer_state_to_ui();
    }

    pub(super) fn apply_hierarchy_pointer_state_to_ui(&self) {
        let pane_surface_host = self.pane_surface_host();
        pane_surface_host.set_hierarchy_scroll_px(self.hierarchy_pointer_state.scroll_offset);
        pane_surface_host.set_hovered_hierarchy_index(
            self.hierarchy_pointer_state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
    }
}

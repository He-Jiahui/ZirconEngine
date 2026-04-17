use super::*;
use crate::host::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::host::slint_host::menu_pointer::build_workbench_menu_pointer_layout;

impl SlintEditorHost {
    pub(super) fn sync_menu_pointer_layout(
        &mut self,
        chrome: &crate::EditorChromeSnapshot,
        preset_names: &[String],
    ) {
        let root_shell_frames = self.template_bridge.root_shell_frames();
        self.menu_pointer_layout = build_workbench_menu_pointer_layout(
            chrome,
            self.shell_size,
            preset_names,
            self.active_layout_preset.as_deref(),
            Some(&root_shell_frames),
        );
        self.menu_pointer_bridge.sync(
            self.menu_pointer_layout.clone(),
            self.menu_pointer_state.clone(),
        );
        self.apply_menu_pointer_state_to_ui();
    }

    pub(super) fn apply_menu_pointer_state_to_ui(&self) {
        self.ui
            .set_file_menu_button_frame(frame_rect(self.menu_pointer_layout.button_frames[0]));
        self.ui
            .set_edit_menu_button_frame(frame_rect(self.menu_pointer_layout.button_frames[1]));
        self.ui
            .set_selection_menu_button_frame(frame_rect(self.menu_pointer_layout.button_frames[2]));
        self.ui
            .set_view_menu_button_frame(frame_rect(self.menu_pointer_layout.button_frames[3]));
        self.ui
            .set_window_menu_button_frame(frame_rect(self.menu_pointer_layout.button_frames[4]));
        self.ui
            .set_help_menu_button_frame(frame_rect(self.menu_pointer_layout.button_frames[5]));
        self.ui.set_open_menu_index(
            self.menu_pointer_state
                .open_menu_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
        self.ui.set_hovered_menu_index(
            self.menu_pointer_state
                .hovered_menu_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
        self.ui.set_hovered_menu_item_index(
            self.menu_pointer_state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
        self.ui
            .set_window_menu_scroll_px(self.menu_pointer_state.popup_scroll_offset);
        self.ui
            .set_window_menu_popup_height_px(self.menu_pointer_layout.window_popup_height);
    }

    pub(super) fn sync_activity_rail_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
        geometry: &WorkbenchShellGeometry,
    ) {
        let root_shell_frames = self.template_bridge.root_shell_frames();
        self.activity_rail_pointer_bridge
            .sync(build_workbench_activity_rail_pointer_layout(
                model,
                geometry,
                &self.chrome_metrics,
                Some(&root_shell_frames),
            ));
    }

    pub(super) fn sync_host_page_pointer_layout(&mut self, model: &WorkbenchViewModel) {
        let root_shell_frames = self.template_bridge.root_shell_frames();
        self.host_page_pointer_bridge
            .sync(build_workbench_host_page_pointer_layout(
                model,
                &self.chrome_metrics,
                Some(&root_shell_frames),
            ));
    }

    pub(super) fn sync_document_tab_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
        geometry: &WorkbenchShellGeometry,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) {
        let root_shell_frames = self.template_bridge.root_shell_frames();
        self.document_tab_pointer_bridge
            .sync(build_workbench_document_tab_pointer_layout(
                model,
                geometry,
                &self.chrome_metrics,
                Some(&root_shell_frames),
                floating_window_projection_bundle,
            ));
    }

    pub(super) fn sync_drawer_header_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
        geometry: &WorkbenchShellGeometry,
    ) {
        let root_shell_frames = self.template_bridge.root_shell_frames();
        self.drawer_header_pointer_bridge
            .sync(build_workbench_drawer_header_pointer_layout(
                model,
                geometry,
                &self.chrome_metrics,
                Some(&root_shell_frames),
            ));
    }

    pub(super) fn sync_welcome_recent_pointer_layout(
        &mut self,
        welcome: &crate::WelcomePaneSnapshot,
    ) {
        let pane_size = self
            .resolve_welcome_recent_pointer_size()
            .unwrap_or(self.welcome_recent_pointer_size);
        if pane_size.width <= 0.0 || pane_size.height <= 0.0 {
            self.apply_welcome_recent_pointer_state_to_ui();
            return;
        }
        self.welcome_recent_pointer_size = pane_size;

        self.welcome_recent_pointer_bridge.sync(
            WelcomeRecentPointerLayout {
                pane_size,
                recent_project_paths: welcome
                    .recent_projects
                    .iter()
                    .map(|recent| recent.path.clone())
                    .collect(),
            },
            self.welcome_recent_pointer_state.clone(),
        );
        self.apply_welcome_recent_pointer_state_to_ui();
    }

    fn resolve_welcome_recent_pointer_size(&self) -> Option<UiSize> {
        if self.welcome_recent_pointer_size.width > 0.0
            && self.welcome_recent_pointer_size.height > 0.0
        {
            return Some(self.welcome_recent_pointer_size);
        }

        self.template_bridge
            .control_frame(callback_dispatch::PANE_SURFACE_CONTROL_ID)
            .map(|frame| UiSize::new(frame.width.max(0.0), frame.height.max(0.0)))
            .filter(|size| size.width > 0.0 && size.height > 0.0)
    }

    pub(super) fn apply_welcome_recent_pointer_state_to_ui(&self) {
        self.ui
            .set_welcome_recent_scroll_px(self.welcome_recent_pointer_state.scroll_offset);
        self.ui.set_hovered_welcome_recent_index(
            self.welcome_recent_pointer_state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
        self.ui.set_hovered_welcome_recent_action(
            match self.welcome_recent_pointer_state.hovered_action {
                Some(WelcomeRecentPointerAction::Open) => 0,
                Some(WelcomeRecentPointerAction::Remove) => 1,
                None => -1,
            },
        );
    }

    pub(super) fn sync_hierarchy_pointer_layout(&mut self, scene_entries: &[crate::SceneEntry]) {
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
        self.ui
            .set_hierarchy_scroll_px(self.hierarchy_pointer_state.scroll_offset);
        self.ui.set_hovered_hierarchy_index(
            self.hierarchy_pointer_state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
    }

    pub(super) fn sync_detail_pointer_layouts(&mut self, chrome: &crate::EditorChromeSnapshot) {
        self.sync_console_pointer_layout(chrome);
        self.sync_inspector_pointer_layout();
        self.sync_browser_asset_details_pointer_layout(&chrome.asset_browser);
    }

    pub(super) fn sync_console_pointer_layout(&mut self, chrome: &crate::EditorChromeSnapshot) {
        if !self.console_scroll_surface.has_size() {
            self.apply_console_pointer_state_to_ui();
            return;
        }

        let size = self.console_scroll_surface.size();
        self.console_scroll_surface.sync(console_scroll_layout(
            size,
            console_content_extent(chrome.status_line.as_str(), size.width, false, ""),
        ));
        self.apply_console_pointer_state_to_ui();
    }

    pub(super) fn apply_console_pointer_state_to_ui(&self) {
        self.ui
            .set_console_scroll_px(self.console_scroll_surface.scroll_offset());
    }

    pub(super) fn sync_inspector_pointer_layout(&mut self) {
        if !self.inspector_scroll_surface.has_size() {
            self.apply_inspector_pointer_state_to_ui();
            return;
        }

        self.inspector_scroll_surface.sync(inspector_scroll_layout(
            self.inspector_scroll_surface.size(),
        ));
        self.apply_inspector_pointer_state_to_ui();
    }

    pub(super) fn apply_inspector_pointer_state_to_ui(&self) {
        self.ui
            .set_inspector_scroll_px(self.inspector_scroll_surface.scroll_offset());
    }

    pub(super) fn sync_browser_asset_details_pointer_layout(
        &mut self,
        snapshot: &crate::workbench::snapshot::AssetWorkspaceSnapshot,
    ) {
        if !self.browser_asset_details_scroll_surface.has_size() {
            self.apply_browser_asset_details_pointer_state_to_ui();
            return;
        }

        self.browser_asset_details_scroll_surface
            .sync(asset_details_scroll_layout(
                self.browser_asset_details_scroll_surface.size(),
                &snapshot.selection,
            ));
        self.apply_browser_asset_details_pointer_state_to_ui();
    }

    pub(super) fn apply_browser_asset_details_pointer_state_to_ui(&self) {
        self.ui.set_browser_asset_details_scroll_px(
            self.browser_asset_details_scroll_surface.scroll_offset(),
        );
    }

    pub(super) fn sync_asset_pointer_layouts(&mut self, chrome: &crate::EditorChromeSnapshot) {
        self.sync_asset_pointer_layout("activity", &chrome.asset_activity);
        self.sync_asset_pointer_layout("browser", &chrome.asset_browser);
    }

    pub(super) fn sync_asset_pointer_layout(
        &mut self,
        surface_mode: &str,
        snapshot: &crate::workbench::snapshot::AssetWorkspaceSnapshot,
    ) {
        let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return;
        };

        surface.tree_bridge.sync(
            AssetFolderTreePointerLayout::from_snapshot(snapshot, surface.tree_size),
            surface.tree_state.clone(),
        );
        surface.content_bridge.sync(
            AssetContentListPointerLayout::from_snapshot(snapshot, surface.content_size),
            surface.content_state.clone(),
        );
        surface.references.bridge.sync(
            AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.references,
                surface.references.size,
            ),
            surface.references.state.clone(),
        );
        surface.used_by.bridge.sync(
            AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.used_by,
                surface.used_by.size,
            ),
            surface.used_by.state.clone(),
        );
        self.apply_asset_pointer_state_to_ui(surface_mode);
    }

    pub(super) fn apply_asset_pointer_state_to_ui(&self, surface_mode: &str) {
        let Some(surface) = self.asset_surface_pointer_state(surface_mode) else {
            return;
        };

        let tree_hovered = surface
            .tree_state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);
        let content_hovered = surface
            .content_state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);
        let references_hovered = surface
            .references
            .state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);
        let used_by_hovered = surface
            .used_by
            .state
            .hovered_row_index
            .map(|index| index as i32)
            .unwrap_or(-1);

        match surface_mode {
            "activity" => {
                self.ui.set_activity_asset_tree_hovered_index(tree_hovered);
                self.ui
                    .set_activity_asset_tree_scroll_px(surface.tree_state.scroll_offset);
                self.ui
                    .set_activity_asset_content_hovered_index(content_hovered);
                self.ui
                    .set_activity_asset_content_scroll_px(surface.content_state.scroll_offset);
                self.ui
                    .set_activity_asset_references_hovered_index(references_hovered);
                self.ui.set_activity_asset_references_scroll_px(
                    surface.references.state.scroll_offset,
                );
                self.ui
                    .set_activity_asset_used_by_hovered_index(used_by_hovered);
                self.ui
                    .set_activity_asset_used_by_scroll_px(surface.used_by.state.scroll_offset);
            }
            "browser" => {
                self.ui.set_browser_asset_tree_hovered_index(tree_hovered);
                self.ui
                    .set_browser_asset_tree_scroll_px(surface.tree_state.scroll_offset);
                self.ui
                    .set_browser_asset_content_hovered_index(content_hovered);
                self.ui
                    .set_browser_asset_content_scroll_px(surface.content_state.scroll_offset);
                self.ui
                    .set_browser_asset_references_hovered_index(references_hovered);
                self.ui
                    .set_browser_asset_references_scroll_px(surface.references.state.scroll_offset);
                self.ui
                    .set_browser_asset_used_by_hovered_index(used_by_hovered);
                self.ui
                    .set_browser_asset_used_by_scroll_px(surface.used_by.state.scroll_offset);
            }
            _ => {}
        }
    }

    pub(super) fn asset_surface_pointer_state(
        &self,
        surface_mode: &str,
    ) -> Option<&AssetSurfacePointerState> {
        match surface_mode {
            "activity" => Some(&self.activity_asset_pointer),
            "browser" => Some(&self.browser_asset_pointer),
            _ => None,
        }
    }

    pub(super) fn asset_surface_pointer_state_mut(
        &mut self,
        surface_mode: &str,
    ) -> Option<&mut AssetSurfacePointerState> {
        match surface_mode {
            "activity" => Some(&mut self.activity_asset_pointer),
            "browser" => Some(&mut self.browser_asset_pointer),
            _ => None,
        }
    }

    pub(super) fn asset_workspace_snapshot_for_pointer(
        &self,
        surface_mode: &str,
    ) -> Option<crate::workbench::snapshot::AssetWorkspaceSnapshot> {
        let snapshot = self.runtime.editor_snapshot();
        match surface_mode {
            "activity" => Some(snapshot.asset_activity),
            "browser" => Some(snapshot.asset_browser),
            _ => None,
        }
    }

    pub(super) fn asset_reference_layout(
        snapshot: &crate::workbench::snapshot::AssetWorkspaceSnapshot,
        list_kind: &str,
        pane_size: UiSize,
    ) -> Option<AssetReferenceListPointerLayout> {
        match list_kind {
            "references" => Some(AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.references,
                pane_size,
            )),
            "used_by" => Some(AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.used_by,
                pane_size,
            )),
            _ => None,
        }
    }
}

fn frame_rect(frame: UiFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

use super::super::*;
use crate::ui::retained_host::menu_pointer::build_host_menu_pointer_layout;
use crate::ui::retained_host::{HostMenuStateData, UiHostContext};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_menu_pointer_layout(
        &mut self,
        model: &WorkbenchViewModel,
        chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
        preset_names: &[String],
    ) {
        let outer_shell_frames = self.template_bridge.outer_shell_frames();
        let next_layout = build_host_menu_pointer_layout(
            &model.menu_bar,
            chrome,
            self.shell_size,
            preset_names,
            self.active_layout_preset.as_deref(),
            Some(&outer_shell_frames),
        );
        if self.menu_pointer_layout.as_ref() != &next_layout {
            self.menu_pointer_layout = Arc::new(next_layout);
        }
        let pointer_changed = self.menu_pointer_bridge.sync_shared(
            Arc::clone(&self.menu_pointer_layout),
            &self.menu_pointer_state,
        );
        if pointer_changed {
            self.apply_menu_pointer_state_to_ui();
        }
    }

    pub(in crate::ui::retained_host::app) fn apply_menu_pointer_state_to_ui(&self) {
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
}

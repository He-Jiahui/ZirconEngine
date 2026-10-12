use std::sync::Arc;

use crate::ui::workbench::window_registry::MenuOverflowMode;
use zircon_runtime_interface::ui::layout::UiFrame;

use super::menu_item_spec::MenuItemSpec;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostMenuPointerLayout {
    pub shell_frame: UiFrame,
    pub menu_bar_frame: UiFrame,
    pub button_frames: Vec<UiFrame>,
    pub menu_bar_content_width: f32,
    pub popup_widths: Arc<[f32]>,
    pub intrinsic_popup_widths: Arc<[f32]>,
    pub save_project_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub delete_enabled: bool,
    pub preset_names: Arc<[String]>,
    pub active_preset_name: Arc<str>,
    pub resolved_preset_name: Arc<str>,
    pub window_popup_height: f32,
    pub menu_overflow_mode: MenuOverflowMode,
    pub menus: Arc<[Vec<MenuItemSpec>]>,
}

impl Default for HostMenuPointerLayout {
    fn default() -> Self {
        Self {
            shell_frame: UiFrame::default(),
            menu_bar_frame: UiFrame::default(),
            button_frames: Vec::new(),
            menu_bar_content_width: 0.0,
            popup_widths: Vec::new().into(),
            intrinsic_popup_widths: Vec::new().into(),
            save_project_enabled: false,
            undo_enabled: false,
            redo_enabled: false,
            delete_enabled: false,
            preset_names: Vec::new().into(),
            active_preset_name: Arc::from(""),
            resolved_preset_name: Arc::from("rider"),
            window_popup_height: 72.0,
            menu_overflow_mode: MenuOverflowMode::Auto,
            menus: Vec::new().into(),
        }
    }
}

impl HostMenuPointerLayout {
    pub(in crate::ui::retained_host::menu_pointer) fn popup_semantics_equal(
        &self,
        other: &Self,
    ) -> bool {
        self.save_project_enabled == other.save_project_enabled
            && self.undo_enabled == other.undo_enabled
            && self.redo_enabled == other.redo_enabled
            && self.delete_enabled == other.delete_enabled
            && shared_or_equal(&self.preset_names, &other.preset_names)
            && shared_or_equal(&self.active_preset_name, &other.active_preset_name)
            && shared_or_equal(&self.resolved_preset_name, &other.resolved_preset_name)
            && shared_or_equal(&self.menus, &other.menus)
    }
}

fn shared_or_equal<T: ?Sized + PartialEq>(left: &Arc<T>, right: &Arc<T>) -> bool {
    Arc::ptr_eq(left, right) || left == right
}

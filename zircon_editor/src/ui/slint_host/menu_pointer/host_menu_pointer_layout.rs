use zircon_runtime_interface::ui::layout::UiFrame;

use super::menu_item_spec::MenuItemSpec;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostMenuPointerLayout {
    pub shell_frame: UiFrame,
    pub button_frames: [UiFrame; 7],
    pub save_project_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub delete_enabled: bool,
    pub preset_names: Vec<String>,
    pub active_preset_name: String,
    pub resolved_preset_name: String,
    pub window_popup_height: f32,
    pub menus: Vec<Vec<MenuItemSpec>>,
}

impl Default for HostMenuPointerLayout {
    fn default() -> Self {
        Self {
            shell_frame: UiFrame::default(),
            button_frames: [UiFrame::default(); 7],
            save_project_enabled: false,
            undo_enabled: false,
            redo_enabled: false,
            delete_enabled: false,
            preset_names: Vec::new(),
            active_preset_name: String::new(),
            resolved_preset_name: "rider".to_string(),
            window_popup_height: 72.0,
            menus: Vec::new(),
        }
    }
}

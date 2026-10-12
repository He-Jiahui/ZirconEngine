/// Menu indices use -1 as the closed/no-hover sentinel so a fresh host never paints a popup.
#[derive(Clone, PartialEq)]
pub(crate) struct HostMenuStateData {
    pub open_menu_index: i32,
    pub hovered_menu_index: i32,
    pub hovered_menu_item_index: i32,
    pub hovered_menu_item_path: Vec<usize>,
    pub open_submenu_path: Vec<usize>,
    pub menu_bar_scroll_px: f32,
    pub window_menu_scroll_px: f32,
    pub window_menu_popup_height_px: f32,
}

impl Default for HostMenuStateData {
    fn default() -> Self {
        Self {
            open_menu_index: -1,
            hovered_menu_index: -1,
            hovered_menu_item_index: -1,
            hovered_menu_item_path: Vec::new(),
            open_submenu_path: Vec::new(),
            menu_bar_scroll_px: 0.0,
            window_menu_scroll_px: 0.0,
            window_menu_popup_height_px: 0.0,
        }
    }
}

use slint::SharedString;

#[derive(Clone, Default)]
pub(crate) struct HostMenuStateData {
    pub open_menu_index: i32,
    pub hovered_menu_index: i32,
    pub hovered_menu_item_index: i32,
    pub window_menu_scroll_px: f32,
    pub window_menu_popup_height_px: f32,
}

#[derive(Clone, Default)]
pub(crate) struct HostDragStateData {
    pub active_drag_target_group: SharedString,
    pub drag_active: bool,
    pub drag_tab_id: SharedString,
    pub drag_tab_title: SharedString,
    pub drag_tab_icon_key: SharedString,
    pub drag_source_group: SharedString,
    pub drag_pointer_x: f32,
    pub drag_pointer_y: f32,
}

#[derive(Clone, Default)]
pub(crate) struct HostResizeStateData {
    pub resize_active: bool,
    pub resize_group: SharedString,
    pub resize_pointer_x: f32,
    pub resize_pointer_y: f32,
}

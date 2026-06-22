use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default, PartialEq)]
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

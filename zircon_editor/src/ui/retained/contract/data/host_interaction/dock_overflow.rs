use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, PartialEq)]
pub(crate) struct HostDockOverflowMenuStateData {
    pub open: bool,
    pub surface_key: SharedString,
    pub hovered_tab_index: i32,
    pub scroll_offset: f32,
}

impl Default for HostDockOverflowMenuStateData {
    fn default() -> Self {
        Self {
            open: false,
            surface_key: SharedString::default(),
            hovered_tab_index: -1,
            scroll_offset: 0.0,
        }
    }
}

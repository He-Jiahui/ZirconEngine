use super::super::super::super::super::paint_theme::current_host_palette;

pub(super) struct PopupSeparatorStyle {
    pub fill: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn popup_separator_style() -> PopupSeparatorStyle {
    PopupSeparatorStyle {
        fill: current_host_palette().separator_soft,
        border: None,
        border_width: 0.0,
        radius: 0.0,
    }
}

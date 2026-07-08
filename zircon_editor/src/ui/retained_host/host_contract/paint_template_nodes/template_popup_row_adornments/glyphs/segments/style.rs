pub(super) struct PopupAdornmentSegmentStyle {
    pub fill: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn popup_adornment_segment_style(fill: [u8; 4]) -> PopupAdornmentSegmentStyle {
    PopupAdornmentSegmentStyle {
        fill,
        border: None,
        border_width: 0.0,
        radius: 1.0,
    }
}

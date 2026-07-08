use super::super::super::super::super::paint_theme::current_host_palette;

pub(super) struct PopupAdornmentAssetStyle {
    pub fill: [u8; 4],
    pub cutout_fill: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub body_radius: f32,
    pub tab_radius: f32,
    pub cutout_radius: f32,
}

pub(super) fn popup_adornment_asset_style(fill: [u8; 4]) -> PopupAdornmentAssetStyle {
    PopupAdornmentAssetStyle {
        fill,
        cutout_fill: current_host_palette().popup,
        border: None,
        border_width: 0.0,
        body_radius: 1.5,
        tab_radius: 1.0,
        cutout_radius: 0.5,
    }
}

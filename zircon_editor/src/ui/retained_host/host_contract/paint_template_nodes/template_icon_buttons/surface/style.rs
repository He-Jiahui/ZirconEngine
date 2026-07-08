use super::super::super::style_selector::WorkbenchIconButtonStyle;

pub(super) struct IconButtonSurfaceCommandStyle {
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn icon_button_surface_command_style(
    style: WorkbenchIconButtonStyle,
) -> Option<IconButtonSurfaceCommandStyle> {
    let has_border = style.border.is_some() && style.border_width > 0.0;
    if style.background.is_none() && !has_border {
        return None;
    }
    Some(IconButtonSurfaceCommandStyle {
        background: style.background,
        border: style.border,
        border_width: style.border_width,
        radius: style.radius,
    })
}

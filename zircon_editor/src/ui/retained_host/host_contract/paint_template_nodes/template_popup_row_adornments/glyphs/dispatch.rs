use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::geometry::popup_row_adornment_rect;
use super::super::selection::PopupRowAdornmentKind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_adornment(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: PopupRowAdornmentKind<'_>,
    color: [u8; 4],
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    let Some(rect) = popup_row_adornment_rect(row_rect, clip) else {
        return;
    };
    let icon_name = match kind {
        PopupRowAdornmentKind::Check => "checkmark",
        PopupRowAdornmentKind::Chevron => "chevron-right",
        PopupRowAdornmentKind::Icon(icon_name) => icon_name,
    };
    push_icon_asset_pixels(
        commands,
        icon_name,
        &rect,
        clip,
        order,
        Some(color),
        opacity,
    );
}

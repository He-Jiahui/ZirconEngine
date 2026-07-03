use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::metrics::POPUP_ROW_ORDER_OFFSET;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchPopupRowStyle,
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    if let Some(background) = style.background {
        commands.push(HostPaintCommand::quad(
            row_rect.clone(),
            Some(clip.clone()),
            order + POPUP_ROW_ORDER_OFFSET + 1,
            Some(background),
            style.outline,
            if style.outline.is_some() { 1.0 } else { 0.0 },
            3.0,
            opacity,
        ));
    }
}

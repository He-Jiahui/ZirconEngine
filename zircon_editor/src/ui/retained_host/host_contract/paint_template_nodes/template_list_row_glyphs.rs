use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod geometry;
mod segments;
mod selection;
mod shapes;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::{
    list_row_adornment_kind, ListRowAdornmentKind,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_adornment(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let adornment = geometry::list_row_adornment_rect(rect);
    match list_row_adornment_kind(node) {
        ListRowAdornmentKind::Check => {
            shapes::push_check_mark(commands, &adornment, clip, order, color, opacity);
        }
        ListRowAdornmentKind::Chevron => {
            shapes::push_right_chevron(commands, &adornment, clip, order, color, opacity);
        }
        ListRowAdornmentKind::DisabledDiamond => {
            shapes::push_disabled_diamond(commands, &adornment, clip, order, opacity);
        }
    }
}

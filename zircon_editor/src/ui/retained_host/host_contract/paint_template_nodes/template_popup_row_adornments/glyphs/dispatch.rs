use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::popup_row_adornment_rect;
use super::super::selection::PopupRowAdornmentKind;
use super::{
    assets::{push_folder_adornment, push_save_adornment},
    symbols::{
        push_check_adornment, push_chevron_adornment, push_plus_adornment, push_trash_adornment,
    },
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_adornment(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: PopupRowAdornmentKind,
    color: [u8; 4],
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    let rect = popup_row_adornment_rect(row_rect);
    match kind {
        PopupRowAdornmentKind::Check => {
            push_check_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Chevron => {
            push_chevron_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Plus => {
            push_plus_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Folder => {
            push_folder_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Save => {
            push_save_adornment(commands, &rect, clip, order, color, opacity);
        }
        PopupRowAdornmentKind::Trash => {
            push_trash_adornment(commands, &rect, clip, order, color, opacity);
        }
    }
}

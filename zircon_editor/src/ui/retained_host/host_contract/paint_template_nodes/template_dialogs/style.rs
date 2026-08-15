mod colors;
mod palette;
mod severity;
mod state;
mod variants;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use colors::{
    cancel_action_paint, confirm_action_paint, dialog_action_paint, dialog_body_color,
    dialog_border_color, dialog_surface_color, dialog_title_color, DialogActionPaint,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use palette::dialog_palette_from_host;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use severity::severity_mark_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use state::{
    confirm_enabled, dialog_unavailable,
};

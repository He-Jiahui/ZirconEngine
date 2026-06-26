mod geometry;
mod icon_glyphs;
mod segments;
mod signals;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::{
    centered_rect, STATUS_ICON_GLYPH_SIZE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icon_glyphs::{
    push_status_icon_glyph, StatusIconKind,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::push_status_signal_icon;

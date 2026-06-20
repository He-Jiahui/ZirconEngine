mod geometry;
mod icon_glyphs;
mod segments;
mod signals;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::warning_mark_segments;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::{
    centered_rect, normalized_status_mark_width, STATUS_ICON_GLYPH_SIZE, STATUS_ITEM_ICON_SIZE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icon_glyphs::{
    push_status_icon_glyph, StatusIconKind,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use segments::push_down_chevron;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::push_status_signal_icon;

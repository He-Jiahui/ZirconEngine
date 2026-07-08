mod entry;
mod glyph;
mod layout;
mod metrics;
mod style;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_button_content;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use glyph::button_glyph;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_buttons) use metrics::{
    button_content_metrics_from_host, button_label_paint_style_with_preferences,
};

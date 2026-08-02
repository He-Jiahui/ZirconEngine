mod identity;
mod metrics;
mod segments;
mod shapes;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    ButtonGlyph, button_glyph_for_key,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::button_icon_size;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::button_icon_size_from_host;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shapes::push_button_glyph;

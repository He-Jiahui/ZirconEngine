mod identity;
mod metrics;
mod shapes;
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    section_title_icon, SectionTitleIcon,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    section_title_glyph_metrics, section_title_glyph_metrics_from_host,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shapes::push_section_icon;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use style::{
    section_icon_color, section_title_glyph_palette_from_host,
};

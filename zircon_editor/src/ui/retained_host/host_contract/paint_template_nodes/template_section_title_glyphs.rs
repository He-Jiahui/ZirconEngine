mod identity;
mod metrics;
mod segments;
mod shapes;
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    section_title_icon, SectionTitleIcon,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    SECTION_ICON_GAP, SECTION_ICON_SIZE,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shapes::push_section_icon;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use style::{
    section_icon_color, SECTION_TRANSFORM_GLYPH,
};

mod chevron;
mod identity;
mod metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chevron::{
    chip_can_paint_chevron, push_chip_chevron,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::chip_has_chevron;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::chip_glyph_chevron_reserve;

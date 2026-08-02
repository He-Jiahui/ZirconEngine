mod commands;
mod geometry;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::{
    push_ripple_commands, ripple_is_visible,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::{
    RIPPLE_DIAMETER_EXPANSION, ripple_diameter, ripple_rect,
};

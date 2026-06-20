mod colors;
mod overlay;
mod shape;
mod state;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use colors::{
    paper_background_color, paper_border_color, paper_border_width,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use overlay::paper_dark_overlay;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use shape::paper_corner_radius;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use state::{
    paper_elevation, paper_is_outlined,
};

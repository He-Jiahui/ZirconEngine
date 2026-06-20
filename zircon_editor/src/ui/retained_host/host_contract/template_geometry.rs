mod bounds;
mod frame;

pub(in crate::ui::retained_host::host_contract) use bounds::{
    template_nodes_bounds, template_popup_bounds,
};
pub(in crate::ui::retained_host::host_contract) use frame::frame_from_template_node;

#[cfg(test)]
#[path = "template_geometry_tests.rs"]
mod tests;

mod surface_frame;
mod template_node;

pub(crate) use template_node::build_pane_template_surface_frame;
pub(in crate::ui::retained_host::host_contract) use template_node::{
    hit_test_pane_template_node, hit_test_workbench_window_template_node, TemplateNodePointerHit,
};

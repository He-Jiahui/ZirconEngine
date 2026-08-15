mod surface_frame;
mod template_node;

pub(crate) use template_node::{
    build_pane_template_surface_frame, rebuild_pane_template_hit_artifacts,
};
pub(in crate::ui::retained_host::host_contract) use template_node::{
    hit_test_pane_template_node, hit_test_workbench_window_template_node_with_index,
    HostPaneTemplateHitIndex, HostWorkbenchHitIndex, TemplateNodePointerHit,
};

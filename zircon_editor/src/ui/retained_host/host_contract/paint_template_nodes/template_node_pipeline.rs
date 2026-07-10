mod clip;
mod draw;
mod transform;

#[cfg(test)]
mod test_support;

pub(in crate::ui::retained_host::host_contract) use draw::{
    draw_template_nodes, draw_template_nodes_with_transform, has_template_nodes,
};
pub(in crate::ui::retained_host::host_contract) use transform::TemplateNodePaintTransform;

#[cfg(test)]
pub(crate) use test_support::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
};

#[cfg(test)]
#[path = "template_node_pipeline_tests/mod.rs"]
mod tests;

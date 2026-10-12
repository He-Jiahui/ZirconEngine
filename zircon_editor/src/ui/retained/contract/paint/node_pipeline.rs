#[path = "node_pipeline/clip.rs"]
mod clip;
#[path = "node_pipeline/draw.rs"]
mod draw;
#[path = "node_pipeline/hover.rs"]
mod hover;
#[path = "node_pipeline/press.rs"]
mod press;
#[path = "node_pipeline/transform.rs"]
mod transform;

#[path = "node_pipeline/test_support.rs"]
mod test_support;

pub(in crate::ui::retained_host::host_contract) use draw::{
    draw_template_nodes, draw_template_nodes_with_transform, has_template_nodes,
};
pub(in crate::ui::retained_host::host_contract) use transform::TemplateNodePaintTransform;

pub(crate) use test_support::paint_template_nodes_for_evidence_with_background;

#[cfg(test)]
pub(crate) use test_support::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
};

#[cfg(test)]
#[path = "node_pipeline_tests/tests/mod.rs"]
mod tests;

use super::*;
use crate::graphics::feature::RenderFeaturePassDescriptor;
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

#[test]
fn resource_plan_unions_explicit_cull_root_usage_without_inferring_from_name() {
    let descriptor = RenderFeatureDescriptor::new(
        "typed-cull-root",
        Vec::new(),
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "terminal-write",
                QueueLane::Graphics,
            )
            .write_present_external_texture_with_ops(
                "test.terminal-output",
                RenderGraphAttachmentOps::clear_store(),
            ),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "terminal-read",
                QueueLane::Graphics,
            )
            .read_external_texture("test.terminal-output"),
        ],
    );

    let plan =
        pipeline_graph_resources(&[descriptor]).expect("typed resource usage should aggregate");
    let terminal = plan
        .get("test.terminal-output")
        .expect("terminal external resource plan");

    assert!(terminal.usage.present);
    assert!(!terminal.usage.readback);
    assert!(!terminal.usage.persistent);
    assert_eq!(terminal.kind, RenderFeatureResourceKind::External);
    assert_eq!(
        terminal.external_binding,
        RenderGraphExternalResourceBinding::report_only_texture()
    );
}

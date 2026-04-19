use crate::render_graph::{QueueLane, RenderGraphBuilder, RenderGraphError};

#[test]
fn compile_rejects_cycles() {
    let mut builder = RenderGraphBuilder::new("frame");
    let a = builder.add_pass("a", QueueLane::Graphics);
    let b = builder.add_pass("b", QueueLane::Graphics);
    builder.add_dependency(a, b).unwrap();
    builder.add_dependency(b, a).unwrap();

    let error = builder.compile().unwrap_err();
    assert!(matches!(error, RenderGraphError::CycleDetected { .. }));
}

use crate::render_graph::{QueueLane, RenderGraphBuilder};

#[test]
fn compile_orders_passes_by_declared_dependencies() {
    let mut builder = RenderGraphBuilder::new("frame");
    let depth = builder.add_pass("depth-prepass", QueueLane::Graphics);
    let shadow = builder.add_pass("shadow", QueueLane::Graphics);
    let lighting = builder.add_pass("lighting", QueueLane::Graphics);
    builder.add_dependency(depth, lighting).unwrap();
    builder.add_dependency(shadow, lighting).unwrap();

    let graph = builder.compile().unwrap();
    let ordered = graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ordered, vec!["depth-prepass", "shadow", "lighting"]);
}

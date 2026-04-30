use crate::render_graph::{QueueLane, RenderGraphBuilder};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

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

#[test]
fn compile_preserves_declared_dependencies_on_compiled_passes() {
    let mut builder = RenderGraphBuilder::new("frame");
    let depth = builder.add_pass("depth-prepass", QueueLane::Graphics);
    let shadow = builder.add_pass("shadow", QueueLane::Graphics);
    let lighting = builder.add_pass("lighting", QueueLane::Graphics);
    builder.add_dependency(depth, lighting).unwrap();
    builder.add_dependency(shadow, lighting).unwrap();

    let graph = builder.compile().unwrap();
    let lighting_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "lighting")
        .unwrap();

    assert_eq!(lighting_pass.dependencies, vec![depth, shadow]);
}

#[test]
fn compile_exposes_inferred_resource_dependencies_on_compiled_passes() {
    let mut builder = RenderGraphBuilder::new("frame");
    let color = builder.create_transient_texture(TextureDesc::new(
        "scene-color",
        64,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();

    let graph = builder.compile().unwrap();
    let final_blit_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "final-blit")
        .unwrap();

    assert_eq!(final_blit_pass.dependencies, vec![opaque]);
    assert_eq!(graph.stats().total_dependency_count, 1);
}

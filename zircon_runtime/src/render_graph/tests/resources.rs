use crate::render_graph::{
    QueueLane, RenderGraphBuilder, RenderGraphError, RenderGraphResourceKind,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn graph_tracks_transient_lifetimes_and_resource_edges() {
    let mut builder = RenderGraphBuilder::new("frame");
    let color = builder.create_transient_texture(TextureDesc::new(
        "scene-color",
        128,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let depth = builder.create_transient_texture(TextureDesc::new(
        "scene-depth",
        128,
        64,
        TextureFormat::Depth32Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let backbuffer = builder.import_external_resource("backbuffer");

    let prepass = builder.add_pass("depth-prepass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);

    builder.write_texture(prepass, depth).unwrap();
    builder.read_texture(opaque, depth).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.write_external(final_blit, backbuffer).unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec!["depth-prepass", "opaque", "final-blit"]
    );
    assert_eq!(graph.queue_lane_count(QueueLane::Graphics), 3);

    let color_lifetime = graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "scene-color")
        .unwrap();
    assert_eq!(
        color_lifetime.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(color_lifetime.first_pass, 1);
    assert_eq!(color_lifetime.last_pass, 2);
}

#[test]
fn graph_rejects_transient_read_without_producer() {
    let mut builder = RenderGraphBuilder::new("frame");
    let buffer = builder.create_transient_buffer(BufferDesc::new(
        "visible-instances",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let pass = builder.add_pass("clustered-lighting", QueueLane::AsyncCompute);
    builder.read_buffer(pass, buffer).unwrap();

    let error = builder.compile().unwrap_err();
    assert!(matches!(
        error,
        RenderGraphError::ReadBeforeProducer { resource, pass }
            if resource == "visible-instances" && pass == "clustered-lighting"
    ));
}

#[test]
fn graph_rejects_write_after_write_without_dependency() {
    let mut builder = RenderGraphBuilder::new("frame");
    let color = builder.create_transient_texture(TextureDesc::new(
        "scene-color",
        128,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let a = builder.add_pass("opaque", QueueLane::Graphics);
    let b = builder.add_pass("debug-overdraw", QueueLane::Graphics);
    builder.write_texture(a, color).unwrap();
    builder.write_texture(b, color).unwrap();

    let error = builder.compile().unwrap_err();
    assert!(matches!(
        error,
        RenderGraphError::WriteAfterWriteMissingDependency { resource, .. }
            if resource == "scene-color"
    ));
}

#[test]
fn graph_culls_unused_resource_writer_but_keeps_external_output_chain() {
    let mut builder = RenderGraphBuilder::new("frame");
    let unused = builder.create_transient_texture(TextureDesc::new(
        "unused",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let color = builder.create_transient_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let backbuffer = builder.import_external_resource("backbuffer");

    let unused_pass = builder.add_pass("unused-pass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    builder.write_texture(unused_pass, unused).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.write_external(final_blit, backbuffer).unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| (pass.name.as_str(), pass.culled))
            .collect::<Vec<_>>(),
        vec![
            ("unused-pass", true),
            ("opaque", false),
            ("final-blit", false)
        ]
    );
}

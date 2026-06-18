use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBuilder, RenderGraphError, RenderGraphResource,
    RenderGraphResourceUsageFlags,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

fn color_desc(name: &str) -> TextureDesc {
    TextureDesc::new(
        name,
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    )
}

#[test]
fn render_graph_culls_passes_unreachable_from_present_root() {
    let mut builder = RenderGraphBuilder::new("present-root-culling");
    let unused = builder.create_texture(color_desc("unused"));
    let color = builder.create_texture(color_desc("scene-color"));
    let output = builder.import_external_resource("viewport-output");

    let unused_pass = builder.add_pass("unused-pass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    builder.write_texture(unused_pass, unused).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.write_external(final_blit, output).unwrap();

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
            ("final-blit", false),
        ]
    );
    assert_eq!(graph.stats().culled_pass_count, 1);
}

#[test]
fn render_graph_non_root_external_write_is_culled() {
    let mut builder = RenderGraphBuilder::new("explicit-external-root");
    let color = builder.create_texture(color_desc("scene-color"));
    let output = builder.import_external_resource("viewport-output");
    let debug_output = builder.import_external_resource_with_usage(
        "debug-output",
        RenderGraphResourceUsageFlags::default(),
    );

    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    let debug = builder.add_pass("debug-export", QueueLane::Graphics);
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.write_external(final_blit, output).unwrap();
    builder.write_external(debug, debug_output).unwrap();

    let graph = builder.compile().unwrap();
    assert!(
        graph
            .passes()
            .iter()
            .find(|pass| pass.name == "debug-export")
            .unwrap()
            .culled
    );
}

#[test]
fn render_graph_readback_marked_buffer_keeps_producer_alive() {
    let mut builder = RenderGraphBuilder::new("readback-root");
    let buffer = builder.create_buffer(BufferDesc::new(
        "occlusion-results",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));
    builder
        .mark_readback(RenderGraphResource::TransientBuffer(buffer))
        .unwrap();

    let write = builder.add_pass("write-readback", QueueLane::AsyncCompute);
    builder.write_buffer(write, buffer).unwrap();

    let graph = builder.compile().unwrap();
    assert!(!graph.passes()[0].culled);
    assert!(graph
        .resource_lifetime_by_name("occlusion-results")
        .is_some());
}

#[test]
fn render_graph_persistent_texture_keeps_producer_alive() {
    let mut builder = RenderGraphBuilder::new("persistent-root");
    let history = builder.create_texture(color_desc("history.current.scene-color"));
    builder.mark_persistent(history).unwrap();

    let write = builder.add_pass("taa-resolve", QueueLane::Graphics);
    builder.write_texture(write, history).unwrap();

    let graph = builder.compile().unwrap();
    let lifetime = graph
        .resource_lifetime_by_name("history.current.scene-color")
        .unwrap();
    assert!(!graph.passes()[0].culled);
    assert!(lifetime.usage.persistent);
}

#[test]
fn render_graph_side_effect_pass_survives_culling() {
    let mut builder = RenderGraphBuilder::new("side-effect-root");
    let upload = builder.add_pass("timestamp-query", QueueLane::Graphics);
    builder
        .set_pass_flags(
            upload,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(graph.stats().executable_pass_count, 1);
    assert!(!graph.passes()[0].culled);
}

#[test]
fn render_graph_missing_cull_root_is_compile_error() {
    let mut builder = RenderGraphBuilder::new("missing-root");
    let scratch = builder.create_texture(color_desc("scratch"));
    let pass = builder.add_pass("scratch-only", QueueLane::Graphics);
    builder.write_texture(pass, scratch).unwrap();

    let error = builder.compile().unwrap_err();
    assert!(matches!(
        error,
        RenderGraphError::MissingCullRoot { graph_name } if graph_name == "missing-root"
    ));
}

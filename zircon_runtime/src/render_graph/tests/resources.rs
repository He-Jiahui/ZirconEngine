use crate::render_graph::{
    QueueLane, RenderGraphBuilder, RenderGraphError, RenderGraphResourceAccessKind,
    RenderGraphResourceDesc, RenderGraphResourceKind,
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
    let stats = graph.stats();
    assert_eq!(stats.total_pass_count, 3);
    assert_eq!(stats.executable_pass_count, 3);
    assert_eq!(stats.culled_pass_count, 0);
    assert_eq!(stats.queue_lane_count(QueueLane::Graphics), 3);
    assert_eq!(stats.queue_lane_count(QueueLane::AsyncCompute), 0);
    assert_eq!(stats.resource_lifetime_count, 3);
    assert_eq!(stats.total_resource_access_count, 5);
    assert_eq!(stats.read_resource_access_count, 2);
    assert_eq!(stats.write_resource_access_count, 3);
    assert_eq!(stats.external_output_count, 1);

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
    assert!(matches!(
        &color_lifetime.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.width == 128
                && desc.height == 64
                && desc.format == TextureFormat::Rgba8UnormSrgb
    ));

    let opaque_resources = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque")
        .unwrap()
        .resources
        .iter()
        .map(|access| (access.name.as_str(), access.kind, access.access))
        .collect::<Vec<_>>();
    assert_eq!(
        opaque_resources,
        vec![
            (
                "scene-depth",
                RenderGraphResourceKind::TransientTexture,
                RenderGraphResourceAccessKind::Read,
            ),
            (
                "scene-color",
                RenderGraphResourceKind::TransientTexture,
                RenderGraphResourceAccessKind::Write,
            ),
        ]
    );
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
fn graph_resolves_resource_producers_after_manual_dependency_ordering() {
    let mut builder = RenderGraphBuilder::new("frame");
    let color = builder.create_transient_texture(TextureDesc::new(
        "scene-color",
        64,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));

    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.add_dependency(opaque, final_blit).unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec!["opaque", "final-blit"]
    );
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
    let stats = graph.stats();
    assert_eq!(stats.total_pass_count, 3);
    assert_eq!(stats.executable_pass_count, 2);
    assert_eq!(stats.culled_pass_count, 1);
    assert_eq!(stats.queue_lane_count(QueueLane::Graphics), 2);
    assert_eq!(stats.resource_lifetime_count, 2);
    assert_eq!(stats.total_resource_access_count, 4);
    assert_eq!(stats.external_output_count, 1);
}

#[test]
fn graph_culling_keeps_manual_dependencies_of_live_passes() {
    let mut builder = RenderGraphBuilder::new("frame");
    let setup_scratch =
        builder.create_transient_buffer(BufferDesc::new("setup-scratch", 16, BufferUsage::STORAGE));
    let output = builder.import_external_resource("viewport-output");

    let setup = builder.add_pass("manual-setup", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_buffer(setup, setup_scratch).unwrap();
    builder.write_external(present, output).unwrap();
    builder.add_dependency(setup, present).unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| (pass.name.as_str(), pass.culled))
            .collect::<Vec<_>>(),
        vec![("manual-setup", false), ("present", false)]
    );
}

#[test]
fn graph_builds_transient_aliasing_plan_for_non_overlapping_lifetimes() {
    let mut builder = RenderGraphBuilder::new("aliasing");
    let history = builder.create_transient_texture(TextureDesc::new(
        "history",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let lighting = builder.create_transient_texture(TextureDesc::new(
        "lighting",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let resolved = builder.create_transient_texture(TextureDesc::new(
        "resolved",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_external_resource("viewport-output");

    let write_history = builder.add_pass("write-history", QueueLane::Graphics);
    let light = builder.add_pass("lighting", QueueLane::Graphics);
    let resolve = builder.add_pass("resolve", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);

    builder.write_texture(write_history, history).unwrap();
    builder.read_texture(light, history).unwrap();
    builder.write_texture(light, lighting).unwrap();
    builder.read_texture(resolve, lighting).unwrap();
    builder.write_texture(resolve, resolved).unwrap();
    builder.read_texture(present, resolved).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder.compile().unwrap();
    let plan = graph.transient_allocation_plan();

    assert_eq!(plan.texture_slot_count, 2);
    assert_eq!(plan.buffer_slot_count, 0);
    assert_eq!(plan.slot_for("history"), Some(0));
    assert_eq!(plan.slot_for("lighting"), Some(1));
    assert_eq!(plan.slot_for("resolved"), Some(0));
    assert_eq!(plan.slot_for("viewport-output"), None);
}

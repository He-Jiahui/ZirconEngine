use super::RgResourceResolver;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, TransientResourcePool,
};
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBuilder, RenderGraphExternalResourceBinding,
    RenderGraphResource, RenderGraphResourceAccessKind,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn rg_resource_resolver_materialization_indices_follow_topologically_reordered_passes() {
    let mut builder = RenderGraphBuilder::new("resolver-compiled-indices");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    builder.write_texture(producer, color).unwrap();
    builder.read_texture(consumer, color).unwrap();
    builder.add_dependency(producer, consumer).unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(graph.passes()[0].id, producer);
    assert_eq!(graph.passes()[1].id, consumer);
    let color = graph
        .resource_declaration_by_name("scene-color")
        .expect("compiled graph retains scene-color")
        .resource;
    let resolver = RgResourceResolver::new(&graph, consumer);

    assert!(resolver.pass_declares_resource(color));
    assert!(resolver.pass_declares_resource_access(color, RenderGraphResourceAccessKind::Read));
    assert!(!resolver.pass_declares_resource_access(color, RenderGraphResourceAccessKind::Write));
    assert_eq!(resolver.pass_resources().len(), 1);
    assert_eq!(resolver.pass_resources()[0].name, "scene-color");
    assert_eq!(
        resolver
            .exact_transient_access_by_name("scene-color", RenderGraphResourceAccessKind::Read,)
            .unwrap(),
        graph.access_id_for(consumer, color, RenderGraphResourceAccessKind::Read)
    );
}

#[test]
fn rg_resource_resolver_requires_pass_declared_access_before_physical_texture_lookup() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("resolver-physical-texture");
    let depth = builder.create_texture(TextureDesc::new(
        "scene-depth",
        16,
        16,
        TextureFormat::Depth32Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_present_external_resource("viewport-output");
    let depth_prepass = builder.add_pass("depth-prepass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_texture(depth_prepass, depth).unwrap();
    builder.read_texture(opaque, depth).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(present, color).unwrap();
    builder.write_external(present, output).unwrap();
    let graph = builder.compile().unwrap();
    let opaque_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque")
        .unwrap();
    let mut resources = RenderGraphExecutionResources::new();
    let mut transient_pool = TransientResourcePool::default();
    transient_pool.begin_frame(backend.device_profile());
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            &graph,
            &mut transient_pool,
        )
        .unwrap();
    let resolver =
        RgResourceResolver::new(&graph, opaque_pass.id).with_physical_resources(&resources);

    resolver
        .texture_view_by_name("scene-depth", RenderGraphResourceAccessKind::Read)
        .expect("declared depth read resolves through physical table");
    let error = resolver
        .texture_view_by_name("scene-color", RenderGraphResourceAccessKind::Read)
        .unwrap_err();

    assert!(
        error.contains("did not declare Read access for resource `scene-color`"),
        "{error}"
    );
}

#[test]
fn rg_resource_resolver_uses_exact_transient_subresource_binding() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("resolver-exact-mip");
    let texture = builder.create_texture(
        TextureDesc::new(
            "mip-chain",
            32,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(4),
    );
    let producer = builder.add_pass("mip-producer", QueueLane::Graphics);
    let consumer = builder.add_pass("mip-consumer", QueueLane::Graphics);
    let producer_version = builder
        .write_texture_with_access_versioned(
            producer,
            texture,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1),
            crate::render_graph::RenderGraphResourceAccessIntent::ColorAttachment,
            Some(crate::render_graph::RenderGraphAttachmentOps::clear_store()),
        )
        .unwrap();
    builder
        .read_texture_with_access_from_version(
            consumer,
            producer_version,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1),
            crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
                crate::render_graph::RenderGraphShaderStages::FRAGMENT,
            ),
        )
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let consumer_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.id == consumer)
        .unwrap();
    let access_id = graph
        .access_id_for(
            consumer,
            graph
                .resource_declaration_by_name("mip-chain")
                .unwrap()
                .resource,
            RenderGraphResourceAccessKind::Read,
        )
        .unwrap();
    let mut resources = RenderGraphExecutionResources::new();
    let mut transient_pool = TransientResourcePool::default();
    transient_pool.begin_frame(backend.device_profile());
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            &graph,
            &mut transient_pool,
        )
        .unwrap();
    let resolver =
        RgResourceResolver::new(&graph, consumer_pass.id).with_physical_resources(&resources);

    resolver
        .texture_view_by_name("mip-chain", RenderGraphResourceAccessKind::Read)
        .expect("exact transient mip access should resolve");
    assert_eq!(
        resources.transient_access_key(access_id).unwrap().access_id,
        access_id
    );
    assert_eq!(
        resolver
            .exact_transient_access_by_name("mip-chain", RenderGraphResourceAccessKind::Read)
            .unwrap(),
        Some(access_id)
    );
}

#[test]
fn rg_resource_resolver_uses_exact_persistent_texture_alias_access_view() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("resolver-exact-persistent-mip");
    let texture = builder.create_texture(
        TextureDesc::new(
            "persistent-mip-chain",
            32,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(4),
    );
    builder.mark_persistent(texture).unwrap();
    let texture_alias = builder
        .create_texture_view_alias(
            "persistent-mip-view",
            texture,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .unwrap();
    let producer = builder.add_pass("persistent-mip-producer", QueueLane::Graphics);
    let consumer = builder.add_pass("persistent-mip-consumer", QueueLane::Graphics);
    let producer_version = builder
        .write_texture_with_access_versioned(
            producer,
            texture_alias,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(0),
            crate::render_graph::RenderGraphResourceAccessIntent::ColorAttachment,
            Some(crate::render_graph::RenderGraphAttachmentOps::clear_store()),
        )
        .unwrap();
    builder
        .read_texture_with_access_from_version(
            consumer,
            producer_version,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(0),
            crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
                crate::render_graph::RenderGraphShaderStages::FRAGMENT,
            ),
        )
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();
    let graph = builder.compile().unwrap();
    let access_id = graph
        .access_id_for(
            consumer,
            RenderGraphResource::TransientTexture(texture_alias),
            RenderGraphResourceAccessKind::Read,
        )
        .unwrap();
    assert_eq!(
        graph.persistent_texture_backing_resource(RenderGraphResource::TransientTexture(
            texture_alias
        )),
        Some(RenderGraphResource::TransientTexture(texture))
    );
    let mut resources = RenderGraphExecutionResources::new();
    let mut transient_pool = TransientResourcePool::default();
    transient_pool.begin_frame(backend.device_profile());
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            &graph,
            &mut transient_pool,
        )
        .unwrap();
    let resolver = RgResourceResolver::new(&graph, consumer).with_physical_resources(&resources);

    assert_eq!(
        resolver
            .exact_graph_owned_texture_access_by_name(
                "persistent-mip-view",
                RenderGraphResourceAccessKind::Read,
            )
            .unwrap(),
        Some(access_id)
    );
    resolver
        .texture_view_by_name("persistent-mip-view", RenderGraphResourceAccessKind::Read)
        .expect("persistent texture alias resolves through its exact access lease");
}

#[test]
fn rg_resource_resolver_keeps_unknown_external_on_legacy_path() {
    let mut builder = RenderGraphBuilder::new("resolver-external-access-kind");
    let unknown = builder.import_present_external_resource("legacy-output");
    let typed = builder.import_present_external_buffer_with_binding(
        "typed-output",
        BufferDesc::new("typed-output", 256, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::report_only_buffer(),
    );
    let pass = builder.add_pass("external-consumer", QueueLane::AsyncCompute);
    builder.write_external(pass, unknown).unwrap();
    builder.read_external(pass, typed).unwrap();
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let resolver = RgResourceResolver::new(&graph, pass);
    assert_eq!(
        resolver
            .exact_external_access_by_name("legacy-output", RenderGraphResourceAccessKind::Write)
            .unwrap(),
        None
    );
    assert!(resolver
        .exact_external_access_by_name("typed-output", RenderGraphResourceAccessKind::Read)
        .unwrap()
        .is_some());
}

#[test]
fn rg_resource_resolver_resolves_typed_external_buffer_through_access_lease() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("resolver-typed-external-buffer");
    let external = builder.import_present_external_buffer_with_binding(
        "typed-output",
        BufferDesc::new("typed-output", 256, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = builder.add_pass("external-consumer", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, external).unwrap();
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();
    let graph = builder.compile().unwrap();
    let native = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("resolver-typed-external-buffer"),
        size: 256,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.insert_buffer("typed-output", native.clone());
    resources
        .materialize_external_access_bindings(&graph)
        .unwrap();
    let resolver = RgResourceResolver::new(&graph, pass).with_physical_resources(&resources);
    let resolved = resolver
        .buffer_by_name("typed-output", RenderGraphResourceAccessKind::Write)
        .unwrap();
    assert_eq!(resolved.size(), native.size());
}

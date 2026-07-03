use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

use crate::core::framework::render::{
    ComputeDispatchBuilder, ComputeKernelRef, RenderShaderEntryPointDescriptor, RenderShaderStage,
    ShaderAssetKind, ShaderDispatchExtent, ShaderResourceAccess, ShaderResourceDescriptor,
    ShaderResourceKind,
};
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps,
    RenderGraphAttachmentStoreOp, RenderGraphBuilder, RenderGraphComputeDispatchExtent,
    RenderGraphComputeWorkload, RenderGraphDumpResourceDesc, RenderGraphError, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

mod transient_aliasing;

#[test]
fn graph_tracks_transient_lifetimes_and_resource_edges() {
    let mut builder = RenderGraphBuilder::new("frame");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        128,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let depth = builder.create_texture(TextureDesc::new(
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
    assert_eq!(graph.resource_declarations().len(), 3);
    let color_resource = RenderGraphResource::TransientTexture(color);
    assert_eq!(
        graph
            .resource_declarations()
            .iter()
            .map(|resource| (resource.name.as_str(), resource.resource, resource.imported))
            .collect::<Vec<_>>(),
        vec![
            (
                "scene-color",
                RenderGraphResource::TransientTexture(color),
                false,
            ),
            (
                "scene-depth",
                RenderGraphResource::TransientTexture(depth),
                false,
            ),
            (
                "backbuffer",
                RenderGraphResource::External(backbuffer),
                true,
            ),
        ]
    );
    assert_eq!(
        graph
            .resource_declaration(color_resource)
            .unwrap()
            .name
            .as_str(),
        "scene-color"
    );
    assert_eq!(
        graph
            .resource_declaration_by_name("scene-depth")
            .unwrap()
            .resource,
        RenderGraphResource::TransientTexture(depth)
    );

    let color_lifetime = graph.resource_lifetime(color_resource).unwrap();
    assert_eq!(color_lifetime.resource, color_resource);
    assert_eq!(
        graph
            .resource_lifetime_by_name("scene-color")
            .unwrap()
            .resource,
        color_resource
    );
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
fn graph_records_attachment_clear_load_store_ops() {
    let mut builder = RenderGraphBuilder::new("attachment-ops");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_external_resource("viewport-output");

    let clear = builder.add_pass("clear-scene", QueueLane::Graphics);
    let composite = builder.add_pass("composite", QueueLane::Graphics);
    builder
        .write_texture_with_ops(clear, color, RenderGraphAttachmentOps::clear_store())
        .unwrap();
    builder.read_texture(composite, color).unwrap();
    builder
        .write_external_with_ops(composite, output, RenderGraphAttachmentOps::load_store())
        .unwrap();

    let graph = builder.compile().unwrap();
    let clear_color = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "clear-scene")
        .unwrap()
        .resources
        .iter()
        .find(|resource| resource.name == "scene-color")
        .unwrap();
    assert_eq!(
        clear_color.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Clear,
            store: RenderGraphAttachmentStoreOp::Store,
        })
    );

    let output = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "composite")
        .unwrap()
        .resources
        .iter()
        .find(|resource| resource.name == "viewport-output")
        .unwrap();
    assert_eq!(
        output.attachment_ops,
        Some(RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: RenderGraphAttachmentStoreOp::Store,
        })
    );
}

#[test]
fn graph_records_storage_writes_without_attachment_ops() {
    let mut builder = RenderGraphBuilder::new("compute-storage");
    let storage_texture = builder.create_texture(TextureDesc::new(
        "ambient-occlusion",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    ));
    let storage_external = builder.import_external_resource("cluster-output");
    let output = builder.import_external_resource("viewport-output");

    let ssao = builder.add_pass("ssao-evaluate", QueueLane::AsyncCompute);
    let compose = builder.add_pass("compose", QueueLane::Graphics);
    let readback = builder.add_pass("cluster-readback", QueueLane::AsyncCompute);
    builder
        .write_storage_texture(ssao, storage_texture)
        .unwrap();
    builder.read_texture(compose, storage_texture).unwrap();
    builder.write_external(compose, output).unwrap();
    builder
        .write_storage_external(readback, storage_external)
        .unwrap();

    let graph = builder.compile().unwrap();
    let ssao_storage = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "ssao-evaluate")
        .unwrap()
        .resources
        .iter()
        .find(|resource| resource.name == "ambient-occlusion")
        .unwrap();
    assert_eq!(ssao_storage.access, RenderGraphResourceAccessKind::Write);
    assert_eq!(ssao_storage.attachment_ops, None);

    let external_storage = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "cluster-readback")
        .unwrap()
        .resources
        .iter()
        .find(|resource| resource.name == "cluster-output")
        .unwrap();
    assert_eq!(external_storage.kind, RenderGraphResourceKind::External);
    assert_eq!(
        external_storage.access,
        RenderGraphResourceAccessKind::Write
    );
    assert_eq!(external_storage.attachment_ops, None);
    assert_eq!(graph.queue_lane_count(QueueLane::AsyncCompute), 2);
}

#[test]
fn graph_preserves_sparse_texture_reservations_without_dense_transient_slot() {
    let mut builder = RenderGraphBuilder::new("sparse-texture-reservation");
    let virtual_pages = builder.create_texture(
        TextureDesc::new(
            "virtual-terrain-pages",
            8192,
            8192,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_DST,
        )
        .with_mip_levels(10)
        .with_sparse_residency(),
    );
    let output = builder.import_external_resource("viewport-output");

    let page_update = builder.add_pass("terrain-page-update", QueueLane::AsyncCompute);
    let sample = builder.add_pass("terrain-sample", QueueLane::Graphics);
    builder
        .write_storage_texture(page_update, virtual_pages)
        .unwrap();
    builder.read_texture(sample, virtual_pages).unwrap();
    builder.write_external(sample, output).unwrap();

    let graph = builder.compile().unwrap();
    let lifetime = graph
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "virtual-terrain-pages")
        .unwrap();

    assert!(lifetime.is_sparse_reserved_texture());
    assert!(matches!(
        &lifetime.desc,
        RenderGraphResourceDesc::Texture(desc) if desc.is_sparse_reserved()
    ));
    assert_eq!(graph.stats().sparse_texture_lifetime_count, 1);
    let allocation_plan = graph.transient_allocation_plan();
    assert_eq!(allocation_plan.texture_slot_count, 0);
    assert_eq!(allocation_plan.sparse_texture_slot_count, 1);
}

#[test]
fn graph_preserves_compute_workload_metadata() {
    let mut builder = RenderGraphBuilder::new("compute-workload");
    let light_list = builder.create_buffer(BufferDesc::new(
        "light-list",
        256,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));

    let clustered = builder.add_pass("light-grid-build", QueueLane::AsyncCompute);
    builder
        .mark_readback(RenderGraphResource::TransientBuffer(light_list))
        .unwrap();
    builder
        .set_compute_workload(
            clustered,
            RenderGraphComputeWorkload::cluster_grid("zircon-cluster-pipeline", [8, 8, 1]),
        )
        .unwrap();
    builder.write_buffer(clustered, light_list).unwrap();

    let graph = builder.compile().unwrap();
    let workload = graph.passes()[0].compute_workload.as_ref().unwrap();

    assert_eq!(workload.pipeline_label, "zircon-cluster-pipeline");
    assert_eq!(workload.workgroup_size, [8, 8, 1]);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::ClusterGrid
    );
}

#[test]
fn graph_accepts_shader_compute_dispatch_plan_as_workload() {
    let shader = AssetReference::from_locator(
        ResourceLocator::parse("builtin://shaders/compute/clustered_lighting").unwrap(),
    );
    let mut dispatch = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader, "cs_main"));
    dispatch
        .with_pipeline_label("zircon-cluster-pipeline")
        .with_workgroup_size([8, 8, 1])
        .bind_storage_write("light-list")
        .dispatch_extent(ShaderDispatchExtent::ClusterGrid);
    let dispatch = dispatch
        .build(
            ShaderAssetKind::Compute,
            &[RenderShaderEntryPointDescriptor {
                name: "cs_main".to_string(),
                stage: RenderShaderStage::Compute,
            }],
            &[ShaderResourceDescriptor {
                name: "light-list".to_string(),
                kind: ShaderResourceKind::StorageBuffer,
                access: Some(ShaderResourceAccess::Write),
            }],
        )
        .unwrap();

    let mut builder = RenderGraphBuilder::new("shader-compute-workload");
    let light_list = builder.create_buffer(BufferDesc::new(
        "light-list",
        256,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));
    let pass = builder.add_pass("light-grid-build", QueueLane::AsyncCompute);
    builder
        .mark_readback(RenderGraphResource::TransientBuffer(light_list))
        .unwrap();
    builder
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::from_shader_dispatch(&dispatch),
        )
        .unwrap();
    builder.write_buffer(pass, light_list).unwrap();

    let graph = builder.compile().unwrap();
    let workload = graph.passes()[0].compute_workload.as_ref().unwrap();

    assert_eq!(workload.pipeline_label, "zircon-cluster-pipeline");
    assert_eq!(workload.workgroup_size, [8, 8, 1]);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::ClusterGrid
    );
}

#[test]
fn graph_rejects_transient_attachment_load_without_producer() {
    let mut builder = RenderGraphBuilder::new("load-without-producer");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let pass = builder.add_pass("opaque", QueueLane::Graphics);
    builder
        .write_texture_with_ops(pass, color, RenderGraphAttachmentOps::load_store())
        .unwrap();

    let error = builder.compile().unwrap_err();

    assert!(matches!(
        error,
        RenderGraphError::LoadBeforeProducer { resource, pass }
            if resource == "scene-color" && pass == "opaque"
    ));
}

#[test]
fn graph_rejects_read_after_discarded_transient_attachment_store() {
    let mut builder = RenderGraphBuilder::new("discarded-store");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let discard = builder.add_pass("scratch-lighting", QueueLane::Graphics);
    let sample = builder.add_pass("sample-lighting", QueueLane::Graphics);
    builder
        .write_texture_with_ops(discard, color, RenderGraphAttachmentOps::clear_discard())
        .unwrap();
    builder.read_texture(sample, color).unwrap();

    let error = builder.compile().unwrap_err();

    assert!(matches!(
        error,
        RenderGraphError::ReadAfterDiscardedStore {
            resource,
            pass,
            producer,
        } if resource == "scene-color" && pass == "sample-lighting" && producer == "scratch-lighting"
    ));
}

#[test]
fn graph_rejects_transient_read_without_producer() {
    let mut builder = RenderGraphBuilder::new("frame");
    let buffer = builder.create_buffer(BufferDesc::new(
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
fn graph_rejects_duplicate_resource_names() {
    let mut builder = RenderGraphBuilder::new("frame");
    builder.create_texture(TextureDesc::new(
        "scene-color",
        128,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    builder.import_external_resource("scene-color");

    let error = builder.compile().unwrap_err();
    assert!(matches!(
        error,
        RenderGraphError::DuplicateResourceName { resource }
            if resource == "scene-color"
    ));
}

#[test]
fn graph_resolves_resource_producers_after_manual_dependency_ordering() {
    let mut builder = RenderGraphBuilder::new("frame");
    let color = builder.create_texture(TextureDesc::new(
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
    builder
        .set_pass_flags(
            final_blit,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();
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
    let color = builder.create_texture(TextureDesc::new(
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
    let unused = builder.create_texture(TextureDesc::new(
        "unused",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let color = builder.create_texture(TextureDesc::new(
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

    let unused_resource = RenderGraphResource::TransientTexture(unused);
    let live_color = RenderGraphResource::TransientTexture(color);
    assert_eq!(graph.resource_declarations().len(), 3);
    assert_eq!(
        graph
            .resource_declaration(unused_resource)
            .unwrap()
            .name
            .as_str(),
        "unused"
    );
    assert_eq!(
        graph
            .resource_declaration_by_name("unused")
            .unwrap()
            .resource,
        unused_resource
    );
    assert!(graph.resource_lifetime(unused_resource).is_none());
    assert!(graph.resource_lifetime_by_name("unused").is_none());
    assert_eq!(
        graph.resource_lifetime(live_color).unwrap().name.as_str(),
        "scene-color"
    );
}

#[test]
fn render_graph_dump_lists_pass_order_resources_and_culled() {
    let mut builder = RenderGraphBuilder::new("dump-contract");
    let unused = builder.create_texture(TextureDesc::new(
        "unused",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_external_resource("viewport-output");

    let unused_pass = builder.add_pass("unused-pass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_texture(unused_pass, unused).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(present, color).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder.compile().unwrap();
    let dump = graph.dump();

    assert_eq!(dump.graph_name, "dump-contract");
    assert_eq!(
        dump.pass_rows
            .iter()
            .map(|pass| (pass.order, pass.name.as_str(), pass.culled))
            .collect::<Vec<_>>(),
        vec![
            (0, "unused-pass", true),
            (1, "opaque", false),
            (2, "present", false)
        ]
    );
    let opaque_row = dump
        .pass_rows
        .iter()
        .find(|pass| pass.name == "opaque")
        .unwrap();
    assert_eq!(opaque_row.resources.len(), 1);
    assert_eq!(opaque_row.resources[0].name, "scene-color");
    assert_eq!(
        opaque_row.resources[0].access,
        RenderGraphResourceAccessKind::Write
    );

    let color_row = dump
        .resource_rows
        .iter()
        .find(|resource| resource.name == "scene-color")
        .unwrap();
    assert!(color_row.live);
    assert_eq!(color_row.first_pass, Some(1));
    assert_eq!(color_row.last_pass, Some(2));
    assert_eq!(color_row.transient_slot, Some(0));
    assert_eq!(color_row.size_bytes, Some(4096));
    assert!(matches!(
        color_row.desc,
        RenderGraphDumpResourceDesc::Texture {
            width: 32,
            height: 32,
            format: TextureFormat::Rgba8UnormSrgb,
            ..
        }
    ));

    let unused_row = dump
        .resource_rows
        .iter()
        .find(|resource| resource.name == "unused")
        .unwrap();
    assert!(!unused_row.live);
    assert_eq!(unused_row.transient_slot, None);

    let text = dump.to_text();
    assert!(text.contains("render_graph name=dump-contract"));
    assert!(text.contains("pass[0] id=0 name=unused-pass"));
    assert!(text.contains("culled=true"));
    assert!(text.contains("resource name=scene-color"));
    assert!(text.contains("lifetime=1..2"));
    let transient_slot_line = text
        .lines()
        .find(|line| line.contains("slot kind=TransientTexture index=0 "))
        .unwrap();
    assert!(transient_slot_line.contains("bucket="));
    assert!(transient_slot_line.contains("bytes_reserved=4096"));
}

#[test]
fn graph_culling_keeps_manual_dependencies_of_live_passes() {
    let mut builder = RenderGraphBuilder::new("frame");
    let setup_scratch =
        builder.create_buffer(BufferDesc::new("setup-scratch", 16, BufferUsage::STORAGE));
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

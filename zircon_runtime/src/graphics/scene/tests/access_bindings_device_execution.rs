use super::*;
use crate::graphics::backend::RenderBackend;
use crate::graphics::pipeline::{RenderGraphExecutionPass, RenderPassStage};
use crate::graphics::scene::scene_renderer::graph_execution::TransientResourcePool;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBufferRange, RenderGraphBuilder,
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessKind, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

fn explicit_storage_buffer_graph() -> (CompiledRenderGraph, crate::render_graph::RenderPassId) {
    let mut builder = RenderGraphBuilder::new("device-execution-buffer-lease");
    let buffer = builder.create_buffer(BufferDesc::new(
        "storage-buffer",
        128,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let pass = builder.add_pass("storage-writer", QueueLane::AsyncCompute);
    builder
        .access_buffer(
            pass,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(32, Some(64)),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .expect("explicit storage access should compile");
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("storage writer must remain live");
    (
        builder.compile().expect("storage graph should compile"),
        pass,
    )
}

fn cross_queue_storage_buffer_graph() -> (
    CompiledRenderGraph,
    crate::render_graph::RenderPassId,
    crate::render_graph::RenderPassId,
) {
    let mut builder = RenderGraphBuilder::new("device-execution-cross-queue-buffer");
    let buffer = builder.create_buffer(BufferDesc::new(
        "cross-queue-storage-buffer",
        128,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let writer = builder.add_pass("async-writer", QueueLane::AsyncCompute);
    let reader = builder.add_pass("graphics-reader", QueueLane::Graphics);
    let range = RenderGraphBufferRange::new(32, Some(64));
    let version = builder
        .write_buffer_with_access_versioned(
            writer,
            buffer,
            range,
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .expect("writer should produce the exact storage-buffer version");
    builder
        .read_buffer_with_access_from_version(
            reader,
            version,
            range,
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .expect("reader should consume the selected buffer window");
    builder
        .set_pass_flags(
            reader,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("reader must remain a live graph root");

    (
        builder.compile().expect("cross-queue graph should compile"),
        writer,
        reader,
    )
}

fn execution_pass_for(
    graph: &CompiledRenderGraph,
    pass_id: crate::render_graph::RenderPassId,
) -> (RenderGraphExecutionPass, Vec<RenderGraphResourceAccessId>) {
    let graph_pass_index = graph.indexed_pass(pass_id).unwrap().0;
    let graph_pass = &graph.passes()[graph_pass_index];
    let access_ids = (0..graph_pass.resources.len())
        .map(|ordinal| graph.access_id_at(pass_id, ordinal).unwrap())
        .collect::<Vec<_>>();
    let device_access_bindings = access_ids
        .iter()
        .map(|access| *graph.access_allocation_binding(*access).unwrap())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let device_transitions_before = graph
        .resource_state_plan()
        .transitions()
        .iter()
        .filter(|transition| access_ids.contains(&transition.to_access))
        .copied()
        .collect::<Vec<_>>()
        .into_boxed_slice();

    (
        RenderGraphExecutionPass {
            graph_pass_index,
            stage: RenderPassStage::PostProcess,
            device_access_bindings,
            device_transitions_before,
        },
        access_ids,
    )
}

#[test]
fn device_execution_rejects_a_plan_without_a_materialized_device_epoch() {
    let (graph, pass) = explicit_storage_buffer_graph();
    let (execution_pass, access_ids) = execution_pass_for(&graph, pass);
    let resources = RenderGraphExecutionResources::new();

    let error = resources
        .validate_device_execution_pass(&graph, &execution_pass, &access_ids)
        .expect_err("compiled rows without a current device lease are not executable");

    assert!(
        error.contains("device epoch"),
        "unexpected validation error: {error}"
    );
}

#[test]
fn device_execution_validates_the_compiled_buffer_range_against_its_physical_lease() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (graph, pass) = explicit_storage_buffer_graph();
    let (execution_pass, access_ids) = execution_pass_for(&graph, pass);
    let mut pool = TransientResourcePool::default();
    pool.begin_frame(backend.device_profile());
    let mut resources = RenderGraphExecutionResources::new();
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            &graph,
            &mut pool,
        )
        .expect("the graph's live buffer range should materialize");

    resources
        .validate_device_execution_pass(&graph, &execution_pass, &access_ids)
        .expect("the device plan and materialized exact buffer lease should agree");
    assert_eq!(
        resources
            .transient_buffer_binding_for_access(access_ids[0])
            .unwrap()
            .1,
        32..96,
        "native buffer binding must retain the compiler's byte window"
    );
}

#[test]
fn device_execution_validates_cross_queue_buffer_producer_and_consumer_leases() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (graph, writer, reader) = cross_queue_storage_buffer_graph();
    let (writer_plan, writer_accesses) = execution_pass_for(&graph, writer);
    let (reader_plan, reader_accesses) = execution_pass_for(&graph, reader);
    assert_eq!(reader_plan.device_transitions_before.len(), 1);
    let transition = reader_plan.device_transitions_before[0];
    assert_eq!(transition.from_access, writer_accesses[0]);
    assert_eq!(transition.to_access, reader_accesses[0]);
    assert_eq!(
        transition.range,
        crate::render_graph::RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(
            32,
            Some(64)
        ))
    );
    assert!(transition.crosses_queue());

    let mut pool = TransientResourcePool::default();
    pool.begin_frame(backend.device_profile());
    let mut resources = RenderGraphExecutionResources::new();
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            &graph,
            &mut pool,
        )
        .expect("the selected buffer window should materialize once for both passes");

    resources
        .validate_device_execution_pass(&graph, &writer_plan, &writer_accesses)
        .expect("producer pass should resolve its physical device lease");
    resources
        .validate_device_execution_pass(&graph, &reader_plan, &reader_accesses)
        .expect("consumer pass should resolve its ordered physical device lease");
    let (writer_buffer, writer_range) = resources
        .transient_buffer_binding_for_access(writer_accesses[0])
        .expect("producer buffer lease");
    let (reader_buffer, reader_range) = resources
        .transient_buffer_binding_for_access(reader_accesses[0])
        .expect("consumer buffer lease");
    assert_eq!(
        resources.transient_physical_allocation_for_access(writer_accesses[0]),
        resources.transient_physical_allocation_for_access(reader_accesses[0]),
        "both graph accesses must resolve through the same compiler-selected allocation"
    );
    assert_eq!(writer_range, 32..96);
    assert_eq!(reader_range, writer_range);
    assert_eq!(writer_buffer.size(), reader_buffer.size());
}

#[test]
fn device_execution_accepts_a_full_scope_view_only_transient_texture_lease() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let desc = TextureDesc::new(
        "view-only-transient-texture",
        32,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    )
    .with_mip_levels(2);
    let mut builder = RenderGraphBuilder::new("device-execution-view-only-texture");
    let texture = builder.create_texture(desc.clone());
    let pass = builder.add_pass("texture-reader", QueueLane::Graphics);
    builder
        .access_texture(
            pass,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::ColorAttachment,
            None,
        )
        .expect("full-scope texture read should compile");
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("texture reader must remain a live graph root");
    let graph = builder.compile().expect("texture graph should compile");
    let (execution_pass, access_ids) = execution_pass_for(&graph, pass);

    let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("view-only-transient-texture"),
        size: wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: desc.depth_or_array_layers(),
        },
        mip_level_count: desc.mip_levels,
        sample_count: desc.sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_texture_view_with_physical_desc(
        "view-only-transient-texture",
        &view,
        desc,
    );
    let mut pool = TransientResourcePool::default();
    pool.begin_frame(backend.device_profile());
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            &graph,
            &mut pool,
        )
        .expect("a producer-supplied full texture view should materialize");

    resources
        .validate_device_execution_pass(&graph, &execution_pass, &access_ids)
        .expect("the exact full-scope view is a valid device lease for this pass");
}

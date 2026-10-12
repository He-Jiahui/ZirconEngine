use super::*;
use crate::graphics::backend::RenderBackend;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBufferRange, RenderGraphBuilder,
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceAccessRange, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn required_external_lease_is_fail_closed_when_physical_buffer_is_missing() {
    let mut builder = RenderGraphBuilder::new("external-lease-missing");
    let buffer = builder.import_present_external_buffer_with_binding(
        "external-buffer",
        BufferDesc::new("external-buffer", 256, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = builder.add_pass("external-writer", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, buffer).unwrap();
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
    let resources = RenderGraphExecutionResources::new();
    let error = RenderGraphExecutionExternalAccessBindings::materialize(&resources, &graph)
        .expect_err("required external leases must be present before encoding");
    assert!(error.contains("required external buffer `external-buffer`"));
}

#[test]
fn external_lease_table_is_keyed_by_compiled_access_identity() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut builder = RenderGraphBuilder::new("external-lease-access-id");
    let buffer = builder.import_present_external_buffer_with_binding(
        "external-buffer",
        BufferDesc::new("external-buffer", 256, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = builder.add_pass("external-writer", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, buffer).unwrap();
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
        label: Some("external-lease-access-id-buffer"),
        size: 256,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.insert_buffer("external-buffer", native);
    resources
        .materialize_external_access_bindings(&graph)
        .expect("typed external lease should materialize");
    let access_id = graph.access_id_at(pass, 0).unwrap();
    let (_, range) = resources
        .external_buffer_binding_for_access(access_id)
        .expect("compiled access identity should resolve its lease");
    assert_eq!(range, 0..256);
}

#[test]
fn external_lease_table_preserves_concrete_buffer_access_window() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut builder = RenderGraphBuilder::new("external-lease-buffer-window");
    let buffer = builder.import_present_external_buffer_with_binding(
        "external-buffer",
        BufferDesc::new("external-buffer", 256, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = builder.add_pass("external-writer", QueueLane::AsyncCompute);
    builder
        .access_external(
            pass,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(32, Some(64))),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
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
        label: Some("external-lease-buffer-window"),
        size: 256,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.insert_buffer("external-buffer", native);
    resources
        .materialize_external_access_bindings(&graph)
        .unwrap();
    let access_id = graph.access_id_at(pass, 0).unwrap();
    let (_, range) = resources
        .external_buffer_binding_for_access(access_id)
        .unwrap();
    assert_eq!(range, 32..96);
}

#[test]
fn external_texture_lease_materializes_exact_scope_from_physical_backing() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (graph, pass, desc) = exact_external_texture_graph("exact-external-texture");
    let texture = create_external_texture(&backend, "exact-external-texture", &desc);
    let default_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_texture("external-texture", &texture, &default_view, desc);

    resources
        .materialize_external_access_bindings(&graph)
        .expect("physical texture backing must materialize its exact mip lease");

    let access_id = graph.access_id_at(pass, 0).unwrap();
    assert!(resources
        .external_texture_view_for_access(access_id)
        .is_ok());
}

#[test]
fn exact_external_texture_scope_rejects_a_view_only_physical_lease() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (graph, _pass, desc) = exact_external_texture_graph("view-only-external-texture");
    let texture = create_external_texture(&backend, "view-only-external-texture", &desc);
    let default_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_texture_view_with_physical_desc(
        "external-texture",
        &default_view,
        desc,
    );

    let error = resources
        .materialize_external_access_bindings(&graph)
        .expect_err("a default view cannot represent a partial mip lease");

    assert!(error.contains("requires subresource scope"));
    assert!(error.contains("physical lease is view-only"));
}

#[test]
fn exact_full_external_texture_scope_accepts_a_view_only_physical_lease() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let desc = external_texture_desc("full-view-only-external-texture");
    let mut builder = RenderGraphBuilder::new("full-view-only-external-texture");
    let texture = builder.import_present_external_texture_with_binding(
        "external-texture",
        desc.clone(),
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass = builder.add_pass("external-reader", QueueLane::AsyncCompute);
    builder
        .read_external_with_access(
            pass,
            texture,
            RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full()),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    builder.set_pass_flags(pass, non_cullable()).unwrap();
    let graph = builder.compile().unwrap();
    let native = create_external_texture(&backend, "full-view-only-external-texture", &desc);
    let default_view = native.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_texture_view_with_physical_desc(
        "external-texture",
        &default_view,
        desc,
    );

    resources
        .materialize_external_access_bindings(&graph)
        .expect("a full-scope access may reuse its producer-supplied default view");
    let access_id = graph.access_id_at(pass, 0).unwrap();
    assert!(resources
        .external_texture_view_for_access(access_id)
        .is_ok());
}

fn exact_external_texture_graph(
    name: &'static str,
) -> (
    CompiledRenderGraph,
    crate::render_graph::RenderPassId,
    TextureDesc,
) {
    let desc = external_texture_desc(name);
    let mut builder = RenderGraphBuilder::new(name);
    let texture = builder.import_present_external_texture_with_binding(
        "external-texture",
        desc.clone(),
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass = builder.add_pass("external-reader", QueueLane::AsyncCompute);
    builder
        .read_external_with_access(
            pass,
            texture,
            RenderGraphResourceAccessRange::Texture(
                RenderGraphTextureSubresourceRange::single_mip(2),
            ),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    builder.set_pass_flags(pass, non_cullable()).unwrap();
    (builder.compile().unwrap(), pass, desc)
}

fn external_texture_desc(name: &'static str) -> TextureDesc {
    TextureDesc::new(
        name,
        32,
        16,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED,
    )
    .with_mip_levels(4)
}

fn create_external_texture(
    backend: &RenderBackend,
    label: &'static str,
    desc: &TextureDesc,
) -> wgpu::Texture {
    backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: desc.depth_or_array_layers(),
        },
        mip_level_count: desc.mip_levels,
        sample_count: desc.sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn non_cullable() -> PassFlags {
    PassFlags {
        has_side_effects: true,
        ..PassFlags::default()
    }
}

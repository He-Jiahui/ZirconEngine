use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::backend::RenderBackend;
use crate::render_graph::{
    CompiledRenderGraph, PassFlags, QueueLane, RenderGraphBuilder, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphTextureSubresourceRange,
};
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

use super::super::render_graph_execution_resources::RenderGraphExecutionResources;
use super::*;

#[test]
fn non_storage_texture_formats_reject_declared_storage_usage() {
    for format in [
        TextureFormat::R8Unorm,
        TextureFormat::R16Float,
        TextureFormat::Rg16Float,
        TextureFormat::Rg11b10Ufloat,
    ] {
        let error = storage_requested_usages_for(format).unwrap_err();

        assert!(error.contains("does not support declared STORAGE usage"));
    }
}

#[test]
fn storage_texture_formats_request_storage_binding() {
    for format in [
        TextureFormat::R32Float,
        TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba16Float,
        TextureFormat::Rgba32Float,
    ] {
        let usages = storage_requested_usages_for(format).unwrap();

        assert!(usages.contains(wgpu::TextureUsages::STORAGE_BINDING));
    }
}

fn storage_requested_usages_for(format: TextureFormat) -> Result<wgpu::TextureUsages, String> {
    wgpu_texture_usages(
        format,
        TextureUsage::RENDER_ATTACHMENT
            | TextureUsage::SAMPLED
            | TextureUsage::STORAGE
            | TextureUsage::COPY_SRC
            | TextureUsage::COPY_DST,
    )
}

#[test]
fn materialization_rejects_sparse_reservations_before_backend_allocation() {
    let mut builder = RenderGraphBuilder::new("sparse-materialization-rejection");
    let sparse = builder.create_texture(
        TextureDesc::new(
            "sparse-pages",
            128,
            128,
            TextureFormat::Rgba8Unorm,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_sparse_residency(),
    );
    let pass = builder.add_pass("materialize", QueueLane::Graphics);
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: true,
                has_side_effects: true,
            },
        )
        .unwrap();
    builder.write_storage_texture(pass, sparse).unwrap();
    assert!(matches!(
        builder.compile(),
        Err(crate::render_graph::RenderGraphError::SparseTextureUnsupported { resource })
            if resource == "sparse-pages"
    ));
}

#[test]
fn materialization_aliases_compatible_transient_texture_slots() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("compatible-texture-aliasing");
    let first = builder.create_texture(TextureDesc::new(
        "first-color",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let second = builder.create_texture(TextureDesc::new(
        "second-color",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_present_external_resource("viewport-output");
    let first_write = builder.add_pass("first-write", QueueLane::Graphics);
    let first_read = builder.add_pass("first-read", QueueLane::Graphics);
    let second_write = builder.add_pass("second-write", QueueLane::Graphics);
    let second_read = builder.add_pass("second-read", QueueLane::Graphics);
    builder.write_texture(first_write, first).unwrap();
    builder.read_texture(first_read, first).unwrap();
    builder.write_texture(second_write, second).unwrap();
    builder.read_texture(second_read, second).unwrap();
    builder.write_external(second_read, output).unwrap();
    builder.add_dependency(first_write, first_read).unwrap();
    builder.add_dependency(first_read, second_write).unwrap();
    builder.add_dependency(second_write, second_read).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert_eq!(graph.transient_allocation_plan().texture_slot_count, 1);
    assert!(resources.has_texture_view("first-color"));
    assert!(resources.has_texture_view("second-color"));
    assert!(resources.owned_texture("first-color").is_some());
    assert!(resources.owned_texture("second-color").is_some());
    let report = resources.resource_report();
    assert_eq!(
        report.owned_texture_count, 1,
        "compatible non-overlapping logical textures should share one WGPU backing texture"
    );
    assert_eq!(report.external_texture_view_count, 0);
    assert_eq!(report.texture_view_count, 2);
    let alias_report = resources.resource_alias_report();
    let first_alias = texture_alias_for(&alias_report, "first-color");
    let second_alias = texture_alias_for(&alias_report, "second-color");
    assert_eq!(first_alias.backing_name, second_alias.backing_name);
    assert!(
        first_alias
            .backing_name
            .starts_with("rg-transient-texture-allocation-")
    );
}

#[test]
fn materialization_allocates_graph_owned_persistent_texture_outside_alias_slots() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("persistent-texture-materialization");
    let history_source = builder.create_texture(TextureDesc::new(
        "history-source",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
    ));
    builder.mark_persistent(history_source).unwrap();
    let write = builder.add_pass("write-history-source", QueueLane::Graphics);
    let read = builder.add_pass("read-history-source", QueueLane::Graphics);
    builder.write_texture(write, history_source).unwrap();
    builder.read_texture(read, history_source).unwrap();
    builder.add_dependency(write, read).unwrap();
    builder
        .set_pass_flags(
            read,
            crate::render_graph::PassFlags {
                has_side_effects: true,
                ..crate::render_graph::PassFlags::default()
            },
        )
        .unwrap();
    let graph = builder.compile().unwrap();
    let write_access = graph.access_id_at(write, 0).unwrap();
    let read_access = graph.access_id_at(read, 0).unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert_eq!(graph.transient_allocation_plan().texture_slot_count, 0);
    assert!(resources.has_texture_view("history-source"));
    assert!(resources.owned_texture("history-source").is_some());
    assert_eq!(resources.resource_report().owned_texture_count, 1);
    assert_eq!(resources.persistent_texture_access_binding_count(), 2);
    assert_eq!(resources.persistent_texture_backing_count(), 1);
    assert!(
        resources
            .graph_owned_texture_for_access(write_access)
            .is_ok()
    );
    assert!(
        resources
            .graph_owned_texture_for_access(read_access)
            .is_ok()
    );
    assert_eq!(
        texture_alias_for(&resources.resource_alias_report(), "history-source").backing_name,
        "rg-persistent-texture-history-source"
    );
}

#[test]
fn materialization_prebuilds_persistent_texture_views_for_exact_mip_accesses() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("persistent-texture-access-views");
    let history_source = builder.create_texture(
        TextureDesc::new(
            "history-mips",
            32,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(4),
    );
    builder.mark_persistent(history_source).unwrap();
    let write = builder.add_pass("write-history-mip", QueueLane::Graphics);
    let read = builder.add_pass("read-history-mip", QueueLane::Graphics);
    let written_version = builder
        .write_texture_with_access_versioned(
            write,
            history_source,
            RenderGraphTextureSubresourceRange::single_mip(2),
            crate::render_graph::RenderGraphResourceAccessIntent::ColorAttachment,
            Some(crate::render_graph::RenderGraphAttachmentOps::clear_store()),
        )
        .unwrap();
    builder
        .read_texture_with_access_from_version(
            read,
            written_version,
            RenderGraphTextureSubresourceRange::single_mip(2),
            crate::render_graph::RenderGraphResourceAccessIntent::sampled_texture(
                crate::render_graph::RenderGraphShaderStages::FRAGMENT,
            ),
        )
        .unwrap();
    builder
        .set_pass_flags(
            read,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();
    let graph = builder.compile().unwrap();
    let write_access = graph.access_id_at(write, 0).unwrap();
    let read_access = graph.access_id_at(read, 0).unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert!(
        resources
            .persistent_texture_view_for_access(write_access)
            .is_ok()
    );
    assert!(
        resources
            .persistent_texture_view_for_access(read_access)
            .is_ok()
    );
    assert_eq!(resources.persistent_texture_access_binding_count(), 2);
    assert_eq!(resources.persistent_texture_view_count(), 2);
    assert_eq!(resources.persistent_texture_backing_count(), 1);
}

#[test]
fn materialization_receives_incompatible_texture_resources_in_separate_graph_slots() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("incompatible-texture-aliasing");
    let large = builder.create_texture(TextureDesc::new(
        "large-color",
        64,
        64,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let small = builder.create_texture(TextureDesc::new(
        "small-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_present_external_resource("viewport-output");
    let large_write = builder.add_pass("large-write", QueueLane::Graphics);
    let large_read = builder.add_pass("large-read", QueueLane::Graphics);
    let small_write = builder.add_pass("small-write", QueueLane::Graphics);
    let small_read = builder.add_pass("small-read", QueueLane::Graphics);
    builder.write_texture(large_write, large).unwrap();
    builder.read_texture(large_read, large).unwrap();
    builder.write_texture(small_write, small).unwrap();
    builder.read_texture(small_read, small).unwrap();
    builder.write_external(small_read, output).unwrap();
    builder.add_dependency(large_write, large_read).unwrap();
    builder.add_dependency(large_read, small_write).unwrap();
    builder.add_dependency(small_write, small_read).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert_eq!(
        graph.transient_allocation_plan().texture_slot_count,
        2,
        "the neutral graph plan now keeps WGPU-incompatible texture descriptors in separate buckets"
    );
    assert!(resources.has_texture_view("large-color"));
    assert!(resources.has_texture_view("small-color"));
    let report = resources.resource_report();
    assert_eq!(
        report.owned_texture_count, 2,
        "WGPU-incompatible logical textures should arrive in separate graph allocation buckets"
    );
    assert_eq!(report.external_texture_view_count, 0);
    assert_eq!(report.texture_view_count, 2);
    let alias_report = resources.resource_alias_report();
    let large_alias = texture_alias_for(&alias_report, "large-color");
    let small_alias = texture_alias_for(&alias_report, "small-color");
    assert_ne!(
        large_alias.backing_name, small_alias.backing_name,
        "different descriptor buckets can both use slot zero but must materialize distinct WGPU backings"
    );
    assert!(
        large_alias
            .backing_name
            .starts_with("rg-transient-texture-bucket-")
    );
    assert!(
        small_alias
            .backing_name
            .starts_with("rg-transient-texture-bucket-")
    );
}

#[test]
fn materialization_overrides_preimported_terminal_aa_output_with_owned_transient() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let final_alias = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("final-target-alias"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut builder = RenderGraphBuilder::new("terminal-aa-input-materialization");
    let tonemapped = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::TONEMAPPED,
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let terminal_output = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output =
        builder.import_present_external_resource(PostProcessGraphResourceNames::FINAL_COLOR);
    let fxaa = builder.add_pass("fxaa", QueueLane::Graphics);
    let output_transfer = builder.add_pass("output-transfer", QueueLane::Graphics);
    builder.read_texture(fxaa, tonemapped).unwrap();
    builder.write_texture(fxaa, terminal_output).unwrap();
    builder
        .read_texture(output_transfer, terminal_output)
        .unwrap();
    builder.write_external(output_transfer, output).unwrap();
    builder.add_dependency(fxaa, output_transfer).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_texture_alias(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        &final_alias,
    );

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert!(
        resources
            .owned_texture(PostProcessGraphResourceNames::FINAL_COMPOSITED)
            .is_some(),
        "terminal AA output must replace the preimported final-color alias with an owned transient"
    );
    let report = resources.resource_report();
    assert_eq!(report.owned_texture_count, 1);
    assert_eq!(report.external_texture_view_count, 0);
}

#[test]
fn materialization_preserves_imported_persistent_texture_without_pool_backing() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let history_texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("history-scene-color"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut builder = RenderGraphBuilder::new("persistent-history-materialization");
    let history = builder.create_texture(TextureDesc::new(
        "history.current.scene-color",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    builder.mark_persistent(history).unwrap();
    let write_history = builder.add_pass("write-history", QueueLane::Graphics);
    builder.write_texture(write_history, history).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_texture_alias("history.current.scene-color", &history_texture);

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert!(resources.has_texture_view("history.current.scene-color"));
    assert!(
        resources
            .owned_texture("history.current.scene-color")
            .is_none()
    );
    let report = resources.resource_report();
    assert_eq!(report.owned_texture_count, 0);
    assert_eq!(report.external_texture_view_count, 1);
}

#[test]
fn materialization_aliases_transient_buffer_slots() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("compatible-buffer-aliasing");
    let first = builder.create_buffer(BufferDesc::new(
        "first-indirect",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let second = builder.create_buffer(BufferDesc::new(
        "second-indirect",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let output = builder.import_present_external_resource("viewport-output");
    let first_write = builder.add_pass("first-buffer-write", QueueLane::Graphics);
    let first_read = builder.add_pass("first-buffer-read", QueueLane::Graphics);
    let second_write = builder.add_pass("second-buffer-write", QueueLane::Graphics);
    let second_read = builder.add_pass("second-buffer-read", QueueLane::Graphics);
    builder.write_buffer(first_write, first).unwrap();
    builder.read_buffer(first_read, first).unwrap();
    builder.write_buffer(second_write, second).unwrap();
    builder.read_buffer(second_read, second).unwrap();
    builder.write_external(second_read, output).unwrap();
    builder.add_dependency(first_write, first_read).unwrap();
    builder.add_dependency(first_read, second_write).unwrap();
    builder.add_dependency(second_write, second_read).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert_eq!(graph.transient_allocation_plan().buffer_slot_count, 1);
    assert!(resources.has_buffer("first-indirect"));
    assert!(resources.has_buffer("second-indirect"));
    let report = resources.resource_report();
    assert_eq!(
        report.buffer_count, 1,
        "compatible non-overlapping logical buffers should share one WGPU backing buffer"
    );
    assert_eq!(report.texture_view_count, 0);
    assert_eq!(report.total_bound_resource_count, 1);
    let alias_report = resources.resource_alias_report();
    let first_alias = buffer_alias_for(&alias_report, "first-indirect");
    let second_alias = buffer_alias_for(&alias_report, "second-indirect");
    assert_eq!(first_alias.backing_name, second_alias.backing_name);
    assert!(
        first_alias
            .backing_name
            .starts_with("rg-transient-buffer-allocation-")
    );
}

#[test]
fn materialization_binds_only_transient_exact_accesses_to_device_resources() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("exact-transient-access-bindings");
    let pyramid = builder.create_texture(
        TextureDesc::new(
            "reflection-pyramid",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let coarse = builder
        .create_texture_view_alias(
            "reflection-pyramid-coarse",
            pyramid,
            RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("declare coarse reflection view");
    let clusters = builder.create_buffer(BufferDesc::new(
        "visible-clusters",
        256,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let history = builder.create_texture(TextureDesc::new(
        "history-color",
        64,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    builder.mark_persistent(history).unwrap();
    let output = builder.import_present_external_resource("viewport-output");
    let build = builder.add_pass("build", QueueLane::Graphics);
    let resolve = builder.add_pass("resolve-coarse", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_texture(build, pyramid).unwrap();
    builder.write_buffer(build, clusters).unwrap();
    builder.write_texture(build, history).unwrap();
    builder.read_texture(resolve, pyramid).unwrap();
    builder.write_texture(resolve, coarse).unwrap();
    builder.read_texture(present, coarse).unwrap();
    builder.read_buffer(present, clusters).unwrap();
    builder.write_external(present, output).unwrap();
    builder.add_dependency(build, resolve).unwrap();
    builder.add_dependency(resolve, present).unwrap();

    let graph = builder.compile().expect("compile exact access graph");
    let alias_write = graph
        .access_id_for(
            resolve,
            RenderGraphResource::TransientTexture(coarse),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("alias write access");
    let buffer_read = graph
        .access_id_for(
            present,
            RenderGraphResource::TransientBuffer(clusters),
            RenderGraphResourceAccessKind::Read,
        )
        .expect("buffer read access");
    let persistent_write = graph
        .access_id_for(
            build,
            RenderGraphResource::TransientTexture(history),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("persistent write access");
    let output_write = graph
        .access_id_for(
            present,
            RenderGraphResource::External(output),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("external output access");
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph)
        .expect("materialize exact transient access bindings");

    assert_eq!(resources.transient_access_binding_count(), 6);
    assert_eq!(resources.transient_texture_backing_count(), 1);
    let report = resources.resource_report();
    assert_eq!(
        report.access_binding_report.transient_access_binding_count,
        6
    );
    assert_eq!(
        report
            .access_binding_report
            .transient_texture_access_binding_count,
        4
    );
    assert_eq!(
        report
            .access_binding_report
            .transient_buffer_access_binding_count,
        2
    );
    assert_eq!(report.access_binding_report.unique_texture_view_count, 2);
    assert_eq!(report.access_binding_report.reused_texture_view_count, 2);
    assert!(
        resources
            .transient_texture_view_for_access(alias_write)
            .is_ok()
    );
    assert!(resources.transient_texture_for_access(alias_write).is_ok());
    assert_eq!(
        resources.transient_access_key(alias_write),
        graph.versioned_access_key(alias_write)
    );
    assert_eq!(
        resources.transient_physical_allocation_for_access(alias_write),
        graph.physical_allocation_id_for_access(alias_write)
    );
    assert!(
        resources
            .transient_buffer_slice_for_access(buffer_read)
            .is_ok()
    );
    assert!(
        resources
            .transient_texture_view_for_access(persistent_write)
            .is_err()
    );
    assert!(
        resources
            .transient_texture_for_access(persistent_write)
            .is_err()
    );
    assert!(
        resources
            .persistent_texture_for_access(persistent_write)
            .is_ok()
    );
    assert!(
        resources
            .graph_owned_texture_for_access(persistent_write)
            .is_ok()
    );
    assert_eq!(resources.persistent_texture_access_binding_count(), 1);
    assert_eq!(resources.persistent_texture_backing_count(), 1);
    assert!(
        resources
            .transient_texture_view_for_access(output_write)
            .is_err()
    );
}

#[test]
fn materialization_does_not_bind_diagnostic_accesses_from_culled_passes() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("culled-exact-access-bindings");
    let culled_scratch = builder.create_texture(TextureDesc::new(
        "culled-scratch",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let live_color = builder.create_texture(TextureDesc::new(
        "live-color",
        32,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_present_external_resource("viewport-output");
    let culled = builder.add_pass("culled-scratch-writer", QueueLane::Graphics);
    let live = builder.add_pass("live-writer", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_texture(culled, culled_scratch).unwrap();
    builder.write_texture(live, live_color).unwrap();
    builder.read_texture(present, live_color).unwrap();
    builder.write_external(present, output).unwrap();
    builder.add_dependency(live, present).unwrap();

    let graph = builder.compile().expect("compile culled access graph");
    let culled_access = graph
        .access_id_for(
            culled,
            RenderGraphResource::TransientTexture(culled_scratch),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("culled writer should retain a diagnostic access ID");
    assert!(graph.versioned_access_key(culled_access).is_some());
    assert!(graph.access_allocation_binding(culled_access).is_none());

    let mut resources = RenderGraphExecutionResources::new();
    materialize_with_transient_pool(&mut resources, &backend, &graph)
        .expect("materialize live transient exact access bindings");

    assert_eq!(resources.transient_access_binding_count(), 2);
    assert!(!resources.has_texture_view("culled-scratch"));
    assert!(
        resources
            .transient_texture_view_for_access(culled_access)
            .is_err()
    );
    assert!(
        resources
            .transient_physical_allocation_for_access(culled_access)
            .is_none()
    );
}

#[test]
fn materialization_exposes_owned_texture_mip_views() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("mipped-materialization");
    let pyramid = builder.create_texture(
        TextureDesc::new(
            "mipped-pyramid",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let pass = builder.add_pass("write-mip-zero", QueueLane::Graphics);
    builder.write_texture(pass, pyramid).unwrap();
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
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert!(resources.has_texture_view("mipped-pyramid"));
    assert!(
        resources
            .owned_texture_mip_view("mipped-pyramid", 1)
            .is_ok()
    );
    assert_eq!(
        resources
            .owned_texture_mip_view("mipped-pyramid", 3)
            .unwrap_err(),
        "render graph execution texture resource `mipped-pyramid` mip level 3 is outside mip_levels 3"
    );
}

#[test]
fn materialization_exposes_owned_cube_storage_texture_array_views() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("cube-storage-view-materialization");
    let cube = builder.create_texture(
        TextureDesc::new(
            "environment.ibl.pmrem",
            64,
            64,
            TextureFormat::Rgba16Float,
            TextureUsage::STORAGE | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
        )
        .with_dimension(TextureDimension::Cube)
        .with_depth(6)
        .with_mip_levels(4),
    );
    let pass = builder.add_pass("env.ibl_prefilter.mip2", QueueLane::AsyncCompute);
    builder.write_storage_texture(pass, cube).unwrap();
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
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert!(
        resources
            .owned_texture_view_with_descriptor(
                "environment.ibl.pmrem",
                &ibl_pmrem_storage_view_descriptor(2)
            )
            .is_ok(),
        "IBL PMREM passes need a Cube backing exposed as one D2Array storage view per mip"
    );
    let mut invalid_mip = ibl_pmrem_storage_view_descriptor(4);
    assert_eq!(
        resources
            .owned_texture_view_with_descriptor("environment.ibl.pmrem", &invalid_mip)
            .unwrap_err(),
        "render graph execution texture resource `environment.ibl.pmrem` view mip range [4..5) is outside mip_levels 4"
    );
    invalid_mip.base_mip_level = 2;
    invalid_mip.array_layer_count = Some(7);
    assert_eq!(
        resources
            .owned_texture_view_with_descriptor("environment.ibl.pmrem", &invalid_mip)
            .unwrap_err(),
        "render graph execution texture resource `environment.ibl.pmrem` view array range [0..7) is outside depth/array_layers 6"
    );
    invalid_mip.array_layer_count = Some(6);
    invalid_mip.usage = Some(wgpu::TextureUsages::COPY_DST);
    let error = resources
        .owned_texture_view_with_descriptor("environment.ibl.pmrem", &invalid_mip)
        .unwrap_err();
    assert!(error.contains("view usage"), "{error}");
    assert!(error.contains("COPY_DST"), "{error}");
    assert!(error.contains("not allowed by texture usages"), "{error}");
}

fn ibl_pmrem_storage_view_descriptor(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("test-ibl-pmrem-storage-view"),
        format: Some(wgpu::TextureFormat::Rgba16Float),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        usage: Some(wgpu::TextureUsages::STORAGE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(6),
    }
}

#[test]
fn materialization_aliases_declared_texture_view_to_parent_mip() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("declared-mip-aliases");
    let reflection_pyramid = builder.create_texture(
        TextureDesc::new(
            "arbitrary-pyramid",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let reflection_pyramid_coarse = builder
        .create_texture_view_alias(
            "arbitrary-pyramid-coarse",
            reflection_pyramid,
            RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("declare arbitrary parent mip alias");
    let output = builder.import_present_external_resource("viewport-output");
    let reflection_pass = builder.add_pass("reflection-pyramid", QueueLane::Graphics);
    builder
        .write_texture(reflection_pass, reflection_pyramid)
        .unwrap();
    let reflection_coarse_pass = builder.add_pass("reflection-pyramid-coarse", QueueLane::Graphics);
    builder
        .read_texture(reflection_coarse_pass, reflection_pyramid)
        .unwrap();
    builder
        .write_texture(reflection_coarse_pass, reflection_pyramid_coarse)
        .unwrap();
    let output_pass = builder.add_pass("output", QueueLane::Graphics);
    builder
        .read_texture(output_pass, reflection_pyramid_coarse)
        .unwrap();
    builder.write_external(output_pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    materialize_with_transient_pool(&mut resources, &backend, &graph).unwrap();

    assert!(resources.has_texture_view("arbitrary-pyramid"));
    assert!(resources.has_texture_view("arbitrary-pyramid-coarse"));
    assert!(resources.owned_texture("arbitrary-pyramid").is_some());
    assert!(
        resources
            .owned_texture("arbitrary-pyramid-coarse")
            .is_none()
    );
    let report = resources.resource_report();
    assert_eq!(report.external_texture_view_count, 0);
    assert_eq!(report.owned_texture_count, 1);
    assert_eq!(report.texture_view_count, 2);
}

fn texture_alias_for<'a>(
    report: &'a crate::core::framework::render::RenderGraphExecutionAliasReport,
    logical_name: &str,
) -> &'a crate::core::framework::render::RenderGraphExecutionAliasRecord {
    report
        .texture_aliases
        .iter()
        .find(|record| record.logical_name == logical_name)
        .unwrap()
}

fn materialize_with_transient_pool(
    resources: &mut RenderGraphExecutionResources,
    backend: &RenderBackend,
    graph: &CompiledRenderGraph,
) -> Result<(), String> {
    let mut pool = TransientResourcePool::default();
    pool.begin_frame(backend.device_profile());
    resources.materialize_transient_resources_with_pool(
        &backend.device,
        backend.device_profile(),
        graph,
        &mut pool,
    )
}

fn buffer_alias_for<'a>(
    report: &'a crate::core::framework::render::RenderGraphExecutionAliasReport,
    logical_name: &str,
) -> &'a crate::core::framework::render::RenderGraphExecutionAliasRecord {
    report
        .buffer_aliases
        .iter()
        .find(|record| record.logical_name == logical_name)
        .unwrap()
}

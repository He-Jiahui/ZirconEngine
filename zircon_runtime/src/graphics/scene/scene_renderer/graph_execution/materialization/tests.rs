use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::backend::RenderBackend;
use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

use super::super::render_graph_execution_resources::RenderGraphExecutionResources;
use super::*;

#[test]
fn non_storage_texture_formats_do_not_request_storage_binding() {
    for format in [
        TextureFormat::R8Unorm,
        TextureFormat::R16Float,
        TextureFormat::Rg16Float,
        TextureFormat::Rg11b10Ufloat,
    ] {
        let usages = storage_requested_usages_for(format);

        assert!(usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT));
        assert!(usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
        assert!(!usages.contains(wgpu::TextureUsages::STORAGE_BINDING));
        assert!(usages.contains(wgpu::TextureUsages::COPY_SRC));
        assert!(usages.contains(wgpu::TextureUsages::COPY_DST));
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
        let usages = storage_requested_usages_for(format);

        assert!(usages.contains(wgpu::TextureUsages::STORAGE_BINDING));
    }
}

fn storage_requested_usages_for(format: TextureFormat) -> wgpu::TextureUsages {
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
fn materialization_creates_dense_transients_and_skips_sparse_reservations() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("materialization");
    let shadow = builder.create_texture(TextureDesc::new(
        "shadow-atlas",
        64,
        64,
        TextureFormat::Depth32Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
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
    let scratch = builder.create_buffer(BufferDesc::new(
        "scratch",
        16,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
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
    builder.write_texture(pass, shadow).unwrap();
    builder.write_storage_texture(pass, sparse).unwrap();
    builder.write_buffer(pass, scratch).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

    assert!(resources.has_texture_view("shadow-atlas"));
    assert!(
        !resources.has_texture_view("sparse-pages"),
        "sparse reservations must not be silently backed by a dense WGPU texture"
    );
    assert!(resources.has_buffer("scratch"));
    assert!(resources.has_bound_resource("shadow-atlas"));
    assert!(resources.has_bound_resource("scratch"));
    assert!(!resources.has_bound_resource("sparse-pages"));
    assert_eq!(
        resources.resource_report(),
        crate::core::framework::render::RenderGraphExecutionResourceReport::new(1, 0, 1, 1)
    );
    let materialization_report = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap();
    assert_eq!(materialization_report.required_texture_count, 1);
    assert_eq!(materialization_report.bound_texture_count, 1);
    assert_eq!(materialization_report.required_buffer_count, 1);
    assert_eq!(materialization_report.bound_buffer_count, 1);
    assert_eq!(materialization_report.sparse_texture_reservation_count, 1);
    assert_eq!(materialization_report.missing_resource_count(), 0);
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
    let output = builder.import_external_resource("viewport-output");
    let first_write = builder.add_pass("first-write", QueueLane::Graphics);
    let first_read = builder.add_pass("first-read", QueueLane::Graphics);
    let second_write = builder.add_pass("second-write", QueueLane::Graphics);
    let second_read = builder.add_pass("second-read", QueueLane::Graphics);
    builder.write_texture(first_write, first).unwrap();
    builder.read_texture(first_read, first).unwrap();
    builder.write_texture(second_write, second).unwrap();
    builder.read_texture(second_read, second).unwrap();
    builder.write_external(second_read, output).unwrap();
    builder.add_dependency(first_read, second_write).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

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
    assert!(first_alias
        .backing_name
        .starts_with("rg-transient-texture-bucket-"));
    assert!(first_alias.backing_name.ends_with("-slot-0"));
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
    let output = builder.import_external_resource("viewport-output");
    let large_write = builder.add_pass("large-write", QueueLane::Graphics);
    let large_read = builder.add_pass("large-read", QueueLane::Graphics);
    let small_write = builder.add_pass("small-write", QueueLane::Graphics);
    let small_read = builder.add_pass("small-read", QueueLane::Graphics);
    builder.write_texture(large_write, large).unwrap();
    builder.read_texture(large_read, large).unwrap();
    builder.write_texture(small_write, small).unwrap();
    builder.read_texture(small_read, small).unwrap();
    builder.write_external(small_read, output).unwrap();
    builder.add_dependency(large_read, small_write).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

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
    assert!(large_alias
        .backing_name
        .starts_with("rg-transient-texture-bucket-"));
    assert!(small_alias
        .backing_name
        .starts_with("rg-transient-texture-bucket-"));
}

#[test]
fn materialization_overrides_preimported_terminal_aa_input_with_owned_transient() {
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
    let terminal_input = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_external_resource(PostProcessGraphResourceNames::FINAL_COLOR);
    let output_transfer = builder.add_pass("output-transfer", QueueLane::Graphics);
    let fxaa = builder.add_pass("fxaa", QueueLane::Graphics);
    builder
        .write_texture(output_transfer, terminal_input)
        .unwrap();
    builder.read_texture(fxaa, terminal_input).unwrap();
    builder.write_external(fxaa, output).unwrap();
    builder.add_dependency(output_transfer, fxaa).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_texture_alias(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        &final_alias,
    );

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

    assert!(
        resources
            .owned_texture(PostProcessGraphResourceNames::FINAL_COMPOSITED)
            .is_some(),
        "terminal AA input must replace the preimported final-color alias with an owned transient"
    );
    let report = resources.resource_report();
    assert_eq!(report.owned_texture_count, 1);
    assert_eq!(report.external_texture_view_count, 0);
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
    let output = builder.import_external_resource("viewport-output");
    let first_write = builder.add_pass("first-buffer-write", QueueLane::Graphics);
    let first_read = builder.add_pass("first-buffer-read", QueueLane::Graphics);
    let second_write = builder.add_pass("second-buffer-write", QueueLane::Graphics);
    let second_read = builder.add_pass("second-buffer-read", QueueLane::Graphics);
    builder.write_buffer(first_write, first).unwrap();
    builder.read_buffer(first_read, first).unwrap();
    builder.write_buffer(second_write, second).unwrap();
    builder.read_buffer(second_read, second).unwrap();
    builder.write_external(second_read, output).unwrap();
    builder.add_dependency(first_read, second_write).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

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
    assert!(first_alias
        .backing_name
        .starts_with("rg-transient-buffer-bucket-"));
    assert!(first_alias.backing_name.ends_with("-slot-0"));
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
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

    assert!(resources.has_texture_view("mipped-pyramid"));
    assert!(resources
        .owned_texture_mip_view("mipped-pyramid", 1)
        .is_ok());
    assert_eq!(
        resources
            .owned_texture_mip_view("mipped-pyramid", 3)
            .unwrap_err(),
        "render graph execution texture resource `mipped-pyramid` mip level 3 is outside mip_levels 3"
    );
}

#[test]
fn materialization_aliases_ssr_reflection_coarse_pyramid_to_parent_mip_view() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("ssr-mip-aliases");
    let reflection_pyramid = builder.create_texture(
        TextureDesc::new(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let reflection_pyramid_coarse = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        32,
        16,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_external_resource("viewport-output");
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

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

    assert!(resources.has_texture_view(
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID
    ));
    assert!(resources.has_texture_view(
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
    ));
    assert!(resources
        .owned_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID)
        .is_some());
    assert!(resources
        .owned_texture(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
        )
        .is_none());
    let report = resources.resource_report();
    assert_eq!(report.external_texture_view_count, 0);
    assert_eq!(report.owned_texture_count, 1);
    assert_eq!(report.texture_view_count, 2);
}

#[test]
fn materialization_allocates_ssr_reflection_coarse_resource_when_parent_has_no_coarse_mip() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut builder = RenderGraphBuilder::new("ssr-small-pyramid");
    let reflection_pyramid = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        1,
        1,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let reflection_pyramid_coarse = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        1,
        1,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_external_resource("viewport-output");
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

    resources
        .materialize_transient_resources(&backend.device, &graph)
        .unwrap();

    assert!(resources
        .owned_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID)
        .is_some());
    assert!(resources
        .owned_texture(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
        )
        .is_some());
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

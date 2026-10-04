use super::RenderGraphExecutionResources;
use crate::graphics::backend::RenderBackend;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBuilder, RenderGraphExternalResourceBinding,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn materialization_validation_reuses_compiled_lifetime_name_index() {
    let source = include_str!("../materialization_validation.rs");
    let ordered_tree_set = ["BTree", "Set"].concat();

    assert!(
        !source.contains(&ordered_tree_set),
        "per-frame validation should not rebuild the compiled lifetime name index"
    );
    assert!(
        source.contains("graph.resource_lifetime_by_name(name)"),
        "stale-binding validation should reuse the compiled graph lookup index"
    );
}

#[test]
fn materialization_validation_reports_unbound_compiled_lifetimes() {
    let mut builder = RenderGraphBuilder::new("unbound-materialization");
    let texture = builder.create_texture(TextureDesc::new(
        "scene-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let buffer = builder.create_buffer(BufferDesc::new(
        "light-list",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let output = builder.import_present_external_resource("viewport-output");
    let pass = builder.add_pass("write", QueueLane::Graphics);
    builder.write_texture(pass, texture).unwrap();
    builder.write_buffer(pass, buffer).unwrap();
    builder.write_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let resources = RenderGraphExecutionResources::new();

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error.contains("buffer `light-list`"));
    assert!(error.contains("texture `scene-color`"));
}

#[test]
fn materialization_validation_reports_unbound_external_lifetimes_without_failing() {
    let mut builder = RenderGraphBuilder::new("external-materialization-report");
    let output = builder.import_present_external_resource("viewport-output");
    let pass = builder.add_pass("write", QueueLane::Graphics);
    builder.write_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let resources = RenderGraphExecutionResources::new();

    let report = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap();

    assert_eq!(report.required_external_count, 0);
    assert_eq!(report.bound_required_external_count, 0);
    assert_eq!(report.missing_required_external_count, 0);
    assert_eq!(report.report_only_external_count, 1);
    assert_eq!(report.bound_report_only_external_count, 0);
    assert_eq!(report.missing_report_only_external_count, 1);
    assert_eq!(report.external_count(), 1);
    assert_eq!(report.bound_external_count(), 0);
    assert_eq!(report.missing_external_count(), 1);
    assert!(report.materialized_resources_complete());
    assert!(!report.is_complete());
}

#[test]
fn materialization_validation_reports_unbound_typed_optional_external_without_failing() {
    let mut builder = RenderGraphBuilder::new("typed-optional-external-report");
    let previous_color = builder.import_present_external_resource_with_binding(
        "history.previous-color",
        RenderGraphExternalResourceBinding::report_only_texture(),
    );
    let previous_exposure = builder.import_present_external_resource_with_binding(
        "history.previous-exposure",
        RenderGraphExternalResourceBinding::report_only_buffer(),
    );
    let pass = builder.add_pass("read", QueueLane::Graphics);
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: true,
                has_side_effects: true,
            },
        )
        .unwrap();
    builder.read_external(pass, previous_color).unwrap();
    builder.read_external(pass, previous_exposure).unwrap();
    let graph = builder.compile().unwrap();
    let resources = RenderGraphExecutionResources::new();

    let report = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap();

    assert_eq!(report.required_external_count, 0);
    assert_eq!(report.bound_required_external_count, 0);
    assert_eq!(report.missing_required_external_count, 0);
    assert_eq!(report.report_only_external_count, 2);
    assert_eq!(report.bound_report_only_external_count, 0);
    assert_eq!(report.missing_report_only_external_count, 2);
    assert_eq!(report.external_count(), 2);
    assert_eq!(report.bound_external_count(), 0);
    assert_eq!(report.missing_external_count(), 2);
    assert!(report.materialized_resources_complete());
    assert!(!report.is_complete());
}

#[test]
fn schema_backed_external_texture_retains_its_compiled_physical_contract() {
    let expected = TextureDesc::new(
        "plugin.compute-output",
        16,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    );
    let mut builder = RenderGraphBuilder::new("schema-backed-external-texture-contract");
    let output = builder.import_present_external_texture_with_binding(
        "plugin.compute-output",
        expected.clone(),
        RenderGraphExternalResourceBinding::report_only_texture(),
    );
    let pass = builder.add_pass("plugin-compute", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();

    assert_eq!(
        graph
            .resource_lifetime_by_name("plugin.compute-output")
            .and_then(|lifetime| lifetime.external_texture_desc.as_ref()),
        Some(&expected)
    );
}

#[test]
fn schema_backed_external_texture_rejects_a_bound_view_without_physical_descriptor() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let expected = TextureDesc::new(
        "plugin.compute-output",
        16,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    );
    let mut builder = RenderGraphBuilder::new("schema-backed-external-texture-missing-desc");
    let output = builder.import_present_external_texture_with_binding(
        "plugin.compute-output",
        expected,
        RenderGraphExternalResourceBinding::report_only_texture(),
    );
    let pass = builder.add_pass("plugin-compute", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("schema-backed-external-texture-missing-desc"),
        size: wgpu::Extent3d {
            width: 16,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_texture_view(
        "plugin.compute-output",
        texture.create_view(&wgpu::TextureViewDescriptor::default()),
    );

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error
        .contains("external texture `plugin.compute-output` is missing its physical descriptor"));
}

#[test]
fn schema_backed_external_texture_rejects_an_incompatible_physical_descriptor() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let expected = TextureDesc::new(
        "plugin.compute-output",
        16,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    );
    let mut builder = RenderGraphBuilder::new("schema-backed-external-texture-mismatch");
    let output = builder.import_present_external_texture_with_binding(
        "plugin.compute-output",
        expected,
        RenderGraphExternalResourceBinding::report_only_texture(),
    );
    let pass = builder.add_pass("plugin-compute", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("schema-backed-external-texture-mismatch"),
        size: wgpu::Extent3d {
            width: 16,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    let actual = TextureDesc::new(
        "plugin.compute-output",
        16,
        8,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_texture_view_with_physical_desc(
        "plugin.compute-output",
        &view,
        actual,
    );

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error.contains("external texture `plugin.compute-output` physical descriptor does not satisfy its compiled contract"));
    assert!(error.contains("format"));
}

#[test]
fn schema_backed_external_texture_accepts_a_physical_descriptor_with_extra_usage() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let expected = TextureDesc::new(
        "plugin.compute-output",
        16,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    );
    let mut builder = RenderGraphBuilder::new("schema-backed-external-texture-compatible");
    let output = builder.import_present_external_texture_with_binding(
        "plugin.compute-output",
        expected,
        RenderGraphExternalResourceBinding::report_only_texture(),
    );
    let pass = builder.add_pass("plugin-compute", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("schema-backed-external-texture-compatible"),
        size: wgpu::Extent3d {
            width: 16,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let actual = TextureDesc::new(
        "plugin.compute-output",
        16,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_SRC,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_texture_view_with_physical_desc(
        "plugin.compute-output",
        &view,
        actual,
    );

    let report = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap();

    assert_eq!(report.bound_report_only_external_count, 1);
}

#[test]
fn schema_backed_external_buffer_rejects_a_bound_buffer_without_physical_descriptor() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let expected = BufferDesc::new(
        "plugin.compute-input",
        16,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    );
    let mut builder = RenderGraphBuilder::new("schema-backed-external-buffer-missing-desc");
    let input = builder.import_present_external_buffer_with_binding(
        "plugin.compute-input",
        expected,
        RenderGraphExternalResourceBinding::report_only_buffer(),
    );
    let pass = builder.add_pass("plugin-compute", QueueLane::AsyncCompute);
    builder.read_external(pass, input).unwrap();
    let graph = builder.compile().unwrap();
    let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("schema-backed-external-buffer-missing-desc"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.insert_buffer("plugin.compute-input", buffer);

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(
        error.contains("external buffer `plugin.compute-input` is missing its physical descriptor")
    );
}

#[test]
fn schema_backed_external_buffer_accepts_a_larger_physical_buffer_with_extra_usage() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let expected = BufferDesc::new(
        "plugin.compute-input",
        16,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    );
    let mut builder = RenderGraphBuilder::new("schema-backed-external-buffer-compatible");
    let input = builder.import_present_external_buffer_with_binding(
        "plugin.compute-input",
        expected,
        RenderGraphExternalResourceBinding::report_only_buffer(),
    );
    let pass = builder.add_pass("plugin-compute", QueueLane::AsyncCompute);
    builder.read_external(pass, input).unwrap();
    let graph = builder.compile().unwrap();
    let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("schema-backed-external-buffer-compatible"),
        size: 32,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_buffer_with_physical_desc(
        "plugin.compute-input",
        &buffer,
        BufferDesc::new(
            "plugin.compute-input",
            32,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        ),
    );

    let report = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap();

    assert_eq!(report.bound_report_only_external_count, 1);
}

#[test]
fn schema_backed_external_buffer_rejects_a_descriptor_larger_than_the_wgpu_buffer() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let expected = BufferDesc::new(
        "plugin.compute-input",
        16,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    );
    let mut builder = RenderGraphBuilder::new("schema-backed-external-buffer-oversized-desc");
    let input = builder.import_present_external_buffer_with_binding(
        "plugin.compute-input",
        expected,
        RenderGraphExternalResourceBinding::report_only_buffer(),
    );
    let pass = builder.add_pass("plugin-compute", QueueLane::AsyncCompute);
    builder.read_external(pass, input).unwrap();
    let graph = builder.compile().unwrap();
    let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("schema-backed-external-buffer-oversized-desc"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_buffer_with_physical_desc(
        "plugin.compute-input",
        &buffer,
        BufferDesc::new(
            "plugin.compute-input",
            32,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        ),
    );

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error.contains("physical WGPU buffer size 16"));
    assert!(error.contains("physical descriptor size 32"));
}

#[test]
fn materialization_validation_reports_required_and_report_only_external_coverage_separately() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut builder = RenderGraphBuilder::new("mixed-external-coverage-report");
    let required_indirect = builder.import_present_external_resource_with_binding(
        "mesh.indirect-args",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let optional_history = builder.import_present_external_resource_with_binding(
        "history.previous-color",
        RenderGraphExternalResourceBinding::report_only_texture(),
    );
    let pass = builder.add_pass("read", QueueLane::Graphics);
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: true,
                has_side_effects: true,
            },
        )
        .unwrap();
    builder.read_external(pass, required_indirect).unwrap();
    builder.read_external(pass, optional_history).unwrap();
    let graph = builder.compile().unwrap();
    let mut resources = RenderGraphExecutionResources::new();
    let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-test-required-external-bound-buffer"),
        size: 64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    resources.insert_buffer("mesh.indirect-args", buffer);

    let report = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap();

    assert_eq!(report.required_external_count, 1);
    assert_eq!(report.bound_required_external_count, 1);
    assert_eq!(report.missing_required_external_count, 0);
    assert_eq!(report.report_only_external_count, 1);
    assert_eq!(report.bound_report_only_external_count, 0);
    assert_eq!(report.missing_report_only_external_count, 1);
    assert_eq!(report.external_count(), 2);
    assert_eq!(report.bound_external_count(), 1);
    assert_eq!(report.missing_external_count(), 1);
    assert!(report.materialized_resources_complete());
    assert!(!report.is_complete());
}

#[test]
fn materialization_validation_fails_unbound_required_external_buffer() {
    let mut builder = RenderGraphBuilder::new("required-external-buffer-materialization");
    let indirect_args = builder.import_present_external_resource_with_binding(
        "mesh.indirect-args",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = builder.add_pass("hzb-occlusion-cull", QueueLane::AsyncCompute);
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: true,
                has_side_effects: true,
            },
        )
        .unwrap();
    builder.read_external(pass, indirect_args).unwrap();
    let graph = builder.compile().unwrap();
    let resources = RenderGraphExecutionResources::new();

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error.contains("required external resource bindings"));
    assert!(error.contains("external buffer `mesh.indirect-args`"));
}

#[test]
fn materialization_validation_fails_unbound_required_external_texture() {
    let mut builder = RenderGraphBuilder::new("required-external-texture-materialization");
    let previous_hzb = builder.import_present_external_resource_with_binding(
        "history.previous-hzb",
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass = builder.add_pass("hzb-occlusion-cull", QueueLane::AsyncCompute);
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: true,
                has_side_effects: true,
            },
        )
        .unwrap();
    builder.read_external(pass, previous_hzb).unwrap();
    let graph = builder.compile().unwrap();
    let resources = RenderGraphExecutionResources::new();

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error.contains("required external resource bindings"));
    assert!(error.contains("external texture `history.previous-hzb`"));
}

#[test]
fn materialization_validation_rejects_stale_texture_binding_outside_live_lifetimes() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut builder = RenderGraphBuilder::new("stale-texture-binding-validation");
    let live_output = builder.import_present_external_resource("viewport-output");
    let pass = builder.add_pass("write", QueueLane::Graphics);
    builder.write_external(pass, live_output).unwrap();
    let graph = builder.compile().unwrap();
    let stale_texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-test-stale-graph-texture"),
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_texture_view(
        "culled-scene-color",
        stale_texture.create_view(&wgpu::TextureViewDescriptor::default()),
    );

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error.contains("stale resource bindings outside live compiled lifetimes"));
    assert!(error.contains("texture `culled-scene-color`"));
}

#[test]
fn materialization_validation_rejects_stale_buffer_binding_outside_live_lifetimes() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut builder = RenderGraphBuilder::new("stale-buffer-binding-validation");
    let live_output = builder.import_present_external_resource("viewport-output");
    let pass = builder.add_pass("write", QueueLane::Graphics);
    builder.write_external(pass, live_output).unwrap();
    let graph = builder.compile().unwrap();
    let stale_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-test-stale-graph-buffer"),
        size: 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut resources = RenderGraphExecutionResources::new();
    resources.insert_buffer("culled-light-list", stale_buffer);

    let error = resources
        .validate_materialized_graph_resources(&graph)
        .unwrap_err();

    assert!(error.contains("stale resource bindings outside live compiled lifetimes"));
    assert!(error.contains("buffer `culled-light-list`"));
}

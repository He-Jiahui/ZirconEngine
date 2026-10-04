use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderFrameExtract, RenderPluginRendererOutputs,
    RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::backend::{read_buffer_bytes, BufferByteReadback, RenderBackend};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassExecutorId, RenderPassExecutorRegistry,
};
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::ViewportRenderFrame;
use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphBufferRange,
    RenderGraphBuilder, RenderGraphComputePassMetadata, RenderGraphComputeShaderSource,
    RenderGraphComputeWorkload, RenderGraphExternalResourceBinding,
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessRange, RenderGraphShaderStages,
};
use crate::rhi::{BufferDesc, BufferUsage};
use crate::scene::world::World;
use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

use super::{
    direct_dispatch, expand_wgsl_modules, per_pixel_dispatch, resolved_wgsl_source,
    storage_write_resources, ComputeDispatch, RenderPassExecutionContext,
    RenderPassGpuExecutionContext, COMPUTE_GENERIC_EXECUTOR_ID,
};

#[test]
fn generic_compute_native_resource_creates_use_the_pass_factory() {
    let executor = include_str!("../generic_compute_executor.rs");
    let production = executor
        .split_once("#[cfg(test)]")
        .map(|(source, _)| source)
        .expect("generic compute production source must precede tests");
    let cache = include_str!("../compute_pipeline_cache.rs");

    assert!(production.contains("RenderPassGpuResourceFactory"));
    assert!(
        production.contains("pipeline_cache.resolve(\n            gpu.device,\n            gpu,")
    );
    assert!(production.contains("gpu.create_bind_group(&wgpu::BindGroupDescriptor"));
    assert!(!production.contains("gpu.device.create_bind_group"));
    assert!(cache.contains("resource_factory: &impl RenderPassGpuResourceFactory"));
    for create in [
        "resource_factory.create_bind_group_layout",
        "resource_factory.create_pipeline_layout",
        "resource_factory.create_shader_module",
        "resource_factory.create_compute_pipeline",
    ] {
        assert!(cache.contains(create), "missing factory route `{create}`");
    }
}

#[test]
fn storage_write_scan_reports_outputs_without_inventing_history_ownership() {
    let metadata = RenderGraphComputePassMetadata::new(
        RenderGraphComputeShaderSource::wgsl("ssao", "@compute fn cs_main() {}"),
        "cs_main",
        vec![BindingSchemaEntry::new(
            0,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            ComputeBindingKind::StorageTextureWrite,
        )],
    );

    let writes = storage_write_resources(&metadata);

    assert_eq!(
        writes.resources,
        vec![PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string()]
    );
    let production = include_str!("../generic_compute_executor.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("generic compute executor source");
    assert!(!production.contains("FrameHistorySlot::AmbientOcclusion"));
}

#[test]
fn generic_compute_expands_project_shader_modules_before_pipeline_creation() {
    let source = "#include <project::compute::math>\n@compute fn cs_main() {}";
    let expanded = expand_wgsl_modules(
        source,
        vec![
            crate::graphics::shader::template::ShaderTemplateInclude::new(
                "project::compute::math",
                "fn compute_identity(value: u32) -> u32 { return value; }",
            ),
        ],
    )
    .expect("project module source expands");

    assert!(!expanded.contains("#include"));
    assert!(expanded.contains("compute_identity"));
    assert!(expanded.contains("fn cs_main"));
}

#[test]
fn generic_compute_asset_shader_reports_missing_streamer_context() {
    let metadata = RenderGraphComputePassMetadata::new(
        RenderGraphComputeShaderSource::asset(AssetReference::from_locator(
            ResourceLocator::parse("res://shaders/compute/reduce.zshader").unwrap(),
        )),
        "cs_main",
        Vec::new(),
    );

    let Err(error) = resolved_wgsl_source("reduce", None, &metadata) else {
        panic!("an asset shader without a resource streamer must fail");
    };
    assert!(error.contains("has no resource streamer"));
    assert!(error.contains("res://shaders/compute/reduce.zshader"));
}

#[test]
fn generic_compute_rejects_direct_dispatch_outside_device_limits() {
    let limits = wgpu::Limits {
        max_compute_workgroups_per_dimension: 64,
        ..wgpu::Limits::default()
    };
    let workload = RenderGraphComputeWorkload::fixed("oversized-dispatch", [1, 1, 1], [65, 1, 1]);

    let Err(error) = direct_dispatch(&workload, [65, 1, 1], &limits) else {
        panic!("direct dispatch beyond the device limit must fail");
    };

    assert!(error.contains("oversized-dispatch"));
    assert!(error.contains("per-dimension limit 64"));
}

#[test]
fn generic_compute_per_pixel_groups_match_target_extent() {
    let workload =
        RenderGraphComputeWorkload::per_pixel("per-pixel-reduce", [8, 8, 1], "scene-color", [8, 8]);

    let dispatch = per_pixel_dispatch(&workload, [1_920, 1_080], [8, 8], &wgpu::Limits::default())
        .expect("valid per-pixel dispatch should fit default limits");

    let ComputeDispatch::Direct(groups) = dispatch else {
        panic!("per-pixel dispatch must be direct");
    };
    assert_eq!(groups, [240, 135, 1]);
}

#[test]
fn generic_executor_records_fixed_storage_dispatch() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut graph_builder = RenderGraphBuilder::new("generic-compute-dispatch");
    let output = graph_builder.import_present_external_resource_with_binding(
        "output",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass_id = graph_builder.add_pass_with_executor(
        "generic-reduce",
        QueueLane::AsyncCompute,
        Some(COMPUTE_GENERIC_EXECUTOR_ID),
    );
    graph_builder.read_external(pass_id, output).unwrap();
    graph_builder.write_external(pass_id, output).unwrap();
    graph_builder
        .set_pass_flags(
            pass_id,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph_builder
        .set_compute_workload(
            pass_id,
            RenderGraphComputeWorkload::fixed("generic-reduce", [1, 1, 1], [4, 1, 1]),
        )
        .unwrap();
    graph_builder
        .set_compute_pass_metadata(
            pass_id,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl(
                    "generic-reduce",
                    "@group(1) @binding(0) var<storage, read_write> output: array<u32>;\n@compute @workgroup_size(1) fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) { output[invocation_id.x] = invocation_id.x; }",
                ),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "output",
                    ComputeBindingKind::StorageBufferReadWrite,
                )],
            ),
        )
        .unwrap();
    let graph = graph_builder.compile().unwrap();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "generic-reduce")
        .unwrap();

    let mut resources = RenderGraphExecutionResources::new();
    let output_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("generic-compute-output"),
        size: 16,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    resources.insert_buffer("output", output_buffer.clone());
    resources
        .materialize_external_access_bindings(&graph)
        .expect("external generic-compute leases should be materialized before encoding");
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("generic-compute-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("generic-compute-empty-scene-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("generic-compute-empty-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
        &scene_bind_group,
        &resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_resource_resolver(&graph, pass.id)
        .with_compute_workload(pass.compute_workload.as_ref())
        .with_compute_pass_metadata(pass.compute_pass_metadata.as_ref())
        .with_compute_binding_access_packet(graph.compute_binding_access_packet(pass.id))
        .with_gpu(gpu);

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap();
    let dispatches = context.gpu_mut().unwrap().take_compute_dispatches();
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].pipeline_label, "generic-reduce");
    assert_eq!(dispatches[0].dispatch_groups, [4, 1, 1]);
    assert_eq!(
        dispatches[0].storage_write_resources,
        ["output".to_string()]
    );
    drop(context);
    backend.queue.submit([encoder.finish()]);
    let output_bytes = read_buffer_bytes(
        &backend.device,
        &backend.queue,
        &output_buffer,
        BufferByteReadback {
            source_offset: 0,
            byte_len: 16,
            label: "generic-compute-output-readback",
        },
    )
    .expect("generic compute output should be readable after submission");
    assert_eq!(
        bytemuck::cast_slice::<u8, u32>(&output_bytes),
        &[0, 1, 2, 3]
    );
}

#[test]
fn generic_executor_records_indirect_storage_dispatch() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut graph_builder = RenderGraphBuilder::new("generic-compute-indirect-dispatch");
    let dispatch_args = graph_builder.import_present_external_buffer_with_binding(
        "dispatch-args",
        BufferDesc::new(
            "dispatch-args",
            12,
            BufferUsage::STORAGE | BufferUsage::INDIRECT | BufferUsage::COPY_DST,
        ),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let output = graph_builder.import_present_external_resource_with_binding(
        "indirect-output",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass_id = graph_builder.add_pass_with_executor(
        "generic-indirect",
        QueueLane::AsyncCompute,
        Some(COMPUTE_GENERIC_EXECUTOR_ID),
    );
    graph_builder
        .read_external_with_access(
            pass_id,
            dispatch_args,
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(0, Some(12))),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    graph_builder.read_external(pass_id, output).unwrap();
    graph_builder.write_external(pass_id, output).unwrap();
    graph_builder
        .set_pass_flags(
            pass_id,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph_builder
        .set_compute_workload(
            pass_id,
            RenderGraphComputeWorkload::from_buffer(
                "generic-indirect",
                [1, 1, 1],
                "dispatch-args",
                0,
            ),
        )
        .unwrap();
    graph_builder
        .set_compute_pass_metadata(
            pass_id,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl(
                    "generic-indirect",
                    "@group(1) @binding(0) var<storage, read> dispatch_args: array<u32>;\n@group(1) @binding(1) var<storage, read_write> output: array<u32>;\n@compute @workgroup_size(1) fn cs_main() { if (dispatch_args[0] == 0u) { return; } output[0] = dispatch_args[1]; }",
                ),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(
                        0,
                        "dispatch-args",
                        ComputeBindingKind::StorageBufferRead,
                    ),
                    BindingSchemaEntry::new(
                        1,
                        "indirect-output",
                        ComputeBindingKind::StorageBufferReadWrite,
                    ),
                ],
            ),
        )
        .unwrap();
    let graph = graph_builder.compile().unwrap();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "generic-indirect")
        .unwrap();

    let mut resources = RenderGraphExecutionResources::new();
    let indirect_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("generic-compute-indirect-arguments"),
        size: 12,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    backend
        .queue
        .write_buffer(&indirect_buffer, 0, &1_u32.to_le_bytes());
    backend
        .queue
        .write_buffer(&indirect_buffer, 4, &1_u32.to_le_bytes());
    backend
        .queue
        .write_buffer(&indirect_buffer, 8, &1_u32.to_le_bytes());
    resources.insert_buffer("dispatch-args", indirect_buffer);
    let output_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("generic-compute-indirect-output"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    resources.insert_buffer("indirect-output", output_buffer.clone());
    resources
        .materialize_external_access_bindings(&graph)
        .expect("external generic-compute leases should be materialized before encoding");
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("generic-compute-indirect-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("generic-compute-empty-scene-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("generic-compute-empty-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
        &scene_bind_group,
        &resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_resource_resolver(&graph, pass.id)
        .with_compute_workload(pass.compute_workload.as_ref())
        .with_compute_pass_metadata(pass.compute_pass_metadata.as_ref())
        .with_compute_binding_access_packet(graph.compute_binding_access_packet(pass.id))
        .with_compute_dispatch_access_packet(graph.compute_dispatch_access_packet(pass.id))
        .with_gpu(gpu);

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap();
    let dispatches = context.gpu_mut().unwrap().take_compute_dispatches();
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0].pipeline_label, "generic-indirect");
    assert!(!dispatches[0].dispatch_groups_known);
    assert_eq!(dispatches[0].dispatch_groups, [0, 1, 1]);
    assert_eq!(
        dispatches[0].storage_write_resources,
        ["indirect-output".to_string()]
    );
    drop(context);
    backend.queue.submit([encoder.finish()]);
    let output_bytes = read_buffer_bytes(
        &backend.device,
        &backend.queue,
        &output_buffer,
        BufferByteReadback {
            source_offset: 0,
            byte_len: 4,
            label: "generic-compute-indirect-output-readback",
        },
    )
    .expect("indirect compute output should be readable after submission");
    assert_eq!(bytemuck::cast_slice::<u8, u32>(&output_bytes), &[1]);
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

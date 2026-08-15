use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{strip_wgsl_include_directives, wgsl_include_paths};
use crate::graphics::feature::COMPUTE_GENERIC_EXECUTOR_ID;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::shader::template::{ShaderModuleRegistry, ShaderTemplateInclude};
use crate::render_graph::{
    ComputeBindingKind, RenderGraphComputeDispatchExtent, RenderGraphComputePassMetadata,
    RenderGraphComputeShaderSource, RenderGraphComputeWorkload, RenderGraphResourceAccessKind,
};

use super::compute_pipeline_cache::ComputePipelineCache;
use super::{RenderPassExecutionContext, RenderPassExecutor, RenderPassGpuExecutionContext};

mod binding_resolver;
mod buffer_binding;
mod per_pixel_extent;
mod texture_view;

use binding_resolver::{resolve_bindings, ResolvedComputeBinding};
use per_pixel_extent::per_pixel_target_extent;
use texture_view::resolve_compute_texture_view;

pub(super) fn generic_compute_executor() -> Arc<dyn RenderPassExecutor> {
    Arc::new(GenericComputeExecutor::default())
}

#[derive(Default)]
struct GenericComputeExecutor {
    pipeline_cache: Mutex<ComputePipelineCache>,
}

impl RenderPassExecutor for GenericComputeExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        let pass_name = context.pass_name.clone();
        let metadata = context.compute_pass_metadata().ok_or_else(|| {
            format!(
                "compute executor `{COMPUTE_GENERIC_EXECUTOR_ID}` for pass `{}` requires compute pass metadata",
                pass_name
            )
        })?;
        let workload = context.compute_workload().ok_or_else(|| {
            format!(
                "compute executor `{COMPUTE_GENERIC_EXECUTOR_ID}` for pass `{}` requires a compute workload",
                pass_name
            )
        })?;
        let per_pixel_access = per_pixel_resource_access(context, workload);
        let context_streamer = context.resource_streamer();
        let gpu = context.require_gpu()?;
        let streamer = context_streamer.or_else(|| gpu.resource_streamer());
        let bindings = resolve_bindings(gpu, metadata)?;
        let shader = resolved_wgsl_source(&pass_name, streamer, metadata)?;
        let dispatch = resolve_dispatch(gpu, metadata, workload, per_pixel_access)?;
        let storage_write_resources = storage_write_resources(metadata);

        let mut pipeline_cache = self
            .pipeline_cache
            .lock()
            .map_err(|_| "generic compute pipeline cache lock is poisoned".to_string())?;
        let binding_layouts = bindings
            .iter()
            .map(|binding| binding.layout.clone())
            .collect::<Vec<_>>();
        let (pipeline, bind_group_layout) = pipeline_cache.get_or_create(
            gpu.device,
            gpu.scene_bind_group_layout(),
            &shader.label,
            shader.source.as_ref(),
            &metadata.entry_point,
            workload.workgroup_size,
            &binding_layouts,
        )?;
        drop(pipeline_cache);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&shader.label),
            layout: &bind_group_layout,
            entries: &bindings
                .iter()
                .map(ResolvedComputeBinding::bind_group_entry)
                .collect::<Vec<_>>(),
        });
        let mut compute_pass = gpu
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(&shader.label),
                timestamp_writes: None,
            });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, gpu.scene_bind_group, &[]);
        compute_pass.set_bind_group(1, &bind_group, &[]);
        match &dispatch {
            ComputeDispatch::Direct(groups) => {
                compute_pass.dispatch_workgroups(groups[0], groups[1], groups[2]);
            }
            ComputeDispatch::Indirect { buffer, offset } => {
                compute_pass.dispatch_workgroups_indirect(buffer, *offset);
            }
        }
        drop(compute_pass);

        match dispatch {
            ComputeDispatch::Direct(groups) => gpu.record_compute_dispatch(
                &pass_name,
                COMPUTE_GENERIC_EXECUTOR_ID,
                &shader.label,
                workload.workgroup_size,
                groups,
                storage_write_resources,
            ),
            ComputeDispatch::Indirect { .. } => gpu.record_indirect_compute_dispatch(
                &pass_name,
                COMPUTE_GENERIC_EXECUTOR_ID,
                &shader.label,
                workload.workgroup_size,
                storage_write_resources,
            ),
        }
        Ok(())
    }
}

struct ResolvedComputeShaderSource<'a> {
    label: String,
    source: Cow<'a, str>,
}

fn resolved_wgsl_source<'a>(
    pass_name: &str,
    streamer: Option<&'a ResourceStreamer>,
    metadata: &'a RenderGraphComputePassMetadata,
) -> Result<ResolvedComputeShaderSource<'a>, String> {
    match &metadata.shader {
        RenderGraphComputeShaderSource::Wgsl { label, source } => Ok(ResolvedComputeShaderSource {
            label: label.clone(),
            source: expand_wgsl_modules(source, Vec::new())?,
        }),
        RenderGraphComputeShaderSource::Asset { asset } => {
            let streamer = streamer.ok_or_else(|| {
                format!(
                    "compute pass `{pass_name}` references asset shader `{asset}`, but its GPU context has no resource streamer"
                )
            })?;
            let asset_manager = streamer.asset_manager().map_err(|error| {
                format!("compute pass `{pass_name}` cannot access shader asset `{asset}`: {error}")
            })?;
            let shader_id = asset_manager
                .resolve_asset_id(&asset.locator)
                .ok_or_else(|| {
                    format!(
                        "compute pass `{pass_name}` references unresolved shader asset `{asset}`"
                    )
                })?;
            let source = streamer.shader_source(&shader_id).ok_or_else(|| {
                format!(
                    "compute pass `{pass_name}` shader asset `{asset}` is not prepared; prepare it through ResourceStreamer before graph execution"
                )
            })?;
            Ok(ResolvedComputeShaderSource {
                label: format!("compute.asset:{asset}"),
                source: expand_wgsl_modules(
                    source,
                    streamer.shader_module_include_sources(&shader_id),
                )?,
            })
        }
    }
}

fn expand_wgsl_modules<'a>(
    source: &'a str,
    module_includes: Vec<ShaderTemplateInclude>,
) -> Result<Cow<'a, str>, String> {
    let roots = wgsl_include_paths(source);
    if roots.is_empty() {
        return Ok(Cow::Borrowed(source));
    }
    let registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
        roots.iter().cloned(),
        module_includes,
    );
    let resolved = registry
        .resolve_roots(roots)
        .map_err(|error| format!("compute shader module resolution failed: {error:?}"))?;
    let mut expanded = String::with_capacity(
        source.len()
            + resolved
                .ordered_sources
                .iter()
                .map(|include| include.source.len() + include.token.len() + 16)
                .sum::<usize>(),
    );
    for include in resolved.ordered_sources {
        expanded.push_str("// include: ");
        expanded.push_str(&include.token);
        expanded.push('\n');
        expanded.push_str(&include.source);
        expanded.push('\n');
    }
    expanded.push_str(&strip_wgsl_include_directives(source));
    Ok(Cow::Owned(expanded))
}

fn per_pixel_resource_access(
    context: &RenderPassExecutionContext<'_>,
    workload: &RenderGraphComputeWorkload,
) -> Option<RenderGraphResourceAccessKind> {
    let RenderGraphComputeDispatchExtent::PerPixel { target, .. } = &workload.dispatch_extent
    else {
        return None;
    };
    if context.declares_resource_name_access(target, RenderGraphResourceAccessKind::Write) {
        Some(RenderGraphResourceAccessKind::Write)
    } else {
        Some(RenderGraphResourceAccessKind::Read)
    }
}

enum ComputeDispatch {
    Direct([u32; 3]),
    Indirect { buffer: wgpu::Buffer, offset: u64 },
}

fn resolve_dispatch(
    gpu: &RenderPassGpuExecutionContext<'_>,
    metadata: &RenderGraphComputePassMetadata,
    workload: &RenderGraphComputeWorkload,
    per_pixel_access: Option<RenderGraphResourceAccessKind>,
) -> Result<ComputeDispatch, String> {
    match &workload.dispatch_extent {
        RenderGraphComputeDispatchExtent::Fixed(groups) => {
            direct_dispatch(workload, *groups, &gpu.device.limits())
        }
        RenderGraphComputeDispatchExtent::FromBuffer { buffer, offset } => {
            Ok(ComputeDispatch::Indirect {
                buffer: gpu
                    .require_buffer(buffer, RenderGraphResourceAccessKind::Read)?
                    .clone(),
                offset: *offset,
            })
        }
        RenderGraphComputeDispatchExtent::PerPixel { target, local_size } => {
            let access = per_pixel_access.unwrap_or(RenderGraphResourceAccessKind::Read);
            let extent = gpu.require_texture_desc(target, access)?;
            per_pixel_dispatch(
                workload,
                per_pixel_target_extent(metadata, target, &extent)?,
                *local_size,
                &gpu.device.limits(),
            )
        }
        unsupported_extent => Err(format!(
            "compute executor `{COMPUTE_GENERIC_EXECUTOR_ID}` does not support dispatch extent {unsupported_extent:?}"
        )),
    }
}

fn per_pixel_dispatch(
    workload: &RenderGraphComputeWorkload,
    target_extent: [u32; 2],
    local_size: [u32; 2],
    limits: &wgpu::Limits,
) -> Result<ComputeDispatch, String> {
    direct_dispatch(
        workload,
        [
            target_extent[0].max(1).div_ceil(local_size[0].max(1)),
            target_extent[1].max(1).div_ceil(local_size[1].max(1)),
            1,
        ],
        limits,
    )
}

fn direct_dispatch(
    workload: &RenderGraphComputeWorkload,
    groups: [u32; 3],
    limits: &wgpu::Limits,
) -> Result<ComputeDispatch, String> {
    if groups
        .iter()
        .any(|group_count| *group_count > limits.max_compute_workgroups_per_dimension)
    {
        return Err(format!(
            "compute pipeline `{}` direct dispatch groups {groups:?} exceed the device per-dimension limit {}",
            workload.pipeline_label, limits.max_compute_workgroups_per_dimension
        ));
    }
    Ok(ComputeDispatch::Direct(groups))
}

fn storage_write_resources(metadata: &RenderGraphComputePassMetadata) -> Vec<String> {
    metadata
        .bindings
        .iter()
        .filter(|binding| {
            matches!(
                binding.kind,
                ComputeBindingKind::StorageBufferReadWrite
                    | ComputeBindingKind::StorageTextureWrite
            )
        })
        .map(|binding| binding.resource.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::asset::ProjectAssetManager;
    use crate::core::framework::render::{
        RenderFrameExtract, RenderPluginRendererOutputs, RenderWorldSnapshotHandle,
    };
    use crate::core::math::UVec2;
    use crate::graphics::backend::{read_buffer_bytes, BufferByteReadback, RenderBackend};
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphExecutionResources, RenderPassExecutorId, RenderPassExecutorRegistry,
    };
    use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
    use crate::graphics::ViewportRenderFrame;
    use crate::render_graph::{
        BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphBuilder,
        RenderGraphComputePassMetadata, RenderGraphComputeShaderSource, RenderGraphComputeWorkload,
        RenderGraphExternalResourceBinding,
    };
    use crate::scene::world::World;
    use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

    use super::{
        direct_dispatch, expand_wgsl_modules, per_pixel_dispatch, resolved_wgsl_source,
        ComputeDispatch, RenderPassExecutionContext, RenderPassGpuExecutionContext,
        COMPUTE_GENERIC_EXECUTOR_ID,
    };

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

        let error = resolved_wgsl_source("reduce", None, &metadata).unwrap_err();
        assert!(error.contains("has no resource streamer"));
        assert!(error.contains("res://shaders/compute/reduce.zshader"));
    }

    #[test]
    fn generic_compute_rejects_direct_dispatch_outside_device_limits() {
        let limits = wgpu::Limits {
            max_compute_workgroups_per_dimension: 64,
            ..wgpu::Limits::default()
        };
        let workload =
            RenderGraphComputeWorkload::fixed("oversized-dispatch", [1, 1, 1], [65, 1, 1]);

        let error = direct_dispatch(&workload, [65, 1, 1], &limits)
            .expect_err("direct dispatch beyond the device limit must fail");

        assert!(error.contains("oversized-dispatch"));
        assert!(error.contains("per-dimension limit 64"));
    }

    #[test]
    fn generic_compute_per_pixel_groups_match_target_extent() {
        let workload = RenderGraphComputeWorkload::per_pixel(
            "per-pixel-reduce",
            [8, 8, 1],
            "scene-color",
            [8, 8],
        );

        let dispatch =
            per_pixel_dispatch(&workload, [1_920, 1_080], [8, 8], &wgpu::Limits::default())
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
        let output = graph_builder.import_external_resource_with_binding(
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
        let dispatch_args = graph_builder.import_external_resource_with_binding(
            "dispatch-args",
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let output = graph_builder.import_external_resource_with_binding(
            "indirect-output",
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass_id = graph_builder.add_pass_with_executor(
            "generic-indirect",
            QueueLane::AsyncCompute,
            Some(COMPUTE_GENERIC_EXECUTOR_ID),
        );
        graph_builder.read_external(pass_id, dispatch_args).unwrap();
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
            .write_buffer(&indirect_buffer, 0, &1_u32.to_ne_bytes());
        backend
            .queue
            .write_buffer(&indirect_buffer, 4, &1_u32.to_ne_bytes());
        backend
            .queue
            .write_buffer(&indirect_buffer, 8, &1_u32.to_ne_bytes());
        resources.insert_buffer("dispatch-args", indirect_buffer);
        let output_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("generic-compute-indirect-output"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        resources.insert_buffer("indirect-output", output_buffer.clone());
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
}

#[cfg(test)]
mod per_pixel_product;

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{strip_wgsl_include_directives, wgsl_include_paths};
use crate::graphics::feature::COMPUTE_GENERIC_EXECUTOR_ID;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::shader::template::{ShaderModuleRegistry, ShaderTemplateInclude};
use crate::render_graph::{
    CompiledRenderGraphComputeDispatchAccess, CompiledRenderGraphComputeDispatchAccessPacket,
    ComputeBindingKind, RenderGraphComputeDispatchExtent, RenderGraphComputePassMetadata,
    RenderGraphComputeShaderSource, RenderGraphComputeWorkload, RenderGraphResourceKind,
    RenderGraphVersionedAccessKey,
};

use super::compute_pipeline_cache::ComputePipelineCache;
use super::{
    RenderGraphComputeDispatchRecord, RenderPassExecutionContext, RenderPassExecutor,
    RenderPassGpuExecutionContext, RenderPassGpuResourceFactory,
};

mod binding_resolver;
mod buffer_binding;
mod texture_view;

use binding_resolver::{resolve_bindings, ResolvedComputeBinding};

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
        let binding_access_packet = context.compute_binding_access_packet().ok_or_else(|| {
            format!(
                "compute executor `{COMPUTE_GENERIC_EXECUTOR_ID}` for pass `{}` requires a compiler binding access packet",
                pass_name
            )
        })?;
        let dispatch_access_packet = context.compute_dispatch_access_packet();
        let context_streamer = context.resource_streamer();
        let gpu = context.require_gpu()?;
        let streamer = context_streamer.or_else(|| gpu.resource_streamer());
        let bindings = resolve_bindings(gpu, metadata, binding_access_packet)?;
        let shader = resolved_wgsl_source(&pass_name, streamer, metadata)?;
        let dispatch = resolve_dispatch(gpu, workload, dispatch_access_packet)?;
        let storage_write_resources = storage_write_resources(metadata);

        let mut pipeline_cache = self
            .pipeline_cache
            .lock()
            .map_err(|_| "generic compute pipeline cache lock is poisoned".to_string())?;
        let binding_layouts = bindings
            .iter()
            .map(|binding| binding.layout.clone())
            .collect::<Vec<_>>();
        let resolved_pipeline = pipeline_cache.resolve(
            gpu.device,
            gpu,
            gpu.scene_bind_group_layout(),
            &shader.label,
            shader.source.as_ref(),
            &metadata.entry_point,
            workload.workgroup_size,
            &binding_layouts,
            &workload.pipeline_fallback_policy,
            gpu.resources.device_epoch(),
        )?;
        drop(pipeline_cache);
        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&shader.label),
            layout: &resolved_pipeline.bind_group_layout,
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
        compute_pass.set_pipeline(&resolved_pipeline.pipeline);
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

        let dispatch_record = match dispatch {
            ComputeDispatch::Direct(groups) => RenderGraphComputeDispatchRecord::new(
                &pass_name,
                COMPUTE_GENERIC_EXECUTOR_ID,
                &shader.label,
                workload.workgroup_size,
                groups,
                storage_write_resources.resources,
            ),
            ComputeDispatch::Indirect { .. } => RenderGraphComputeDispatchRecord::new(
                &pass_name,
                COMPUTE_GENERIC_EXECUTOR_ID,
                &shader.label,
                workload.workgroup_size,
                [0, 1, 1],
                storage_write_resources.resources,
            )
            .with_gpu_indirect_dispatch_groups(),
        }
        .with_pipeline_resolution(resolved_pipeline.resolution);
        gpu.push_compute_dispatch_record(dispatch_record);
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

enum ComputeDispatch {
    Direct([u32; 3]),
    Indirect { buffer: wgpu::Buffer, offset: u64 },
}

fn resolve_dispatch(
    gpu: &RenderPassGpuExecutionContext<'_>,
    workload: &RenderGraphComputeWorkload,
    dispatch_access_packet: Option<&CompiledRenderGraphComputeDispatchAccessPacket>,
) -> Result<ComputeDispatch, String> {
    match &workload.dispatch_extent {
        RenderGraphComputeDispatchExtent::Fixed(groups) => {
            direct_dispatch(workload, *groups, &gpu.device.limits())
        }
        RenderGraphComputeDispatchExtent::FromBuffer { .. } => {
            let packet = require_dispatch_access_packet(workload, dispatch_access_packet)?;
            let CompiledRenderGraphComputeDispatchAccess::Indirect { access, offset } =
                packet.dispatch
            else {
                return Err(dispatch_packet_kind_mismatch(workload, "indirect buffer"));
            };
            Ok(ComputeDispatch::Indirect {
                buffer: resolve_dispatch_buffer(gpu, access)?,
                offset,
            })
        }
        RenderGraphComputeDispatchExtent::PerPixel { .. } => {
            let packet = require_dispatch_access_packet(workload, dispatch_access_packet)?;
            let CompiledRenderGraphComputeDispatchAccess::PerPixel {
                target_extent,
                local_size,
                ..
            } = packet.dispatch
            else {
                return Err(dispatch_packet_kind_mismatch(workload, "per-pixel texture"));
            };
            per_pixel_dispatch(workload, target_extent, local_size, &gpu.device.limits())
        }
        unsupported_extent => Err(format!(
            "compute executor `{COMPUTE_GENERIC_EXECUTOR_ID}` does not support dispatch extent {unsupported_extent:?}"
        )),
    }
}

fn require_dispatch_access_packet<'a>(
    workload: &RenderGraphComputeWorkload,
    packet: Option<&'a CompiledRenderGraphComputeDispatchAccessPacket>,
) -> Result<&'a CompiledRenderGraphComputeDispatchAccessPacket, String> {
    packet.ok_or_else(|| {
        format!(
            "compute pipeline `{}` dynamic dispatch requires a compiler dispatch access packet",
            workload.pipeline_label
        )
    })
}

fn dispatch_packet_kind_mismatch(workload: &RenderGraphComputeWorkload, expected: &str) -> String {
    format!(
        "compute pipeline `{}` compiler dispatch access packet does not select an {expected}",
        workload.pipeline_label
    )
}

fn resolve_dispatch_buffer(
    gpu: &RenderPassGpuExecutionContext<'_>,
    access: RenderGraphVersionedAccessKey,
) -> Result<wgpu::Buffer, String> {
    match access.resource.kind() {
        RenderGraphResourceKind::TransientBuffer => gpu
            .resources
            .transient_buffer_binding_for_access(access.access_id)
            .map(|(buffer, _)| buffer.clone()),
        RenderGraphResourceKind::External => gpu
            .resources
            .external_buffer_binding_for_access(access.access_id)
            .map(|(buffer, _)| buffer.clone()),
        RenderGraphResourceKind::TransientTexture => Err(format!(
            "compute dynamic dispatch access {:?} resolves to a transient texture, not a buffer",
            access.access_id
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

struct StorageWriteResources {
    resources: Vec<String>,
}

fn storage_write_resources(metadata: &RenderGraphComputePassMetadata) -> StorageWriteResources {
    let mut resources = Vec::new();
    for binding in &metadata.bindings {
        if !matches!(
            binding.kind,
            ComputeBindingKind::StorageBufferReadWrite | ComputeBindingKind::StorageTextureWrite
        ) {
            continue;
        }
        resources.push(binding.resource.clone());
    }
    StorageWriteResources { resources }
}

#[cfg(test)]
#[path = "tests/generic_compute_executor.rs"]
mod tests;

#[cfg(test)]
#[path = "generic_compute_executor/tests/per_pixel_product.rs"]
mod per_pixel_product;

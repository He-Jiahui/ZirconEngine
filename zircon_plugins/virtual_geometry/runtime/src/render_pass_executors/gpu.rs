use std::{
    borrow::Cow,
    collections::BTreeMap,
    mem::size_of,
    sync::{Mutex, OnceLock},
};

use bytemuck::{Pod, Zeroable};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryDebugSnapshot, ViewProjectionMatrixPair,
};
use zircon_runtime::core::math::Mat4;
use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RenderPassDeviceEpoch, RenderPassGpuExecutionContext,
    RenderPassGpuResourceFactory,
};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

const PAGE_REQUESTS: &str = "virtual-geometry-page-requests";
const VISIBLE_CLUSTERS: &str = "virtual-geometry-visible-clusters";
const FEEDBACK: &str = "virtual-geometry-feedback";
const SCENE_DEPTH: &str = "scene-depth";
const SCENE_COLOR: &str = "scene-color";
const WORKGROUP_SIZE: u32 = 64;
const NODE_CULL_DISPATCH_GROUPS: u32 = 64;
const NODE_CULL_PASS_NAME: &str = "virtual-geometry-node-cluster-cull";
const NODE_CULL_EXECUTOR_ID: &str = "virtual-geometry.node-cluster-cull";
const NODE_CULL_PIPELINE_LABEL: &str = "zircon-virtual-geometry-node-cluster-cull";

struct NodeCullPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClusterTrianglePipelineKey {
    device_epoch: RenderPassDeviceEpoch,
    target_format: Option<wgpu::TextureFormat>,
    depth_format: Option<wgpu::TextureFormat>,
}

struct ClusterTrianglePipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

static NODE_CULL_PIPELINE: OnceLock<Mutex<Option<(RenderPassDeviceEpoch, NodeCullPipeline)>>> =
    OnceLock::new();
static DEPTH_CLUSTER_TRIANGLE_PIPELINE: OnceLock<
    Mutex<Option<(ClusterTrianglePipelineKey, ClusterTrianglePipeline)>>,
> = OnceLock::new();
static COLOR_CLUSTER_TRIANGLE_PIPELINE: OnceLock<
    Mutex<Option<(ClusterTrianglePipelineKey, ClusterTrianglePipeline)>>,
> = OnceLock::new();

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ClusterVertex {
    clip_position: [f32; 4],
    cluster_id: u32,
    _padding: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ClusterCandidate {
    cluster_id: u32,
    page_id: u32,
}

pub(super) fn execute_prepare(gpu: &mut RenderPassGpuExecutionContext<'_>) -> Result<(), String> {
    let output = gpu.require_buffer_binding(PAGE_REQUESTS, RenderGraphResourceAccessKind::Write)?;
    let capacity = binding_word_capacity(&output);
    let output_buffer = output.buffer.clone();
    let output_offset = output.offset;
    if capacity == 0 {
        return Err("virtual geometry page-request graph buffer has no u32 capacity".to_string());
    }

    let ids = resident_page_ids(gpu.virtual_geometry_debug_snapshot());
    if ids.len() > capacity.saturating_sub(1) {
        return Err(format!(
            "virtual geometry frame has {} resident pages but the graph buffer admits {}",
            ids.len(),
            capacity.saturating_sub(1)
        ));
    }
    let mut words = Vec::with_capacity(ids.len().saturating_add(1));
    words.push(u32::try_from(ids.len()).unwrap_or(u32::MAX));
    words.extend(ids);
    gpu.buffer_upload_recorder().write_buffer(
        &output_buffer,
        output_offset,
        bytemuck::cast_slice(&words),
    );
    Ok(())
}

pub(super) fn execute_node_cluster_cull(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<(), String> {
    let input = gpu.require_buffer_binding(PAGE_REQUESTS, RenderGraphResourceAccessKind::Read)?;
    let output =
        gpu.require_buffer_binding(VISIBLE_CLUSTERS, RenderGraphResourceAccessKind::Write)?;
    let output_capacity = binding_word_capacity(&output).saturating_sub(1);
    let input_buffer = input.buffer.clone();
    let input_offset = input.offset;
    let input_size = input.size;
    let output_buffer = output.buffer.clone();
    let output_offset = output.offset;
    let output_size = output.size;
    if output_capacity == 0 {
        return Err(
            "virtual geometry visible-cluster graph buffer has no record capacity".to_string(),
        );
    }

    let mut candidates = cluster_candidates(gpu.virtual_geometry_debug_snapshot());
    if candidates.len() > output_capacity {
        return Err(format!(
            "virtual geometry frame has {} cluster candidates but the graph buffer admits {}",
            candidates.len(),
            output_capacity
        ));
    }
    let dispatch_count = candidates.len();
    candidates.insert(
        0,
        ClusterCandidate {
            cluster_id: u32::try_from(dispatch_count).unwrap_or(u32::MAX),
            page_id: 0,
        },
    );
    gpu.buffer_upload_recorder().write_buffer(
        &output_buffer,
        output_offset,
        bytemuck::bytes_of(&0_u32),
    );

    let device_epoch = gpu.device_epoch().ok_or_else(|| {
        "virtual geometry node/cull requires a materialized device epoch".to_string()
    })?;
    let mut native = gpu.native_context();
    let factory = &native;
    let candidate_buffer = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-cluster-candidates"),
        contents: bytemuck::cast_slice(&candidates),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mut pipeline_cache = NODE_CULL_PIPELINE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| "virtual geometry node/cull pipeline cache is poisoned".to_string())?;
    if pipeline_cache
        .as_ref()
        .is_none_or(|(cached_epoch, _)| *cached_epoch != device_epoch)
    {
        *pipeline_cache = Some((device_epoch, create_node_cull_pipeline(factory)));
    }
    let pipeline = &pipeline_cache
        .as_ref()
        .expect("node/cull pipeline cache must be initialized")
        .1;
    let bind_group = factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-virtual-geometry-node-cluster-cull-bind-group"),
        layout: &pipeline.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &input_buffer,
                    offset: input_offset,
                    size: input_size,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: candidate_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &output_buffer,
                    offset: output_offset,
                    size: output_size,
                }),
            },
        ],
    });
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-virtual-geometry-node-cluster-cull"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline.pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(NODE_CULL_DISPATCH_GROUPS, 1, 1);
    drop(pass);
    drop(pipeline_cache);
    drop(native);
    gpu.record_compute_dispatch(
        NODE_CULL_PASS_NAME,
        NODE_CULL_EXECUTOR_ID,
        NODE_CULL_PIPELINE_LABEL,
        [WORKGROUP_SIZE, 1, 1],
        [NODE_CULL_DISPATCH_GROUPS, 1, 1],
        vec![VISIBLE_CLUSTERS.to_string()],
    );
    Ok(())
}

pub(super) fn execute_page_feedback(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<(), String> {
    let input =
        gpu.require_buffer_binding(VISIBLE_CLUSTERS, RenderGraphResourceAccessKind::Read)?;
    let output = gpu.require_buffer_binding(FEEDBACK, RenderGraphResourceAccessKind::Write)?;
    let copy_size = binding_size(&input).min(binding_size(&output));
    let input_buffer = input.buffer.clone();
    let input_offset = input.offset;
    let output_buffer = output.buffer.clone();
    let output_offset = output.offset;
    if copy_size < size_of::<u32>() as u64 {
        return Err("virtual geometry feedback copy has no u32 payload capacity".to_string());
    }
    gpu.native_context().encoder.copy_buffer_to_buffer(
        &input_buffer,
        input_offset,
        &output_buffer,
        output_offset,
        copy_size - copy_size % size_of::<u32>() as u64,
    );
    Ok(())
}

fn create_node_cull_pipeline(factory: &impl RenderPassGpuResourceFactory) -> NodeCullPipeline {
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-virtual-geometry-node-cluster-cull-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "shaders/node_cluster_cull.wgsl"
        ))),
    });
    let bind_group_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-virtual-geometry-node-cluster-cull-bind-group-layout"),
        entries: &[
            storage_buffer_layout_entry(0, true, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(1, true, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(2, false, wgpu::ShaderStages::COMPUTE),
        ],
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-virtual-geometry-node-cluster-cull-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-virtual-geometry-node-cluster-cull-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    NodeCullPipeline {
        bind_group_layout,
        pipeline,
    }
}

pub(super) fn execute_visbuffer(gpu: &mut RenderPassGpuExecutionContext<'_>) -> Result<(), String> {
    let depth_view = gpu
        .require_texture_view(SCENE_DEPTH, RenderGraphResourceAccessKind::Write)?
        .clone();
    let vertices = cluster_vertices(gpu);
    if vertices.is_empty() {
        return Ok(());
    }
    encode_cluster_triangles(gpu, &vertices, ClusterTriangleTarget::Depth(&depth_view))
}

pub(super) fn execute_debug_overlay(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<(), String> {
    let Some(snapshot) = gpu.virtual_geometry_debug_snapshot() else {
        return Ok(());
    };
    if !snapshot.debug.visualize_visbuffer {
        return Ok(());
    }
    let color_view = gpu
        .require_texture_view(SCENE_COLOR, RenderGraphResourceAccessKind::Write)?
        .clone();
    let vertices = cluster_vertices(gpu);
    if vertices.is_empty() {
        return Ok(());
    }
    encode_cluster_triangles(gpu, &vertices, ClusterTriangleTarget::Color(&color_view))
}

fn resident_page_ids(snapshot: Option<&RenderVirtualGeometryDebugSnapshot>) -> Vec<u32> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    snapshot
        .resident_page_payloads
        .iter()
        .map(|page| page.page_id)
        .collect()
}

fn cluster_candidates(
    snapshot: Option<&RenderVirtualGeometryDebugSnapshot>,
) -> Vec<ClusterCandidate> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    snapshot
        .selected_clusters
        .iter()
        .filter(|cluster| {
            cluster.state
                == zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState::Resident
        })
        .map(|cluster| ClusterCandidate {
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
        })
        .collect()
}

fn cluster_vertices(gpu: &RenderPassGpuExecutionContext<'_>) -> Vec<ClusterVertex> {
    let Some(snapshot) = gpu.virtual_geometry_debug_snapshot() else {
        return Vec::new();
    };
    let camera = gpu.frame_extract().view.selected_effective_camera();
    let clip_from_world = ViewProjectionMatrixPair::from_camera(&camera, gpu.viewport_size())
        .clip_from_world_unjittered;
    let mut instance_transforms = snapshot
        .selected_clusters
        .iter()
        .filter_map(|cluster| {
            let instance_index = usize::try_from(cluster.instance_index?).ok()?;
            let instance = snapshot.instances.get(instance_index)?;
            Some((cluster.cluster_id, instance.transform.matrix()))
        })
        .collect::<BTreeMap<_, _>>();
    for cluster in &snapshot.leaf_clusters {
        if instance_transforms.contains_key(&cluster.cluster_id) {
            continue;
        }
        let Some(instance) = snapshot
            .instances
            .iter()
            .find(|instance| instance.entity == cluster.entity)
        else {
            continue;
        };
        instance_transforms.insert(cluster.cluster_id, instance.transform.matrix());
    }

    let mut vertices = Vec::new();
    for page in &snapshot.resident_page_payloads {
        for range in &page.cluster_ranges {
            let Some(transform) = instance_transforms.get(&range.cluster_id).copied() else {
                continue;
            };
            let Some(source_vertices) = payload_range(page, range.vertex_start, range.vertex_count)
            else {
                continue;
            };
            append_cluster_vertices(
                &mut vertices,
                clip_from_world * transform,
                range.cluster_id,
                source_vertices,
            );
        }
    }
    vertices
}

fn payload_range(
    page: &zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayload,
    start: u32,
    count: u32,
) -> Option<&[zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayloadVertex]> {
    let start = usize::try_from(start).ok()?;
    let end = start.checked_add(usize::try_from(count).ok()?)?;
    page.vertices.get(start..end)
}

fn append_cluster_vertices(
    output: &mut Vec<ClusterVertex>,
    clip_from_local: Mat4,
    cluster_id: u32,
    source: &[zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayloadVertex],
) {
    output.extend(source.iter().map(|vertex| ClusterVertex {
        clip_position: (clip_from_local * vertex.position.extend(1.0)).to_array(),
        cluster_id,
        _padding: [0; 3],
    }));
}

#[derive(Clone, Copy)]
enum ClusterTriangleTarget<'a> {
    Depth(&'a wgpu::TextureView),
    Color(&'a wgpu::TextureView),
}

fn encode_cluster_triangles(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    vertices: &[ClusterVertex],
    target: ClusterTriangleTarget<'_>,
) -> Result<(), String> {
    let visible =
        gpu.require_buffer_binding(VISIBLE_CLUSTERS, RenderGraphResourceAccessKind::Read)?;
    let visible_buffer = visible.buffer.clone();
    let visible_offset = visible.offset;
    let visible_size = visible.size;
    let target_format = gpu.target_format();
    let depth_format = gpu.depth_format();
    let device_epoch = gpu.device_epoch().ok_or_else(|| {
        "virtual geometry raster requires a materialized device epoch".to_string()
    })?;
    let pipeline_key = match target {
        ClusterTriangleTarget::Depth(_) => ClusterTrianglePipelineKey {
            device_epoch,
            target_format: None,
            depth_format: Some(depth_format),
        },
        ClusterTriangleTarget::Color(_) => ClusterTrianglePipelineKey {
            device_epoch,
            target_format: Some(target_format),
            depth_format: None,
        },
    };
    let mut native = gpu.native_context();
    let factory = &native;
    let vertex_buffer = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-cluster-triangle-vertices"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let pipeline_cache = match target {
        ClusterTriangleTarget::Depth(_) => &DEPTH_CLUSTER_TRIANGLE_PIPELINE,
        ClusterTriangleTarget::Color(_) => &COLOR_CLUSTER_TRIANGLE_PIPELINE,
    };
    let mut pipeline_cache = pipeline_cache
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| "virtual geometry cluster raster pipeline cache is poisoned".to_string())?;
    if pipeline_cache
        .as_ref()
        .is_none_or(|(cached_key, _)| *cached_key != pipeline_key)
    {
        *pipeline_cache = Some((
            pipeline_key,
            create_cluster_triangle_pipeline(factory, target_format, depth_format, target),
        ));
    }
    let pipeline = &pipeline_cache
        .as_ref()
        .expect("cluster triangle pipeline cache must be initialized")
        .1;
    let bind_group = factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-virtual-geometry-cluster-triangle-bind-group"),
        layout: &pipeline.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &visible_buffer,
                    offset: visible_offset,
                    size: visible_size,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: vertex_buffer.as_entire_binding(),
            },
        ],
    });
    let color_attachment = match target {
        ClusterTriangleTarget::Color(view) => Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        }),
        ClusterTriangleTarget::Depth(_) => None,
    };
    let color_attachments = [color_attachment];
    let depth_stencil_attachment = match target {
        ClusterTriangleTarget::Depth(view) => Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        ClusterTriangleTarget::Color(_) => None,
    };
    let mut pass = native
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-virtual-geometry-cluster-triangle-raster"),
            color_attachments: match target {
                ClusterTriangleTarget::Color(_) => &color_attachments,
                ClusterTriangleTarget::Depth(_) => &[],
            },
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(&pipeline.pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.draw(0..u32::try_from(vertices.len()).unwrap_or(u32::MAX), 0..1);
    Ok(())
}

fn create_cluster_triangle_pipeline(
    factory: &impl RenderPassGpuResourceFactory,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    target: ClusterTriangleTarget<'_>,
) -> ClusterTrianglePipeline {
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-virtual-geometry-cluster-triangle-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "shaders/cluster_triangles.wgsl"
        ))),
    });
    let bind_group_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-virtual-geometry-cluster-triangle-bind-group-layout"),
        entries: &[
            storage_buffer_layout_entry(
                0,
                true,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ),
            storage_buffer_layout_entry(1, true, wgpu::ShaderStages::VERTEX),
        ],
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-virtual-geometry-cluster-triangle-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let color_targets = [Some(wgpu::ColorTargetState {
        format: target_format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    })];
    let (fragment_entry, targets, depth_stencil) = match target {
        ClusterTriangleTarget::Depth(_) => (
            Some("fs_depth"),
            &[][..],
            Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
        ),
        ClusterTriangleTarget::Color(_) => (Some("fs_color"), &color_targets[..], None),
    };
    let pipeline = factory.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-virtual-geometry-cluster-triangle-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: fragment_entry,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets,
        }),
        multiview_mask: None,
        cache: None,
    });
    ClusterTrianglePipeline {
        bind_group_layout,
        pipeline,
    }
}

fn storage_buffer_layout_entry(
    binding: u32,
    read_only: bool,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn binding_size(binding: &wgpu::BufferBinding<'_>) -> u64 {
    binding
        .size
        .map(|size| size.get())
        .unwrap_or_else(|| binding.buffer.size().saturating_sub(binding.offset))
}

fn binding_word_capacity(binding: &wgpu::BufferBinding<'_>) -> usize {
    usize::try_from(binding_size(binding) / size_of::<u32>() as u64).unwrap_or(usize::MAX)
}

#[cfg(test)]
#[path = "gpu/tests/cases.rs"]
mod tests;

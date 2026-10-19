use std::{
    borrow::Cow,
    collections::HashMap,
    mem::size_of,
    ops::Range,
    sync::{Mutex, OnceLock},
};

use bytemuck::{Pod, Zeroable};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionData, ViewProjectionMatrixPair,
};
use zircon_runtime::core::math::Mat4;
use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RenderPassDeviceEpoch, RenderPassGpuExecutionContext,
    RenderPassGpuResourceFactory,
};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;
use zircon_runtime::rhi::TextureFormat;

const PAGE_REQUESTS: &str = "virtual-geometry-page-requests";
const VISIBLE_CLUSTERS: &str = "virtual-geometry-visible-clusters";
const NONRESIDENT_PAGE_REQUESTS: &str = "virtual-geometry-nonresident-page-requests";
const VISBUFFER64: &str = "virtual-geometry-visbuffer64";
const VISIBLE_CANDIDATES: &str = "virtual-geometry-visible-candidate-indices";
const INDIRECT_ARGS: &str = "virtual-geometry-indirect-args";
const INDIRECT_AUTHORITY: &str = "virtual-geometry-indirect-authority";
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

struct IndirectClusterTrianglePipeline {
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
static INDIRECT_DEPTH_CLUSTER_TRIANGLE_PIPELINE: OnceLock<
    Mutex<Option<(ClusterTrianglePipelineKey, IndirectClusterTrianglePipeline)>>,
> = OnceLock::new();

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ClusterVertex {
    clip_position: [f32; 4],
    cluster_id: u32,
    candidate_index: u32,
    authority_index: u32,
    instance_index: u32,
    lineage_depth: u32,
    frontier_rank: u32,
    _padding: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ClusterCandidate {
    cluster_id: u32,
    page_id: u32,
    candidate_index: u32,
    instance_index: u32,
    lod_level: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PackedClusterRange {
    pub entity: u64,
    pub instance_index: u32,
    pub page_id: u32,
    pub cluster_id: u32,
    pub cluster_ordinal: u32,
    pub candidate_index: u32,
    pub draw_ref_index: u32,
    pub original_index: u32,
    pub submission_index: Option<u32>,
    pub draw_ref_rank: Option<u32>,
    pub submission_slot: Option<u32>,
    pub authority_index: u32,
    pub vertices: Range<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct VisibleClusterReadbackKey {
    pub frame_id: u64,
    pub viewport: [u32; 2],
    pub device_epoch: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CullGpuReadbackWords {
    pub visible_clusters: Vec<u32>,
    pub requested_pages: Vec<u32>,
    pub visbuffer64: Vec<u32>,
}

struct PendingVisibleClusterReadback {
    buffer: wgpu::Buffer,
    word_lengths: [usize; 3],
    wait_for_submission: Box<dyn FnOnce() -> Result<(), String> + Send>,
}

#[derive(Default)]
pub(crate) struct VisibleClusterReadbackCollector {
    pending: Mutex<HashMap<VisibleClusterReadbackKey, PendingVisibleClusterReadback>>,
}

impl VisibleClusterReadbackCollector {
    pub(crate) fn read_words(&self, key: &VisibleClusterReadbackKey) -> Result<Vec<u32>, String> {
        self.read_outputs(key)
            .map(|outputs| outputs.visible_clusters)
    }

    pub(crate) fn read_outputs(
        &self,
        key: &VisibleClusterReadbackKey,
    ) -> Result<CullGpuReadbackWords, String> {
        let pending = self
            .pending
            .lock()
            .map_err(|_| "virtual geometry readback collector is poisoned".to_string())?
            .remove(key)
            .ok_or_else(|| format!("no virtual geometry readback is pending for {key:?}"))?;
        let slice = pending.buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result.map_err(|error| error.to_string()));
        });
        (pending.wait_for_submission)()?;
        receiver
            .recv_timeout(std::time::Duration::from_secs(30))
            .map_err(|error| {
                format!("virtual geometry readback callback did not finish: {error}")
            })??;
        let mapped = slice.get_mapped_range();
        let words = bytemuck::try_cast_slice::<u8, u32>(&mapped)
            .map_err(|error| format!("virtual geometry readback is not u32-aligned: {error:?}"))?;
        let [visible_len, request_len, visbuffer_len] = pending.word_lengths;
        let outputs = CullGpuReadbackWords {
            visible_clusters: words[..visible_len].to_vec(),
            requested_pages: words[visible_len..visible_len + request_len].to_vec(),
            visbuffer64: words
                [visible_len + request_len..visible_len + request_len + visbuffer_len]
                .to_vec(),
        };
        drop(mapped);
        pending.buffer.unmap();
        Ok(outputs)
    }
}

pub(crate) fn record_visible_cluster_readback(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    collector: &VisibleClusterReadbackCollector,
    frame_id: u64,
) -> Result<VisibleClusterReadbackKey, String> {
    let mut sources = Vec::with_capacity(3);
    let mut word_lengths = [0_usize; 3];
    let mut total_size = 0_u64;
    for (index, name) in [VISIBLE_CLUSTERS, NONRESIDENT_PAGE_REQUESTS, VISBUFFER64]
        .into_iter()
        .enumerate()
    {
        let binding = gpu.require_buffer_binding(name, RenderGraphResourceAccessKind::Write)?;
        let byte_size = binding_size(&binding);
        if byte_size == 0 || byte_size % size_of::<u32>() as u64 != 0 {
            return Err(format!(
                "virtual geometry {name} binding has invalid readback size"
            ));
        }
        word_lengths[index] = usize::try_from(byte_size / size_of::<u32>() as u64)
            .map_err(|_| format!("virtual geometry {name} readback exceeds usize"))?;
        sources.push((
            binding.buffer.clone(),
            binding.offset,
            byte_size,
            total_size,
        ));
        total_size = total_size
            .checked_add(byte_size)
            .ok_or_else(|| "virtual geometry readback byte length overflow".to_string())?;
    }
    let byte_len = usize::try_from(total_size)
        .map_err(|_| "virtual geometry combined readback exceeds usize".to_string())?;
    let epoch = gpu
        .device_epoch()
        .ok_or_else(|| "virtual geometry readback requires a device epoch".to_string())?;
    let viewport = gpu.viewport_size();
    let key = VisibleClusterReadbackKey {
        frame_id,
        viewport: [viewport.x, viewport.y],
        device_epoch: format!("{epoch:?}"),
    };
    let mut pending = collector
        .pending
        .lock()
        .map_err(|_| "virtual geometry readback collector is poisoned".to_string())?;
    if pending.contains_key(&key) {
        return Err(format!(
            "virtual geometry readback key was reused before consumption: {key:?}"
        ));
    }
    let mut native = gpu.native_context();
    let staging = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-cull-and-visbuffer64-readback"),
        contents: &vec![0_u8; byte_len],
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    });
    for (buffer, offset, size, staging_offset) in sources {
        native
            .encoder
            .copy_buffer_to_buffer(&buffer, offset, &staging, staging_offset, size);
    }
    let wait_for_submission = Box::new(native.readback_completion_waiter());
    drop(native);
    pending.insert(
        key.clone(),
        PendingVisibleClusterReadback {
            buffer: staging,
            word_lengths,
            wait_for_submission,
        },
    );
    Ok(key)
}

pub(super) fn execute_prepare(gpu: &mut RenderPassGpuExecutionContext<'_>) -> Result<(), String> {
    let output = gpu.require_buffer_binding(PAGE_REQUESTS, RenderGraphResourceAccessKind::Write)?;
    let capacity = binding_word_capacity(&output);
    let output_buffer = output.buffer.clone();
    let output_offset = output.offset;
    if capacity == 0 {
        return Err("virtual geometry page-request graph buffer has no u32 capacity".to_string());
    }

    let ids = resident_page_ids(gpu.virtual_geometry_execution_data());
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
    let prepared_cpu_frame = gpu
        .virtual_geometry_execution_data()
        .and_then(|data| data.prepared_cpu_frame.clone());
    // The prepared CPU uploads are separate from candidate shader/readback outputs.
    // The encoded startup dispatch retains its launch and output GPU resources.
    let prepared_uploads = prepared_cpu_frame.as_ref().map(|frame| {
        let mut native = gpu.native_context();
        super::materialize_prepared_cull(&mut native, &frame.inspection)
    });
    let prepared_sections = prepared_uploads
        .as_ref()
        .map(|prepared| prepared.native_inspection_sections())
        .transpose()?
        .unwrap_or_default();
    let source_words = super::selection::node_selection_source_words(gpu)?;
    let startup_buffer = prepared_sections
        .iter()
        .find(|(id, _, _)| *id == 5)
        .map(|(_, buffer, _)| buffer.clone());
    if source_words[21] != 0 && startup_buffer.is_none() {
        return Err("prepared GPU instance work lease is absent".into());
    }
    let decisions = gpu.require_buffer_binding(
        "virtual-geometry-cull-decisions",
        RenderGraphResourceAccessKind::Write,
    )?;
    let decision_buffer = decisions.buffer.clone();
    let decision_offset = decisions.offset;
    let decision_size = decisions.size;
    let decision_bytes = binding_size(&decisions);
    if decision_bytes != (8 + 4096 * 33) * 4 {
        return Err("VG graph decision buffer has a different physical ABI".into());
    }
    let input = gpu.require_buffer_binding(PAGE_REQUESTS, RenderGraphResourceAccessKind::Read)?;
    let output =
        gpu.require_buffer_binding(VISIBLE_CLUSTERS, RenderGraphResourceAccessKind::Write)?;
    let output_capacity = binding_word_capacity(&output).saturating_sub(1);
    let visible_candidates =
        gpu.require_buffer_binding(VISIBLE_CANDIDATES, RenderGraphResourceAccessKind::Write)?;
    let candidate_output_capacity = binding_word_capacity(&visible_candidates).saturating_sub(1);
    let requests = gpu.require_buffer_binding(
        NONRESIDENT_PAGE_REQUESTS,
        RenderGraphResourceAccessKind::Write,
    )?;
    let visbuffer =
        gpu.require_buffer_binding(VISBUFFER64, RenderGraphResourceAccessKind::Write)?;
    let input_buffer = input.buffer.clone();
    let input_offset = input.offset;
    let input_size = input.size;
    let request_buffer = requests.buffer.clone();
    let request_offset = requests.offset;
    let request_size = requests.size;
    let visbuffer_buffer = visbuffer.buffer.clone();
    let visbuffer_offset = visbuffer.offset;
    let visbuffer_size = visbuffer.size;
    let output_buffer = output.buffer.clone();
    let output_offset = output.offset;
    let output_size = output.size;
    let candidate_output_buffer = visible_candidates.buffer.clone();
    let candidate_output_offset = visible_candidates.offset;
    let candidate_output_size = visible_candidates.size;
    let output_byte_size = binding_size(&output);
    let candidate_output_byte_size = binding_size(&visible_candidates);
    let request_byte_size = binding_size(&requests);
    let visbuffer_byte_size = binding_size(&visbuffer);
    if output_capacity == 0 {
        return Err(
            "virtual geometry visible-cluster graph buffer has no record capacity".to_string(),
        );
    }

    let candidates = cluster_candidates(gpu.virtual_geometry_execution_data());
    if candidates.len() > output_capacity || candidates.len() > candidate_output_capacity {
        return Err(format!(
            "virtual geometry frame has {} cluster candidates but the graph buffers admit {} cluster IDs and {} candidate indices",
            candidates.len(),
            output_capacity,
            candidate_output_capacity
        ));
    }
    gpu.buffer_upload_recorder().write_buffer(
        &output_buffer,
        output_offset,
        bytemuck::bytes_of(&0_u32),
    );
    gpu.buffer_upload_recorder().write_buffer(
        &candidate_output_buffer,
        candidate_output_offset,
        bytemuck::bytes_of(&0_u32),
    );

    let device_epoch = gpu.device_epoch().ok_or_else(|| {
        "virtual geometry node/cull requires a materialized device epoch".to_string()
    })?;
    let mut native = gpu.native_context();
    native
        .encoder
        .clear_buffer(&output_buffer, output_offset, Some(output_byte_size));
    native.encoder.clear_buffer(
        &candidate_output_buffer,
        candidate_output_offset,
        Some(candidate_output_byte_size),
    );
    native
        .encoder
        .clear_buffer(&request_buffer, request_offset, Some(request_byte_size));
    native.encoder.clear_buffer(
        &visbuffer_buffer,
        visbuffer_offset,
        Some(visbuffer_byte_size),
    );
    native
        .encoder
        .clear_buffer(&decision_buffer, decision_offset, Some(decision_bytes));
    let factory = &native;
    let source_buffer = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-selection-source"),
        contents: bytemuck::cast_slice(&source_words),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let startup_buffer = startup_buffer.unwrap_or_else(|| {
        factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-virtual-geometry-empty-startup-work"),
            contents: bytemuck::bytes_of(&0_u32),
            usage: wgpu::BufferUsages::STORAGE,
        })
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
                resource: source_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &output_buffer,
                    offset: output_offset,
                    size: output_size,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &candidate_output_buffer,
                    offset: candidate_output_offset,
                    size: candidate_output_size,
                }),
            },
            buffer_bind_group_entry(4, &request_buffer, request_offset, request_size),
            buffer_bind_group_entry(5, &visbuffer_buffer, visbuffer_offset, visbuffer_size),
            wgpu::BindGroupEntry {
                binding: 6,
                resource: startup_buffer.as_entire_binding(),
            },
            buffer_bind_group_entry(7, &decision_buffer, decision_offset, decision_size),
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
    // Diagnostics observe the same physical source prefix consumed by the shader.
    let mut sections = vec![(0, source_buffer.clone(), 0..14 * 4)];
    sections.extend(prepared_sections);
    register_native_inspection(gpu, sections)?;
    gpu.record_compute_dispatch(
        NODE_CULL_PASS_NAME,
        NODE_CULL_EXECUTOR_ID,
        NODE_CULL_PIPELINE_LABEL,
        [WORKGROUP_SIZE, 1, 1],
        [NODE_CULL_DISPATCH_GROUPS, 1, 1],
        vec![
            VISIBLE_CLUSTERS.to_string(),
            VISIBLE_CANDIDATES.to_string(),
            NONRESIDENT_PAGE_REQUESTS.to_string(),
            VISBUFFER64.to_string(),
            "virtual-geometry-cull-decisions".to_string(),
        ],
    );
    Ok(())
}

pub(super) fn execute_page_feedback(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<(), String> {
    let input = gpu.require_buffer_binding(
        NONRESIDENT_PAGE_REQUESTS,
        RenderGraphResourceAccessKind::Read,
    )?;
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
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("node_cluster_cull.wgsl"))),
    });
    let bind_group_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-virtual-geometry-node-cluster-cull-bind-group-layout"),
        entries: &[
            storage_buffer_layout_entry(0, true, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(1, true, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(2, false, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(3, false, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(4, false, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(5, false, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(6, true, wgpu::ShaderStages::COMPUTE),
            storage_buffer_layout_entry(7, false, wgpu::ShaderStages::COMPUTE),
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
    let (vertices, ranges) = cluster_vertices_with_ranges(gpu)?;
    if vertices.is_empty() {
        return Ok(());
    }
    encode_cluster_triangles_indirect(gpu, &vertices, &ranges, &depth_view)
}

pub(super) fn execute_debug_overlay(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<(), String> {
    let Some(snapshot) = gpu.virtual_geometry_execution_data() else {
        return Ok(());
    };
    if !snapshot.debug.visualize_visbuffer {
        return Ok(());
    }
    let color_view = gpu
        .require_texture_view(SCENE_COLOR, RenderGraphResourceAccessKind::Write)?
        .clone();
    let vertices = cluster_vertices(gpu)?;
    if vertices.is_empty() {
        return Ok(());
    }
    encode_cluster_triangles(gpu, &vertices, ClusterTriangleTarget::Color(&color_view))
}

// Pure input projections retain the original unit fixture without making debug storage
// a production execution dependency.
trait ClusterExecutionSource {
    fn resident_page_payloads(
        &self,
    ) -> &[zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayload];
    fn selected_clusters(
        &self,
    ) -> &[zircon_runtime::core::framework::render::RenderVirtualGeometrySelectedCluster];
}
impl ClusterExecutionSource for RenderVirtualGeometryExecutionData {
    fn resident_page_payloads(
        &self,
    ) -> &[zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayload] {
        &self.resident_page_payloads
    }
    fn selected_clusters(
        &self,
    ) -> &[zircon_runtime::core::framework::render::RenderVirtualGeometrySelectedCluster] {
        &self.selected_clusters
    }
}
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryDebugSnapshot;
#[cfg(test)]
impl ClusterExecutionSource for RenderVirtualGeometryDebugSnapshot {
    fn resident_page_payloads(
        &self,
    ) -> &[zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayload] {
        &self.resident_page_payloads
    }
    fn selected_clusters(
        &self,
    ) -> &[zircon_runtime::core::framework::render::RenderVirtualGeometrySelectedCluster] {
        &self.selected_clusters
    }
}

fn resident_page_ids(snapshot: Option<&impl ClusterExecutionSource>) -> Vec<u32> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    snapshot
        .resident_page_payloads()
        .iter()
        .map(|page| page.page_id)
        .collect()
}

fn cluster_candidates(snapshot: Option<&impl ClusterExecutionSource>) -> Vec<ClusterCandidate> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    snapshot
        .selected_clusters()
        .iter()
        .enumerate()
        .map(|(candidate_index, cluster)| ClusterCandidate {
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            candidate_index: u32::try_from(candidate_index).unwrap_or(u32::MAX),
            instance_index: cluster.instance_index.unwrap_or(u32::from(u16::MAX)),
            lod_level: u32::from(cluster.lod_level),
        })
        .collect()
}

fn cluster_vertices(gpu: &RenderPassGpuExecutionContext<'_>) -> Result<Vec<ClusterVertex>, String> {
    Ok(cluster_vertices_with_ranges(gpu)?.0)
}

/// Packs the exact vertices used by the raster consumer and records every source cluster's
/// half-open vertex ranges. Repeated cluster IDs retain all ranges in append order.

pub(super) fn cluster_vertices_with_ranges(
    gpu: &RenderPassGpuExecutionContext<'_>,
) -> Result<(Vec<ClusterVertex>, Vec<PackedClusterRange>), String> {
    let Some(data) = gpu.virtual_geometry_execution_data() else {
        return Ok((Vec::new(), Vec::new()));
    };
    let rows = zircon_runtime::graphics::virtual_geometry_indexed_source_rows(data)?;
    let camera = gpu.frame_extract().view.selected_effective_camera();
    let clip_from_world = ViewProjectionMatrixPair::from_camera(&camera, gpu.viewport_size())
        .clip_from_world_unjittered;
    let mut vertices = Vec::new();
    let mut ranges = Vec::new();
    for row in rows {
        let identity = row.identity();
        zircon_runtime::core::framework::render::RenderVirtualGeometryExtract::checked_cluster_row_indices(
            &data.instances, &data.author_cluster_catalog, identity.entity,
            identity.stable_instance_key, identity.instance_index).map_err(str::to_owned)?;
        let local_to_world =
            match identity.instance_index {
                Some(index) => {
                    let Ok(index) = usize::try_from(index) else {
                        return Err("raster source instance index exceeds usize".into());
                    };
                    let Some(instance) = data.instances.get(index) else {
                        return Err("raster source instance is absent".into());
                    };
                    instance.transform.matrix()
                }
                None => {
                    if data.instances.iter().any(|instance| instance.entity == identity.entity)
                    || identity.stable_instance_key
                        != zircon_runtime::core::framework::render::render_mesh_stable_instance_key(
                            identity.entity,
                            0,
                        )
                {
                    return Err("raster legacy source has a conflicting instance domain".into());
                }
                    let mut meshes = gpu
                        .frame_extract()
                        .geometry
                        .meshes
                        .iter()
                        .filter(|mesh| mesh.node_id == identity.entity);
                    let Some(mesh) = meshes.next() else {
                        return Err("raster legacy source has no mesh transform".into());
                    };
                    if meshes.next().is_some() {
                        return Err("raster legacy source has ambiguous mesh transforms".into());
                    }
                    mesh.transform.matrix()
                }
            };
        let Some(page) = data.resident_page_payloads.get(row.page_index()) else {
            return Err("raster source physical payload is absent".into());
        };
        let Some(range) = page.cluster_ranges.get(row.range_index()) else {
            return Err("raster source physical range is absent".into());
        };
        let Some(source) = payload_range(page, range.vertex_start, range.vertex_count) else {
            return Err("raster source physical vertex range is invalid".into());
        };
        let Ok(start) = u32::try_from(vertices.len()) else {
            return Err("raster packed vertex start exceeds u32".into());
        };
        let Ok(count) = u32::try_from(source.len()) else {
            return Err("raster packed vertex count exceeds u32".into());
        };
        let Some(end) = start.checked_add(count) else {
            return Err("raster packed vertex end overflow".into());
        };
        append_cluster_vertices(
            &mut vertices,
            clip_from_world * local_to_world,
            row.cluster_id(),
            row.candidate_index(),
            row.source_row(),
            identity.instance_index.unwrap_or(u32::MAX),
            row.lineage_depth(),
            row.frontier_rank(),
            source,
        );
        ranges.push(PackedClusterRange {
            entity: identity.entity,
            instance_index: identity.instance_index.unwrap_or(u32::MAX),
            page_id: identity.page_id,
            cluster_id: row.cluster_id(),
            cluster_ordinal: row.cluster_ordinal(),
            candidate_index: row.candidate_index(),
            draw_ref_index: identity.draw_ref_index,
            original_index: identity.original_index,
            submission_index: identity.submission_index,
            draw_ref_rank: identity.draw_ref_rank,
            submission_slot: identity.submission_slot,
            authority_index: row.source_row(),
            vertices: start..end,
        });
    }
    Ok((vertices, ranges))
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
    candidate_index: u32,
    authority_index: u32,
    instance_index: u32,
    lineage_depth: u32,
    frontier_rank: u32,
    source: &[zircon_runtime::core::framework::render::RenderVirtualGeometryPagePayloadVertex],
) {
    output.extend(source.iter().map(|vertex| ClusterVertex {
        clip_position: (clip_from_local * vertex.position.extend(1.0)).to_array(),
        cluster_id,
        candidate_index,
        authority_index,
        instance_index,
        lineage_depth,
        frontier_rank,
        _padding: [0; 2],
    }));
}

#[derive(Clone, Copy)]
enum ClusterTriangleTarget<'a> {
    Depth(&'a wgpu::TextureView),
    Color(&'a wgpu::TextureView),
}

fn cluster_color_texture_format(format: TextureFormat) -> Result<wgpu::TextureFormat, String> {
    let format = match format {
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::R16Float => wgpu::TextureFormat::R16Float,
        TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
        TextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
        TextureFormat::Rg11b10Ufloat => wgpu::TextureFormat::Rg11b10Ufloat,
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::Depth24Plus
        | TextureFormat::Depth24PlusStencil8
        | TextureFormat::Depth32Float => {
            return Err(format!(
                "virtual geometry raster target `{SCENE_COLOR}` must be a color texture, got `{format:?}`"
            ));
        }
    };
    Ok(format)
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
    let target_format = match target {
        ClusterTriangleTarget::Depth(_) => gpu.target_format(),
        ClusterTriangleTarget::Color(_) => cluster_color_texture_format(
            gpu.require_texture_desc(SCENE_COLOR, RenderGraphResourceAccessKind::Write)?
                .format,
        )?,
    };
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

fn encode_cluster_triangles_indirect(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    vertices: &[ClusterVertex],
    ranges: &[PackedClusterRange],
    depth_view: &wgpu::TextureView,
) -> Result<(), String> {
    let candidates =
        gpu.require_buffer_binding(VISIBLE_CANDIDATES, RenderGraphResourceAccessKind::Read)?;
    let args = gpu.require_buffer_binding(INDIRECT_ARGS, RenderGraphResourceAccessKind::Read)?;
    let authority =
        gpu.require_buffer_binding(INDIRECT_AUTHORITY, RenderGraphResourceAccessKind::Read)?;
    let depth_format = gpu.depth_format();
    let device_epoch = gpu.device_epoch().ok_or_else(|| {
        "virtual geometry indirect raster requires a materialized device epoch".to_string()
    })?;
    let pipeline_key = ClusterTrianglePipelineKey {
        device_epoch,
        target_format: None,
        depth_format: Some(depth_format),
    };
    let mut native = gpu.native_context();
    let factory = &native;
    let vertex_buffer = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-indirect-cluster-vertices"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mut pipeline_cache = INDIRECT_DEPTH_CLUSTER_TRIANGLE_PIPELINE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| "virtual geometry indirect raster pipeline cache is poisoned".to_string())?;
    if pipeline_cache
        .as_ref()
        .is_none_or(|(cached_key, _)| *cached_key != pipeline_key)
    {
        *pipeline_cache = Some((
            pipeline_key,
            create_indirect_cluster_triangle_pipeline(factory, depth_format),
        ));
    }
    let pipeline = &pipeline_cache
        .as_ref()
        .expect("indirect pipeline initialized")
        .1;
    let candidates_buffer = candidates.buffer.clone();
    let candidates_offset = candidates.offset;
    let candidates_size = candidates.size;
    let authority_buffer = authority.buffer.clone();
    let authority_offset = authority.offset;
    let authority_size = authority.size;
    let bind_group = factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-virtual-geometry-indirect-raster-bind-group"),
        layout: &pipeline.bind_group_layout,
        entries: &[
            buffer_bind_group_entry(0, &candidates_buffer, candidates_offset, candidates_size),
            wgpu::BindGroupEntry {
                binding: 1,
                resource: vertex_buffer.as_entire_binding(),
            },
            buffer_bind_group_entry(2, &authority_buffer, authority_offset, authority_size),
        ],
    });
    let args_buffer = args.buffer.clone();
    let args_offset = args.offset;
    let args_size = args
        .size
        .map(|size| size.get())
        .unwrap_or_else(|| args.buffer.size().saturating_sub(args.offset));
    if ranges.iter().any(|range| {
        let offset = 4_u64 + u64::from(range.authority_index) * 16;
        offset.checked_add(16).is_none_or(|end| end > args_size)
    }) {
        return Err(
            "virtual geometry indirect argument range exceeds its graph binding".to_string(),
        );
    }
    let attachments = [];
    let mut pass = native
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-virtual-geometry-indirect-cluster-raster"),
            color_attachments: &attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(&pipeline.pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    for range in ranges {
        let offset = args_offset + 4 + u64::from(range.authority_index) * 16;
        pass.draw_indirect(&args_buffer, offset);
    }
    drop(pass);
    drop(pipeline_cache);
    drop(native);
    Ok(())
}

fn buffer_bind_group_entry(
    binding: u32,
    buffer: &wgpu::Buffer,
    offset: u64,
    size: Option<std::num::NonZeroU64>,
) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset,
            size,
        }),
    }
}

fn create_indirect_cluster_triangle_pipeline(
    factory: &impl RenderPassGpuResourceFactory,
    depth_format: wgpu::TextureFormat,
) -> IndirectClusterTrianglePipeline {
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-virtual-geometry-indirect-cluster-triangle-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
            "cluster_triangles_indirect.wgsl"
        ))),
    });
    let bind_group_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-virtual-geometry-indirect-cluster-triangle-layout"),
        entries: &[
            storage_buffer_layout_entry(
                0,
                true,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ),
            storage_buffer_layout_entry(1, true, wgpu::ShaderStages::VERTEX),
            storage_buffer_layout_entry(
                2,
                true,
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ),
        ],
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-virtual-geometry-indirect-cluster-triangle-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = factory.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-virtual-geometry-indirect-cluster-triangle-pipeline"),
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: depth_format,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_depth"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[],
        }),
        multiview_mask: None,
        cache: None,
    });
    IndirectClusterTrianglePipeline {
        bind_group_layout,
        pipeline,
    }
}

fn create_cluster_triangle_pipeline(
    factory: &impl RenderPassGpuResourceFactory,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    target: ClusterTriangleTarget<'_>,
) -> ClusterTrianglePipeline {
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-virtual-geometry-cluster-triangle-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("cluster_triangles.wgsl"))),
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

/// The decoder captures physical layouts, never the CPU inspection values.
fn register_native_inspection(
    gpu: &RenderPassGpuExecutionContext<'_>,
    sections: Vec<(u32, wgpu::Buffer, std::ops::Range<u64>)>,
) -> Result<(), String> {
    use zircon_runtime::core::framework::render::*;
    let mut layouts = Vec::with_capacity(sections.len());
    let mut total = 0_u64;
    for (id, _, range) in &sections {
        let bytes = range
            .end
            .checked_sub(range.start)
            .ok_or("reversed startup section")?;
        let end = total
            .checked_add(bytes)
            .ok_or("startup section length overflow")?;
        layouts.push((*id, total, end));
        total = end;
    }
    // The map callback owns these exact source buffers until cancellation or decoding.
    let leases = sections.clone();
    gpu.register_virtual_geometry_native_inspection(sections, Box::new(move |mapped, outputs| {
        let _keepalive = &leases;
        if u64::try_from(mapped.len()).map_err(|_| "startup map length overflow")? != total {
            return Err("startup mapped allocation length mismatch".into());
        }
        let mut words = std::collections::BTreeMap::new();
        for (id, start, end) in layouts {
            let start = usize::try_from(start).map_err(|_| "startup start overflow")?;
            let end = usize::try_from(end).map_err(|_| "startup end overflow")?;
            let raw = mapped.get(start..end).ok_or("startup section exceeds mapped allocation")?;
            if raw.len() % 4 != 0 { return Err("startup section is not word aligned".into()); }
            let stream = raw.chunks_exact(4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect::<Vec<_>>();
            if words.insert(id, stream).is_some() { return Err("duplicate startup schema section".into()); }
        }
        let cull_words = words.remove(&0).ok_or("actual cull-input section is absent")?;
        if cull_words.len() != RenderVirtualGeometryCullInputSnapshot::GPU_WORD_COUNT
            || cull_words[13] > 3 || (cull_words[11] != u32::MAX && cull_words[11] > u8::MAX as u32)
            || cull_words[12] & !15 != 0 {
            return Err("invalid native cull-input schema".into());
        }
        let cull_input = RenderVirtualGeometryCullInputSnapshot::from_packed_words(&cull_words)
            .ok_or("invalid native cull-input record")?;
        fn fixed<T>(words: Option<Vec<u32>>, stride: usize, decode: impl Fn(&[u32]) -> Option<T>)
            -> Result<Option<T>, String> {
            words.map(|words| {
                if words.len() != stride { return Err("native fixed startup section length mismatch".into()); }
                decode(&words).ok_or_else(|| "native fixed startup record is invalid".into())
            }).transpose()
        }
        fn rows<T>(words: Option<Vec<u32>>, stride: usize, mip: Option<usize>,
            decode: impl Fn(&[u32]) -> Option<T>) -> Result<Vec<T>, String> {
            let Some(words) = words else { return Ok(Vec::new()); };
            if words.len() % stride != 0 { return Err("native startup row stride mismatch".into()); }
            words.chunks_exact(stride).map(|row| {
                if mip.is_some_and(|index| row[index] != u32::MAX && row[index] > u8::MAX as u32) {
                    return Err("native startup forced mip exceeds its ABI".into());
                }
                decode(row).ok_or_else(|| "native startup row is invalid".into())
            }).collect()
        }
        let global_state = fixed(words.remove(&1),
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::GPU_WORD_COUNT,
            RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::from_packed_words)?;
        let dispatch_setup = fixed(words.remove(&2),
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::GPU_WORD_COUNT,
            RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot::from_packed_words)?;
        let launch_worklist = words.remove(&3).map(|words| {
            let value = RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot::from_packed_words(&words)
                .ok_or("native launch-worklist record is invalid")?;
            if value.packed_words().len() != words.len() {
                return Err("native launch-worklist trailing or missing rows".to_owned());
            }
            Ok(value)
        }).transpose()?;
        let instance_seeds = rows(words.remove(&4),
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT, None,
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::from_packed_words)?;
        let instance_work_items = rows(words.remove(&5),
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::GPU_WORD_COUNT, Some(9),
            RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::from_packed_words)?;
        let cluster_work_items = rows(words.remove(&6),
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::GPU_WORD_COUNT, Some(7),
            RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::from_packed_words)?;
        let hierarchy_child_ids = words.remove(&7).unwrap_or_default();
        let child_work_items = rows(words.remove(&8),
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem::GPU_WORD_COUNT, Some(10),
            RenderVirtualGeometryNodeAndClusterCullChildWorkItem::from_packed_words)?;
        let traversal_records = rows(words.remove(&9),
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord::GPU_WORD_COUNT, Some(14),
            RenderVirtualGeometryNodeAndClusterCullTraversalRecord::from_packed_words)?;
        let page_request_ids = words.remove(&10).unwrap_or_default();
        if !words.is_empty() { return Err("unknown native startup schema section".into()); }
        if let Some(global) = &global_state {
            if global.cull_input != cull_input {
                return Err("actual native global state and cull input disagree".into());
            }
        }
        if let Some(launch) = &launch_worklist {
            if Some(&launch.global_state) != global_state.as_ref()
                || Some(launch.dispatch_setup) != dispatch_setup
                || launch.instance_seeds != instance_seeds {
                return Err("actual native startup sections disagree with the launch-worklist".into());
            }
        }
        if let Some(setup) = &dispatch_setup {
            if usize::try_from(setup.instance_seed_count).map_err(|_| "startup count overflow")?
                != instance_seeds.len() || instance_work_items.len() != instance_seeds.len() {
                return Err("actual startup compute count disagrees with the uploaded launch count".into());
            }
        }
        let startup = global_state.is_some().then_some(RenderVirtualGeometryNativeStartupReadback {
            global_state, dispatch_setup, launch_worklist, instance_seeds, instance_work_items,
            cluster_work_items, hierarchy_child_ids, child_work_items, traversal_records, page_request_ids,
        });
        // This branch follows actual, validated terminal GPU row counts after both passes completed.
        let selected_cluster_source = if outputs.selected_clusters.is_empty() {
            RenderVirtualGeometrySelectedClusterSource::RenderPathClearOnly
        } else { RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections };
        let hardware_rasterization_source = if outputs.hardware_rasterization_records.is_empty() {
            RenderVirtualGeometryHardwareRasterizationSource::RenderPathClearOnly
        } else { RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections };
        let visbuffer64_source = if outputs.visbuffer64_entries.is_empty() {
            RenderVirtualGeometryVisBuffer64Source::RenderPathClearOnly
        } else { RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections };
        Ok(RenderVirtualGeometryNativeReadbackInspection {
            cull_input, startup, selected_cluster_source, hardware_rasterization_source, visbuffer64_source,
        })
    }))
}

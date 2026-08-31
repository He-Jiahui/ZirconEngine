use crate::core::framework::render::{IblBakeArtifactRequest, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::CompiledRenderGraph;

use super::realtime_ibl_graph_plan::{
    RealtimeIblGraphPlan, RealtimeIblGraphSlotResources, RealtimeIblGraphTextureResources,
};
use super::realtime_ibl_time_slice::{IblRealtimeBufferSlot, RealtimeIblFrameBatch};

pub(in crate::graphics) mod execution_resource_cache;

use execution_resource_cache::{
    RealtimeIblExecutionResourceCache, RealtimeIblExecutionResourceCacheStats,
    RealtimeIblExecutionResourceResolution,
};

const CUBE_FACE_COUNT: u32 = 6;
const IBL_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(in crate::graphics) struct RealtimeIblGpuResources {
    slot_a: RealtimeIblGpuSlotResources,
    slot_b: RealtimeIblGpuSlotResources,
    execution_resource_cache: RealtimeIblExecutionResourceCache,
}

struct RealtimeIblGpuSlotResources {
    source: RealtimeIblGpuTextureResources,
    pmrem: RealtimeIblGpuTextureResources,
    sh9: wgpu::Buffer,
}

struct RealtimeIblGpuTextureResources {
    _texture: wgpu::Texture,
    sampled: wgpu::TextureView,
    sampled_mips: Vec<wgpu::TextureView>,
    storage_mips: Vec<wgpu::TextureView>,
}

impl RealtimeIblGpuResources {
    pub(in crate::graphics) fn new(
        device: &wgpu::Device,
        request: &IblBakeArtifactRequest,
    ) -> Self {
        Self {
            slot_a: RealtimeIblGpuSlotResources::new(device, request, "a"),
            slot_b: RealtimeIblGpuSlotResources::new(device, request, "b"),
            execution_resource_cache: RealtimeIblExecutionResourceCache::default(),
        }
    }

    /// Caches only immutable handles for one compiled graph topology. A cache
    /// entry belongs to this device-owned allocation and is never shared across
    /// a resource-layout change or a new runtime instance.
    pub(in crate::graphics) fn execution_resources_for(
        &mut self,
        request: &IblBakeArtifactRequest,
        batch: &RealtimeIblFrameBatch,
        plan: &RealtimeIblGraphPlan,
        graph: &CompiledRenderGraph,
        required_resource_names: &[String],
        cpu_timing_enabled: bool,
    ) -> Result<RealtimeIblExecutionResourceResolution<'_>, String> {
        let slot_a = &self.slot_a;
        let slot_b = &self.slot_b;
        self.execution_resource_cache.resolve(
            request,
            batch,
            plan,
            graph,
            required_resource_names,
            cpu_timing_enabled,
            |resources| bind_graph_plan(slot_a, slot_b, plan, required_resource_names, resources),
        )
    }

    #[cfg(test)]
    fn execution_resource_cache_stats(&self) -> RealtimeIblExecutionResourceCacheStats {
        self.execution_resource_cache.stats()
    }

    pub(in crate::graphics) fn bind_graph_plan(
        &self,
        plan: &RealtimeIblGraphPlan,
        required_resource_names: &[String],
        resources: &mut RenderGraphExecutionResources,
    ) -> Result<(), String> {
        bind_graph_plan(
            &self.slot_a,
            &self.slot_b,
            plan,
            required_resource_names,
            resources,
        )
    }

    fn slot(&self, slot: IblRealtimeBufferSlot) -> &RealtimeIblGpuSlotResources {
        match slot {
            IblRealtimeBufferSlot::A => &self.slot_a,
            IblRealtimeBufferSlot::B => &self.slot_b,
        }
    }

    pub(in crate::graphics) fn source_sampled(
        &self,
        slot: IblRealtimeBufferSlot,
    ) -> &wgpu::TextureView {
        &self.slot(slot).source.sampled
    }

    pub(in crate::graphics) fn source_sampled_mip(
        &self,
        slot: IblRealtimeBufferSlot,
        mip_level: u32,
    ) -> Result<&wgpu::TextureView, String> {
        self.slot(slot)
            .source
            .sampled_mips
            .get(mip_level as usize)
            .ok_or_else(|| format!("realtime IBL source sampled mip {mip_level} is unavailable"))
    }

    pub(in crate::graphics) fn source_storage_mip(
        &self,
        slot: IblRealtimeBufferSlot,
        mip_level: u32,
    ) -> Result<&wgpu::TextureView, String> {
        self.slot(slot)
            .source
            .storage_mips
            .get(mip_level as usize)
            .ok_or_else(|| format!("realtime IBL source storage mip {mip_level} is unavailable"))
    }

    pub(in crate::graphics) fn pmrem_storage_mip(
        &self,
        slot: IblRealtimeBufferSlot,
        mip_level: u32,
    ) -> Result<&wgpu::TextureView, String> {
        self.slot(slot)
            .pmrem
            .storage_mips
            .get(mip_level as usize)
            .ok_or_else(|| format!("realtime IBL PMREM storage mip {mip_level} is unavailable"))
    }

    pub(in crate::graphics) fn pmrem_sampled(
        &self,
        slot: IblRealtimeBufferSlot,
    ) -> &wgpu::TextureView {
        &self.slot(slot).pmrem.sampled
    }

    pub(in crate::graphics) fn sh9(&self, slot: IblRealtimeBufferSlot) -> &wgpu::Buffer {
        &self.slot(slot).sh9
    }
}

fn bind_graph_plan(
    slot_a: &RealtimeIblGpuSlotResources,
    slot_b: &RealtimeIblGpuSlotResources,
    plan: &RealtimeIblGraphPlan,
    required_resource_names: &[String],
    resources: &mut RenderGraphExecutionResources,
) -> Result<(), String> {
    bind_slot(
        slot_resource(slot_a, slot_b, plan.ready.slot),
        &plan.ready,
        required_resource_names,
        resources,
    )?;
    bind_slot(
        slot_resource(slot_a, slot_b, plan.work.slot),
        &plan.work,
        required_resource_names,
        resources,
    )
}

fn bind_slot(
    gpu: &RealtimeIblGpuSlotResources,
    graph: &RealtimeIblGraphSlotResources,
    required_resource_names: &[String],
    resources: &mut RenderGraphExecutionResources,
) -> Result<(), String> {
    bind_texture_views(
        &graph.source,
        &gpu.source,
        required_resource_names,
        resources,
    )?;
    bind_texture_views(&graph.pmrem, &gpu.pmrem, required_resource_names, resources)?;
    if is_required_resource(required_resource_names, &graph.sh9.name) {
        resources.bind_execution_owned_buffer(&graph.sh9.name, &graph.sh9.name, &gpu.sh9);
    }
    Ok(())
}

fn slot_resource<'slots>(
    slot_a: &'slots RealtimeIblGpuSlotResources,
    slot_b: &'slots RealtimeIblGpuSlotResources,
    slot: IblRealtimeBufferSlot,
) -> &'slots RealtimeIblGpuSlotResources {
    match slot {
        IblRealtimeBufferSlot::A => slot_a,
        IblRealtimeBufferSlot::B => slot_b,
    }
}

impl RealtimeIblGpuSlotResources {
    fn new(device: &wgpu::Device, request: &IblBakeArtifactRequest, slot_label: &str) -> Self {
        Self {
            source: RealtimeIblGpuTextureResources::new(
                device,
                &format!("zircon-realtime-ibl-{slot_label}-source"),
                request.source_face_size(),
                request.source_mip_count(),
            ),
            pmrem: RealtimeIblGpuTextureResources::new(
                device,
                &format!("zircon-realtime-ibl-{slot_label}-pmrem"),
                request.pmrem_face_size(),
                request.pmrem_mip_count(),
            ),
            sh9: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("zircon-realtime-ibl-{slot_label}-sh9")),
                size: IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }
}

impl RealtimeIblGpuTextureResources {
    fn new(device: &wgpu::Device, label: &str, face_size: u32, mip_count: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: face_size,
                height: face_size,
                depth_or_array_layers: CUBE_FACE_COUNT,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: IBL_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let sampled = texture.create_view(&sampled_view_descriptor(0, mip_count));
        let sampled_mips = (0..mip_count)
            .map(|mip_level| texture.create_view(&sampled_view_descriptor(mip_level, 1)))
            .collect();
        let storage_mips = (0..mip_count)
            .map(|mip_level| texture.create_view(&storage_view_descriptor(mip_level)))
            .collect();
        Self {
            _texture: texture,
            sampled,
            sampled_mips,
            storage_mips,
        }
    }
}

fn bind_texture_views(
    graph: &RealtimeIblGraphTextureResources,
    gpu: &RealtimeIblGpuTextureResources,
    required_resource_names: &[String],
    resources: &mut RenderGraphExecutionResources,
) -> Result<(), String> {
    if graph.sampled_mips.len() != gpu.sampled_mips.len()
        || graph.storage_mips.len() != gpu.storage_mips.len()
    {
        return Err(format!(
            "realtime IBL graph texture `{}` view counts do not match its GPU allocation",
            graph.sampled.name
        ));
    }
    if is_required_resource(required_resource_names, &graph.sampled.name) {
        resources.import_borrowed_texture_view(&graph.sampled.name, &gpu.sampled);
    }
    for (logical, view) in graph.sampled_mips.iter().zip(&gpu.sampled_mips) {
        if is_required_resource(required_resource_names, &logical.name) {
            resources.import_borrowed_texture_view(&logical.name, view);
        }
    }
    for (logical, view) in graph.storage_mips.iter().zip(&gpu.storage_mips) {
        if is_required_resource(required_resource_names, &logical.name) {
            resources.import_borrowed_texture_view(&logical.name, view);
        }
    }
    Ok(())
}

fn is_required_resource(required_resource_names: &[String], name: &str) -> bool {
    required_resource_names
        .binary_search_by(|candidate| candidate.as_str().cmp(name))
        .is_ok()
}

fn sampled_view_descriptor(
    base_mip_level: u32,
    mip_level_count: u32,
) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-realtime-ibl-sampled-cube-view"),
        format: Some(IBL_TEXTURE_FORMAT),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level,
        mip_level_count: Some(mip_level_count),
        base_array_layer: 0,
        array_layer_count: Some(CUBE_FACE_COUNT),
    }
}

fn storage_view_descriptor(mip_level: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("zircon-realtime-ibl-storage-mip-view"),
        format: Some(IBL_TEXTURE_FORMAT),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        usage: Some(wgpu::TextureUsages::STORAGE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(CUBE_FACE_COUNT),
    }
}

#[cfg(test)]
mod tests;

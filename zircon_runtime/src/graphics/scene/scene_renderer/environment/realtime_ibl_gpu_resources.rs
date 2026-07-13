use std::collections::HashSet;

use crate::core::framework::render::{IblBakeArtifactRequest, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::CompiledRenderGraph;

use super::realtime_ibl_graph_plan::{
    RealtimeIblGraphPlan, RealtimeIblGraphSlotResources, RealtimeIblGraphTextureResources,
};
use super::realtime_ibl_time_slice::IblRealtimeBufferSlot;

const CUBE_FACE_COUNT: u32 = 6;
const IBL_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(in crate::graphics) struct RealtimeIblGpuResources {
    slot_a: RealtimeIblGpuSlotResources,
    slot_b: RealtimeIblGpuSlotResources,
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
        }
    }

    pub(in crate::graphics) fn bind_graph_plan(
        &self,
        plan: &RealtimeIblGraphPlan,
        graph: &CompiledRenderGraph,
        resources: &mut RenderGraphExecutionResources,
    ) -> Result<(), String> {
        let live_resource_names = graph
            .resource_lifetimes()
            .iter()
            .map(|lifetime| lifetime.name.as_str())
            .collect::<HashSet<_>>();
        self.bind_slot(&plan.ready, &live_resource_names, resources)?;
        self.bind_slot(&plan.work, &live_resource_names, resources)
    }

    fn bind_slot(
        &self,
        graph: &RealtimeIblGraphSlotResources,
        live_resource_names: &HashSet<&str>,
        resources: &mut RenderGraphExecutionResources,
    ) -> Result<(), String> {
        let gpu = self.slot(graph.slot);
        bind_texture_views(&graph.source, &gpu.source, live_resource_names, resources)?;
        bind_texture_views(&graph.pmrem, &gpu.pmrem, live_resource_names, resources)?;
        if live_resource_names.contains(graph.sh9.name.as_str()) {
            resources.bind_execution_owned_buffer(&graph.sh9.name, &graph.sh9.name, &gpu.sh9);
        }
        Ok(())
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
    live_resource_names: &HashSet<&str>,
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
    if live_resource_names.contains(graph.sampled.name.as_str()) {
        resources.import_borrowed_texture_view(&graph.sampled.name, &gpu.sampled);
    }
    for (logical, view) in graph.sampled_mips.iter().zip(&gpu.sampled_mips) {
        if live_resource_names.contains(logical.name.as_str()) {
            resources.import_borrowed_texture_view(&logical.name, view);
        }
    }
    for (logical, view) in graph.storage_mips.iter().zip(&gpu.storage_mips) {
        if live_resource_names.contains(logical.name.as_str()) {
            resources.import_borrowed_texture_view(&logical.name, view);
        }
    }
    Ok(())
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

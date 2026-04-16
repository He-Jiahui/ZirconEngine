use super::super::deferred::DeferredSceneResources;
use super::super::hybrid_gi::HybridGiGpuResources;
use super::super::mesh::MeshPipelineCache;
use super::super::overlay::ViewportOverlayRenderer;
use super::super::particle::ParticleRenderer;
use super::super::post_process::ScenePostProcessResources;
use super::super::prepass::NormalPrepassPipeline;
use super::super::virtual_geometry::VirtualGeometryGpuResources;

pub(crate) struct SceneRendererCore {
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) scene_uniform_buffer: wgpu::Buffer,
    pub(super) scene_bind_group: wgpu::BindGroup,
    pub(super) model_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) mesh_pipelines: MeshPipelineCache,
    pub(super) normal_prepass: NormalPrepassPipeline,
    pub(super) deferred: DeferredSceneResources,
    pub(super) particle_renderer: ParticleRenderer,
    pub(super) post_process: ScenePostProcessResources,
    pub(super) overlay_renderer: ViewportOverlayRenderer,
    pub(super) hybrid_gi: HybridGiGpuResources,
    pub(super) virtual_geometry: VirtualGeometryGpuResources,
}

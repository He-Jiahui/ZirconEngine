use std::sync::Arc;

use super::super::deferred::DeferredSceneResources;
use super::super::hybrid_gi::HybridGiGpuResources;
use super::super::mesh::MeshPipelineCache;
use super::super::overlay::{EmptyViewportIconSource, ViewportIconSource, ViewportOverlayRenderer};
use super::super::particle::ParticleRenderer;
use super::super::post_process::ScenePostProcessResources;
use super::super::prepass::NormalPrepassPipeline;
use super::super::primitives::SceneUniform;
use super::super::virtual_geometry::VirtualGeometryGpuResources;
use super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        Self::new_with_icon_source(
            device,
            queue,
            target_format,
            Arc::new(EmptyViewportIconSource),
        )
    }

    pub(crate) fn new_with_icon_source(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Self {
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-scene-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let scene_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-scene-uniform"),
            size: std::mem::size_of::<SceneUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });
        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-model-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-texture-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let mesh_pipelines = MeshPipelineCache::new(
            device,
            target_format,
            &scene_bind_group_layout,
            &model_bind_group_layout,
            &texture_bind_group_layout,
        );
        let normal_prepass =
            NormalPrepassPipeline::new(device, &scene_bind_group_layout, &model_bind_group_layout);
        let deferred = DeferredSceneResources::new(
            device,
            &scene_bind_group_layout,
            &model_bind_group_layout,
            &texture_bind_group_layout,
            target_format,
        );
        let particle_renderer =
            ParticleRenderer::new(device, &scene_bind_group_layout, target_format);
        let post_process = ScenePostProcessResources::new(device, queue, target_format);
        let overlay_renderer = ViewportOverlayRenderer::new(
            device,
            target_format,
            &scene_bind_group_layout,
            &texture_bind_group_layout,
            icon_source,
        );
        let hybrid_gi = HybridGiGpuResources::new(device);
        let virtual_geometry = VirtualGeometryGpuResources::new(device);

        Self {
            texture_bind_group_layout,
            scene_uniform_buffer,
            scene_bind_group,
            model_bind_group_layout,
            mesh_pipelines,
            normal_prepass,
            deferred,
            particle_renderer,
            post_process,
            overlay_renderer,
            hybrid_gi,
            virtual_geometry,
        }
    }
}

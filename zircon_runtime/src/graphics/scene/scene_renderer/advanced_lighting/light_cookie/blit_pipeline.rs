use crate::graphics::scene::resources::ResourceStreamer;

use super::frame_plan::CookieAtlasEntry;

const BLIT_SHADER: &str = include_str!("shaders/blit.wgsl");

pub(super) struct LightCookieAtlasBlitPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl LightCookieAtlasBlitPipeline {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-light-cookie-blit-bind-group-layout"),
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
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-light-cookie-blit-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-light-cookie-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-light-cookie-atlas-blit"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            bind_group_layout,
            pipeline,
        }
    }

    pub(super) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        streamer: &ResourceStreamer,
        entries: &[CookieAtlasEntry],
        cell_size: u32,
    ) -> usize {
        let draws = entries
            .iter()
            .filter_map(|entry| {
                let resource = streamer.texture(Some(entry.texture));
                (resource.id == Some(entry.texture)).then(|| {
                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("zircon-light-cookie-blit-bind-group"),
                        layout: &self.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(resource.view()),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(resource.sampler()),
                            },
                        ],
                    });
                    let slot = entry.metadata.misc[2];
                    let viewport = [
                        (slot % super::COOKIE_ATLAS_GRID_SIZE) * cell_size,
                        (slot / super::COOKIE_ATLAS_GRID_SIZE) * cell_size,
                    ];
                    (viewport, bind_group)
                })
            })
            .collect::<Vec<_>>();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("LightCookieAtlasBuildPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        for (viewport, bind_group) in &draws {
            pass.set_viewport(
                viewport[0] as f32,
                viewport[1] as f32,
                cell_size as f32,
                cell_size as f32,
                0.0,
                1.0,
            );
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        draws.len()
    }
}

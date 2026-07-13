use crate::graphics::scene::scene_renderer::sprite::SpriteVertex;

use super::OIT_DRAW_SHADER_SOURCE;

const OIT_SPRITE_SHADER: &str = r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

struct SpriteVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct SpriteVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: SpriteVertexInput) -> SpriteVertexOutput {
    var output: SpriteVertexOutput;
    output.clip_position = scene.view_proj * vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

@fragment
fn fs_oit(input: SpriteVertexOutput) {
    oit_draw(
        input.clip_position,
        textureSample(sprite_texture, sprite_sampler, input.uv) * input.color,
    );
}
"#;

pub(in crate::graphics::scene::scene_renderer) struct OitFragmentStorePipeline {
    depth_format: wgpu::TextureFormat,
    sprite_texture_layout: wgpu::BindGroupLayout,
    sprite_pipeline: wgpu::RenderPipeline,
}

impl OitFragmentStorePipeline {
    pub(in crate::graphics::scene::scene_renderer) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        oit_layout: &wgpu::BindGroupLayout,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let sprite_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-oit-sprite-texture-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-oit-sprite-empty-layout"),
            entries: &[],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-oit-sprite-pipeline-layout"),
            bind_group_layouts: &[
                Some(scene_layout),
                Some(&sprite_texture_layout),
                Some(&empty_layout),
                Some(&empty_layout),
                Some(oit_layout),
            ],
            immediate_size: 0,
        });
        let shader_source = format!("{OIT_DRAW_SHADER_SOURCE}\n{OIT_SPRITE_SHADER}");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-oit-sprite-shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-oit-sprite-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[SpriteVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_oit"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            depth_format,
            sprite_texture_layout,
            sprite_pipeline,
        }
    }

    pub(in crate::graphics::scene::scene_renderer) const fn depth_format(
        &self,
    ) -> wgpu::TextureFormat {
        self.depth_format
    }

    pub(in crate::graphics::scene::scene_renderer) fn sprite_pipeline(
        &self,
    ) -> &wgpu::RenderPipeline {
        &self.sprite_pipeline
    }

    pub(in crate::graphics::scene::scene_renderer) fn create_sprite_texture_bind_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-oit-sprite-texture-bind-group"),
            layout: &self.sprite_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}

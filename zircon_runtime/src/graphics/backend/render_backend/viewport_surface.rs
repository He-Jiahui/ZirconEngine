use std::num::NonZeroIsize;

use crate::core::framework::render::{RenderNativeSurfaceTarget, RenderViewportSurfaceDescriptor};
use crate::core::math::UVec2;
use crate::graphics::types::GraphicsError;

use super::render_backend::RenderBackend;

const SURFACE_FRAME_LATENCY: u32 = 2;
const PRESENT_BLIT_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0)
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, input.uv);
}
"#;

pub(crate) struct ViewportSurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    blit: SurfaceBlitResources,
}

impl ViewportSurface {
    pub(crate) fn size(&self) -> UVec2 {
        UVec2::new(self.config.width, self.config.height)
    }

    pub(crate) fn present_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source_view: &wgpu::TextureView,
    ) -> Result<(), GraphicsError> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(GraphicsError::SurfaceStatus("surface validation error"));
            }
        };
        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.blit
            .blit(device, queue, source_view, &target_view, self.size());
        surface_texture.present();
        Ok(())
    }
}

impl RenderBackend {
    pub(crate) fn create_viewport_surface(
        &self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<ViewportSurface, GraphicsError> {
        let surface = self.create_surface(descriptor.target)?;
        let config = configure_surface(&surface, &self.adapter, &self.device, descriptor.size)?;
        let blit = SurfaceBlitResources::new(&self.device, config.format);
        Ok(ViewportSurface {
            surface,
            config,
            blit,
        })
    }

    fn create_surface(
        &self,
        target: RenderNativeSurfaceTarget,
    ) -> Result<wgpu::Surface<'static>, GraphicsError> {
        match target {
            RenderNativeSurfaceTarget::Win32 { hwnd, hinstance } => {
                let hwnd = required_nonzero_isize(hwnd, "invalid win32 hwnd")?;
                let mut window = wgpu::rwh::Win32WindowHandle::new(hwnd);
                window.hinstance = optional_nonzero_isize(hinstance)?;
                let raw_window_handle = wgpu::rwh::RawWindowHandle::Win32(window);
                let raw_display_handle =
                    wgpu::rwh::RawDisplayHandle::Windows(wgpu::rwh::WindowsDisplayHandle::new());
                let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(raw_display_handle),
                    raw_window_handle,
                };
                // The app owns the native window and unbinds/drops the runtime surface before
                // the window is destroyed. The ABI carries raw handles, so wgpu must receive the
                // unsafe raw-handle target instead of a borrowed winit window object.
                unsafe { self.instance.create_surface_unsafe(target) }.map_err(Into::into)
            }
        }
    }
}

struct SurfaceBlitResources {
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl SurfaceBlitResources {
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-present-blit-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-present-blit-bind-group-layout"),
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
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-present-blit-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-present-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(PRESENT_BLIT_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-present-blit-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            sampler,
            bind_group_layout,
            pipeline,
        }
    }

    fn blit(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        size: UVec2,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-present-blit-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-present-blit-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("zircon-present-blit-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_viewport(0.0, 0.0, size.x as f32, size.y as f32, 0.0, 1.0);
            pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
    }
}

fn configure_surface(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    size: UVec2,
) -> Result<wgpu::SurfaceConfiguration, GraphicsError> {
    let size = UVec2::new(size.x.max(1), size.y.max(1));
    let caps = surface.get_capabilities(adapter);
    let Some(format) = choose_surface_format(&caps.formats) else {
        return Err(GraphicsError::SurfaceStatus(
            "surface has no compatible formats",
        ));
    };
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.x,
        height: size.y,
        present_mode: choose_present_mode(&caps.present_modes),
        desired_maximum_frame_latency: SURFACE_FRAME_LATENCY,
        alpha_mode: caps
            .alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto),
        view_formats: vec![],
    };
    surface.configure(device, &config);
    Ok(config)
}

fn choose_surface_format(formats: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    [
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
    ]
    .into_iter()
    .find(|format| formats.contains(format))
    .or_else(|| formats.first().copied())
}

fn choose_present_mode(present_modes: &[wgpu::PresentMode]) -> wgpu::PresentMode {
    if present_modes.contains(&wgpu::PresentMode::AutoVsync) {
        wgpu::PresentMode::AutoVsync
    } else {
        wgpu::PresentMode::Fifo
    }
}

fn required_nonzero_isize(value: u64, error: &'static str) -> Result<NonZeroIsize, GraphicsError> {
    if value == 0 || value > isize::MAX as u64 {
        return Err(GraphicsError::SurfaceStatus(error));
    }
    Ok(NonZeroIsize::new(value as isize).expect("value checked above"))
}

fn optional_nonzero_isize(value: Option<u64>) -> Result<Option<NonZeroIsize>, GraphicsError> {
    value
        .map(|value| required_nonzero_isize(value, "invalid win32 hinstance"))
        .transpose()
}

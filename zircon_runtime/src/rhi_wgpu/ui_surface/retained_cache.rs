use wgpu::util::DeviceExt;

use super::geometry::ImageVertex;

pub(super) struct WgpuRetainedSurfaceCache {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    restore_vertices: wgpu::Buffer,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    initialized: bool,
}

impl WgpuRetainedSurfaceCache {
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Self {
        let size = (size.0.max(1), size.1.max(1));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-retained-cache"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-ui-retained-cache-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        let restore_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-ui-retained-cache-restore-vertices"),
            contents: bytemuck::cast_slice(&restore_quad_vertices()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            _texture: texture,
            view,
            bind_group,
            restore_vertices,
            size,
            format,
            initialized: false,
        }
    }

    pub(super) fn resize(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) {
        *self = Self::new(device, format, size, layout, sampler);
    }

    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(super) fn initialized(&self) -> bool {
        self.initialized
    }

    pub(super) fn matches(&self, format: wgpu::TextureFormat, size: (u32, u32)) -> bool {
        self.format == format && self.size == (size.0.max(1), size.1.max(1))
    }

    pub(super) fn mark_initialized(&mut self) {
        self.initialized = true;
    }

    pub(super) fn record_restore(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        image_pipeline: &wgpu::RenderPipeline,
        target_view: &wgpu::TextureView,
        surface_size: (u32, u32),
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-ui-retained-cache-restore-pass"),
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
        pass.set_viewport(
            0.0,
            0.0,
            surface_size.0.max(1) as f32,
            surface_size.1.max(1) as f32,
            0.0,
            1.0,
        );
        pass.set_pipeline(image_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.restore_vertices.slice(..));
        pass.draw(0..6, 0..1);
    }
}

fn restore_quad_vertices() -> [ImageVertex; 6] {
    [
        ImageVertex {
            position: [-1.0, 1.0],
            uv: [0.0, 0.0],
        },
        ImageVertex {
            position: [1.0, 1.0],
            uv: [1.0, 0.0],
        },
        ImageVertex {
            position: [-1.0, -1.0],
            uv: [0.0, 1.0],
        },
        ImageVertex {
            position: [-1.0, -1.0],
            uv: [0.0, 1.0],
        },
        ImageVertex {
            position: [1.0, 1.0],
            uv: [1.0, 0.0],
        },
        ImageVertex {
            position: [1.0, -1.0],
            uv: [1.0, 1.0],
        },
    ]
}

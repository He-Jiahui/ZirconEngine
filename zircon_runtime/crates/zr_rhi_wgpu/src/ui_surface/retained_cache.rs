//! 普通损伤基线与缩放投影缓存具有不同有效性，只有成功提交后才能更新缓存状态。
const UI_SURFACE_COPY_BYTES_PER_PIXEL: u64 = 4;

pub(super) struct WgpuRetainedSurfaceCache {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    state: RetainedSurfaceState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RetainedSurfaceState {
    #[default]
    Uninitialized,
    OrdinaryBaseline,
    ResizeProjection(u64),
}

impl RetainedSurfaceState {
    const fn ordinary_baseline_ready(self) -> bool {
        matches!(self, Self::OrdinaryBaseline)
    }

    const fn is_projection_ready(self, generation: Option<u64>) -> bool {
        matches!(
            (self, generation),
            (Self::ResizeProjection(cached), Some(requested)) if cached == requested
        )
    }
}

impl WgpuRetainedSurfaceCache {
    pub(super) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            size,
            format,
            state: RetainedSurfaceState::Uninitialized,
        }
    }

    pub(super) fn resize(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) {
        *self = Self::new(device, format, size);
    }

    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub(super) const fn size(&self) -> (u32, u32) {
        self.size
    }

    pub(super) fn ordinary_baseline_ready(&self) -> bool {
        self.state.ordinary_baseline_ready()
    }

    pub(super) fn matches(&self, format: wgpu::TextureFormat, size: (u32, u32)) -> bool {
        self.format == format && self.size == (size.0.max(1), size.1.max(1))
    }

    // 缩放投影只对精确内容代际可复用，不能充当普通损伤更新的完整基线。
    pub(super) fn is_projection_ready(&self, generation: Option<u64>) -> bool {
        self.state.is_projection_ready(generation)
    }

    pub(super) fn mark_ordinary_baseline_ready(&mut self) {
        self.state = RetainedSurfaceState::OrdinaryBaseline;
    }

    pub(super) fn mark_projection_ready(&mut self, generation: Option<u64>) {
        self.state = generation.map_or(
            RetainedSurfaceState::Uninitialized,
            RetainedSurfaceState::ResizeProjection,
        );
    }

    pub(super) const fn copy_requires_target_clear(&self, surface_size: (u32, u32)) -> bool {
        surface_size.0 > self.size.0 || surface_size.1 > self.size.1
    }

    pub(super) fn record_copy_to_surface(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture: &wgpu::Texture,
        surface_view: &wgpu::TextureView,
        surface_size: (u32, u32),
    ) -> u64 {
        if self.copy_requires_target_clear(surface_size) {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("zircon-ui-retained-cache-target-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
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
        }
        let copy_size = (
            self.size.0.min(surface_size.0),
            self.size.1.min(surface_size.1),
        );
        encoder.copy_texture_to_texture(
            self.texture.as_image_copy(),
            surface_texture.as_image_copy(),
            wgpu::Extent3d {
                width: copy_size.0.max(1),
                height: copy_size.1.max(1),
                depth_or_array_layers: 1,
            },
        );
        retained_copy_byte_count(self.format, copy_size)
    }
}

fn retained_copy_byte_count(format: wgpu::TextureFormat, surface_size: (u32, u32)) -> u64 {
    let bytes_per_pixel = match format {
        wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb
        | wgpu::TextureFormat::Rgba8UnormSrgb => UI_SURFACE_COPY_BYTES_PER_PIXEL,
        _ => 0,
    };
    u64::from(surface_size.0.max(1))
        .saturating_mul(u64::from(surface_size.1.max(1)))
        .saturating_mul(bytes_per_pixel)
}

#[cfg(test)]
#[path = "tests/retained_cache.rs"]
mod tests;

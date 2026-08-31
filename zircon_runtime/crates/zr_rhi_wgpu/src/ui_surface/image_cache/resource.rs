use super::super::shared_image_registry::WgpuUiImageSurfacePin;
use super::super::WgpuUiExternalImage;

pub(in super::super) struct WgpuUiImageResource {
    pub(in super::super) bind_group: wgpu::BindGroup,
    pub(super) shared_allocation_pin: Option<WgpuUiImageSurfacePin>,
    pub(super) size: (u32, u32),
    pub(super) byte_size: u64,
    pub(super) last_touched_present: u64,
    pub(super) last_uploaded_generation: Option<u64>,
}

impl WgpuUiImageResource {
    pub(super) fn from_external(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        image: WgpuUiExternalImage,
        byte_size: u64,
        last_touched_present: u64,
    ) -> Self {
        let view = image.create_sample_view();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-ui-external-image-bind-group"),
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
        Self {
            bind_group,
            shared_allocation_pin: image.shared_allocation_pin,
            size: (image.width, image.height),
            byte_size,
            last_touched_present,
            last_uploaded_generation: Some(image.generation),
        }
    }
}

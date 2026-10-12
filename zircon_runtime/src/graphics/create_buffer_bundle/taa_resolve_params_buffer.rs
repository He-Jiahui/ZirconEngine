use crate::graphics::scene::scene_renderer::temporal::taa::taa_resolve_params::TaaResolveParams;

pub(super) fn taa_resolve_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-taa-resolve-params"),
        size: std::mem::size_of::<TaaResolveParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

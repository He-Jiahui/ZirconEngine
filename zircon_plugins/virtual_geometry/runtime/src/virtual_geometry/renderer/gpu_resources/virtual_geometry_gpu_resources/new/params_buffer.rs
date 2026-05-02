use super::super::super::virtual_geometry_uploader_params::VirtualGeometryUploaderParams;

pub(super) fn create_uploader_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-uploader-params"),
        size: std::mem::size_of::<VirtualGeometryUploaderParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

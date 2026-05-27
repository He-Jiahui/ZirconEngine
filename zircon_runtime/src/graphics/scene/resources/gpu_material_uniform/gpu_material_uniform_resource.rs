use crate::core::framework::render::RenderMaterialPropertyUniformPayload;
use wgpu::util::DeviceExt;

pub(crate) const GPU_MATERIAL_UNIFORM_MIN_SIZE: usize = 64;

pub(crate) struct GpuMaterialUniformResource {
    #[allow(dead_code)]
    pub(in crate::graphics::scene::resources) buffer: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    pub(crate) payload_byte_len: u64,
    #[allow(dead_code)]
    pub(crate) buffer_byte_len: u64,
}

impl GpuMaterialUniformResource {
    pub(crate) fn from_payload(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        payload: &RenderMaterialPropertyUniformPayload,
    ) -> Self {
        let contents = padded_uniform_contents(payload);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-material-property-uniform-buffer"),
            contents: &contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-material-property-uniform-bind-group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            buffer,
            bind_group,
            payload_byte_len: payload.bytes.len() as u64,
            buffer_byte_len: contents.len() as u64,
        }
    }
}

fn padded_uniform_contents(payload: &RenderMaterialPropertyUniformPayload) -> Vec<u8> {
    let mut contents = payload.bytes.clone();
    contents.resize(contents.len().max(GPU_MATERIAL_UNIFORM_MIN_SIZE), 0);
    contents
}

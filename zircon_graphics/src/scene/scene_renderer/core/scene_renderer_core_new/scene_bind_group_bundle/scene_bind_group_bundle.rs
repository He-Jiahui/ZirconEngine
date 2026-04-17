pub(super) struct SceneBindGroupBundle {
    pub(super) layout: wgpu::BindGroupLayout,
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
}

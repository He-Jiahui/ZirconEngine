pub(super) fn create_cluster_buffer(
    device: &wgpu::Device,
    cluster_buffer_bytes: usize,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-cluster-buffer"),
        size: cluster_buffer_bytes.max(16) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

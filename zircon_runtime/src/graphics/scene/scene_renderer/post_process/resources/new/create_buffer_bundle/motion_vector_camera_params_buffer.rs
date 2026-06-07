use super::super::super::super::motion_vector_camera_params::MotionVectorCameraParams;

pub(super) fn motion_vector_camera_params_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-motion-vector-camera-params"),
        size: std::mem::size_of::<MotionVectorCameraParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

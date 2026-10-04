/// CPU MeshVertex 到 wgpu 顶点流的固定布局；位置、蒙皮、切线等通道位置须与各 mesh 管线一致。
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuMeshVertex {
    pub(crate) position: [f32; 3],
    pub(crate) normal: [f32; 3],
    pub(crate) uv: [f32; 2],
    pub(crate) joint_indices: [u16; 4],
    pub(crate) joint_weights: [f32; 4],
    pub(crate) tangent: [f32; 4],
    pub(crate) color: [f32; 4],
    pub(crate) uv1: [f32; 2],
}

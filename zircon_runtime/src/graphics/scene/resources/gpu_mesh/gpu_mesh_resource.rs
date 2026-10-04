use crate::core::math::Vec3;

/// 设备侧网格快照由 ResourceStreamer 按资产修订持有；绘制命令共享缓冲，
/// 间接绘制顺序签名参与同优先级命令的稳定排序，资源有效性由修订另行控制。
pub(crate) struct GpuMeshResource {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
    pub(super) indirect_order_signature: u64,
    pub(crate) wire_segments: Vec<[Vec3; 2]>,
    pub(crate) bounds_min: Vec3,
    pub(crate) bounds_max: Vec3,
}

impl GpuMeshResource {
    pub(crate) const fn indirect_order_signature(&self) -> u64 {
        self.indirect_order_signature
    }
}

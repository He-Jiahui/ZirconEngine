use std::sync::Arc;

use crate::core::resource::ResourceId;

use super::super::GpuMeshResource;

/// 模型资源把多个已解析 primitive 的 GPU 网格保持为一组；调用方按模型修订替换整组，
/// 绘制侧通过 Arc 保留当前帧仍在使用的旧网格。
pub(crate) struct GpuModelResource {
    pub(super) id: ResourceId,
    pub(crate) meshes: Vec<Arc<GpuMeshResource>>,
}

impl GpuModelResource {
    pub(crate) const fn id(&self) -> ResourceId {
        self.id
    }
}

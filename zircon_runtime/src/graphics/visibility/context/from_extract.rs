use crate::core::framework::render::RenderFrameExtract;

use super::super::declarations::VisibilityContext;

impl VisibilityContext {
    /// 没有前帧历史时构建当前帧可见性；增量 BVH、粒子和虚拟几何缓存会按首次提交处理。
    pub fn from_extract(value: &RenderFrameExtract) -> Self {
        Self::from_extract_with_history(value, None)
    }
}

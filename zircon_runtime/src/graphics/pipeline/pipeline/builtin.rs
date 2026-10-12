//! 内建管线提供默认视图的作者模板，后续仍走与自定义管线相同的编译及能力校验。
use crate::core::framework::render::{CorePipelineKind, RenderPipelineHandle};

use crate::graphics::pipeline::declarations::RenderPipelineAsset;

impl RenderPipelineAsset {
    pub(crate) const DEFAULT_FORWARD_PLUS_HANDLE: RenderPipelineHandle =
        RenderPipelineHandle::new(1);
    pub(crate) const DEFAULT_DEFERRED_HANDLE: RenderPipelineHandle = RenderPipelineHandle::new(2);
    pub(crate) const DEFAULT_CORE2D_HANDLE: RenderPipelineHandle = RenderPipelineHandle::new(3);

    pub(crate) const fn default_handle_for_core_pipeline(
        core_pipeline: CorePipelineKind,
    ) -> RenderPipelineHandle {
        match core_pipeline {
            CorePipelineKind::Core2d => Self::DEFAULT_CORE2D_HANDLE,
            CorePipelineKind::Core3d => Self::DEFAULT_FORWARD_PLUS_HANDLE,
        }
    }

    pub fn builtin(handle: RenderPipelineHandle) -> Option<Self> {
        match handle.raw() {
            raw if raw == Self::DEFAULT_FORWARD_PLUS_HANDLE.raw() => {
                Some(Self::default_forward_plus())
            }
            raw if raw == Self::DEFAULT_DEFERRED_HANDLE.raw() => Some(Self::default_deferred()),
            raw if raw == Self::DEFAULT_CORE2D_HANDLE.raw() => Some(Self::default_core2d()),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "tests/builtin.rs"]
mod tests;

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
mod tests {
    use crate::core::framework::render::CorePipelineKind;

    use super::RenderPipelineAsset;

    #[test]
    fn default_pipeline_handles_match_builtin_assets() {
        assert_eq!(
            RenderPipelineAsset::default_handle_for_core_pipeline(CorePipelineKind::Core3d),
            RenderPipelineAsset::default_forward_plus().handle
        );
        assert_eq!(
            RenderPipelineAsset::DEFAULT_DEFERRED_HANDLE,
            RenderPipelineAsset::default_deferred().handle
        );
        assert_eq!(
            RenderPipelineAsset::default_handle_for_core_pipeline(CorePipelineKind::Core2d),
            RenderPipelineAsset::default_core2d().handle
        );
    }
}

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

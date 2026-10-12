use crate::scene::dynamic_scene::DynamicSceneError;

pub(super) fn scene_asset_error(
    context: impl AsRef<str>,
    error: impl std::fmt::Display,
) -> DynamicSceneError {
    DynamicSceneError::SceneAsset {
        reason: format!("{}: {error}", context.as_ref()),
    }
}

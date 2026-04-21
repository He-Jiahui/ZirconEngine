use super::super::error::EditorBindingDispatchError;

pub(super) fn parse_asset_surface(
    surface: &str,
) -> Result<crate::core::editor_event::EditorAssetSurface, EditorBindingDispatchError> {
    match surface {
        "activity" => Ok(crate::core::editor_event::EditorAssetSurface::Activity),
        "browser" => Ok(crate::core::editor_event::EditorAssetSurface::Browser),
        _ => Err(EditorBindingDispatchError::StateMutation(format!(
            "unknown asset surface {surface}"
        ))),
    }
}

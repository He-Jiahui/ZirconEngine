use super::super::error::EditorBindingDispatchError;

pub(super) fn parse_asset_surface(
    surface: &str,
) -> Result<crate::EditorAssetSurface, EditorBindingDispatchError> {
    match surface {
        "activity" => Ok(crate::EditorAssetSurface::Activity),
        "browser" => Ok(crate::EditorAssetSurface::Browser),
        _ => Err(EditorBindingDispatchError::StateMutation(format!(
            "unknown asset surface {surface}"
        ))),
    }
}

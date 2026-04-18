use super::super::error::EditorBindingDispatchError;

pub(super) fn parse_asset_view_mode(
    mode: &str,
) -> Result<crate::EditorAssetViewMode, EditorBindingDispatchError> {
    match mode {
        "list" => Ok(crate::EditorAssetViewMode::List),
        "thumbnail" => Ok(crate::EditorAssetViewMode::Thumbnail),
        _ => Err(EditorBindingDispatchError::StateMutation(format!(
            "unknown asset view mode {mode}"
        ))),
    }
}

use super::super::error::EditorBindingDispatchError;

pub(super) fn parse_asset_view_mode(
    mode: &str,
) -> Result<crate::core::editor_event::EditorAssetViewMode, EditorBindingDispatchError> {
    match mode {
        "list" => Ok(crate::core::editor_event::EditorAssetViewMode::List),
        "thumbnail" => Ok(crate::core::editor_event::EditorAssetViewMode::Thumbnail),
        _ => Err(EditorBindingDispatchError::StateMutation(format!(
            "unknown asset view mode {mode}"
        ))),
    }
}

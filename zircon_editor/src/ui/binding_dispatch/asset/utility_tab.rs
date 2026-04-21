use super::super::error::EditorBindingDispatchError;

pub(super) fn parse_asset_utility_tab(
    tab: &str,
) -> Result<crate::core::editor_event::EditorAssetUtilityTab, EditorBindingDispatchError> {
    match tab {
        "preview" => Ok(crate::core::editor_event::EditorAssetUtilityTab::Preview),
        "references" => Ok(crate::core::editor_event::EditorAssetUtilityTab::References),
        "metadata" => Ok(crate::core::editor_event::EditorAssetUtilityTab::Metadata),
        "plugins" => Ok(crate::core::editor_event::EditorAssetUtilityTab::Plugins),
        _ => Err(EditorBindingDispatchError::StateMutation(format!(
            "unknown asset utility tab {tab}"
        ))),
    }
}

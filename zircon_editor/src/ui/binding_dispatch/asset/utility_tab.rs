use super::super::error::EditorBindingDispatchError;

pub(super) fn parse_asset_utility_tab(
    tab: &str,
) -> Result<crate::EditorAssetUtilityTab, EditorBindingDispatchError> {
    match tab {
        "preview" => Ok(crate::EditorAssetUtilityTab::Preview),
        "references" => Ok(crate::EditorAssetUtilityTab::References),
        "metadata" => Ok(crate::EditorAssetUtilityTab::Metadata),
        "plugins" => Ok(crate::EditorAssetUtilityTab::Plugins),
        _ => Err(EditorBindingDispatchError::StateMutation(format!(
            "unknown asset utility tab {tab}"
        ))),
    }
}

use zircon_runtime_interface::ui::{template::parse_component_reference, v2::UiV2AssetError};

pub(crate) enum UiV2WidgetImportReference<'a> {
    WholeAsset(&'a str),
    Component {
        asset_id: &'a str,
        component: &'a str,
    },
}

pub(crate) fn parse_v2_component_reference<'a>(
    owner_asset_id: &str,
    reference: &'a str,
) -> Result<(&'a str, &'a str), UiV2AssetError> {
    parse_component_reference(reference).map_err(|error| UiV2AssetError::InvalidDocument {
        asset_id: owner_asset_id.to_string(),
        detail: error.to_string(),
    })
}

/// V2 widget imports support both complete component assets and one named component.
/// A named component always passes through the shared reference parser so malformed
/// fragments fail while sources are being loaded instead of becoming a later miss.
pub(crate) fn parse_v2_widget_import_reference<'a>(
    owner_asset_id: &str,
    reference: &'a str,
) -> Result<UiV2WidgetImportReference<'a>, UiV2AssetError> {
    if reference.contains('#') {
        let (asset_id, component) = parse_v2_component_reference(owner_asset_id, reference)?;
        Ok(UiV2WidgetImportReference::Component {
            asset_id,
            component,
        })
    } else {
        Ok(UiV2WidgetImportReference::WholeAsset(reference))
    }
}

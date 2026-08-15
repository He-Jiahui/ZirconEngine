use zircon_runtime_interface::ui::template::{
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION, UiAssetDocument, UiAssetHeader, UiAssetImports,
    UiAssetKind,
};

use super::merge::{theme_base_name, theme_display_name};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalStyleDraft {
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
    pub(crate) display_name: String,
}

pub(crate) fn can_promote_local_theme_to_external_style_asset(document: &UiAssetDocument) -> bool {
    !document.tokens.is_empty() || !document.stylesheets.is_empty()
}

pub(crate) fn default_external_style_draft(
    source_asset_id: &str,
    source_display_name: &str,
) -> UiAssetExternalStyleDraft {
    let base_name = theme_base_name(source_asset_id);
    UiAssetExternalStyleDraft {
        asset_id: format!("res://ui/themes/{base_name}_theme.zui"),
        document_id: format!("ui.theme.{base_name}_theme"),
        display_name: theme_display_name(source_display_name, &base_name),
    }
}

pub(crate) fn promote_local_theme_to_external_style_asset(
    document: &mut UiAssetDocument,
    style_asset_id: &str,
    style_document_id: &str,
    display_name: &str,
) -> Option<UiAssetDocument> {
    if !can_promote_local_theme_to_external_style_asset(document) {
        return None;
    }

    let promoted_theme = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: style_document_id.to_string(),
            version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            display_name: display_name.to_string(),
        },
        imports: UiAssetImports {
            widgets: Vec::new(),
            styles: document.imports.styles.clone(),
            resources: Vec::new(),
        },
        tokens: std::mem::take(&mut document.tokens),
        root: None,
        components: Default::default(),
        stylesheets: std::mem::take(&mut document.stylesheets),
    };

    document.imports.styles.clear();
    document.imports.styles.push(style_asset_id.to_string());

    Some(promoted_theme)
}

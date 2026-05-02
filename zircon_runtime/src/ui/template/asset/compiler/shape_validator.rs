use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiAssetKind};

pub(super) fn validate_document_shape(document: &UiAssetDocument) -> Result<(), UiAssetError> {
    match document.asset.kind {
        UiAssetKind::Layout | UiAssetKind::Widget => {
            if document.root.is_none() {
                return Err(UiAssetError::InvalidDocument {
                    asset_id: document.asset.id.clone(),
                    detail: "layout/widget assets require [root]".to_string(),
                });
            }
        }
        UiAssetKind::Style => {}
    }
    Ok(())
}

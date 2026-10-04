use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiAssetKind};

// 只守住资产类别的最低结构要求；style 可作为导入而没有根，实际编译可实例化文档时仍由展开入口要求根节点。
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

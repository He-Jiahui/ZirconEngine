use crate::asset::assets::{ImportedAsset, UiIconAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

// .icon.toml 作者文档以 Texture 类别登记，但保留 UiIcon 的语义 ID、默认尺寸及 SVG/bitmap 来源；
// 外部图像引用由资产契约提取，导入入口只负责生成带源 URI 的 typed outcome。
pub(crate) fn import_ui_icon_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let asset = UiIconAsset::from_toml_str(document).map_err(|source| {
        AssetImportError::UiIconDocument {
            context: "parse ui icon asset",
            source,
        }
    })?;

    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::UiIcon(asset),
    ))
}

#[cfg(test)]
#[path = "tests/import_ui_icon_asset_plugins07_ui_icon_source_tests.rs"]
mod plugins07_ui_icon_source_tests;

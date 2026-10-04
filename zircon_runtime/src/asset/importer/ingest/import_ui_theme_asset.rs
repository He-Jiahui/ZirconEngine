use crate::asset::assets::{ImportedAsset, UiThemeAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

// .theme.toml 以 UiStyle 类别进入目录，仍保留 UiTheme 文档以供 UI 主题加载器使用。
pub(crate) fn import_ui_theme_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let asset = UiThemeAsset::from_toml_str(document).map_err(|source| {
        AssetImportError::UiThemeDocument {
            context: "parse ui theme asset",
            source,
        }
    })?;

    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::UiTheme(asset),
    ))
}

#[cfg(test)]
#[path = "tests/import_ui_theme_asset_plugins07_ui_theme_source_tests.rs"]
mod plugins07_ui_theme_source_tests;

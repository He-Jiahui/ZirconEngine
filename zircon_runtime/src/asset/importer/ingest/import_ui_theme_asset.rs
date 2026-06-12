use crate::asset::assets::{ImportedAsset, UiThemeAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_ui_theme_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let asset = UiThemeAsset::from_toml_str(&document).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse ui theme asset {}: {error}",
            context.source_path.display()
        ))
    })?;

    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::UiTheme(asset),
    ))
}

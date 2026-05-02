use crate::asset::assets::{ImportedAsset, SceneAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_scene(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let scene = SceneAsset::from_toml_str(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse scene toml: {error}")))?;
    Ok(AssetImportOutcome::new(ImportedAsset::Scene(scene)))
}

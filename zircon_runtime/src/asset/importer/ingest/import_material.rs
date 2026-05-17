use crate::asset::assets::{ImportedAsset, MaterialAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_material(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let material = MaterialAsset::from_toml_str(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse zmaterial toml: {error}")))?;
    let mut outcome = AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Material(material.clone()),
    )
    .with_dependency(material.shader.locator.clone());
    for (_, texture) in material.all_texture_slots() {
        outcome = outcome.with_dependency(texture.locator.clone());
    }
    Ok(outcome)
}

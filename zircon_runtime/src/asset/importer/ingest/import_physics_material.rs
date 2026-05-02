use crate::asset::assets::ImportedAsset;
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, PhysicsMaterialAsset,
};

pub(crate) fn import_physics_material(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    PhysicsMaterialAsset::from_toml_str(&document)
        .map(ImportedAsset::PhysicsMaterial)
        .map(AssetImportOutcome::new)
        .map_err(|error| AssetImportError::Parse(error.to_string()))
}

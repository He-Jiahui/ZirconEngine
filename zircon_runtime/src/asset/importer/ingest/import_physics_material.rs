use crate::asset::assets::ImportedAsset;
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, PhysicsMaterialAsset,
};

pub(crate) fn import_physics_material(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    PhysicsMaterialAsset::from_toml_str(document)
        .map(ImportedAsset::PhysicsMaterial)
        .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
        .map_err(|error| AssetImportError::Parse(error.to_string()))
}

#[cfg(test)]
#[path = "tests/import_physics_material_plugins07_physics_material_source_tests.rs"]
mod plugins07_physics_material_source_tests;

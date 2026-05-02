use super::primitive_from_indexed_mesh::backfill_virtual_geometry_for_model;
use crate::asset::assets::{ImportedAsset, ModelAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_model(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let mut model = ModelAsset::from_toml_str(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse model toml: {error}")))?;
    backfill_virtual_geometry_for_model(&mut model);
    Ok(AssetImportOutcome::new(ImportedAsset::Model(model)))
}

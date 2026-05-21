use crate::asset::assets::{ImportedAsset, ZMeshDocument};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_zmesh(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let zmesh = ZMeshDocument::from_toml_str(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse zmesh toml: {error}")))?;
    let mesh = zmesh
        .into_mesh_asset(context.uri.clone())
        .map_err(|error| AssetImportError::Parse(format!("validate zmesh: {error}")))?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Mesh(mesh),
    ))
}

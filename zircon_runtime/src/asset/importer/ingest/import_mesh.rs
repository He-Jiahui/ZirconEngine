use crate::asset::assets::{ImportedAsset, ZMeshDocument};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, cook_mesh_sdf_or_fallback_single,
};

pub(crate) fn import_zmesh(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let zmesh = ZMeshDocument::from_toml_str(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse zmesh toml: {error}")))?;
    let mut mesh = zmesh
        .into_mesh_asset(context.uri.clone())
        .map_err(|error| AssetImportError::Parse(format!("validate zmesh: {error}")))?;
    if mesh.mesh_sdf.is_none() {
        if let Some(settings) = context.mesh_sdf_cook_request()?.settings() {
            let primitive = mesh
                .to_model_primitive()
                .map_err(|error| AssetImportError::Parse(format!("validate zmesh: {error}")))?;
            mesh.mesh_sdf =
                cook_mesh_sdf_or_fallback_single(&primitive.vertices, &primitive.indices, settings)
                    .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?;
        }
    }
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Mesh(mesh),
    ))
}

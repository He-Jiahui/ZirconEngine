use super::primitive_from_indexed_mesh::primitive_from_indexed_mesh;
use crate::asset::assets::{ImportedAsset, ModelAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_obj(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let (models, _) = tobj::load_obj(
        &context.source_path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .map_err(|error| AssetImportError::Parse(format!("parse obj: {error}")))?;

    let source_hint = context.uri.to_string();
    let primitives = models
        .into_iter()
        .map(|model| {
            primitive_from_indexed_mesh(
                &model.mesh.positions,
                &model.mesh.normals,
                &model.mesh.texcoords,
                &model.mesh.indices,
                &[],
                &[],
                Some(model.name.as_str()),
                &source_hint,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AssetImportOutcome::new(ImportedAsset::Model(ModelAsset {
        uri: context.uri.clone(),
        primitives,
    })))
}

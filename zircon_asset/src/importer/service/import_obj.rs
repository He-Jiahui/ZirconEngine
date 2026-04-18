use std::path::Path;

use super::primitive_from_indexed_mesh::primitive_from_indexed_mesh;
use super::AssetImporter;
use crate::assets::{ImportedAsset, ModelAsset};
use crate::{AssetImportError, AssetUri};

impl AssetImporter {
    pub(super) fn import_obj(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let (models, _) = tobj::load_obj(
            source_path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|error| AssetImportError::Parse(format!("parse obj: {error}")))?;

        let primitives = models
            .into_iter()
            .map(|model| {
                primitive_from_indexed_mesh(
                    &model.mesh.positions,
                    &model.mesh.normals,
                    &model.mesh.texcoords,
                    &model.mesh.indices,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ImportedAsset::Model(ModelAsset {
            uri: uri.clone(),
            primitives,
        }))
    }
}

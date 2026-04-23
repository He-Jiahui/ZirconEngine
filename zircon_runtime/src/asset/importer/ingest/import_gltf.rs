use std::path::Path;

use super::primitive_from_indexed_mesh::primitive_from_indexed_mesh;
use super::AssetImporter;
use crate::asset::assets::{ImportedAsset, ModelAsset};
use crate::asset::{AssetImportError, AssetUri};

impl AssetImporter {
    pub(super) fn import_gltf(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let (document, buffers, _) = gltf::import(source_path)
            .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
        let mut primitives = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
                let positions = reader
                    .read_positions()
                    .ok_or_else(|| {
                        AssetImportError::Parse("gltf primitive missing positions".to_string())
                    })?
                    .flat_map(|position| position.into_iter())
                    .collect::<Vec<_>>();
                let normals = reader
                    .read_normals()
                    .map(|iter| {
                        iter.flat_map(|normal| normal.into_iter())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let texcoords = reader
                    .read_tex_coords(0)
                    .map(|set| {
                        set.into_f32()
                            .flat_map(|uv| uv.into_iter())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let joint_indices = reader
                    .read_joints(0)
                    .map(|set| set.into_u16().collect::<Vec<_>>())
                    .unwrap_or_default();
                let joint_weights = reader
                    .read_weights(0)
                    .map(|set| set.into_f32().collect::<Vec<_>>())
                    .unwrap_or_default();
                let indices = reader
                    .read_indices()
                    .map(|indices| indices.into_u32().collect::<Vec<_>>())
                    .unwrap_or_else(|| {
                        let vertex_count = positions.len() / 3;
                        (0..vertex_count as u32).collect()
                    });

                primitives.push(primitive_from_indexed_mesh(
                    &positions,
                    &normals,
                    &texcoords,
                    &indices,
                    &joint_indices,
                    &joint_weights,
                )?);
            }
        }

        Ok(ImportedAsset::Model(ModelAsset {
            uri: uri.clone(),
            primitives,
        }))
    }
}

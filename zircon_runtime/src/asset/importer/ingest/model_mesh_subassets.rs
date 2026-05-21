use crate::asset::assets::{ImportedAsset, MeshAsset, ModelAsset};
use crate::asset::{AssetImportOutcome, AssetUri, ImportedAssetEntry};

pub(super) fn model_outcome_with_mesh_subassets(
    root_uri: AssetUri,
    model: ModelAsset,
) -> AssetImportOutcome {
    model.primitives.iter().enumerate().fold(
        AssetImportOutcome::new(root_uri.clone(), ImportedAsset::Model(model.clone())),
        |outcome, (primitive_index, primitive)| {
            let mesh_uri = model_primitive_mesh_uri(&root_uri, primitive_index);
            outcome
                .with_dependency(mesh_uri.clone())
                .with_entry(ImportedAssetEntry::new(
                    mesh_uri.clone(),
                    ImportedAsset::Mesh(MeshAsset::from_model_primitive(mesh_uri, primitive)),
                ))
        },
    )
}

fn model_primitive_mesh_uri(root_uri: &AssetUri, primitive_index: usize) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#Mesh{primitive_index}/Primitive0"))
        .expect("generated model primitive mesh uri must be valid")
}

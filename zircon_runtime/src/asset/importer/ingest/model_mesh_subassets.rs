use crate::asset::assets::{ImportedAsset, MeshAsset, ModelAsset};
use crate::asset::{AssetImportOutcome, AssetReference, AssetUri, ImportedAssetEntry};

pub(super) fn model_outcome_with_mesh_subassets(
    root_uri: AssetUri,
    mut model: ModelAsset,
) -> AssetImportOutcome {
    let mesh_uris = (0..model.primitives.len())
        .map(|primitive_index| model_primitive_mesh_uri(&root_uri, primitive_index))
        .collect::<Vec<_>>();
    for (primitive, mesh_uri) in model.primitives.iter_mut().zip(mesh_uris.iter()) {
        primitive.mesh = Some(AssetReference::from_locator(mesh_uri.clone()));
    }

    mesh_uris.into_iter().zip(model.primitives.iter()).fold(
        AssetImportOutcome::new(root_uri.clone(), ImportedAsset::Model(model.clone())),
        |outcome, (mesh_uri, primitive)| {
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

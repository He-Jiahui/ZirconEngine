use crate::asset::assets::{ImportedAsset, MeshAsset, ModelAsset};
use crate::asset::{AssetImportOutcome, AssetReference, AssetUri, ImportedAssetEntry};

pub(super) fn model_outcome_with_mesh_subassets(
    root_uri: AssetUri,
    mut model: ModelAsset,
) -> AssetImportOutcome {
    let primitive_count = model.primitives.len();
    let mut dependencies = Vec::new();
    let mut mesh_entries = Vec::new();
    reserve_model_subasset_outputs(primitive_count, &mut dependencies, &mut mesh_entries);
    for (primitive_index, primitive) in model.primitives.iter_mut().enumerate() {
        if let Some(mesh) = primitive.mesh.as_ref() {
            dependencies.push(mesh.locator.clone());
            continue;
        }
        let mesh_uri = model_primitive_mesh_uri(&root_uri, primitive_index);
        primitive.mesh = Some(AssetReference::from_locator(mesh_uri.clone()));
        let mut mesh = MeshAsset::from_model_primitive(mesh_uri.clone(), primitive);
        mesh.mesh_sdf = primitive.mesh_sdf.take();
        dependencies.push(mesh_uri.clone());
        mesh_entries.push(ImportedAssetEntry::new(mesh_uri, ImportedAsset::Mesh(mesh)));
    }

    let mut outcome = AssetImportOutcome::new(root_uri, ImportedAsset::Model(model));
    outcome.entries.reserve(mesh_entries.len());
    outcome.entries[0].dependencies = dependencies;
    outcome.entries.extend(mesh_entries);
    outcome
}

fn reserve_model_subasset_outputs<Dependency, Entry>(
    primitive_count: usize,
    dependencies: &mut Vec<Dependency>,
    mesh_entries: &mut Vec<Entry>,
) {
    dependencies.reserve(primitive_count);
    mesh_entries.reserve(primitive_count);
}

fn model_primitive_mesh_uri(root_uri: &AssetUri, primitive_index: usize) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#Mesh{primitive_index}/Primitive0"))
        .expect("generated model primitive mesh uri must be valid")
}

#[cfg(test)]
#[path = "tests/model_mesh_subassets.rs"]
mod tests;

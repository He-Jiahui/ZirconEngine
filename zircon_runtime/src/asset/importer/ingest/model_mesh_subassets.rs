use crate::asset::assets::{ImportedAsset, MeshAsset, ModelAsset};
use crate::asset::{AssetImportOutcome, AssetReference, AssetUri, ImportedAssetEntry};

pub(super) fn model_outcome_with_mesh_subassets(
    root_uri: AssetUri,
    mut model: ModelAsset,
) -> AssetImportOutcome {
    let mut dependencies = Vec::new();
    let mut mesh_entries = Vec::new();
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

    let outcome = dependencies.into_iter().fold(
        AssetImportOutcome::new(root_uri, ImportedAsset::Model(model)),
        AssetImportOutcome::with_dependency,
    );
    mesh_entries
        .into_iter()
        .fold(outcome, AssetImportOutcome::with_entry)
}

fn model_primitive_mesh_uri(root_uri: &AssetUri, primitive_index: usize) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#Mesh{primitive_index}/Primitive0"))
        .expect("generated model primitive mesh uri must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::ModelPrimitiveAsset;

    #[test]
    fn external_mesh_references_are_preserved_as_dependencies() {
        let root_uri = AssetUri::parse("res://models/external.model.toml").unwrap();
        let external_mesh_uri = AssetUri::parse("res://meshes/authoritative.zmesh").unwrap();
        let model = ModelAsset {
            uri: root_uri.clone(),
            primitives: vec![ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
                mesh: Some(AssetReference::from_locator(external_mesh_uri.clone())),
                mesh_sdf: None,
                virtual_geometry: None,
            }],
        };

        let outcome = model_outcome_with_mesh_subassets(root_uri, model);

        assert_eq!(outcome.entries.len(), 1);
        assert_eq!(
            outcome.root_entry().unwrap().dependencies,
            vec![external_mesh_uri.clone()]
        );
        let ImportedAsset::Model(model) = &outcome.root_entry().unwrap().asset else {
            panic!("root import remains a model");
        };
        assert_eq!(
            model.primitives[0].mesh.as_ref().map(|mesh| &mesh.locator),
            Some(&external_mesh_uri)
        );
    }

    #[test]
    fn inline_primitive_is_assetized_without_touching_external_siblings() {
        let root_uri = AssetUri::parse("res://models/mixed.model.toml").unwrap();
        let external_mesh_uri = AssetUri::parse("res://meshes/external.zmesh").unwrap();
        let primitive = |mesh| ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh,
            mesh_sdf: None,
            virtual_geometry: None,
        };
        let model = ModelAsset {
            uri: root_uri.clone(),
            primitives: vec![
                primitive(Some(AssetReference::from_locator(
                    external_mesh_uri.clone(),
                ))),
                primitive(None),
            ],
        };

        let outcome = model_outcome_with_mesh_subassets(root_uri.clone(), model);
        let generated = model_primitive_mesh_uri(&root_uri, 1);

        assert_eq!(outcome.entries.len(), 2);
        assert_eq!(
            outcome.root_entry().unwrap().dependencies,
            vec![external_mesh_uri, generated.clone()]
        );
        assert!(outcome
            .entries
            .iter()
            .any(|entry| entry.locator == generated));
    }
}

use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};

use super::*;
use crate::asset::{AssetUri, AssetUuid, ModelPrimitiveAsset};

#[test]
fn project_subasset_reference_round_trips_through_formal_model_document() {
    let guid: AssetUuid = "f1111111-2222-4333-8444-555555555555".parse().unwrap();
    let locator = AssetUri::parse("res://models/hero.glb#Mesh0").unwrap();
    let model = ModelAsset {
        uri: AssetUri::parse("res://models/hero.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: Some(AssetReference::new(guid, locator.clone())),
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    };

    let persisted = serialize_model(&model, |reference| {
        assert_eq!(reference.locator.label(), Some("Mesh0"));
        Ok(PersistedAssetReference::project(
            AssetRef::try_new(
                reference.uuid,
                RelPath::parse("models/hero.glb").unwrap(),
                Some("Mesh0".to_owned()),
            )
            .unwrap(),
        ))
    })
    .unwrap();
    let reloaded = deserialize_model(&persisted, |reference| {
        let reference = reference.project_ref().expect("project reference");
        assert_eq!(reference.path_hint().as_str(), "models/hero.glb");
        assert_eq!(reference.sub(), Some("Mesh0"));
        Ok(AssetReference::new(guid, locator.clone()))
    })
    .unwrap();

    assert_eq!(reloaded, model);
}

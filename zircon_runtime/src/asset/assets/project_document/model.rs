use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::PersistedAssetReference;

use crate::asset::{AssetReference, ModelAsset, ReferenceResolutionError};

use super::codec::{decode_document, encode_document};
use crate::asset::assets::ProjectDocumentError;

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct ModelAuthoringDocument<R> {
    primitives: Vec<ModelPrimitiveDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct ModelPrimitiveDocument<R> {
    #[serde(default)]
    mesh: Option<R>,
    #[serde(flatten)]
    _rest: toml::Table,
}

pub(in crate::asset::assets) fn validate_model(document: &str) -> Result<(), toml::de::Error> {
    let document = toml::from_str::<ModelAuthoringDocument<PersistedAssetReference>>(document)?;
    for primitive in &document.primitives {
        let _ = primitive.mesh.as_ref();
    }
    Ok(())
}

pub(in crate::asset::assets) fn deserialize_model(
    document: &str,
    mut resolver: impl FnMut(
        &PersistedAssetReference,
    ) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<ModelAsset, ProjectDocumentError> {
    let document = toml::from_str::<ModelAuthoringDocument<PersistedAssetReference>>(document)?;
    let document = map_references(document, |reference| resolver(&reference))?;
    decode_document(document)
}

pub(in crate::asset::assets) fn serialize_model(
    value: &ModelAsset,
    mut resolver: impl FnMut(
        &AssetReference,
    ) -> Result<PersistedAssetReference, ReferenceResolutionError>,
) -> Result<String, ProjectDocumentError> {
    let document = encode_document::<_, ModelAuthoringDocument<AssetReference>>(value)?;
    let document = map_references(document, |reference| resolver(&reference))?;
    Ok(toml::to_string_pretty(&document)?)
}

fn map_references<A, B>(
    document: ModelAuthoringDocument<A>,
    mut map: impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<ModelAuthoringDocument<B>, ReferenceResolutionError> {
    Ok(ModelAuthoringDocument {
        primitives: document
            .primitives
            .into_iter()
            .map(|primitive| {
                Ok(ModelPrimitiveDocument {
                    mesh: primitive.mesh.map(&mut map).transpose()?,
                    _rest: primitive._rest,
                })
            })
            .collect::<Result<_, ReferenceResolutionError>>()?,
        _rest: document._rest,
    })
}

#[cfg(test)]
mod tests {
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
}

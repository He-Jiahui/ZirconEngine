use std::collections::BTreeMap;

use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::PersistedAssetReference;

use crate::asset::{AssetReference, ReferenceResolutionError, ZMaterialDocument};

use super::codec::{decode_document, encode_document};
use crate::asset::assets::ProjectDocumentError;

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct MaterialAuthoringDocument<R> {
    version: u32,
    shader: R,
    #[serde(default)]
    parent: Option<R>,
    #[serde(default)]
    textures: BTreeMap<String, MaterialTextureDocument<R>>,
    #[serde(flatten)]
    _rest: toml::Table,
}

#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "R: Serialize", deserialize = "R: Deserialize<'de>"))]
struct MaterialTextureDocument<R> {
    #[serde(default, flatten)]
    reference: Option<R>,
    #[serde(flatten)]
    _rest: toml::Table,
}

pub(in crate::asset::assets) fn validate_material(document: &str) -> Result<(), toml::de::Error> {
    let document = toml::from_str::<MaterialAuthoringDocument<PersistedAssetReference>>(document)?;
    if document.version != 2 {
        return Err(toml::de::Error::custom(format!(
            "zmaterial v2 document version `{}` is unsupported",
            document.version
        )));
    }
    let _ = (&document.shader, document.parent.as_ref());
    Ok(())
}

pub(in crate::asset::assets) fn deserialize_material(
    document: &str,
    mut resolver: impl FnMut(
        &PersistedAssetReference,
    ) -> Result<AssetReference, ReferenceResolutionError>,
) -> Result<ZMaterialDocument, ProjectDocumentError> {
    let document = toml::from_str::<MaterialAuthoringDocument<PersistedAssetReference>>(document)?;
    let document = map_references(document, |reference| resolver(&reference))?;
    decode_document(document)
}

pub(in crate::asset::assets) fn serialize_material(
    value: &ZMaterialDocument,
    mut resolver: impl FnMut(
        &AssetReference,
    ) -> Result<PersistedAssetReference, ReferenceResolutionError>,
) -> Result<String, ProjectDocumentError> {
    let document = encode_document::<_, MaterialAuthoringDocument<AssetReference>>(value)?;
    let document = map_references(document, |reference| resolver(&reference))?;
    Ok(toml::to_string_pretty(&document)?)
}

fn map_references<A, B>(
    document: MaterialAuthoringDocument<A>,
    mut map: impl FnMut(A) -> Result<B, ReferenceResolutionError>,
) -> Result<MaterialAuthoringDocument<B>, ReferenceResolutionError> {
    Ok(MaterialAuthoringDocument {
        version: document.version,
        shader: map(document.shader)?,
        parent: document.parent.map(&mut map).transpose()?,
        textures: document
            .textures
            .into_iter()
            .map(|(name, texture)| {
                Ok((
                    name,
                    MaterialTextureDocument {
                        reference: texture.reference.map(&mut map).transpose()?,
                        _rest: texture._rest,
                    },
                ))
            })
            .collect::<Result<_, ReferenceResolutionError>>()?,
        _rest: document._rest,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
    use zircon_runtime_interface::resource::ResourceScheme;

    use super::*;
    use crate::asset::assets::{MaterialTextureSlotValue, ZMaterialQueueOverride};
    use crate::asset::{AssetUri, AssetUuid};

    #[test]
    fn builtin_and_project_references_round_trip_across_every_material_reference_field() {
        let parent_guid: AssetUuid = "f2111111-2222-4333-8444-555555555555".parse().unwrap();
        let texture_guid: AssetUuid = "f3111111-2222-4333-8444-555555555555".parse().unwrap();
        let shader = AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr").unwrap());
        let parent = AssetReference::new(
            parent_guid,
            AssetUri::parse("res://materials/base.zmaterial").unwrap(),
        );
        let texture = AssetReference::new(
            texture_guid,
            AssetUri::parse("res://textures/albedo.png").unwrap(),
        );
        let material = ZMaterialDocument {
            version: 2,
            name: Some("Roundtrip".to_owned()),
            shader: shader.clone(),
            parent: Some(parent.clone()),
            options: BTreeMap::new(),
            overrides: BTreeMap::new(),
            textures: BTreeMap::from([(
                "albedo".to_owned(),
                MaterialTextureSlotValue::new(texture.clone()),
            )]),
            queue: Some(ZMaterialQueueOverride { offset: 3 }),
            editor: toml::Table::new(),
            validation_diagnostics: Vec::new(),
        };

        let persisted = serialize_material(&material, |reference| {
            if reference.locator.scheme() == ResourceScheme::Builtin {
                return Ok(PersistedAssetReference::builtin(reference.locator.clone()));
            }
            let path = if reference.uuid == parent_guid {
                "materials/base.zmaterial"
            } else {
                "textures/albedo.png"
            };
            Ok(PersistedAssetReference::project(
                AssetRef::try_new(
                    reference.uuid,
                    RelPath::parse(path).unwrap(),
                    reference.locator.label().map(str::to_owned),
                )
                .unwrap(),
            ))
        })
        .unwrap();
        let reloaded = deserialize_material(&persisted, |reference| {
            if let Some(locator) = reference.builtin_locator() {
                return Ok(AssetReference::from_locator(locator.clone()));
            }
            let reference = reference.project_ref().expect("project reference");
            let mut locator = format!("res://{}", reference.path_hint());
            if let Some(sub) = reference.sub() {
                locator.push('#');
                locator.push_str(sub);
            }
            Ok(AssetReference::new(
                reference.guid(),
                AssetUri::parse(&locator).unwrap(),
            ))
        })
        .unwrap();

        assert_eq!(reloaded, material);
    }

    #[test]
    fn resolver_failure_remains_a_typed_project_document_source() {
        let material = ZMaterialDocument {
            version: 2,
            name: None,
            shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr").unwrap()),
            parent: None,
            options: BTreeMap::new(),
            overrides: BTreeMap::new(),
            textures: BTreeMap::new(),
            queue: None,
            editor: toml::Table::new(),
            validation_diagnostics: Vec::new(),
        };

        let error = serialize_material(&material, |_| {
            Err(ReferenceResolutionError::MissingGuid {
                guid: "f4111111-2222-4333-8444-555555555555".parse().unwrap(),
            })
        })
        .unwrap_err();

        assert!(matches!(
            error,
            ProjectDocumentError::Reference(ReferenceResolutionError::MissingGuid { .. })
        ));
    }
}

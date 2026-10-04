use std::cell::Cell;
use std::collections::BTreeMap;

use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use super::*;
use crate::asset::assets::{MaterialTextureSlotValue, ZMaterialQueueOverride};
use crate::asset::{AssetUri, AssetUuid};

#[test]
fn public_serializer_rejects_unsupported_material_version_before_resolution() {
    let material = ZMaterialDocument {
        version: 1,
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
    let resolver_called = Cell::new(false);

    let error = material
        .to_project_toml_string(|reference| {
            resolver_called.set(true);
            Ok(PersistedAssetReference::builtin(reference.locator.clone()))
        })
        .unwrap_err();

    assert!(!resolver_called.get());
    assert!(matches!(error, ProjectDocumentError::Deserialize { .. }));
    assert!(error.to_string().contains(
        "zmaterial v2 document version `1` is unsupported; migrate material files to version = 2"
    ));
}

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
    let mut texture_slot = MaterialTextureSlotValue::new(texture.clone());
    texture_slot.fallback = Some("white".to_owned());
    texture_slot.uv_channel = 2;
    let material = ZMaterialDocument {
        version: 2,
        name: Some("Roundtrip".to_owned()),
        shader: shader.clone(),
        parent: Some(parent.clone()),
        options: BTreeMap::new(),
        overrides: BTreeMap::new(),
        textures: BTreeMap::from([("albedo".to_owned(), texture_slot)]),
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

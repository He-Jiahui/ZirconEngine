use std::fs;
use std::sync::Arc;

use zircon_runtime_interface::project::RelPath;

use super::*;
use crate::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUri, AssetUuid, ReferenceResolutionError};

#[test]
fn importer_rejects_path_candidate_when_guid_is_unregistered() {
    let root = std::env::temp_dir().join(format!(
        "zircon_import_reference_repair_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("models")).unwrap();
    fs::write(root.join("models/hero.glb"), b"model").unwrap();
    let resolved: AssetUuid = "c1111111-2222-4333-8444-555555555555".parse().unwrap();
    let stale: AssetUuid = "c2111111-2222-4333-8444-555555555555".parse().unwrap();
    let registry = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            AssetUuid::new(),
            AssetUri::parse("res://models/hero.glb").unwrap(),
            AssetKind::Model,
            "digest",
        ),
        AssetRegistryEntry::new(
            resolved,
            AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
            AssetKind::Mesh,
            "mesh-digest",
        ),
    ])
    .unwrap();
    let source = format!(
        "uri = \"res://models/hero.model.toml\"\n\n[[primitives]]\nvertices = []\nindices = []\n\n[primitives.mesh]\nkind = \"project\"\nguid = \"{stale}\"\npath_hint = \"assets/models/hero.glb\"\nsub = \"Mesh0\"\n"
    );
    let context = AssetImportContext::new(
        root.join("models/hero.model.toml"),
        AssetUri::parse("res://models/hero.model.toml").unwrap(),
        source.into_bytes(),
        toml::Table::new(),
    )
    .with_project_resolver(
        Arc::new(registry),
        Arc::new(vec![(RelPath::parse("assets").unwrap(), root.clone())]),
    );

    let error = import_model(&context).unwrap_err();
    assert!(matches!(
        error,
        AssetImportError::ProjectDocument(
            crate::asset::assets::ProjectDocumentError::Reference(
                ReferenceResolutionError::PathOccupiedCandidate {
                    guid,
                    path,
                    candidate_uuid,
                    candidate_path,
                }
            )
        ) if guid == stale
            && path == "assets/models/hero.glb"
            && candidate_uuid == resolved
            && candidate_path == AssetUri::parse("res://models/hero.glb#Mesh0").unwrap()
    ));
    fs::remove_dir_all(root).unwrap();
}

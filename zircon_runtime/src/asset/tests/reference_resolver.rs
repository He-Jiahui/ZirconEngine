use std::fs;

use zircon_runtime_interface::project::AssetRef;

use super::*;
use crate::asset::project::{AssetMetaDocument, AssetSourceUnit};
use crate::asset::registry::AssetRegistryEntry;
use crate::asset::{AssetKind, AssetUuid};

#[test]
fn resolution_keeps_guid_authoritative_and_reports_path_candidates() {
    let root = std::env::temp_dir().join(format!(
        "zircon_reference_resolution_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("models")).unwrap();
    fs::write(root.join("models/a.glb"), "a").unwrap();
    fs::write(root.join("models/b.glb"), "b").unwrap();
    let a: AssetUuid = "e1111111-2222-4333-8444-555555555555".parse().unwrap();
    let b: AssetUuid = "e2111111-2222-4333-8444-555555555555".parse().unwrap();
    let missing: AssetUuid = "e3111111-2222-4333-8444-555555555555".parse().unwrap();
    let a_mesh: AssetUuid = "e4111111-2222-4333-8444-555555555555".parse().unwrap();
    let a_material: AssetUuid = "e5111111-2222-4333-8444-555555555555".parse().unwrap();
    let b_missing_mesh: AssetUuid = "e6111111-2222-4333-8444-555555555555".parse().unwrap();
    let registry = AssetRegistryIndex::from_entries([
        AssetRegistryEntry::new(
            a,
            AssetUri::parse("res://models/a.glb").unwrap(),
            AssetKind::Model,
            "a",
        ),
        AssetRegistryEntry::new(
            b,
            AssetUri::parse("res://models/b.glb").unwrap(),
            AssetKind::Model,
            "b",
        ),
        AssetRegistryEntry::new(
            a_mesh,
            AssetUri::parse("res://models/a.glb#Mesh0").unwrap(),
            AssetKind::Mesh,
            "a-mesh",
        ),
        AssetRegistryEntry::new(
            a_material,
            AssetUri::parse("res://models/a.glb#Material0").unwrap(),
            AssetKind::Material,
            "a-material",
        ),
        AssetRegistryEntry::new(
            b_missing_mesh,
            AssetUri::parse("res://models/b.glb#MissingMesh").unwrap(),
            AssetKind::Mesh,
            "b-missing-mesh",
        ),
    ])
    .unwrap();
    let roots = vec![(RelPath::parse("assets").unwrap(), root.clone())];

    let exact = AssetRef::try_new(a, RelPath::parse("assets/models/a.glb").unwrap(), None).unwrap();
    assert_eq!(
        resolve_project_reference(&registry, &roots, &exact)
            .unwrap()
            .repair,
        None
    );

    let stale_guid = AssetRef::try_new(
        missing,
        RelPath::parse("assets/models/a.glb").unwrap(),
        None,
    )
    .unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &stale_guid),
        Err(ReferenceResolutionError::PathOccupiedCandidate {
            guid,
            path,
            candidate_uuid,
            candidate_path,
        }) if guid == missing
            && path == "assets/models/a.glb"
            && candidate_uuid == a
            && candidate_path == AssetUri::parse("res://models/a.glb").unwrap()
    ));

    let stale_subasset = AssetRef::try_new(
        missing,
        RelPath::parse("assets/models/a.glb").unwrap(),
        Some("Mesh0".to_owned()),
    )
    .unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &stale_subasset),
        Err(ReferenceResolutionError::PathOccupiedCandidate {
            guid,
            path,
            candidate_uuid,
            candidate_path,
        }) if guid == missing
            && path == "assets/models/a.glb"
            && candidate_uuid == a_mesh
            && candidate_path == AssetUri::parse("res://models/a.glb#Mesh0").unwrap()
    ));

    let guid_subasset_conflict = AssetRef::try_new(
        a,
        RelPath::parse("assets/models/a.glb").unwrap(),
        Some("Mesh0".to_owned()),
    )
    .unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &guid_subasset_conflict),
        Err(ReferenceResolutionError::Conflict { guid, path })
            if guid == a && path == "assets/models/a.glb"
    ));

    for requested_guid in [a, missing] {
        let missing_subasset = AssetRef::try_new(
            requested_guid,
            RelPath::parse("assets/models/a.glb").unwrap(),
            Some("MissingMesh".to_owned()),
        )
        .unwrap();
        assert!(matches!(
            resolve_project_reference(&registry, &roots, &missing_subasset),
            Err(ReferenceResolutionError::DanglingSubasset {
                guid,
                path,
                label,
                candidates,
            }) if guid == requested_guid
                && path == "assets/models/a.glb"
                && label == "MissingMesh"
                && candidates == vec![
                    AssetUri::parse("res://models/a.glb#Material0").unwrap(),
                    AssetUri::parse("res://models/a.glb#Mesh0").unwrap(),
                ]
        ));
    }

    // A stale hint cannot replace the known parent's source for label diagnostics.
    for hint in ["assets/models/moved.glb", "assets/models/b.glb"] {
        // B has MissingMesh, while MissingMaterial is absent from both sources.
        for requested_label in ["MissingMesh", "MissingMaterial"] {
            let missing_subasset = AssetRef::try_new(
                a,
                RelPath::parse(hint).unwrap(),
                Some(requested_label.to_owned()),
            )
            .unwrap();
            assert!(matches!(
                resolve_project_reference(&registry, &roots, &missing_subasset),
                Err(ReferenceResolutionError::DanglingSubasset {
                    guid,
                    path,
                    label,
                    candidates,
                }) if guid == a
                    && path == hint
                    && label == requested_label
                    && candidates == vec![
                        AssetUri::parse("res://models/a.glb#Material0").unwrap(),
                        AssetUri::parse("res://models/a.glb#Mesh0").unwrap(),
                    ]
            ));
        }

        for requested_label in ["Mesh0", "Material0"] {
            let parent_subasset_conflict = AssetRef::try_new(
                a,
                RelPath::parse(hint).unwrap(),
                Some(requested_label.to_owned()),
            )
            .unwrap();
            assert!(matches!(
                resolve_project_reference(&registry, &roots, &parent_subasset_conflict),
                Err(ReferenceResolutionError::Conflict { guid, path })
                    if guid == a && path == hint
            ));
        }
    }

    for hint in [
        "assets/models/a.glb",
        "assets/models/moved.glb",
        "assets/models/b.glb",
    ] {
        for requested_label in [None, Some("Material0"), Some("MissingMesh")] {
            let labeled_guid_conflict = AssetRef::try_new(
                a_mesh,
                RelPath::parse(hint).unwrap(),
                requested_label.map(str::to_owned),
            )
            .unwrap();
            assert!(matches!(
                resolve_project_reference(&registry, &roots, &labeled_guid_conflict),
                Err(ReferenceResolutionError::Conflict { guid, path })
                    if guid == a_mesh && path == hint
            ));
        }
    }

    let missing_guid_occupied_label = AssetRef::try_new(
        missing,
        RelPath::parse("assets/models/b.glb").unwrap(),
        Some("MissingMesh".to_owned()),
    )
    .unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &missing_guid_occupied_label),
        Err(ReferenceResolutionError::PathOccupiedCandidate {
            guid,
            path,
            candidate_uuid,
            candidate_path,
        }) if guid == missing
            && path == "assets/models/b.glb"
            && candidate_uuid == b_missing_mesh
            && candidate_path == AssetUri::parse("res://models/b.glb#MissingMesh").unwrap()
    ));

    let missing_guid_stale_label = AssetRef::try_new(
        missing,
        RelPath::parse("assets/models/moved.glb").unwrap(),
        Some("MissingMesh".to_owned()),
    )
    .unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &missing_guid_stale_label),
        Err(ReferenceResolutionError::Dangling { guid, path })
            if guid == missing && path == "assets/models/moved.glb"
    ));

    let stale_path =
        AssetRef::try_new(a, RelPath::parse("assets/models/moved.glb").unwrap(), None).unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &stale_path)
            .unwrap()
            .repair,
        Some(ReferenceRepair {
            kind: ReferenceRepairKind::PathHint,
            ..
        })
    ));

    let dangling = AssetRef::try_new(
        missing,
        RelPath::parse("assets/models/missing.glb").unwrap(),
        None,
    )
    .unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &dangling),
        Err(ReferenceResolutionError::Dangling { .. })
    ));

    let occupied_hint =
        AssetRef::try_new(a, RelPath::parse("assets/models/b.glb").unwrap(), None).unwrap();
    assert!(matches!(
        resolve_project_reference(&registry, &roots, &occupied_hint)
            .unwrap()
            .repair,
        Some(ReferenceRepair { kind: ReferenceRepairKind::PathHint, resolved, .. })
            if resolved.path_hint().as_str() == "assets/models/a.glb"
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn resolution_accepts_compound_zmeta_hint_and_keeps_registered_uuid() {
    let root = std::env::temp_dir().join(format!(
        "zircon_reference_resolution_compound_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let compound_root = root.join("shaders/redirect_surface");
    fs::create_dir_all(&compound_root).unwrap();
    let uuid: AssetUuid = "f1111111-2222-4333-8444-555555555555".parse().unwrap();
    let locator = AssetUri::parse("res://shaders/redirect_surface").unwrap();
    let mut meta = AssetMetaDocument::new(uuid, locator.clone(), AssetKind::Shader);
    meta.unit = AssetSourceUnit::Compound;
    meta.save(root.join("shaders/redirect_surface.zmeta"))
        .unwrap();
    let registry = AssetRegistryIndex::from_entries([AssetRegistryEntry::new(
        uuid,
        locator.clone(),
        AssetKind::Shader,
        "redirect surface",
    )])
    .unwrap();
    let roots = vec![(RelPath::parse("assets").unwrap(), root.clone())];
    let persisted = AssetRef::try_new(
        uuid,
        RelPath::parse("assets/shaders/redirect_surface.zmeta").unwrap(),
        None,
    )
    .unwrap();

    let resolved = resolve_project_reference(&registry, &roots, &persisted).unwrap();

    assert_eq!(resolved.reference.uuid, uuid);
    assert_eq!(resolved.reference.locator, locator);
    assert_eq!(resolved.repair, None);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn persisted_source_mapping_rejects_directory_without_matching_compound_meta() {
    let root = std::env::temp_dir().join(format!(
        "zircon_reference_resolution_unregistered_directory_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders/unregistered")).unwrap();
    let locator = AssetUri::parse("res://shaders/unregistered").unwrap();

    let source = persisted_source_path_for_locator(&root, &locator).unwrap();

    assert_eq!(source, None);
    fs::remove_dir_all(root).unwrap();
}

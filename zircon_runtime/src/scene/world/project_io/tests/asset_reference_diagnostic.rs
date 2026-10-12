use super::*;
use crate::asset::project::ProjectReferenceDiagnosticKind;
use crate::asset::{AssetUri, AssetUuid};
use crate::core::resource::ResourceId;

#[test]
fn typed_scene_errors_project_to_the_runtime_reference_diagnostic_contract() {
    let document = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let locator = AssetUri::parse("res://models/missing.glb").unwrap();
    let uuid = AssetUuid::new();
    let dangling = scene_reference_diagnostic(
        &document,
        ProjectReferenceDiagnosticPhase::Load,
        &SceneProjectError::DanglingAssetReference {
            uuid,
            locator: locator.clone(),
        },
    )
    .unwrap();
    assert!(matches!(
        dangling.kind(),
        ProjectReferenceDiagnosticKind::DanglingAssetReference {
            uuid: observed,
            locator: observed_locator,
        } if *observed == uuid && observed_locator == &locator
    ));

    let persisted = scene_reference_diagnostic(
        &document,
        ProjectReferenceDiagnosticPhase::Load,
        &SceneProjectError::Asset(AssetImportError::ProjectDocument(
            ProjectDocumentError::Reference(ReferenceResolutionError::DanglingSubasset {
                guid: uuid,
                path: "assets/models/hero.glb".to_owned(),
                label: "MissingMesh".to_owned(),
                candidates: Vec::new(),
            }),
        )),
    )
    .unwrap();
    assert!(matches!(
        persisted.kind(),
        ProjectReferenceDiagnosticKind::PersistedDanglingReference {
            uuid: observed,
            path_hint,
            subasset: Some(subasset),
        } if *observed == uuid
            && path_hint.as_ref() == "assets/models/hero.glb"
            && subasset.as_ref() == "MissingMesh"
    ));

    let resource_id = ResourceId::new();
    let unresolved = scene_reference_diagnostic(
        &document,
        ProjectReferenceDiagnosticPhase::Save,
        &SceneProjectError::UnresolvedResourceHandle {
            resource_id,
            role: "material",
        },
    )
    .unwrap();
    assert!(matches!(
        unresolved.kind(),
        ProjectReferenceDiagnosticKind::UnresolvedResourceHandle {
            resource_id: observed,
            role,
        } if *observed == resource_id && role.as_ref() == "material"
    ));
}

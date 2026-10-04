use super::*;

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

#[test]
fn document_publication_replaces_stale_diagnostics_and_keeps_other_documents() {
    let store = ProjectReferenceDiagnosticsStore::default();
    let scene_a = uri("res://scenes/a.scene.toml");
    let scene_b = uri("res://scenes/b.scene.toml");
    let missing_a = AssetUuid::new();
    let missing_b = AssetUuid::new();
    store.replace_document(
        scene_a.clone(),
        ProjectReferenceDiagnosticPhase::Load,
        vec![ProjectReferenceDiagnostic::dangling(
            scene_a.clone(),
            ProjectReferenceDiagnosticPhase::Load,
            missing_a,
            uri("res://models/missing-a.glb"),
        )],
    );
    store.replace_document(
        scene_b.clone(),
        ProjectReferenceDiagnosticPhase::Load,
        vec![ProjectReferenceDiagnostic::dangling(
            scene_b.clone(),
            ProjectReferenceDiagnosticPhase::Load,
            missing_b,
            uri("res://models/missing-b.glb"),
        )],
    );

    let cleared = store.replace_document(
        scene_a.clone(),
        ProjectReferenceDiagnosticPhase::Save,
        Vec::new(),
    );
    let snapshot = store.snapshot();

    assert!(cleared.diagnostics().is_empty());
    assert_eq!(cleared.sequence(), 3);
    assert_eq!(snapshot.sequence(), 3);
    assert_eq!(snapshot.diagnostics_for_document(&scene_a).count(), 0);
    assert_eq!(snapshot.diagnostics_for_document(&scene_b).count(), 1);
}

#[test]
fn latest_event_is_bounded_to_one_document_replacement() {
    let store = ProjectReferenceDiagnosticsStore::default();
    let scene = uri("res://scenes/main.scene.toml");
    let resource_id = ResourceId::new();

    store.replace_document(
        scene.clone(),
        ProjectReferenceDiagnosticPhase::Save,
        vec![ProjectReferenceDiagnostic::unresolved_handle(
            scene.clone(),
            ProjectReferenceDiagnosticPhase::Save,
            resource_id,
            "material",
        )],
    );

    let event = store.latest_event().unwrap();
    assert_eq!(event.document(), &scene);
    assert_eq!(event.phase(), ProjectReferenceDiagnosticPhase::Save);
    assert_eq!(event.diagnostics().len(), 1);
    assert!(matches!(
        event.diagnostics()[0].kind(),
        ProjectReferenceDiagnosticKind::UnresolvedResourceHandle {
            resource_id: observed,
            role,
        } if *observed == resource_id && role.as_ref() == "material"
    ));
}

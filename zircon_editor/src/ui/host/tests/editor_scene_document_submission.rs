#[test]
fn project_scene_installer_prepares_an_authoring_seed_before_replacing_authoring_world() {
    let source = include_str!("../editor_scene_document_submission.rs");
    let prepare_seed = [
        "self",
        "            .manager",
        "            .prepare_authoring_world(document.world().clone())",
    ]
    .join("\n");
    let raw_level_install = [
        "self.state.replace_world(",
        "document.world().clone(), &self.project_path)",
    ]
    .concat();

    assert!(source.contains(&prepare_seed));
    let install_authoring_world = [
        "self.state",
        "            .replace_world(authoring_world, &self.project_path)",
    ]
    .join("\n");
    assert!(source.contains(".prepare_scene_transition()"));
    assert!(source.contains(&install_authoring_world));
    assert!(!source.contains(".create_runtime_level(scene.clone())"));
    assert!(!source.contains(&raw_level_install));
}

#[test]
fn authoring_seed_preparation_does_not_reexpose_the_runtime_level_through_manager_project() {
    let source = include_str!("../editor_manager_project.rs");

    assert!(source.contains(".prepare_authoring_world(scene)"));
    assert!(!source.contains(".create_runtime_level(scene)"));
}

#[test]
fn scene_submission_binds_lifecycle_document_for_new_and_already_active_routes() {
    let source = include_str!("../editor_scene_document_submission.rs");

    assert!(source.contains(
        "SceneDocumentRouteResult::Activated(activation) => activation.activation.document"
    ));
    assert!(source.contains("SceneDocumentRouteResult::AlreadyActive { document } => *document"));
    assert!(source.contains(".state.bind_scene_document(document)"));
}

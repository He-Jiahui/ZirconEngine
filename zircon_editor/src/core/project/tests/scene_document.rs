#![cfg(windows)]

use std::fs;

use zircon_runtime::asset::AssetUri;

use super::super::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, SceneCreateRequest, SceneOpenRequest,
};
use super::temp_root;

#[test]
fn project_authority_opens_a_project_owned_scene_by_canonical_uri() {
    let location = temp_root("scene-open");
    let draft = NewProjectDraft {
        project_name: "Scene Open".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };
    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let root = created.root.clone();
    let mut project = created.into_project();
    project.scan_and_import().unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();

    let document = ProjectAuthority::default()
        .open_scene(&project, SceneOpenRequest::new(scene_uri.clone()))
        .unwrap();

    assert_eq!(document.scene_uri(), &scene_uri);
    assert_eq!(
        document.source_path(),
        root.join("assets").join("scenes").join("main.scene.toml")
    );
    assert!(
        document
            .world()
            .nodes()
            .iter()
            .any(|node| node.name == "Cube")
    );

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn project_authority_creates_a_new_scene_without_overwriting_an_existing_target() {
    let location = temp_root("scene-create");
    let draft = NewProjectDraft {
        project_name: "Scene Create".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };
    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let mut project = created.into_project();
    let scene_uri = AssetUri::parse("res://scenes/secondary.scene.toml").unwrap();
    let request = SceneCreateRequest::new(scene_uri.clone());

    let document = ProjectAuthority::default()
        .create_scene(&mut project, request.clone())
        .unwrap();
    assert_eq!(document.scene_uri(), &scene_uri);
    assert!(document.source_path().is_file());
    let catalog_uris = project
        .registry()
        .values()
        .map(|record| record.primary_locator().to_string())
        .collect::<Vec<_>>();
    assert!(catalog_uris.iter().any(|uri| uri == &scene_uri.to_string()));
    assert!(
        catalog_uris
            .iter()
            .all(|uri| !uri.contains(".zircon-scene-staging-")),
        "project catalog retained a transient scene staging source: {catalog_uris:?}"
    );

    let reopened = ProjectAuthority::default()
        .open_scene(&project, SceneOpenRequest::new(scene_uri.clone()))
        .unwrap();
    assert_eq!(reopened.scene_uri(), &scene_uri);
    assert_eq!(reopened.source_path(), document.source_path());

    fs::write(document.source_path(), "caller-owned scene content").unwrap();
    let error = ProjectAuthority::default()
        .create_scene(&mut project, request)
        .unwrap_err();
    assert!(error.to_string().contains("already exists"));
    assert_eq!(
        fs::read_to_string(document.source_path()).unwrap(),
        "caller-owned scene content"
    );

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn scene_target_uris_cannot_escape_the_project_root_before_reaching_project_authority() {
    assert!(AssetUri::parse("res://../../outside.scene.toml").is_err());
    assert!(AssetUri::parse("res://scenes/../../../outside.scene.toml").is_err());
}

#[test]
fn scene_creation_rejects_a_missing_parent_without_creating_it() {
    let location = temp_root("scene-missing-parent");
    let draft = NewProjectDraft {
        project_name: "Scene Missing Parent".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };
    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let root = created.root.clone();
    let mut project = created.into_project();

    let error = ProjectAuthority::default()
        .create_scene(
            &mut project,
            SceneCreateRequest::new(AssetUri::parse("res://uncreated/new.scene.toml").unwrap()),
        )
        .unwrap_err();

    assert!(error.to_string().contains("parent directory"));
    assert!(!root.join("assets/uncreated").exists());

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn project_authority_rejects_a_non_scene_asset_before_loading_it() {
    let location = temp_root("scene-kind");
    let draft = NewProjectDraft {
        project_name: "Scene Kind".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };
    let created = ProjectAuthority::default().create_project(&draft).unwrap();
    let project = created.into_project();

    let error = ProjectAuthority::default()
        .open_scene(
            &project,
            SceneOpenRequest::new(AssetUri::parse("res://models/cube.obj").unwrap()),
        )
        .unwrap_err();
    assert!(error.to_string().contains("scene asset"));

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

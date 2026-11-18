#![cfg(windows)]

use std::fs;
use std::time::{Duration, Instant};

use zircon_runtime::asset::AssetUri;

use crate::core::jobs::test_job_system;

use super::super::{
    NewProjectDraft, ProjectAuthority, ProjectTemplateId, SceneCreateRequest, SceneOpenRequest,
};
use super::temp_root;

#[test]
fn project_authority_opens_a_project_owned_scene_by_canonical_uri() {
    let location = temp_root("scene-open");
    let draft = NewProjectDraft {
        project_name: "Scene Open".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: ProjectTemplateId::RenderableEmpty,
    };
    let created = ProjectAuthority::default()
        .create_project(
            &draft,
            &crate::tests::support::test_project_creation_provenance(),
        )
        .unwrap();
    let root = created.root.clone();
    let mut project = ProjectAuthority::default()
        .open_resolved_project(created.identity())
        .unwrap()
        .into_project();
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
    assert!(document
        .world()
        .nodes()
        .iter()
        .any(|node| node.name == "Cube"));

    drop(project);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn project_authority_scene_open_job_returns_the_typed_document() {
    let location = temp_root("scene-open-job");
    let draft = NewProjectDraft {
        project_name: "Scene Open Job".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: ProjectTemplateId::RenderableEmpty,
    };
    let created = ProjectAuthority::default()
        .create_project(
            &draft,
            &crate::tests::support::test_project_creation_provenance(),
        )
        .unwrap();
    let mut project = ProjectAuthority::default()
        .open_resolved_project(created.identity())
        .unwrap()
        .into_project();
    project.scan_and_import().unwrap();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let jobs = test_job_system();

    let ticket = ProjectAuthority::default()
        .submit_scene_open(&jobs, project, SceneOpenRequest::new(scene_uri.clone()))
        .unwrap();
    let document = ticket
        .wait_until(Instant::now() + Duration::from_secs(5))
        .expect("scene load job must finish before the test deadline")
        .unwrap();

    assert_eq!(document.scene_uri(), &scene_uri);
    assert!(document
        .world()
        .nodes()
        .iter()
        .any(|node| node.name == "Cube"));
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn project_authority_creates_a_new_scene_without_overwriting_an_existing_target() {
    let location = temp_root("scene-create");
    let draft = NewProjectDraft {
        project_name: "Scene Create".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: ProjectTemplateId::RenderableEmpty,
    };
    let created = ProjectAuthority::default()
        .create_project(
            &draft,
            &crate::tests::support::test_project_creation_provenance(),
        )
        .unwrap();
    let mut project = ProjectAuthority::default()
        .open_resolved_project(created.identity())
        .unwrap()
        .into_project();
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
        template: ProjectTemplateId::RenderableEmpty,
    };
    let created = ProjectAuthority::default()
        .create_project(
            &draft,
            &crate::tests::support::test_project_creation_provenance(),
        )
        .unwrap();
    let root = created.root.clone();
    let mut project = ProjectAuthority::default()
        .open_resolved_project(created.identity())
        .unwrap()
        .into_project();

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
        template: ProjectTemplateId::RenderableEmpty,
    };
    let created = ProjectAuthority::default()
        .create_project(
            &draft,
            &crate::tests::support::test_project_creation_provenance(),
        )
        .unwrap();
    let project = ProjectAuthority::default()
        .open_resolved_project(created.identity())
        .unwrap()
        .into_project();

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

#[test]
fn editor157_project_imports_and_opens_case_preserved_scene_sources() {
    use super::super::ProjectAuthorityError;
    use zircon_runtime::asset::ImportedAsset;
    use zircon_runtime::scene::{components::NodeKind, Scene};
    use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

    struct FixtureLocation(std::path::PathBuf);

    impl Drop for FixtureLocation {
        fn drop(&mut self) {
            let result = fs::remove_dir_all(&self.0);
            if !std::thread::panicking() {
                result.expect("remove the owned scene suffix fixture");
            }
        }
    }

    let location = FixtureLocation(temp_root("editor157-scene-suffix"));
    let authority = ProjectAuthority::default();
    let created = authority
        .create_project(
            &NewProjectDraft {
                project_name: "Scene Suffix".to_string(),
                location: location.0.to_string_lossy().into_owned(),
                template: ProjectTemplateId::RenderableEmpty,
            },
            &crate::tests::support::test_project_creation_provenance(),
        )
        .expect("create and preflight the real project fixture");
    let mut project = authority
        .open_resolved_project(created.identity())
        .expect("open the preflighted project fixture")
        .into_project();
    let scenes = project.primary_project_asset_root().unwrap().join("scenes");
    // Separate basenames avoid aliasing on a case-insensitive Windows volume.
    let cases = [
        ("Cobalt.scene.toml", "Lowercase sentinel"),
        ("MiXeD.ScEnE.ToMl", "Mixed case sentinel"),
        ("UPPER.SCENE.TOML", "Uppercase sentinel"),
    ];
    for (basename, marker) in cases {
        let mut world = Scene::empty_for_project(&project).unwrap();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world.rename_node(entity, marker).unwrap();
        let source = world
            .to_scene_asset(&project)
            .unwrap()
            .to_project_toml_string(|reference| project.persist_runtime_reference(reference))
            .unwrap();
        fs::write(scenes.join(basename), source).unwrap();
    }
    fs::copy(
        scenes.join("Cobalt.scene.toml"),
        scenes.join("Cobalt.scene.toml.backup"),
    )
    .unwrap();

    project
        .scan_and_import()
        .expect("commit a real project scan");
    let catalog_input = project.catalog_input_generation();
    // Check all three durable imports before asking the Editor to open any of them.
    for (basename, marker) in cases {
        let locator = format!("res://scenes/{basename}");
        let uri = AssetUri::parse(&locator).unwrap();
        let record = project
            .registry()
            .get_by_locator(&uri)
            .unwrap_or_else(|| panic!("scan lost exact locator {locator}"));
        assert_eq!(record.primary_locator().to_string(), locator);
        assert_eq!(record.kind, ResourceKind::Scene, "{locator}");
        assert_eq!(record.state, ResourceState::Ready, "{locator}: {record:?}");
        let row = catalog_input
            .record(record.id())
            .unwrap_or_else(|| panic!("missing committed catalog input for {locator}"));
        assert_eq!(row.resource(), record);
        assert_eq!(row.source_path(), scenes.join(basename));
        let ImportedAsset::Scene(artifact) = project
            .load_artifact(&uri)
            .unwrap_or_else(|error| panic!("load the committed artifact for {locator}: {error}"))
        else {
            panic!("the committed artifact for {locator} must be a Scene");
        };
        assert_eq!(artifact.entities.len(), 1, "{locator}");
        assert_eq!(artifact.entities[0].name, marker, "{locator}");
    }

    for locator in [
        "res://scenes/Cobalt.scene.toml.backup",
        "res://scenes/Cobalt.scene.toml#Root",
    ] {
        let request = SceneOpenRequest::new(AssetUri::parse(locator).unwrap());
        let error = authority.open_scene(&project, request).unwrap_err();
        assert!(
            matches!(&error, ProjectAuthorityError::SceneTarget { uri, .. } if uri == locator),
            "expected scene-target rejection for {locator}, got {error:?}"
        );
    }

    let mut open_errors = Vec::new();
    for (basename, marker) in cases {
        let locator = format!("res://scenes/{basename}");
        let uri = AssetUri::parse(&locator).unwrap();
        let document = match authority.open_scene(&project, SceneOpenRequest::new(uri)) {
            Ok(document) => document,
            Err(error) => {
                open_errors.push((locator, error));
                continue;
            }
        };
        assert_eq!(document.scene_uri().to_string(), locator);
        assert_eq!(document.source_path(), scenes.join(basename));
        assert_eq!(
            document.source_path().file_name().unwrap().to_str(),
            Some(basename)
        );
        assert!(document
            .source_revision()
            .matches_path(document.source_path())
            .unwrap());
        let nodes = document.world().nodes();
        assert_eq!(nodes.len(), 1, "{locator}");
        assert_eq!(nodes[0].name, marker, "{locator}");
        assert_eq!(nodes[0].kind, NodeKind::Empty, "{locator}");
    }
    assert!(
        open_errors.is_empty(),
        "ready Scene imports must open at their exact source locators: {open_errors:?}"
    );
}

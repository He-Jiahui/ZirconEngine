use std::path::Path;

use super::super::ProjectAuthorityError;
use super::{canonical_project_root, protect_scene_path, validate_creation_target};

#[test]
fn project_root_validation_rejects_empty_and_blank_paths_before_filesystem_access() {
    for path in [Path::new(""), Path::new(" "), Path::new("\u{2003}")] {
        assert!(matches!(
            canonical_project_root(path),
            Err(ProjectAuthorityError::EmptyProjectPath)
        ));
        assert!(matches!(
            validate_creation_target(path),
            Err(ProjectAuthorityError::EmptyProjectPath)
        ));
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
#[test]
fn scene_path_guard_rejects_unsupported_platforms_before_loading_or_publishing() {
    let root =
        std::env::temp_dir().join(format!("zircon-editor-scene-guard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();

    let error = protect_scene_path(&root, &root.join("assets/scenes/main.scene.toml"), false)
        .err()
        .expect("unsupported platform must reject the scene lease");
    assert!(matches!(
        error,
        ProjectAuthorityError::SceneTarget { reason, .. }
            if reason.contains("no-follow lease support")
    ));

    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(target_os = "linux")]
mod unix {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
    use zircon_runtime::asset::AssetUri;

    use crate::core::jobs::test_job_system;
    use crate::core::project::{
        NewProjectDraft, ProjectAuthority, ProjectTemplateId, SceneCreateRequest, SceneOpenRequest,
    };

    use super::protect_scene_path;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    struct FixtureRoot(PathBuf);

    impl FixtureRoot {
        fn new(label: &str) -> Self {
            let executable = std::env::current_exe().unwrap();
            let directory = ProjectPaths::resolve_existing(executable.parent().unwrap()).unwrap();
            let root = directory
                .operation_path()
                .join("zircon-mvp-fixtures")
                .join(format!(
                    "zircon-editor-unix-filesystem-{label}-{}-{}",
                    std::process::id(),
                    NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }

        fn scene_path(&self) -> PathBuf {
            let directory = self.0.join("assets/scenes");
            fs::create_dir_all(&directory).unwrap();
            directory.join("main.scene.toml")
        }
    }

    impl Drop for FixtureRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn project_fixture(label: &str) -> (FixtureRoot, ProjectManager) {
        let location = FixtureRoot::new(label);
        let created = ProjectAuthority::default()
            .create_project(
                &NewProjectDraft {
                    project_name: "Unix Scene Guard".to_string(),
                    location: location.0.to_string_lossy().into_owned(),
                    template: ProjectTemplateId::RenderableEmpty,
                },
                &crate::tests::support::test_project_creation_provenance(),
            )
            .unwrap();
        let mut project = ProjectAuthority::default()
            .open_resolved_project(created.identity())
            .unwrap()
            .into_project();
        project.scan_and_import().unwrap();
        (location, project)
    }

    fn main_scene_uri() -> AssetUri {
        AssetUri::parse("res://scenes/main.scene.toml").unwrap()
    }

    #[test]
    fn filesystem_project_authority_opens_a_real_unix_scene() {
        let (_location, project) = project_fixture("open");
        let scene_uri = main_scene_uri();

        let document = ProjectAuthority::default()
            .open_scene(&project, SceneOpenRequest::new(scene_uri.clone()))
            .unwrap();

        assert_eq!(document.scene_uri(), &scene_uri);
        assert_eq!(
            document.source_path(),
            project.paths().root().join("assets/scenes/main.scene.toml")
        );
        assert!(document
            .world()
            .nodes()
            .iter()
            .any(|node| node.name == "Cube"));
        let snapshot = protect_scene_path(project.paths().root(), document.source_path(), true)
            .unwrap()
            .read_snapshot()
            .unwrap();
        assert_eq!(document.source_revision(), snapshot.revision());
        assert_eq!(snapshot.bytes(), fs::read(document.source_path()).unwrap());
    }

    #[test]
    fn filesystem_project_authority_new_scene_is_create_only_and_leaves_no_staging_sources() {
        let (_location, mut project) = project_fixture("create");
        let uri = AssetUri::parse("res://scenes/secondary.scene.toml").unwrap();
        let request = SceneCreateRequest::new(uri.clone());
        let document = ProjectAuthority::default()
            .create_scene(&mut project, request.clone())
            .unwrap();
        let reopened = ProjectAuthority::default()
            .open_scene(&project, SceneOpenRequest::new(uri.clone()))
            .unwrap();
        assert_eq!(reopened.source_path(), document.source_path());
        assert_eq!(reopened.source_revision(), document.source_revision());

        let caller_content = b"caller-owned existing scene";
        fs::write(document.source_path(), caller_content).unwrap();
        let error = ProjectAuthority::default()
            .create_scene(&mut project, request)
            .unwrap_err();
        assert!(error.to_string().contains("already exists"), "{error}");
        assert_eq!(fs::read(document.source_path()).unwrap(), caller_content);

        let catalog_uris = project
            .registry()
            .values()
            .map(|record| record.primary_locator().to_string())
            .collect::<Vec<_>>();
        assert!(catalog_uris.iter().any(|entry| entry == &uri.to_string()));
        assert!(
            catalog_uris
                .iter()
                .all(|entry| !entry.contains(".zircon-scene-staging-")),
            "catalog retained a staging scene: {catalog_uris:?}"
        );
        assert!(fs::read_dir(document.source_path().parent().unwrap())
            .unwrap()
            .all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".zircon-scene-staging-")));
    }

    #[test]
    fn filesystem_unix_scene_guard_refuses_parent_leaf_and_dangling_symlinks() {
        let root = FixtureRoot::new("symlinks");
        let source = root.scene_path();
        fs::write(&source, b"real scene").unwrap();
        let linked_parent = root.0.join("linked-scenes");
        symlink(source.parent().unwrap(), &linked_parent).unwrap();
        let linked_leaf = source.with_file_name("linked.scene.toml");
        symlink(&source, &linked_leaf).unwrap();
        let dangling_leaf = source.with_file_name("dangling.scene.toml");
        symlink(source.with_file_name("missing.scene.toml"), &dangling_leaf).unwrap();

        for path in [
            linked_parent.join("main.scene.toml"),
            linked_parent.join("new.scene.toml"),
            linked_leaf,
            dangling_leaf,
        ] {
            for include_file in [false, true] {
                assert!(
                    protect_scene_path(&root.0, &path, include_file).is_err(),
                    "accepted symlink path {} (include_file={include_file})",
                    path.display()
                );
            }
        }

        let linked_root = root.0.join("linked-root");
        symlink(root.0.join("assets"), &linked_root).unwrap();
        assert!(protect_scene_path(
            &linked_root,
            &linked_root.join("scenes/main.scene.toml"),
            true
        )
        .is_err());
    }

    #[test]
    fn filesystem_unix_scene_guard_refuses_parent_traversal_beneath_a_project_prefix() {
        let root = FixtureRoot::new("parent-traversal");
        let project_root = root.0.join("project");
        fs::create_dir_all(project_root.join("assets/scenes")).unwrap();
        let outside = root.0.join("outside.scene.toml");
        fs::write(&outside, b"outside project").unwrap();
        let traversal = project_root.join("assets/scenes/../../../outside.scene.toml");

        for include_file in [false, true] {
            assert!(protect_scene_path(&project_root, &traversal, include_file).is_err());
        }
        assert_eq!(fs::read(outside).unwrap(), b"outside project");
    }

    #[test]
    fn filesystem_project_authority_open_and_new_refuse_symlink_scene_components() {
        let (_location, mut project) = project_fixture("authority-symlinks");
        let scenes = project.paths().root().join("assets/scenes");
        let original_scenes = scenes.with_file_name("original-scenes");
        fs::rename(&scenes, &original_scenes).unwrap();
        symlink(&original_scenes, &scenes).unwrap();

        assert!(ProjectAuthority::default()
            .open_scene(&project, SceneOpenRequest::new(main_scene_uri()))
            .is_err());
        assert!(ProjectAuthority::default()
            .create_scene(
                &mut project,
                SceneCreateRequest::new(AssetUri::parse("res://scenes/new.scene.toml").unwrap())
            )
            .is_err());
        assert!(!original_scenes.join("new.scene.toml").exists());
        assert!(fs::read_dir(&original_scenes).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".zircon-scene-staging-")
        }));
    }

    #[test]
    fn filesystem_unix_scene_guard_detects_replaced_file_even_with_identical_bytes() {
        let root = FixtureRoot::new("file-replacement");
        let source = root.scene_path();
        let bytes = b"the same bytes do not establish file identity";
        fs::write(&source, bytes).unwrap();
        let guard = protect_scene_path(&root.0, &source, true).unwrap();
        let before = guard.read_snapshot().unwrap();
        fs::rename(&source, source.with_file_name("original.scene.toml")).unwrap();
        fs::write(&source, bytes).unwrap();

        assert!(guard.verify().is_err());
        assert!(guard.read_snapshot().is_err());
        let after = protect_scene_path(&root.0, &source, true)
            .unwrap()
            .read_snapshot()
            .unwrap();
        assert_eq!(before.bytes(), after.bytes());
        assert_ne!(before.revision(), after.revision());
    }

    #[test]
    fn filesystem_unix_scene_guard_detects_replaced_parent_before_reading_or_staging() {
        let root = FixtureRoot::new("directory-replacement");
        let source = root.scene_path();
        fs::write(&source, b"original source").unwrap();
        let read_guard = protect_scene_path(&root.0, &source, true).unwrap();
        let new_source = source.with_file_name("new.scene.toml");
        let mut create_guard = protect_scene_path(&root.0, &new_source, false).unwrap();
        let directory = source.parent().unwrap();
        let original_directory = directory.with_file_name("original-scenes");
        fs::rename(directory, &original_directory).unwrap();
        fs::create_dir(directory).unwrap();
        fs::write(&source, b"replacement source").unwrap();

        assert!(read_guard.verify().is_err());
        assert!(read_guard.read_snapshot().is_err());
        assert!(create_guard.verify().is_err());
        let staging = source.with_file_name(".new.scene-staging.scene.toml");
        assert!(create_guard.stage_scene(&staging, b"staged scene").is_err());
        assert!(!staging.exists());
        assert!(!original_directory
            .join(staging.file_name().unwrap())
            .exists());
        assert_eq!(fs::read(&source).unwrap(), b"replacement source");
    }

    #[test]
    fn filesystem_unix_scene_guard_refuses_replaced_staging_and_preserves_foreign_file() {
        let root = FixtureRoot::new("staging-replacement");
        let source = root.scene_path();
        let mut guard = protect_scene_path(&root.0, &source, false).unwrap();
        let staging = source.with_file_name(".new.scene-staging.scene.toml");
        let revision = guard.stage_scene(&staging, b"owned staged scene").unwrap();
        let original_staging = staging.with_file_name("owned-staging.scene.toml");
        fs::rename(&staging, &original_staging).unwrap();
        fs::write(&staging, b"foreign replacement").unwrap();

        assert!(guard.publish_scene(&staging).is_err());
        assert!(guard.remove_owned_file(&staging, &revision).is_err());
        assert!(!source.exists());
        assert_eq!(fs::read(&staging).unwrap(), b"foreign replacement");
        assert_eq!(fs::read(&original_staging).unwrap(), b"owned staged scene");
    }

    #[test]
    fn filesystem_unix_scene_guard_cleanup_uses_leased_parent_after_symlink_replacement() {
        let root = FixtureRoot::new("staging-parent-replacement");
        let source = root.scene_path();
        let mut guard = protect_scene_path(&root.0, &source, false).unwrap();
        let staging = source.with_file_name(".new.scene-staging.scene.toml");
        let revision = guard.stage_scene(&staging, b"owned staged scene").unwrap();
        let parent = source.parent().unwrap();
        let original_parent = parent.with_file_name("leased-scenes");
        let foreign_parent = root.0.join("foreign-scenes");
        fs::create_dir(&foreign_parent).unwrap();
        let foreign_staging = foreign_parent.join(staging.file_name().unwrap());
        fs::write(&foreign_staging, b"foreign staged scene").unwrap();
        fs::rename(parent, &original_parent).unwrap();
        symlink(&foreign_parent, parent).unwrap();

        assert!(guard.verify().is_err());
        assert!(guard.publish_scene(&staging).is_err());
        guard.remove_owned_file(&staging, &revision).unwrap();

        assert!(!original_parent.join(staging.file_name().unwrap()).exists());
        assert!(!original_parent.join(source.file_name().unwrap()).exists());
        assert!(!foreign_parent.join(source.file_name().unwrap()).exists());
        assert_eq!(fs::read(foreign_staging).unwrap(), b"foreign staged scene");
    }

    #[test]
    fn filesystem_project_authority_async_reload_accepts_legitimate_external_scene_edits() {
        let (_location, project) = project_fixture("external-reload");
        let document = ProjectAuthority::default()
            .open_scene(&project, SceneOpenRequest::new(main_scene_uri()))
            .unwrap();
        let original = fs::read_to_string(document.source_path()).unwrap();
        let edited = original.replace("name = \"Cube\"", "name = \"Externally Edited Cube\"");
        assert_ne!(original, edited);
        fs::write(document.source_path(), edited.as_bytes()).unwrap();

        let jobs = test_job_system();
        let ticket = ProjectAuthority::default()
            .submit_scene_open(&jobs, project, SceneOpenRequest::new(main_scene_uri()))
            .unwrap();
        let reloaded = ticket
            .wait_until(Instant::now() + Duration::from_secs(5))
            .expect("asynchronous external-edit reload must finish")
            .unwrap();

        assert_ne!(document.source_revision(), reloaded.source_revision());
        assert!(reloaded
            .world()
            .nodes()
            .iter()
            .any(|node| node.name == "Externally Edited Cube"));
    }

    #[test]
    fn filesystem_project_authority_reload_detects_byte_identical_file_replacement() {
        let (_location, project) = project_fixture("replacement-reload");
        let document = ProjectAuthority::default()
            .open_scene(&project, SceneOpenRequest::new(main_scene_uri()))
            .unwrap();
        let bytes = fs::read(document.source_path()).unwrap();
        fs::rename(
            document.source_path(),
            document.source_path().with_extension("original"),
        )
        .unwrap();
        fs::write(document.source_path(), &bytes).unwrap();

        let reloaded = ProjectAuthority::default()
            .open_scene(&project, SceneOpenRequest::new(main_scene_uri()))
            .unwrap();

        assert_eq!(fs::read(reloaded.source_path()).unwrap(), bytes);
        assert_ne!(document.source_revision(), reloaded.source_revision());
        assert!(reloaded
            .world()
            .nodes()
            .iter()
            .any(|node| node.name == "Cube"));
    }
}

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime_interface::project::RelPath;

use super::*;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

// These are editor-owned bytes: runtime keeps the payload opaque but the fixture retains the
// current versioned envelope and Workbench default shape so the editor decoder can reopen it.
const OLD_WORKSPACE: &[u8] = br#"{
  "$zircon": {
    "header": {
      "schema_id": "zircon.editor.workbench.project-workspace",
      "schema_version": 2
    },
    "payload": {
      "editor_workspace": {
        "workbench": {
          "active_main_page": "workbench",
          "main_pages": [
            {"WorkbenchPage": {"id": "workbench", "title": "Workbench", "activity_window": "window:workbench"}}
          ],
          "activity_windows": {
            "window:workbench": {
              "window_id": "window:workbench",
              "descriptor_id": "editor.workbench_window",
              "host_mode": "EmbeddedMainFrame",
              "activity_drawers": {
                "LeftTop": {"slot": "LeftTop", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "LeftBottom": {"slot": "LeftBottom", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "RightTop": {"slot": "RightTop", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "RightBottom": {"slot": "RightBottom", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "Bottom": {"slot": "Bottom", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 148.0, "visible": true}
              },
              "content_workspace": {"Tabs": {"node_id": "00000000-0000-0000-0000-000000000000", "tabs": [], "active_tab": null}},
              "menu_overflow_mode": "Auto",
              "region_overrides": {},
              "view_overrides": {}
            }
          },
          "floating_windows": []
        },
        "open_view_instances": [],
        "focused_view": null,
        "active_drawers": [],
        "scene_viewport_sessions": {}
      }
    }
  }
}"#;

const NEW_WORKSPACE: &[u8] = br#"{
  "$zircon": {
    "header": {
      "schema_id": "zircon.editor.workbench.project-workspace",
      "schema_version": 2
    },
    "payload": {
      "editor_workspace": {
        "workbench": {
          "active_main_page": "workbench",
          "main_pages": [
            {"WorkbenchPage": {"id": "workbench", "title": "Workbench", "activity_window": "window:workbench"}}
          ],
          "activity_windows": {
            "window:workbench": {
              "window_id": "window:workbench",
              "descriptor_id": "editor.workbench_window",
              "host_mode": "EmbeddedMainFrame",
              "activity_drawers": {
                "LeftTop": {"slot": "LeftTop", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "LeftBottom": {"slot": "LeftBottom", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "RightTop": {"slot": "RightTop", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "RightBottom": {"slot": "RightBottom", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 260.0, "visible": true},
                "Bottom": {"slot": "Bottom", "tab_stack": {"tabs": [], "active_tab": null}, "active_view": null, "mode": "Pinned", "extent": 148.0, "visible": true}
              },
              "content_workspace": {"Tabs": {"node_id": "00000000-0000-0000-0000-000000000000", "tabs": [], "active_tab": null}},
              "menu_overflow_mode": "Auto",
              "region_overrides": {},
              "view_overrides": {}
            }
          },
          "floating_windows": []
        },
        "open_view_instances": [],
        "focused_view": null,
        "active_drawers": ["LeftTop"],
        "scene_viewport_sessions": {}
      }
    }
  }
}"#;

#[test]
fn editor_document_scope_rejects_wrong_physical_root_and_stale_generation() {
    let root = fixture_root("scope");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "Editor document scope",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let manager = ProjectManager::open(&root).unwrap();
    let generation = manager.catalog_input_generation().sequence();

    assert!(manager
        .scoped_editor_document(&EditorDocumentScope::new(root.join("other"), generation))
        .is_err());
    assert!(manager
        .scoped_editor_document(&EditorDocumentScope::new(&root, generation + 1))
        .is_err());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn editor_document_recovery_policy_rejects_retired_and_outside_targets() {
    let root = fixture_root("policy");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    let manifest = ProjectManifest::new(
        "Editor document recovery",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    let policy = EditorDocumentRecoveryPolicy::new(&paths, &manifest).unwrap();
    let scene_path = paths
        .asset_root(&RelPath::project_assets())
        .join("scenes/main.scene.toml");
    let scene_identity = ProjectPaths::resolve_identity(&scene_path).unwrap();
    assert!(policy
        .scene_target_allowed(&scene_path, &scene_identity)
        .unwrap());
    let outside = root.join("outside.scene.toml");
    let outside_identity = ProjectPaths::resolve_identity(&outside).unwrap();
    assert!(!policy
        .scene_target_allowed(&outside, &outside_identity)
        .unwrap());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn crash_publication_recovers_scene_and_opaque_workspace_before_reopen() {
    let root = fixture_root("crash-reopen");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    fs::create_dir_all(paths.asset_root(&RelPath::project_assets()).join("scenes")).unwrap();
    ProjectManifest::new(
        "Editor document crash reopen",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let manager = ProjectManager::open(&root).unwrap();
    let generation = manager.catalog_input_generation().sequence();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    let old_scene = Scene::default();
    let mut new_scene = old_scene.clone();
    let cube = new_scene
        .nodes()
        .iter()
        .find(|node| node.name == "Cube")
        .expect("the fixture Scene must contain a Cube node")
        .clone();
    let mut changed_transform = cube.transform.clone();
    changed_transform.translation.x += 3.75;
    assert!(new_scene
        .update_transform(cube.id, changed_transform)
        .unwrap());
    let old_scene_document = old_scene
        .to_scene_asset(&manager)
        .unwrap()
        .to_project_toml_string(|reference| manager.persist_runtime_reference(reference))
        .unwrap()
        .into_bytes();
    let scene_path = manager.source_path_for_uri(&scene_uri).unwrap();
    fs::write(&scene_path, &old_scene_document).unwrap();
    let workspace_path = paths.derived_root().join(WORKSPACE_FILE);
    fs::write(&workspace_path, OLD_WORKSPACE).unwrap();
    let scoped = manager
        .scoped_editor_document(&EditorDocumentScope::new(&root, generation))
        .unwrap();
    let result = scoped.commit_scene_workspace_with_test_fault(
        &scene_uri,
        &new_scene,
        NEW_WORKSPACE,
        ProjectTransactionFault::CrashAfterTargetReplace(0),
    );
    assert!(result.is_err());
    assert!(paths.derived_root().join(JOURNAL_DIRECTORY).exists());
    drop(scoped);
    drop(manager);

    let reopened = ProjectManager::open(&root).unwrap();
    let journal_directory = paths.derived_root().join(JOURNAL_DIRECTORY);
    assert!(journal_directory.is_dir());
    assert!(fs::read_dir(journal_directory).unwrap().next().is_none());
    assert!(reopened.paths().root().exists());
    assert_eq!(fs::read(&scene_path).unwrap(), old_scene_document);
    assert_eq!(fs::read(&workspace_path).unwrap(), OLD_WORKSPACE);
    let decoded = Scene::load_scene_from_uri(&reopened, &scene_uri).unwrap();
    assert_eq!(decoded.nodes(), old_scene.nodes());

    let reopened_generation = reopened.catalog_input_generation().sequence();
    let reopened_scope = reopened
        .scoped_editor_document(&EditorDocumentScope::new(&root, reopened_generation))
        .unwrap();
    reopened_scope
        .commit_scene_workspace(&scene_uri, &new_scene, NEW_WORKSPACE)
        .unwrap()
        .ensure_durable()
        .unwrap();
    drop(reopened_scope);
    drop(reopened);
    let committed = ProjectManager::open(&root).unwrap();
    let new_scene_document = new_scene
        .to_scene_asset(&committed)
        .unwrap()
        .to_project_toml_string(|reference| committed.persist_runtime_reference(reference))
        .unwrap()
        .into_bytes();
    assert_eq!(fs::read(&scene_path).unwrap(), new_scene_document);
    assert_eq!(fs::read(&workspace_path).unwrap(), NEW_WORKSPACE);
    let committed_scene = Scene::load_scene_from_uri(&committed, &scene_uri).unwrap();
    assert_eq!(committed_scene.nodes(), new_scene.nodes());
    drop(committed);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn none_workspace_save_rejects_scene_directory_reparse_without_mutating_either_owner(
) -> Result<(), DirectoryLinkFixtureError> {
    let root = fixture_root("none-reparse");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    let project_scenes = paths.asset_root(&RelPath::project_assets()).join("scenes");
    fs::create_dir_all(&project_scenes).unwrap();
    ProjectManifest::new(
        "Editor document None reparse",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let manager = ProjectManager::open(&root).unwrap();
    let generation = manager.catalog_input_generation().sequence();

    let outside = fixture_root("none-reparse-outside");
    let outside_scenes = outside.join("scenes");
    fs::create_dir_all(&outside_scenes).unwrap();
    let outside_scene = outside_scenes.join("main.scene.toml");
    let outside_before = br#"outside-scene-before"#;
    fs::write(&outside_scene, outside_before).unwrap();
    let displaced_scenes = root.join(".none-reparse-project-scenes");
    fs::rename(&project_scenes, &displaced_scenes).unwrap();
    create_directory_link(&outside_scenes, &project_scenes)?;

    let workspace_path = paths.derived_root().join(WORKSPACE_FILE);
    let workspace_before = br#"valid-workspace-bytes"#;
    fs::write(&workspace_path, workspace_before).unwrap();
    let scoped = manager
        .scoped_editor_document(&EditorDocumentScope::new(&root, generation))
        .unwrap();
    let result = scoped.commit_scene_without_workspace(
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        &Scene::default(),
    );

    assert!(
        result.is_err(),
        "a Scene target behind a directory reparse must be rejected"
    );
    assert_eq!(fs::read(&outside_scene).unwrap(), outside_before);
    assert_eq!(fs::read(&workspace_path).unwrap(), workspace_before);

    drop(scoped);
    drop(manager);
    remove_directory_link(&project_scenes);
    fs::rename(displaced_scenes, project_scenes).unwrap();
    fs::remove_dir_all(outside).unwrap();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[derive(Debug)]
enum DirectoryLinkFixtureError {
    Unsupported(std::io::Error),
}

impl std::fmt::Display for DirectoryLinkFixtureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(error) => write!(
                formatter,
                "native directory reparse fixture unavailable: {error}"
            ),
        }
    }
}

#[cfg(unix)]
fn create_directory_link(
    target: &std::path::Path,
    link: &std::path::Path,
) -> Result<(), DirectoryLinkFixtureError> {
    std::os::unix::fs::symlink(target, link).map_err(DirectoryLinkFixtureError::Unsupported)
}

#[cfg(windows)]
fn create_directory_link(
    target: &std::path::Path,
    link: &std::path::Path,
) -> Result<(), DirectoryLinkFixtureError> {
    std::os::windows::fs::symlink_dir(target, link).map_err(DirectoryLinkFixtureError::Unsupported)
}

#[cfg(unix)]
fn remove_directory_link(link: &std::path::Path) {
    let _ = fs::remove_file(link);
}

#[cfg(windows)]
fn remove_directory_link(link: &std::path::Path) {
    let _ = fs::remove_dir(link);
}

fn fixture_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-document-{label}-{}-{}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

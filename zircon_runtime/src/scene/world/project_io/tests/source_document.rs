#![cfg(windows)]

use super::*;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::{ImportedAsset, SceneAsset};
use crate::scene::components::NodeKind;
use std::fs;
use std::os::windows::fs::MetadataExt;
use zircon_runtime_interface::project::RelPath;

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let result = fs::remove_dir_all(&self.0);
        if !std::thread::panicking() {
            result.expect("remove the owned source-document fixture");
        }
    }
}

fn fixture(label: &str) -> Fixture {
    let base = Path::new(r"E:\cargo-targets\zircon-local");
    for ancestor in base.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).expect("physical E output root must exist");
        assert!(metadata.is_dir() && metadata.file_attributes() & 0x400 == 0);
    }
    assert!(fs::canonicalize(base)
        .unwrap()
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .eq_ignore_ascii_case(r"E:\cargo-targets\zircon-local"));
    let root = base.join(format!(
        "editor182-observed-source-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).expect("create a new owned fixture");
    Fixture(root)
}

fn project(root: &Path) -> ProjectManager {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    fs::create_dir_all(paths.asset_root(&RelPath::project_assets()).join("scenes")).unwrap();
    let uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
    ProjectManifest::new("Observed scene source", uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    fs::write(
        paths
            .asset_root(&RelPath::project_assets())
            .join("scenes/main.scene.toml"),
        SceneAsset {
            entities: Vec::new(),
        }
        .to_toml_string()
        .unwrap(),
    )
    .unwrap();
    let mut project = ProjectManager::open(root).unwrap();
    project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    project.scan_and_import().unwrap();
    project
}

fn write_world(project: &ProjectManager, count: usize) {
    let mut world = World::empty_for_project(project).unwrap();
    for _ in 0..count {
        world.spawn_node(NodeKind::Empty).unwrap();
    }
    let source = world
        .to_scene_asset(project)
        .unwrap()
        .to_project_toml_string(|reference| project.persist_runtime_reference(reference))
        .unwrap();
    fs::write(
        project
            .source_path_for_uri(&project.manifest().default_scene)
            .unwrap(),
        source,
    )
    .unwrap();
}

#[test]
fn authoring_load_revision_is_bound_to_parsed_source_bytes() {
    let fixture = fixture("binding");
    let project = project(&fixture.0);
    let uri = project.manifest().default_scene.clone();
    write_world(&project, 1);
    let (world, revision) = World::load_scene_document(&project, &uri)
        .unwrap()
        .into_parts();
    assert_eq!(world.nodes().len(), 1);
    write_world(&project, 2);
    let path = project.source_path_for_uri(&uri).unwrap();
    let latest = fs::read(&path).unwrap();
    assert!(!revision.matches_path(&path).unwrap());
    assert!(matches!(
        world.save_scene_to_project(&project, &uri, &revision),
        Err(SceneProjectError::SourceConflict { .. })
    ));
    assert_eq!(fs::read(&path).unwrap(), latest);
}

#[test]
fn cached_artifact_never_becomes_the_new_source_baseline() {
    let fixture = fixture("cache");
    let project = project(&fixture.0);
    let uri = project.manifest().default_scene.clone();
    write_world(&project, 1);
    let ImportedAsset::Scene(cached) = project.load_artifact(&uri).unwrap() else {
        panic!("Scene artifact");
    };
    assert!(
        cached.entities.is_empty(),
        "fixture must retain the old imported A artifact"
    );
    let (world, revision) = World::load_scene_document(&project, &uri)
        .unwrap()
        .into_parts();
    assert_eq!(
        world.nodes().len(),
        1,
        "authoring must parse source B rather than pair artifact A with revision B"
    );
    let first = world
        .save_scene_to_project(&project, &uri, &revision)
        .unwrap();
    assert!(first.is_durable());
    let second = world
        .save_scene_to_project(&project, &uri, first.source_revision())
        .unwrap();
    assert!(
        second.is_durable(),
        "repeat Save uses the actual publication receipt as its new base"
    );
}

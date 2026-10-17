#[cfg(windows)]
use super::ProjectManager;
#[cfg(windows)]
use crate::asset::project::{ProjectManifest, ProjectPaths};
#[cfg(windows)]
use crate::asset::AssetUri;
#[cfg(windows)]
use crate::core::resource::ResourceLocator;
#[cfg(windows)]
use crate::scene::World;
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::process::Command;
#[cfg(windows)]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(windows)]
use zircon_runtime_interface::project::RelPath;

#[cfg(windows)]
struct ReparseFixture {
    root: std::path::PathBuf,
    junction: std::path::PathBuf,
}

#[cfg(windows)]
impl Drop for ReparseFixture {
    fn drop(&mut self) {
        // Remove the junction itself first; never recursively walk its target.
        let _ = fs::remove_dir(&self.junction);
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
#[cfg(windows)]
fn existing_project_source_rechecks_reparse_target_before_world_save() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the epoch")
        .as_nanos();
    let fixture_root = std::env::temp_dir().join(format!("zircon_source_reparse_{nonce}"));
    let root = fixture_root.join("project");
    let outside = fixture_root.join("outside");
    let _cleanup = ReparseFixture {
        root: fixture_root,
        junction: root.join("assets"),
    };
    let paths = ProjectPaths::from_root(&root).expect("resolve test project root");
    paths
        .ensure_layout(&[RelPath::project_assets()])
        .expect("create project layout");
    ProjectManifest::new(
        "SourceReparseBoundary",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .expect("write project manifest");
    let project = ProjectManager::open(&root).expect("open project before mutation");
    let asset_root = paths.asset_root(&RelPath::project_assets());
    fs::create_dir_all(&outside).expect("create outside target");
    fs::create_dir_all(outside.join("scenes")).expect("create outside scene directory");
    fs::write(outside.join("scenes/escape.scene.toml"), "outside")
        .expect("create outside scene target");
    fs::remove_dir_all(&asset_root).expect("remove original asset root");
    let junction_command = format!(
        r#"mklink /J "{}" "{}""#,
        root.join("assets").to_string_lossy().replace('/', "\\"),
        outside.to_string_lossy().replace('/', "\\")
    );
    let junction = Command::new("cmd")
        .args(["/D", "/S", "/C"])
        // cmd /C must receive the literal quotes around these controlled fixture paths.
        .raw_arg(format!(r#""{junction_command}""#))
        .output()
        .expect("start mklink for test junction");
    assert!(
        junction.status.success(),
        "create test junction failed: {}",
        String::from_utf8_lossy(&junction.stderr)
    );

    let uri = AssetUri::parse("res://scenes/escape.scene.toml").unwrap();
    let direct_error = project
        .resolve_source_path_for_uri(&uri)
        .expect_err("physical URI resolution must reject a reparse target outside the root");
    assert!(direct_error
        .to_string()
        .contains("outside registered asset roots"));
    let error = project
        .resolve_existing_or_primary_project_source_path_for_uri(&uri)
        .expect_err("reparse target outside the registered root must be rejected");
    assert!(error.to_string().contains("outside registered asset roots"));

    let outside_path = outside.join("scenes/escape.scene.toml");
    let before_save = fs::read(&outside_path).expect("read external scene before save attempt");
    let world = World::empty_for_project(&project).expect("create project world");
    let scene_locator = ResourceLocator::parse("res://scenes/escape.scene.toml").unwrap();
    let save_error = world
        .save_scene_to_project(
            &project,
            &scene_locator,
            &crate::scene::world::SceneSourceRevision::Missing,
        )
        .expect_err("world save through an outside-root junction must be rejected");
    assert!(save_error
        .to_string()
        .contains("outside registered asset roots"));
    assert_eq!(
        fs::read(&outside_path).expect("read external scene after save attempt"),
        before_save,
        "rejected world save must leave external bytes unchanged"
    );

    drop(project);
}

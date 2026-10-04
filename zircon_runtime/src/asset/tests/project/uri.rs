//! 守护项目与缓存 URI 的规范化及越界拒绝，并在 Windows 上验证大小写虚拟化路径的反向定位。

#[cfg(windows)]
use std::fs;

use crate::core::resource::ResourceScheme;

#[cfg(windows)]
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
#[cfg(windows)]
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::AssetUri;
#[cfg(windows)]
use zircon_runtime_interface::project::RelPath;

#[test]
fn asset_uri_normalizes_res_and_lib_paths() {
    let res = AssetUri::parse("res://textures\\environment/sky.png").unwrap();
    let lib = AssetUri::parse("lib://imports\\model.cache").unwrap();

    assert_eq!(res.scheme(), ResourceScheme::Res);
    assert_eq!(res.path(), "textures/environment/sky.png");
    assert_eq!(res.to_string(), "res://textures/environment/sky.png");
    assert_eq!(lib.scheme(), ResourceScheme::Library);
    assert_eq!(lib.path(), "imports/model.cache");
    assert_eq!(lib.to_string(), "lib://imports/model.cache");
}

#[test]
fn asset_uri_rejects_escape_attempts() {
    assert!(AssetUri::parse("res://../outside.txt").is_err());
    assert!(AssetUri::parse("lib://../../outside.bin").is_err());
    assert!(AssetUri::parse("res://folder/../../outside.txt").is_err());
}

#[cfg(windows)]
#[test]
// Windows 调用方可能交付大小写不同的绝对路径；项目根判定和 URI 生成仍须使用磁盘上的原始拼写。
fn project_manager_resolves_a_case_virtualized_windows_source_path() {
    let root = unique_temp_project_root("project uri \u{8d44}\u{6e90}\u{8def}\u{5f84}");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let assets_root = paths.asset_root(&RelPath::project_assets());
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "Case virtualization",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source = assets_root.join("models/\u{7acb}\u{65b9}\u{4f53}.obj");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "cube").unwrap();
    let virtualized_source = source.to_string_lossy().to_ascii_uppercase();
    let manager = ProjectManager::open(&root).unwrap();

    assert_eq!(
        manager
            .project_asset_root_for_source_path(std::path::Path::new(&virtualized_source))
            .unwrap(),
        assets_root.as_path()
    );
    assert_eq!(
        manager
            .project_uri_for_source_path(std::path::Path::new(&virtualized_source))
            .unwrap(),
        AssetUri::parse("res://models/\u{7acb}\u{65b9}\u{4f53}.obj").unwrap()
    );

    fs::remove_dir_all(root).unwrap();
}

use std::fs;

use super::*;

#[test]
fn project_cover_prefers_zircon_metadata_cover() {
    let root = temp_project_root("cover-priority");
    let metadata_dir = root.join(".zircon");
    fs::create_dir_all(&metadata_dir).unwrap();
    fs::write(root.join("cover.png"), "root").unwrap();
    let expected = metadata_dir.join("cover.png");
    fs::write(&expected, "metadata").unwrap();

    let cover = project_cover_path(&root);
    fs::remove_dir_all(&root).unwrap();

    assert_eq!(cover, Some(expected));
}

#[test]
fn project_cover_accepts_asset_thumbnail_when_root_cover_is_missing() {
    let root = temp_project_root("cover-assets");
    let assets_dir = root.join("Assets");
    fs::create_dir_all(&assets_dir).unwrap();
    let expected = assets_dir.join("thumbnail.jpg");
    fs::write(&expected, "asset").unwrap();

    let cover = project_cover_path(&root);
    fs::remove_dir_all(&root).unwrap();

    assert_eq!(cover, Some(expected));
}

#[test]
fn project_cover_ignores_missing_or_non_directory_roots() {
    let root = std::env::temp_dir().join(format!(
        "zircon-hub-cover-missing-{}",
        crate::projects::now_unix_ms()
    ));

    assert_eq!(project_cover_path(&root), None);
    assert_eq!(project_cover_path(""), None);
}

fn temp_project_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zircon-hub-{label}-{}",
        crate::projects::now_unix_ms()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

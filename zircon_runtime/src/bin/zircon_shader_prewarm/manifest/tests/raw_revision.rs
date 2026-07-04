use std::fs;

use super::super::asset_root_manifest;

#[test]
fn shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_raw_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let shader_path = root.join("simple.wgsl");
    fs::write(&shader_path, "fn simple_a() {}\n").unwrap();

    let first_revision = asset_root_manifest(&root).unwrap().variants[0]
        .key
        .material_revision;
    fs::write(&shader_path, "fn simple_b() {}\n").unwrap();
    let second_revision = asset_root_manifest(&root).unwrap().variants[0]
        .key
        .material_revision;

    assert_ne!(first_revision, 0);
    assert_ne!(second_revision, 0);
    assert_ne!(
        first_revision, second_revision,
        "raw shader source edits must export a new shader prewarm material revision"
    );
    let _ = fs::remove_dir_all(root);
}

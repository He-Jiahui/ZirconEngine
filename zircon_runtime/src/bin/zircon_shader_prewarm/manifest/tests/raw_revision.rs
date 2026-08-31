use std::fs;

use super::super::asset_root_manifest;

#[test]
fn shader_prewarm_asset_root_manifest_does_not_promote_raw_wgsl_module() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_raw_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let shader_path = root.join("simple.wgsl");
    fs::write(&shader_path, "fn simple_a() {}\n").unwrap();

    let first = asset_root_manifest(&root).unwrap();
    fs::write(&shader_path, "fn simple_b() {}\n").unwrap();
    let second = asset_root_manifest(&root).unwrap();

    assert!(first.sources.is_empty());
    assert!(first.variants.is_empty());
    assert!(second.sources.is_empty());
    assert!(second.variants.is_empty());
    let _ = fs::remove_dir_all(root);
}

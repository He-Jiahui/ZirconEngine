use std::fs;

use crate::error::ShaderPrewarmAssetScanError;

use super::super::asset_root_manifest;

#[test]
fn shader_prewarm_asset_root_scan_reports_typed_read_root_error() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_asset_scan_file_root_{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&root);
    let _ = fs::remove_dir_all(&root);
    fs::write(&root, "not a directory").unwrap();

    let error = asset_root_manifest(&root).unwrap_err();

    match error {
        ShaderPrewarmAssetScanError::ReadAssetRoot { path, source: _ } => {
            assert_eq!(path, root);
        }
        other => panic!("expected typed asset-root read error, got {other:?}"),
    }

    let _ = fs::remove_file(root);
}

#[test]
fn shader_prewarm_asset_root_scan_reports_typed_zshader_parse_error() {
    let root = unique_root("zircon_shader_prewarm_asset_scan_bad_zshader");
    fs::create_dir_all(&root).unwrap();
    let zshader_path = root.join("bad.zshader");
    fs::write(&zshader_path, "{not toml").unwrap();

    let error = asset_root_manifest(&root).unwrap_err();

    match error {
        ShaderPrewarmAssetScanError::ParseZShader { path, source: _ } => {
            assert_eq!(path, zshader_path);
        }
        other => panic!("expected typed zshader parse error, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_scan_reports_typed_empty_wgsl_error() {
    let root = unique_root("zircon_shader_prewarm_asset_scan_empty_wgsl");
    fs::create_dir_all(&root).unwrap();
    let wgsl_path = root.join("empty.wgsl");
    fs::write(&wgsl_path, "   \n\t").unwrap();

    let error = asset_root_manifest(&root).unwrap_err();

    match error {
        ShaderPrewarmAssetScanError::EmptyShaderSource { path } => {
            assert_eq!(path, wgsl_path);
        }
        other => panic!("expected typed empty WGSL error, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_scan_reports_typed_zmaterial_parse_error() {
    let root = unique_root("zircon_shader_prewarm_asset_scan_bad_zmaterial");
    fs::create_dir_all(&root).unwrap();
    let material_path = root.join("bad.zmaterial");
    fs::write(&material_path, "{not toml").unwrap();

    let error = asset_root_manifest(&root).unwrap_err();

    match error {
        ShaderPrewarmAssetScanError::ParseZMaterial { path, source: _ } => {
            assert_eq!(path, material_path);
        }
        other => panic!("expected typed zmaterial parse error, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

fn unique_root(prefix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("{prefix}_{}", std::process::id()))
}

//! Small generated projects give product-frame fixtures the production pipeline-cache owner.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetManager, AssetUri};

static NEXT_PROJECT: AtomicU64 = AtomicU64::new(0);

pub fn open_smoke_project(manager: &ProjectAssetManager, label: &str) {
    assert!(
        !label.is_empty()
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')),
        "product project labels must be simple path segments"
    );
    let output = std::env::var_os("ZIRCON_RENDERING_SMOKE_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    assert_physical_output(&output);
    std::fs::create_dir_all(&output).expect("create registered rendering fixture output");
    assert_physical_output(&output);
    let resolved = std::fs::canonicalize(&output).expect("resolve rendering fixture output");
    let resolved = resolved
        .to_str()
        .expect("rendering fixture output should be UTF-8")
        .replace('\\', "/");
    let resolved = resolved.strip_prefix("//?/").unwrap_or(&resolved);
    let requested = output
        .to_str()
        .expect("rendering fixture output should be UTF-8")
        .replace('\\', "/");
    assert!(
        resolved.eq_ignore_ascii_case(&requested),
        "rendering fixture output must retain its physical drive and path"
    );

    let project_root = output.join(format!(
        "{label}-{}-{}",
        std::process::id(),
        NEXT_PROJECT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&project_root).expect("create an exclusively owned product project");
    let paths = ProjectPaths::from_root(&project_root).expect("physical product project paths");
    let manifest = ProjectManifest::new(
        label,
        AssetUri::parse("res://scenes/main.scene.toml").expect("product scene URI"),
        1,
    );
    paths
        .ensure_layout(&manifest.asset_roots)
        .expect("product project layout");
    manifest
        .save(paths.manifest_path())
        .expect("generated product project manifest");
    AssetManager::open_project(
        manager,
        project_root.to_str().expect("product project UTF-8"),
    )
    .expect("activate the product project before renderer construction");
}

fn assert_physical_output(output: &Path) {
    let text = output
        .to_str()
        .expect("rendering fixture output should be UTF-8")
        .replace('\\', "/");
    let bytes = text.as_bytes();
    assert!(
        output.is_absolute()
            && bytes.len() > 3
            && matches!(bytes[0].to_ascii_uppercase(), b'D' | b'E' | b'F')
            && bytes[1..3] == *b":/",
        "rendering fixture output requires a physical D, E or F drive root"
    );
    let mut segments = text[3..].split('/');
    assert!(
        segments
            .next()
            .is_some_and(|segment| segment.eq_ignore_ascii_case("cargo-targets")),
        "rendering fixture output requires the drive-root cargo-targets directory"
    );
    assert!(
        segments.all(|segment| !segment.is_empty()
            && segment != "."
            && segment != ".."
            && !segment.contains(':')),
        "rendering fixture output must not contain path aliases"
    );
    for ancestor in output.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                assert!(
                    !metadata.file_type().is_symlink(),
                    "fixture output has a symlink"
                );
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    assert_eq!(
                        metadata.file_attributes() & 0x400,
                        0,
                        "fixture output has a reparse ancestor"
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("inspect rendering fixture output: {error}"),
        }
    }
}

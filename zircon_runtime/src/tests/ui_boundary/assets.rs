#[test]
fn production_ui_entry_assets_live_under_crate_assets_not_src() {
    let runtime_fixture_source = include_str!("../../ui/runtime_ui/runtime_ui_fixture.rs");
    let runtime_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ui")
        .join("runtime")
        .join("fixtures");

    assert!(
        !runtime_fixture_source.contains("include_str!(\"fixtures/"),
        "runtime UI fixtures should load from crate assets instead of src fixtures"
    );

    for relative in [
        "hud_overlay.ui.toml",
        "pause_menu.ui.toml",
        "settings_dialog.ui.toml",
        "inventory_list.ui.toml",
        "quest_log_dialog.ui.toml",
    ] {
        assert!(
            runtime_assets.join(relative).exists(),
            "expected runtime fixture entry asset {relative} under {:?}",
            runtime_assets
        );
    }

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should live under the repo root");
    for crate_src in [
        repo_root.join("zircon_editor").join("src"),
        repo_root.join("zircon_runtime").join("src"),
    ] {
        let lingering = collect_ui_toml_files(&crate_src);
        assert!(
            lingering.is_empty(),
            "production ui entry assets must not live under src; found {:?}",
            lingering
        );
    }
}

#[test]
fn default_runtime_font_manifest_stays_inside_runtime_assets() {
    let assets_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let manifest_path = assets_root.join("fonts").join("default.font.toml");
    let manifest_source = std::fs::read_to_string(&manifest_path)
        .expect("default runtime font manifest should exist under crate assets");
    let manifest: toml::Value =
        toml::from_str(&manifest_source).expect("default runtime font manifest should parse");
    let source = manifest
        .get("source")
        .and_then(toml::Value::as_str)
        .expect("default runtime font manifest should declare a source");
    let resolved_source = std::fs::canonicalize(
        manifest_path
            .parent()
            .expect("default font manifest should live in a folder")
            .join(source),
    )
    .expect("default runtime font source should resolve on disk");
    let canonical_assets_root =
        std::fs::canonicalize(&assets_root).expect("runtime assets root should resolve");

    assert!(
        resolved_source.starts_with(&canonical_assets_root),
        "default runtime font source should stay inside runtime assets; got {:?} outside {:?}",
        resolved_source,
        canonical_assets_root
    );
}

fn collect_ui_toml_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_ui_toml_files(&path));
            } else if path.extension() == Some(std::ffi::OsStr::new("toml"))
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with(".ui.toml"))
            {
                files.push(path);
            }
        }
    }
    files
}

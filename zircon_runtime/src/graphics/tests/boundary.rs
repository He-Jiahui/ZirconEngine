#[test]
fn graphics_runtime_surface_no_longer_depends_on_legacy_scene_crate() {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));

    assert!(
        !manifest.contains("zircon_scene"),
        "zircon_runtime graphics should stay fully absorbed and stop depending on the legacy zircon_scene crate"
    );
}

#[test]
fn graphics_root_no_longer_exports_legacy_preview_or_render_service_surface() {
    let graphics_mod_source =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/graphics/mod.rs"));

    for forbidden in [
        "pub use backend::RuntimePreviewRenderer;",
        "pub use service::{RenderService, SharedTextureRenderService};",
        "mod service;",
    ] {
        assert!(
            !graphics_mod_source.contains(forbidden),
            "zircon_runtime graphics root should stop exporting legacy surface `{forbidden}`"
        );
    }
}

#[test]
fn graphics_root_no_longer_exports_runtime_ui_host_surface() {
    let graphics_mod_source =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/graphics/mod.rs"));
    let runtime_mod_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/graphics/runtime/mod.rs"
    ));

    for forbidden in [
        "RuntimeUiFixture",
        "RuntimeUiManager",
        "RuntimeUiManagerError",
        "mod ui;",
        "pub use ui::{RuntimeUiFixture, RuntimeUiManager, RuntimeUiManagerError};",
    ] {
        assert!(
            !graphics_mod_source.contains(forbidden) && !runtime_mod_source.contains(forbidden),
            "zircon_runtime graphics should stop owning runtime UI host surface `{forbidden}`"
        );
    }
}

#[test]
fn hybrid_gi_old_probe_trace_types_stay_confined_to_extract_source_adapter() {
    use std::path::{Path, PathBuf};

    let graphics_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/graphics");
    let allowed_adapter = PathBuf::from("hybrid_gi_extract_sources/normalize.rs");
    let mut violations = Vec::new();

    collect_rust_files(
        &graphics_root,
        &graphics_root,
        &mut |relative_path, source| {
            let normalized_path = normalize_test_path(relative_path);
            if normalized_path == allowed_adapter
                || normalized_path.components().any(|component| {
                    matches!(component.as_os_str().to_str(), Some("tests" | "tests.rs"))
                })
            {
                return;
            }

            let normalized_source = source.replace("\r\n", "\n");
            let production_source = normalized_source
                .split("\n#[cfg(test)]\nmod tests")
                .next()
                .unwrap_or(&normalized_source);
            if production_source.contains("RenderHybridGiProbe")
                || production_source.contains("RenderHybridGiTraceRegion")
            {
                violations.push(normalized_path.display().to_string());
            }
        },
    );

    assert!(
        violations.is_empty(),
        "RenderHybridGiProbe / RenderHybridGiTraceRegion should stay behind hybrid_gi_extract_sources::normalize; production leaks: {violations:?}"
    );

    fn collect_rust_files(root: &Path, current: &Path, visit: &mut impl FnMut(&Path, &str)) {
        for entry in std::fs::read_dir(current).expect("read graphics source directory") {
            let entry = entry.expect("read graphics source entry");
            let path = entry.path();
            if path.is_dir() {
                collect_rust_files(root, &path, visit);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                let relative_path = path.strip_prefix(root).expect("path under graphics root");
                let source = std::fs::read_to_string(&path).expect("read graphics source file");
                visit(relative_path, &source);
            }
        }
    }

    fn normalize_test_path(path: &Path) -> PathBuf {
        path.components().collect()
    }
}

#[path = "structure_convention/diagnostics_surface.rs"]
mod diagnostics_surface;
#[path = "structure_convention/facade_surface.rs"]
mod facade_surface;
#[path = "structure_convention/graphics_dead_code/mod.rs"]
mod graphics_dead_code;
#[path = "structure_convention/provider_boilerplate.rs"]
mod provider_boilerplate;
#[path = "structure_convention/runtime_dead_code.rs"]
mod runtime_dead_code;

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    let missing: Vec<_> = required
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(
        missing.is_empty(),
        "{label} missing required anchors: {missing:?}"
    );
}

fn runtime_src_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .join("src")
            .join(relative)
    } else {
        std::path::PathBuf::from("zircon_runtime")
            .join("src")
            .join(relative)
    }
}

fn repo_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .parent()
            .expect("zircon_runtime manifest should live under repository root")
            .join(relative)
    } else {
        std::path::PathBuf::from(relative)
    }
}

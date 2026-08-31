use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::RenderDevice;

const FORBIDDEN_WGPU_SOURCE_NEEDLES: &[&str] =
    &["wgpu::", "use wgpu", "pub use wgpu", "extern crate wgpu"];

const FORBIDDEN_RHI_SEMANTIC_TERMS: &[&str] = &[
    "Mesh", "Material", "Light", "Scene", "Camera", "Ui", "Sprite",
];

#[test]
fn neutral_render_device_contract_is_object_safe() {
    fn accepts_shared_owner(_: Arc<dyn RenderDevice>) {}

    let _: fn(Arc<dyn RenderDevice>) = accepts_shared_owner;
}

#[test]
fn app_editor_framework_and_interface_sources_do_not_import_wgpu_directly() {
    let roots = [
        "zircon_app/src",
        "zircon_editor/src",
        "zircon_runtime/src/core/framework",
        "zircon_runtime_interface/src",
    ];
    let manifests = [
        "zircon_app/Cargo.toml",
        "zircon_editor/Cargo.toml",
        "zircon_runtime_interface/Cargo.toml",
    ];
    let mut violations = Vec::new();

    for root in roots {
        let root = repo_root().join(root);
        let mut sources = Vec::new();
        collect_production_rust_sources(&root, &mut sources);

        for source in sources {
            let text = std::fs::read_to_string(&source).expect("read boundary source");
            let text = production_text(&text);
            for needle in FORBIDDEN_WGPU_SOURCE_NEEDLES {
                if let Some(line) = first_line_containing(text, needle) {
                    violations.push(format!(
                        "{}:{line} contains forbidden WGPU marker `{needle}`",
                        relative_to_repo(&source).display()
                    ));
                }
            }
        }
    }

    for manifest in manifests {
        let manifest_path = repo_root().join(manifest);
        let text = std::fs::read_to_string(&manifest_path).expect("read boundary manifest");
        for (line_index, line) in text.lines().enumerate() {
            if declares_wgpu_dependency(line) {
                violations.push(format!(
                    "{}:{} declares a direct WGPU dependency",
                    relative_to_repo(&manifest_path).display(),
                    line_index + 1
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "app, editor, framework, and interface layers must route through runtime RHI boundaries:\n{}",
        violations.join("\n")
    );
}

#[test]
fn neutral_rhi_descriptor_sources_do_not_depend_on_upper_render_semantics() {
    let sources = [
        "zircon_runtime/crates/zr_rhi/src/capabilities.rs",
        "zircon_runtime/crates/zr_rhi/src/descriptors.rs",
        "zircon_runtime/crates/zr_rhi/src/descriptors/pipeline.rs",
        "zircon_runtime/crates/zr_rhi/src/device.rs",
    ];
    let mut violations = Vec::new();

    for source in sources {
        let source_path = repo_root().join(source);
        let text = std::fs::read_to_string(&source_path).expect("read neutral RHI source");
        for term in FORBIDDEN_RHI_SEMANTIC_TERMS {
            if let Some(line) = first_line_with_upper_layer_term(&text, term) {
                violations.push(format!(
                    "{}:{line} contains upper-layer render term `{term}`",
                    relative_to_repo(&source_path).display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "neutral RHI descriptor/device sources must stay below scene, material, mesh, camera, UI, and sprite semantics:\n{}",
        violations.join("\n")
    );
}

#[test]
fn physical_rhi_crates_enforce_the_neutral_to_wgpu_dependency_direction() {
    let workspace_manifest = read_repo("Cargo.toml");
    let runtime_manifest = read_repo("zircon_runtime/Cargo.toml");
    let neutral_manifest = read_repo("zircon_runtime/crates/zr_rhi/Cargo.toml");
    let backend_manifest = read_repo("zircon_runtime/crates/zr_rhi_wgpu/Cargo.toml");
    let app_manifest = read_repo("zircon_app/Cargo.toml");
    let editor_manifest = read_repo("zircon_editor/Cargo.toml");
    let runtime_root = read_repo("zircon_runtime/src/lib.rs");
    let runtime_facade = read_repo("zircon_runtime/src/rhi.rs");

    for member in [
        "zircon_runtime/crates/zr_rhi",
        "zircon_runtime/crates/zr_rhi_wgpu",
    ] {
        assert!(
            workspace_manifest.contains(&format!("\"{member}\"")),
            "workspace must include physical RHI crate `{member}`"
        );
    }
    assert!(
        !neutral_manifest.contains("wgpu") && !neutral_manifest.contains("zr_rhi_wgpu"),
        "zr_rhi must not name WGPU or its concrete backend crate in any dependency section"
    );
    assert!(
        backend_manifest.contains("wgpu.workspace = true")
            && backend_manifest.contains("zr_rhi.workspace = true"),
        "zr_rhi_wgpu must depend on WGPU and the neutral zr_rhi contract crate"
    );
    assert!(
        runtime_manifest.contains("zr_rhi.workspace = true")
            && runtime_manifest.contains("zr_rhi_wgpu = { workspace = true, optional = true }"),
        "zircon_runtime must always compile neutral contracts and keep only the backend optional"
    );
    for (owner, manifest) in [
        ("zircon_app", app_manifest.as_str()),
        ("zircon_editor", editor_manifest.as_str()),
    ] {
        assert!(
            !manifest.contains("zr_rhi") && !manifest.contains("zr_rhi_wgpu"),
            "{owner} must consume the curated zircon_runtime::rhi facade, not physical RHI crates"
        );
    }
    assert!(
        !runtime_root.contains("rhi_wgpu"),
        "zircon_runtime must not retain the deleted rhi_wgpu module"
    );
    assert!(
        runtime_facade.contains("pub use zr_rhi::{")
            && runtime_facade.contains("zr_rhi_wgpu::WgpuUiSurfacePresenter::new"),
        "zircon_runtime::rhi must remain the curated neutral facade and default backend factory"
    );
    assert!(
        !repo_root().join("zircon_runtime/src/rhi_wgpu").exists()
            && !repo_root().join("zircon_runtime/src/rhi").exists(),
        "the monolithic Runtime RHI source directories must be removed"
    );
}

#[test]
fn wgpu_dependency_detector_covers_supported_toml_forms_without_prefix_false_positives() {
    for declaration in [
        "wgpu = \"29\"",
        "wgpu.workspace = true",
        "wgpu.version = \"29\"",
        "[dependencies.wgpu]",
        "[dev-dependencies.wgpu]",
        "[target.'cfg(windows)'.dependencies.wgpu]",
        "render_backend = { package = \"wgpu\", workspace = true }",
    ] {
        assert!(
            declares_wgpu_dependency(declaration),
            "expected WGPU dependency declaration to be detected: {declaration}"
        );
    }

    for declaration in [
        "wgpu-types.workspace = true",
        "zr_rhi_wgpu.workspace = true",
        "description = \"wgpu\"",
        "# wgpu.workspace = true",
    ] {
        assert!(
            !declares_wgpu_dependency(declaration),
            "non-WGPU dependency text must not be rejected: {declaration}"
        );
    }
}

fn collect_production_rust_sources(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).expect("read boundary source directory") {
        let entry = entry.expect("read boundary source entry");
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == OsStr::new("tests"))
            {
                continue;
            }
            collect_production_rust_sources(&path, sources);
        } else if path.extension().is_some_and(|ext| ext == OsStr::new("rs"))
            && !is_standalone_test_file(&path)
        {
            sources.push(path);
        }
    }
}

fn is_standalone_test_file(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name == "tests.rs" || name.ends_with("_tests.rs"))
}

fn production_text(source: &str) -> &str {
    source.split("\n#[cfg(test)]").next().unwrap_or(source)
}

fn declares_wgpu_dependency(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return false;
    }

    line.starts_with("[dependencies.wgpu]")
        || line.starts_with("[dev-dependencies.wgpu]")
        || (line.starts_with('[') && line.ends_with(".wgpu]"))
        || line.split_once('=').is_some_and(|(key, value)| {
            let key = key.trim();
            key == "wgpu"
                || key.starts_with("wgpu.")
                || value.contains("package = \"wgpu\"")
                || value.contains("package=\"wgpu\"")
        })
}

fn first_line_containing(text: &str, needle: &str) -> Option<usize> {
    text.lines()
        .enumerate()
        .find_map(|(line_index, line)| line.contains(needle).then_some(line_index + 1))
}

fn first_line_with_upper_layer_term(text: &str, term: &str) -> Option<usize> {
    text.lines().enumerate().find_map(|(line_index, line)| {
        contains_upper_layer_term(line, term).then_some(line_index + 1)
    })
}

fn contains_upper_layer_term(text: &str, term: &str) -> bool {
    if term != "Ui" {
        return text.contains(term);
    }

    text.match_indices(term).any(|(index, _)| {
        text[index + term.len()..]
            .chars()
            .next()
            .is_none_or(|next| !next.is_ascii_lowercase())
    })
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("zr_rhi should live under zircon_runtime/crates")
        .to_path_buf()
}

fn relative_to_repo(path: &Path) -> PathBuf {
    path.strip_prefix(repo_root()).unwrap_or(path).to_path_buf()
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative))
        .unwrap_or_else(|error| panic!("failed to read repository path `{relative}`: {error}"))
}

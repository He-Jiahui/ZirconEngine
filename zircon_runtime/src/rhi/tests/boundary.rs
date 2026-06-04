use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const FORBIDDEN_WGPU_SOURCE_NEEDLES: &[&str] =
    &["wgpu::", "use wgpu", "pub use wgpu", "extern crate wgpu"];

const FORBIDDEN_RHI_SEMANTIC_TERMS: &[&str] = &[
    "Mesh", "Material", "Light", "Scene", "Camera", "Ui", "Sprite",
];

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
        "zircon_runtime/src/rhi/capabilities.rs",
        "zircon_runtime/src/rhi/descriptors.rs",
        "zircon_runtime/src/rhi/descriptors/pipeline.rs",
        "zircon_runtime/src/rhi/device.rs",
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
    let line = line.trim_start();
    line.starts_with("wgpu")
        || line.starts_with("[dependencies.wgpu]")
        || line.starts_with("[dev-dependencies.wgpu]")
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
        .expect("runtime crate should live under the workspace root")
        .to_path_buf()
}

fn relative_to_repo(path: &Path) -> PathBuf {
    path.strip_prefix(repo_root()).unwrap_or(path).to_path_buf()
}

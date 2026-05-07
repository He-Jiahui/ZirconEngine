use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const ALLOWED_DEPENDENCIES: &[&str] = &[
    "glam",
    "serde",
    "serde_json",
    "thiserror",
    "toml",
    "unicode-segmentation",
    "uuid",
];

const FORBIDDEN_SOURCE_NEEDLES: &[&str] = &[
    "#[path",
    "include_str!(",
    "include_bytes!(",
    "zircon_runtime/src",
    "zircon_editor/src",
    "zircon_runtime::",
    "zircon_editor::",
    "wgpu::",
    "winit::",
    "slint::",
    "libloading::",
    "tokio::",
    "std::fs",
    "std::net",
    "std::process",
    "std::thread",
    "std::sync",
];

#[test]
fn manifest_dependencies_stay_contract_only() {
    let manifest_path = manifest_dir().join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read interface manifest");
    let manifest: toml::Value = toml::from_str(&manifest).expect("parse interface manifest");
    let dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .expect("interface manifest dependencies table");
    let allowed: BTreeSet<_> = ALLOWED_DEPENDENCIES.iter().copied().collect();
    let actual: BTreeSet<_> = dependencies.keys().map(String::as_str).collect();
    let unexpected: Vec<_> = actual.difference(&allowed).copied().collect();

    assert!(
        unexpected.is_empty(),
        "zircon_runtime_interface may only depend on contract/serialization crates; unexpected dependencies: {unexpected:?}"
    );
    for table in ["build-dependencies", "dev-dependencies"] {
        assert!(
            manifest.get(table).is_none(),
            "zircon_runtime_interface must not grow a `{table}` table without an explicit boundary review"
        );
    }
}

#[test]
fn production_source_does_not_include_or_import_implementation_crates() {
    let sources = production_rust_sources();
    let mut violations = Vec::new();

    for source in sources {
        let text = std::fs::read_to_string(&source).expect("read interface source");
        for needle in FORBIDDEN_SOURCE_NEEDLES {
            if text.contains(needle) {
                violations.push(format!(
                    "{} contains forbidden boundary marker `{needle}`",
                    relative_to_manifest(&source).display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "zircon_runtime_interface source must stay ABI/DTO/serialization-only:\n{}",
        violations.join("\n")
    );
}

fn production_rust_sources() -> Vec<PathBuf> {
    let source_root = manifest_dir().join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);
    sources
}

fn collect_rust_sources(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).expect("read interface source directory") {
        let entry = entry.expect("read interface source entry");
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == OsStr::new("tests"))
            {
                continue;
            }
            collect_rust_sources(&path, sources);
        } else if path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("rs"))
        {
            sources.push(path);
        }
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn relative_to_manifest(path: &Path) -> PathBuf {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_path_buf()
}

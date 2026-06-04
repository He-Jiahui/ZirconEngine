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
    concat!("sli", "nt::"),
    "libloading::",
    "tokio::",
    "std::fs",
    "std::net",
    "std::process",
    "std::thread",
    "std::sync",
];

const EXPECTED_RUNTIME_API_MODULES: &[&str] = &[
    "api_table",
    "constants",
    "events",
    "host_requests",
    "requests",
    "viewport",
];
const RUNTIME_API_FACADE_LINE_BUDGET: usize = 20;
const RUNTIME_API_CHILD_LINE_BUDGET: usize = 700;

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

#[test]
fn runtime_api_surface_stays_folder_backed_by_abi_owner() {
    let root_path = manifest_dir().join("src").join("runtime_api.rs");
    let root_text = std::fs::read_to_string(&root_path).expect("read runtime_api facade");
    let facade_lines = root_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    assert!(
        facade_lines <= RUNTIME_API_FACADE_LINE_BUDGET,
        "runtime_api.rs must stay a small facade over owner modules; found {facade_lines} non-empty lines"
    );
    for forbidden in [
        "#[repr(",
        "pub struct ",
        "pub enum ",
        "pub const ",
        "pub type ",
    ] {
        assert!(
            !root_text.contains(forbidden),
            "runtime_api.rs must not own ABI declarations directly; found `{forbidden}`"
        );
    }

    let module_root = manifest_dir().join("src").join("runtime_api");
    let expected_files: BTreeSet<_> = EXPECTED_RUNTIME_API_MODULES
        .iter()
        .map(|module| format!("{module}.rs"))
        .collect();
    let actual_files: BTreeSet<_> = std::fs::read_dir(&module_root)
        .expect("read runtime_api module directory")
        .map(|entry| {
            entry
                .expect("read runtime_api module entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|file_name| file_name.ends_with(".rs"))
        .collect();

    assert_eq!(
        actual_files, expected_files,
        "runtime_api module owners changed; update the boundary review when adding/removing ABI owner files"
    );

    for module in EXPECTED_RUNTIME_API_MODULES {
        assert!(
            root_text.contains(&format!("mod {module};")),
            "runtime_api.rs must declare `{module}` as an owner module"
        );
        assert!(
            root_text.contains(&format!("pub use {module}::*;")),
            "runtime_api.rs must re-export `{module}` through runtime_api::*"
        );

        let module_path = module_root.join(format!("{module}.rs"));
        let module_text = std::fs::read_to_string(&module_path).expect("read runtime_api owner");
        let module_lines = module_text.lines().count();
        assert!(
            module_lines <= RUNTIME_API_CHILD_LINE_BUDGET,
            "{} must be split before it becomes another support hot spot; found {module_lines} lines",
            relative_to_manifest(&module_path).display()
        );
    }
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

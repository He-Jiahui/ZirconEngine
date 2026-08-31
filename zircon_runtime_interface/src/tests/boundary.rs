use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const ALLOWED_DEPENDENCIES: &[&str] = &[
    "bincode",
    "glam",
    "semver",
    "serde",
    "serde_json",
    "thiserror",
    "toml",
    "unicode-segmentation",
    "uuid",
];

const ALLOWED_DEV_DEPENDENCIES: &[&str] = &[];

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

const CANONICAL_TEXT_SPOOL_OS_NEEDLES: &[&str] = &["std::fs", "std::process", "std::sync"];
const CANONICAL_TEXT_SPOOL_PATH: &str = "src/serialization/text/canonical_spool.rs";

const EXPECTED_RUNTIME_API_DOMAINS: &[&str] = &["abi", "constants", "frame", "host", "session"];
const EXPECTED_RUNTIME_API_OWNER_PATHS: &[&str] = &[
    "abi/api_shape.rs",
    "abi/api_table.rs",
    "abi/host_api_shape.rs",
    "constants.rs",
    "frame/frame_demand.rs",
    "frame/frame_shape.rs",
    "frame/highlight_set.rs",
    "host/host_requests.rs",
    "session/events.rs",
    "session/operation.rs",
    "session/plugin_event_mirror.rs",
    "session/requests.rs",
    "session/session.rs",
    "session/session_identity.rs",
    "session/viewport.rs",
];
const RUNTIME_API_FACADE_LINE_BUDGET: usize = 220;
const RUNTIME_API_FACADE_REEXPORT_BUDGET: usize = 6;
const RUNTIME_API_CHILD_LINE_BUDGET: usize = 700;
const RUNTIME_API_DOMAIN_FACADE_PATHS: &[&str] = &[
    "abi/mod.rs",
    "frame/mod.rs",
    "host/mod.rs",
    "session/mod.rs",
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
    assert!(
        manifest.get("build-dependencies").is_none(),
        "zircon_runtime_interface must not grow build dependencies without an explicit boundary review"
    );
    let allowed_dev: BTreeSet<_> = ALLOWED_DEV_DEPENDENCIES.iter().copied().collect();
    let actual_dev: BTreeSet<_> = manifest
        .get("dev-dependencies")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|dependencies| dependencies.keys().map(String::as_str))
        .collect();
    let unexpected_dev: Vec<_> = actual_dev.difference(&allowed_dev).copied().collect();
    assert!(
        unexpected_dev.is_empty(),
        "zircon_runtime_interface dev dependencies require explicit boundary review; unexpected: {unexpected_dev:?}"
    );
}

#[test]
fn production_source_does_not_include_or_import_implementation_crates() {
    let sources = production_rust_sources();
    let mut violations = Vec::new();

    for source in sources {
        let text = std::fs::read_to_string(&source).expect("read interface source");
        for needle in FORBIDDEN_SOURCE_NEEDLES {
            if template_pack_embedding_is_reviewed(&source, needle)
                || canonical_text_spool_os_access_is_reviewed(&source, needle)
            {
                continue;
            }
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

fn template_pack_embedding_is_reviewed(source: &Path, needle: &str) -> bool {
    needle == "include_bytes!("
        && relative_to_manifest(source) == Path::new("src/project/template_pack/embedded.rs")
}

fn canonical_text_spool_os_access_is_reviewed(source: &Path, needle: &str) -> bool {
    relative_to_manifest(source) == Path::new(CANONICAL_TEXT_SPOOL_PATH)
        && CANONICAL_TEXT_SPOOL_OS_NEEDLES.contains(&needle)
}

#[test]
fn canonical_text_spool_os_exception_stays_exact() {
    let source = manifest_dir().join(CANONICAL_TEXT_SPOOL_PATH);
    let text = std::fs::read_to_string(source).expect("read canonical text spool");
    let actual: Vec<_> = FORBIDDEN_SOURCE_NEEDLES
        .iter()
        .copied()
        .filter(|needle| text.contains(needle))
        .collect();

    assert_eq!(
        actual, CANONICAL_TEXT_SPOOL_OS_NEEDLES,
        "canonical text sorting may use only its reviewed disk-spool OS primitives"
    );
}

#[test]
fn runtime_api_surface_stays_folder_backed_by_abi_owner() {
    let legacy_root_path = manifest_dir().join("src").join("runtime_api.rs");
    assert!(
        !legacy_root_path.exists(),
        "runtime_api.rs was superseded by runtime_api/mod.rs and must not be restored"
    );

    let root_path = manifest_dir()
        .join("src")
        .join("runtime_api")
        .join("mod.rs");
    let root_text = std::fs::read_to_string(&root_path).expect("read runtime_api facade");
    let facade_lines = root_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    assert!(
        facade_lines <= RUNTIME_API_FACADE_LINE_BUDGET,
        "runtime_api/mod.rs must stay a small facade over owner modules; found {facade_lines} non-empty lines"
    );
    let facade_reexport_statements = root_text
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use "))
        .count();
    assert!(
        facade_reexport_statements <= RUNTIME_API_FACADE_REEXPORT_BUDGET,
        "runtime_api/mod.rs must group its curated re-exports by domain; found {facade_reexport_statements} statements"
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
            "runtime_api/mod.rs must not own ABI declarations directly; found `{forbidden}`"
        );
    }

    let module_root = manifest_dir().join("src").join("runtime_api");
    let expected_files: BTreeSet<_> = EXPECTED_RUNTIME_API_OWNER_PATHS
        .iter()
        .map(|path| (*path).to_string())
        .collect();
    let actual_files = runtime_api_owner_sources(&module_root);

    assert_eq!(
        actual_files, expected_files,
        "runtime_api owner paths changed; update the boundary review when adding/removing ABI owner files"
    );

    for domain in EXPECTED_RUNTIME_API_DOMAINS {
        assert!(
            root_text.contains(&format!("mod {domain};")),
            "runtime_api/mod.rs must declare `{domain}` as an ABI owner domain"
        );
        assert!(
            root_text.contains(&format!("pub use {domain}::{{")),
            "runtime_api/mod.rs must explicitly re-export `{domain}` through runtime_api::*"
        );
    }

    assert!(
        !root_text
            .lines()
            .any(|line| line.trim_start().starts_with("pub use ") && line.contains("::*;")),
        "runtime_api/mod.rs must not use glob re-exports"
    );

    for owner_path in EXPECTED_RUNTIME_API_OWNER_PATHS {
        let module_path = module_root.join(owner_path);
        let module_text = std::fs::read_to_string(&module_path).expect("read runtime_api owner");
        let module_lines = module_text.lines().count();
        assert!(
            module_lines <= RUNTIME_API_CHILD_LINE_BUDGET,
            "{} must be split before it becomes another support hot spot; found {module_lines} lines",
            relative_to_manifest(&module_path).display()
        );
    }

    for facade_path in RUNTIME_API_DOMAIN_FACADE_PATHS {
        let facade_path = module_root.join(facade_path);
        let facade_text =
            std::fs::read_to_string(&facade_path).expect("read runtime_api domain facade");
        assert!(
            !facade_text
                .lines()
                .any(|line| { line.trim_start().starts_with("pub use ") && line.contains("::*;") }),
            "{} must explicitly re-export its ABI owner surface",
            relative_to_manifest(&facade_path).display()
        );
    }
}

fn runtime_api_owner_sources(module_root: &Path) -> BTreeSet<String> {
    let mut owner_paths = BTreeSet::new();
    collect_runtime_api_owner_sources(module_root, module_root, &mut owner_paths);
    owner_paths
}

fn collect_runtime_api_owner_sources(
    module_root: &Path,
    path: &Path,
    owner_paths: &mut BTreeSet<String>,
) {
    for entry in std::fs::read_dir(path).expect("read runtime_api owner directory") {
        let entry = entry.expect("read runtime_api owner entry");
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == OsStr::new("tests"))
            {
                continue;
            }
            collect_runtime_api_owner_sources(module_root, &path, owner_paths);
        } else if path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("rs"))
            && path
                .file_name()
                .is_some_and(|name| name != OsStr::new("mod.rs"))
            && !path
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy().ends_with("_tests"))
        {
            owner_paths.insert(
                path.strip_prefix(module_root)
                    .expect("runtime_api owner stays below its module root")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
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

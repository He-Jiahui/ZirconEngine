use std::fs;
use std::path::{Path, PathBuf};

use woc_contract_codegen::{validate_reference_root, ReferenceInventoryError, REFERENCE_COMMIT};

const REFERENCE_FILES: [&str; 7] = [
    "asset_catalog.json",
    "command_catalog.json",
    "parity_scenarios.json",
    "source_manifest.json",
    "test_catalog.json",
    "ui_flow_catalog.json",
    "world_api_catalog.json",
];

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn pinned_reference_catalogs_cover_the_complete_audited_surface() {
    let summary = validate_reference_root(project_root().join("reference/current-head"))
        .expect("reference catalogs must match the pinned target");

    assert_eq!(summary.commands, 165);
    assert_eq!(summary.world_members, 248);
    assert_eq!(summary.test_cases, 14_716);
    assert_eq!(summary.test_files, 1_331);
    assert_eq!(summary.test_generators, 67);
    assert_eq!(summary.parity_scenarios, 54);
    assert_eq!(summary.glbs, 949);
    assert_eq!(summary.ui_flow_sources, 650);
}

#[test]
fn reference_catalog_rows_have_a_reconstruction_ownership_class() {
    const CLASSIFIED_COLLECTIONS: [(&str, &[&str]); 6] = [
        ("command_catalog.json", &["entries"]),
        ("world_api_catalog.json", &["entries", "facets"]),
        ("test_catalog.json", &["entries", "files", "generators"]),
        ("parity_scenarios.json", &["entries"]),
        ("asset_catalog.json", &["entries"]),
        ("ui_flow_catalog.json", &["entries"]),
    ];
    let allowed = ["simulation", "client", "service", "presentation"];

    for (catalog, collections) in CLASSIFIED_COLLECTIONS {
        let path = project_root().join("reference/current-head").join(catalog);
        let source = fs::read_to_string(path).expect("reference catalog must be readable");
        let document: serde_json::Value =
            serde_json::from_str(&source).expect("reference catalog must be JSON");
        for collection in collections {
            let rows = document[*collection]
                .as_array()
                .expect("reference catalog must contain the required row collection");
            assert!(
                rows.iter().all(|row| row["ownership_class"]
                    .as_str()
                    .is_some_and(|class| allowed.contains(&class))),
                "every {catalog} {collection} row must declare a recognized reconstruction ownership class"
            );
        }
    }
}

#[test]
fn missing_catalog_is_rejected() {
    let reference = TemporaryReference::empty("missing");
    assert!(matches!(
        validate_reference_root(&reference.root),
        Err(ReferenceInventoryError::Read { .. })
    ));
}

#[test]
fn renamed_row_is_rejected_even_when_counts_are_unchanged() {
    let reference = TemporaryReference::copy("renamed");
    mutate_command_catalog(&reference.root, |entries| {
        entries[0]["name"] = serde_json::Value::String("renamed_command".to_string());
    });
    assert!(matches!(
        validate_reference_root(&reference.root),
        Err(ReferenceInventoryError::InvalidIdentity { .. })
    ));
}

#[test]
fn duplicate_row_is_rejected() {
    let reference = TemporaryReference::copy("duplicate");
    mutate_command_catalog(&reference.root, |entries| {
        entries.push(entries[0].clone());
    });
    assert!(validate_reference_root(&reference.root).is_err());
}

#[test]
fn count_drift_is_rejected() {
    let reference = TemporaryReference::copy("count-drift");
    mutate_command_catalog(&reference.root, |entries| {
        entries.pop().expect("command catalog must not be empty");
    });
    assert!(validate_reference_root(&reference.root).is_err());
}

#[test]
fn runtime_owned_files_do_not_depend_on_the_reference_checkout() {
    let root = project_root();
    let slash_reference = ["dev", "world-of-claudecraft"].join("/");
    let backslash_reference = ["dev", "world-of-claudecraft"].join("\\");
    for relative in ["zircon-project.toml", "scripts", "native"] {
        inspect_runtime_path(
            &root.join(relative),
            [&slash_reference, &backslash_reference],
        );
    }
}

fn inspect_runtime_path(path: &Path, forbidden_paths: [&str; 2]) {
    let metadata = fs::symlink_metadata(path).expect("runtime-owned path must exist");
    assert!(
        !metadata.file_type().is_symlink(),
        "{} is a symlink",
        path.display()
    );
    if metadata.is_dir() {
        for entry in fs::read_dir(path).expect("runtime-owned directory must be readable") {
            inspect_runtime_path(
                &entry.expect("directory entry must be readable").path(),
                forbidden_paths,
            );
        }
        return;
    }

    let is_text = matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "toml" | "zr" | "zrp" | "json")
    );
    if is_text {
        let source = fs::read_to_string(path).expect("runtime-owned text must be UTF-8");
        assert!(
            forbidden_paths
                .iter()
                .all(|forbidden| !source.contains(*forbidden)),
            "{} contains a runtime dependency on the reference checkout",
            path.display()
        );
    }
}

#[test]
fn reference_commit_is_full_and_pinned() {
    assert_eq!(REFERENCE_COMMIT.len(), 40);
    assert_eq!(REFERENCE_COMMIT, "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a");
}

struct TemporaryReference {
    root: PathBuf,
}

impl TemporaryReference {
    fn empty(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time must follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-woc-reference-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary reference directory must be created");
        Self { root }
    }

    fn copy(label: &str) -> Self {
        let reference = Self::empty(label);
        let source = project_root().join("reference/current-head");
        for name in REFERENCE_FILES {
            fs::copy(source.join(name), reference.root.join(name))
                .expect("reference catalog must copy into the mutation fixture");
        }
        reference
    }
}

impl Drop for TemporaryReference {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("temporary reference directory must be removed");
    }
}

fn mutate_command_catalog(root: &Path, mutate: impl FnOnce(&mut Vec<serde_json::Value>)) {
    let path = root.join("command_catalog.json");
    let source = fs::read_to_string(&path).expect("command catalog fixture must be readable");
    let mut document: serde_json::Value =
        serde_json::from_str(&source).expect("command catalog fixture must be JSON");
    let entries = document["entries"]
        .as_array_mut()
        .expect("command catalog must contain an entries array");
    mutate(entries);
    let mut output =
        serde_json::to_string_pretty(&document).expect("mutated command catalog must serialize");
    output.push('\n');
    fs::write(path, output).expect("mutated command catalog must be written");
}

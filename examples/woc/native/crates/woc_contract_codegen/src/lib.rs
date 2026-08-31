#![forbid(unsafe_code)]

mod contract;
mod projection;

pub use contract::{load_contract_manifest, ContractError, ContractManifest};
pub use projection::{
    generate_projections, verify_projection, write_projection, GeneratedProjections,
    ProjectionError, ProjectionWriteOutcome,
};

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const REFERENCE_COMMIT: &str = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
pub const EXPECTED_SOURCE_FILES: usize = 3_163;
pub const EXPECTED_SOURCE_CHARACTERS: usize = 56_451_702;
pub const EXPECTED_COMMANDS: usize = 165;
pub const EXPECTED_DISPATCH_ONLY_COMMANDS: usize = 9;
pub const EXPECTED_WORLD_MEMBERS: usize = 248;
pub const EXPECTED_WORLD_METHODS: usize = 181;
pub const EXPECTED_WORLD_DATA_MEMBERS: usize = 67;
pub const EXPECTED_WORLD_FACETS: usize = 28;
pub const EXPECTED_TEST_CASES: usize = 14_716;
pub const EXPECTED_TEST_FILES: usize = 1_331;
pub const EXPECTED_TEST_GENERATORS: usize = 67;
pub const EXPECTED_PARITY_SCENARIOS: usize = 54;
pub const EXPECTED_GLBS: usize = 949;
pub const EXPECTED_UI_FLOW_SOURCES: usize = 650;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceInventorySummary {
    pub commands: usize,
    pub dispatch_only_commands: usize,
    pub world_members: usize,
    pub world_methods: usize,
    pub world_data_members: usize,
    pub world_facets: usize,
    pub test_cases: usize,
    pub test_files: usize,
    pub test_generators: usize,
    pub parity_scenarios: usize,
    pub glbs: usize,
    pub ui_flow_sources: usize,
}

#[derive(Debug, Error)]
pub enum ReferenceInventoryError {
    #[error("failed to read reference file {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse reference file {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("reference catalog {catalog} uses commit {actual}, expected {expected}")]
    Commit {
        catalog: String,
        actual: String,
        expected: &'static str,
    },
    #[error("reference inventory mismatch for {field}: actual {actual}, expected {expected}")]
    Count {
        field: &'static str,
        actual: usize,
        expected: usize,
    },
    #[error("reference catalog {catalog} uses schema {actual}, expected 1")]
    Schema { catalog: String, actual: u32 },
    #[error("reference catalog {catalog} contains duplicate key {key}")]
    Duplicate { catalog: &'static str, key: String },
    #[error("reference catalog {catalog} must not be empty")]
    Empty { catalog: &'static str },
    #[error("test case {id} references undeclared test file {file}")]
    UndeclaredTestFile { id: String, file: String },
    #[error("source identity {field} is invalid: {value}")]
    InvalidIdentity { field: &'static str, value: String },
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct Catalog<T> {
    schema_version: u32,
    source_commit: String,
    entries: Vec<T>,
    #[serde(default)]
    facets: Vec<WorldFacetEntry>,
    #[serde(default)]
    files: Vec<TestFileEntry>,
    #[serde(default)]
    generators: Vec<T>,
}

#[derive(Deserialize)]
struct SourceManifest {
    schema_version: u32,
    source_commit: String,
    source_repository: String,
    identities: SourceIdentities,
    catalog_sha256: std::collections::BTreeMap<String, String>,
    audited_totals: SourceTotals,
}

#[derive(Deserialize)]
struct SourceIdentities {
    package_manifest: FileIdentity,
    parity_sources: TreeIdentity,
    golden_directory: TreeIdentity,
}

#[derive(Deserialize)]
struct FileIdentity {
    path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Deserialize)]
struct TreeIdentity {
    file_count: usize,
    sha256: String,
    files: Vec<FileIdentity>,
}

#[derive(Deserialize)]
struct SourceTotals {
    source_files: usize,
    source_characters: usize,
    commands: usize,
    dispatch_only_commands: usize,
    world_members: usize,
    world_methods: usize,
    world_data_members: usize,
    world_facets: usize,
    test_cases: usize,
    test_files: usize,
    test_case_generators: usize,
    parity_scenarios: usize,
    glbs: usize,
    ui_flow_sources: usize,
}

#[derive(Deserialize)]
struct CommandEntry {
    name: String,
    kind: String,
    ownership_class: OwnershipClass,
}

#[derive(Deserialize)]
struct WorldMemberEntry {
    facet: String,
    name: String,
    kind: String,
    ownership_class: OwnershipClass,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum OwnershipClass {
    Simulation,
    Client,
    Service,
    Presentation,
}

#[derive(Deserialize)]
struct WorldFacetEntry {
    name: String,
    ownership_class: OwnershipClass,
}

#[derive(Deserialize)]
struct TestEntry {
    id: String,
    file: String,
    ownership_class: OwnershipClass,
}

#[derive(Deserialize)]
struct TestFileEntry {
    path: String,
    ownership_class: OwnershipClass,
}

#[derive(Deserialize)]
struct NamedEntry {
    name: String,
    ownership_class: OwnershipClass,
}

#[derive(Deserialize)]
struct AssetEntry {
    path: String,
    ownership_class: OwnershipClass,
}

#[derive(Deserialize)]
struct UiFlowEntry {
    id: String,
    ownership_class: OwnershipClass,
}

pub fn validate_reference_root(
    reference_root: impl AsRef<Path>,
) -> Result<ReferenceInventorySummary, ReferenceInventoryError> {
    let root = reference_root.as_ref();
    let source: SourceManifest = read_json(root, "source_manifest.json")?;
    validate_source_manifest(&source)?;
    validate_catalog_identities(&source.catalog_sha256)?;
    let commands: Catalog<CommandEntry> =
        read_catalog(root, "command_catalog.json", &source.catalog_sha256)?;
    let world: Catalog<WorldMemberEntry> =
        read_catalog(root, "world_api_catalog.json", &source.catalog_sha256)?;
    let tests: Catalog<TestEntry> =
        read_catalog(root, "test_catalog.json", &source.catalog_sha256)?;
    let parity: Catalog<NamedEntry> =
        read_catalog(root, "parity_scenarios.json", &source.catalog_sha256)?;
    let assets: Catalog<AssetEntry> =
        read_catalog(root, "asset_catalog.json", &source.catalog_sha256)?;
    let ui: Catalog<UiFlowEntry> =
        read_catalog(root, "ui_flow_catalog.json", &source.catalog_sha256)?;

    validate_unique(
        "command_catalog.json",
        commands.entries.iter().map(|entry| &entry.name),
    )?;
    validate_ownership_classes(
        "command_catalog.json",
        commands.entries.iter().map(|entry| entry.ownership_class),
    )?;
    validate_unique(
        "world_api_catalog.json",
        world.entries.iter().map(|entry| &entry.name),
    )?;
    validate_unique(
        "world_api_catalog.json facets",
        world.facets.iter().map(|facet| &facet.name),
    )?;
    validate_count(
        "world_api_catalog facets",
        world.facets.len(),
        EXPECTED_WORLD_FACETS,
    )?;
    let facet_ownership = world
        .facets
        .iter()
        .map(|facet| (facet.name.as_str(), facet.ownership_class))
        .collect::<std::collections::BTreeMap<_, _>>();
    for entry in &world.entries {
        let Some(ownership_class) = facet_ownership.get(entry.facet.as_str()) else {
            return Err(ReferenceInventoryError::InvalidIdentity {
                field: "world_api_catalog facet",
                value: entry.facet.clone(),
            });
        };
        if entry.ownership_class != *ownership_class {
            return Err(ReferenceInventoryError::InvalidIdentity {
                field: "world_api_catalog ownership_class",
                value: format!("{}:{}", entry.facet, entry.name),
            });
        }
    }
    validate_unique(
        "test_catalog.json",
        tests.entries.iter().map(|entry| &entry.id),
    )?;
    validate_unique(
        "test_catalog.json generators",
        tests.generators.iter().map(|entry| &entry.id),
    )?;
    validate_unique(
        "test_catalog.json files",
        tests.files.iter().map(|entry| &entry.path),
    )?;
    validate_ownership_classes(
        "test_catalog.json",
        tests
            .entries
            .iter()
            .map(|entry| entry.ownership_class)
            .chain(tests.generators.iter().map(|entry| entry.ownership_class))
            .chain(tests.files.iter().map(|entry| entry.ownership_class)),
    )?;
    let test_files = tests
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for entry in &tests.entries {
        if !test_files.contains(entry.file.as_str()) {
            return Err(ReferenceInventoryError::UndeclaredTestFile {
                id: entry.id.clone(),
                file: entry.file.clone(),
            });
        }
    }
    for entry in &tests.generators {
        if !test_files.contains(entry.file.as_str()) {
            return Err(ReferenceInventoryError::UndeclaredTestFile {
                id: entry.id.clone(),
                file: entry.file.clone(),
            });
        }
    }
    validate_unique(
        "parity_scenarios.json",
        parity.entries.iter().map(|entry| &entry.name),
    )?;
    validate_ownership_classes(
        "parity_scenarios.json",
        parity.entries.iter().map(|entry| entry.ownership_class),
    )?;
    validate_unique(
        "asset_catalog.json",
        assets.entries.iter().map(|entry| &entry.path),
    )?;
    validate_ownership_classes(
        "asset_catalog.json",
        assets.entries.iter().map(|entry| entry.ownership_class),
    )?;
    validate_unique(
        "ui_flow_catalog.json",
        ui.entries.iter().map(|entry| &entry.id),
    )?;
    validate_ownership_classes(
        "ui_flow_catalog.json",
        ui.entries.iter().map(|entry| entry.ownership_class),
    )?;
    if ui.entries.is_empty() {
        return Err(ReferenceInventoryError::Empty {
            catalog: "ui_flow_catalog.json",
        });
    }

    let summary = ReferenceInventorySummary {
        commands: commands.entries.len(),
        dispatch_only_commands: commands
            .entries
            .iter()
            .filter(|entry| entry.kind == "dispatch_only")
            .count(),
        world_members: world.entries.len(),
        world_methods: world
            .entries
            .iter()
            .filter(|entry| entry.kind == "method")
            .count(),
        world_data_members: world
            .entries
            .iter()
            .filter(|entry| entry.kind == "data")
            .count(),
        world_facets: world
            .entries
            .iter()
            .map(|entry| entry.facet.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        test_cases: tests.entries.len(),
        test_files: tests.files.len(),
        test_generators: tests.generators.len(),
        parity_scenarios: parity.entries.len(),
        glbs: assets.entries.len(),
        ui_flow_sources: ui.entries.len(),
    };

    validate_count("commands", summary.commands, EXPECTED_COMMANDS)?;
    validate_count(
        "dispatch_only_commands",
        summary.dispatch_only_commands,
        EXPECTED_DISPATCH_ONLY_COMMANDS,
    )?;
    validate_count(
        "world_members",
        summary.world_members,
        EXPECTED_WORLD_MEMBERS,
    )?;
    validate_count(
        "world_methods",
        summary.world_methods,
        EXPECTED_WORLD_METHODS,
    )?;
    validate_count(
        "world_data_members",
        summary.world_data_members,
        EXPECTED_WORLD_DATA_MEMBERS,
    )?;
    validate_count("world_facets", summary.world_facets, EXPECTED_WORLD_FACETS)?;
    validate_count("test_cases", summary.test_cases, EXPECTED_TEST_CASES)?;
    validate_count("test_files", summary.test_files, EXPECTED_TEST_FILES)?;
    validate_count(
        "test_generators",
        summary.test_generators,
        EXPECTED_TEST_GENERATORS,
    )?;
    validate_count(
        "parity_scenarios",
        summary.parity_scenarios,
        EXPECTED_PARITY_SCENARIOS,
    )?;
    validate_count("glbs", summary.glbs, EXPECTED_GLBS)?;
    validate_count(
        "ui_flow_sources",
        summary.ui_flow_sources,
        EXPECTED_UI_FLOW_SOURCES,
    )?;

    validate_count(
        "manifest commands",
        source.audited_totals.commands,
        summary.commands,
    )?;
    validate_count(
        "manifest world_members",
        source.audited_totals.world_members,
        summary.world_members,
    )?;
    validate_count(
        "manifest test_cases",
        source.audited_totals.test_cases,
        summary.test_cases,
    )?;
    validate_count(
        "manifest parity_scenarios",
        source.audited_totals.parity_scenarios,
        summary.parity_scenarios,
    )?;
    validate_count("manifest glbs", source.audited_totals.glbs, summary.glbs)?;

    Ok(summary)
}

fn read_catalog<T: DeserializeOwned>(
    root: &Path,
    name: &str,
    identities: &std::collections::BTreeMap<String, String>,
) -> Result<Catalog<T>, ReferenceInventoryError> {
    let path = root.join(name);
    let source = fs::read_to_string(&path).map_err(|source| ReferenceInventoryError::Read {
        path: path.clone(),
        source,
    })?;
    let expected =
        identities
            .get(name)
            .ok_or_else(|| ReferenceInventoryError::InvalidIdentity {
                field: "catalog_sha256",
                value: format!("missing {name}"),
            })?;
    let digest = Sha256::digest(source.as_bytes());
    let mut actual = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(actual, "{byte:02x}").expect("writing to a String cannot fail");
    }
    if actual != *expected {
        return Err(ReferenceInventoryError::InvalidIdentity {
            field: "catalog_sha256",
            value: format!("{name}: actual {actual}, expected {expected}"),
        });
    }
    let catalog: Catalog<T> = serde_json::from_str(&source)
        .map_err(|source| ReferenceInventoryError::Parse { path, source })?;
    if catalog.source_commit != REFERENCE_COMMIT {
        return Err(ReferenceInventoryError::Commit {
            catalog: name.to_string(),
            actual: catalog.source_commit,
            expected: REFERENCE_COMMIT,
        });
    }
    if catalog.schema_version != 1 {
        return Err(ReferenceInventoryError::Schema {
            catalog: name.to_string(),
            actual: catalog.schema_version,
        });
    }
    Ok(catalog)
}

fn read_json<T: DeserializeOwned>(root: &Path, name: &str) -> Result<T, ReferenceInventoryError> {
    let path = root.join(name);
    let source = fs::read_to_string(&path).map_err(|source| ReferenceInventoryError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&source).map_err(|source| ReferenceInventoryError::Parse { path, source })
}

fn validate_source_manifest(source: &SourceManifest) -> Result<(), ReferenceInventoryError> {
    if source.schema_version != 1 {
        return Err(ReferenceInventoryError::Schema {
            catalog: "source_manifest.json".to_string(),
            actual: source.schema_version,
        });
    }
    if source.source_commit != REFERENCE_COMMIT {
        return Err(ReferenceInventoryError::Commit {
            catalog: "source_manifest.json".to_string(),
            actual: source.source_commit.clone(),
            expected: REFERENCE_COMMIT,
        });
    }
    validate_identity("source_repository", &source.source_repository, |value| {
        value == "world-of-claudecraft"
    })?;
    validate_file_identity("package_manifest", &source.identities.package_manifest)?;
    validate_tree_identity("parity_sources", &source.identities.parity_sources, 8)?;
    validate_tree_identity("golden_directory", &source.identities.golden_directory, 54)?;
    validate_count(
        "source_files",
        source.audited_totals.source_files,
        EXPECTED_SOURCE_FILES,
    )?;
    validate_count(
        "source_characters",
        source.audited_totals.source_characters,
        EXPECTED_SOURCE_CHARACTERS,
    )?;
    validate_count(
        "manifest dispatch_only_commands",
        source.audited_totals.dispatch_only_commands,
        EXPECTED_DISPATCH_ONLY_COMMANDS,
    )?;
    validate_count(
        "manifest world_methods",
        source.audited_totals.world_methods,
        EXPECTED_WORLD_METHODS,
    )?;
    validate_count(
        "manifest world_data_members",
        source.audited_totals.world_data_members,
        EXPECTED_WORLD_DATA_MEMBERS,
    )?;
    validate_count(
        "manifest world_facets",
        source.audited_totals.world_facets,
        EXPECTED_WORLD_FACETS,
    )?;
    validate_count(
        "manifest test_files",
        source.audited_totals.test_files,
        EXPECTED_TEST_FILES,
    )?;
    validate_count(
        "manifest test_case_generators",
        source.audited_totals.test_case_generators,
        EXPECTED_TEST_GENERATORS,
    )?;
    validate_count(
        "manifest ui_flow_sources",
        source.audited_totals.ui_flow_sources,
        EXPECTED_UI_FLOW_SOURCES,
    )?;
    Ok(())
}

fn validate_catalog_identities(
    identities: &std::collections::BTreeMap<String, String>,
) -> Result<(), ReferenceInventoryError> {
    const CATALOGS: [&str; 6] = [
        "asset_catalog.json",
        "command_catalog.json",
        "parity_scenarios.json",
        "test_catalog.json",
        "ui_flow_catalog.json",
        "world_api_catalog.json",
    ];
    validate_count("catalog identities", identities.len(), CATALOGS.len())?;
    for name in CATALOGS {
        let expected =
            identities
                .get(name)
                .ok_or_else(|| ReferenceInventoryError::InvalidIdentity {
                    field: "catalog_sha256",
                    value: format!("missing {name}"),
                })?;
        validate_identity("catalog_sha256", expected, is_sha256)?;
    }
    Ok(())
}

fn validate_file_identity(
    field: &'static str,
    identity: &FileIdentity,
) -> Result<(), ReferenceInventoryError> {
    validate_identity(field, &identity.path, |value| !value.is_empty())?;
    if identity.bytes == 0 {
        return Err(ReferenceInventoryError::InvalidIdentity {
            field,
            value: "zero bytes".to_string(),
        });
    }
    validate_identity(field, &identity.sha256, is_sha256)
}

fn validate_tree_identity(
    field: &'static str,
    identity: &TreeIdentity,
    expected_files: usize,
) -> Result<(), ReferenceInventoryError> {
    validate_count(field, identity.file_count, expected_files)?;
    validate_count(field, identity.files.len(), expected_files)?;
    validate_identity(field, &identity.sha256, is_sha256)?;
    for file in &identity.files {
        validate_file_identity(field, file)?;
    }
    Ok(())
}

fn validate_identity(
    field: &'static str,
    value: &str,
    predicate: impl FnOnce(&str) -> bool,
) -> Result<(), ReferenceInventoryError> {
    if !predicate(value) {
        return Err(ReferenceInventoryError::InvalidIdentity {
            field,
            value: value.to_string(),
        });
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_ownership_classes(
    catalog: &'static str,
    classes: impl IntoIterator<Item = OwnershipClass>,
) -> Result<(), ReferenceInventoryError> {
    let labels = classes
        .into_iter()
        .map(OwnershipClass::label)
        .collect::<std::collections::BTreeSet<_>>();
    if labels.is_empty() {
        return Err(ReferenceInventoryError::Empty { catalog });
    }
    Ok(())
}

impl OwnershipClass {
    fn label(self) -> &'static str {
        match self {
            Self::Simulation => "simulation",
            Self::Client => "client",
            Self::Service => "service",
            Self::Presentation => "presentation",
        }
    }
}

fn validate_unique<'a>(
    catalog: &'static str,
    keys: impl IntoIterator<Item = &'a String>,
) -> Result<(), ReferenceInventoryError> {
    let mut seen = std::collections::BTreeSet::new();
    for key in keys {
        if !seen.insert(key.as_str()) {
            return Err(ReferenceInventoryError::Duplicate {
                catalog,
                key: key.clone(),
            });
        }
    }
    Ok(())
}

fn validate_count(
    field: &'static str,
    actual: usize,
    expected: usize,
) -> Result<(), ReferenceInventoryError> {
    if actual != expected {
        return Err(ReferenceInventoryError::Count {
            field,
            actual,
            expected,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Catalog;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct EntryWithoutDefault {
        value: u32,
    }

    #[test]
    fn generic_catalog_does_not_require_default_entries() {
        let catalog: Catalog<EntryWithoutDefault> = serde_json::from_str(
            r#"{"schema_version":1,"source_commit":"fixture","entries":[{"value":7}]}"#,
        )
        .expect("catalog defaults must not impose T: Default");

        assert_eq!(catalog.entries[0].value, 7);
        assert!(catalog.files.is_empty());
        assert!(catalog.generators.is_empty());
    }
}

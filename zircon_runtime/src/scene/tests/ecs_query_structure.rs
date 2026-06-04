use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_QUERY_STATE_MODULES: &[&str] = &[
    "cached_direct",
    "helpers",
    "mod",
    "mutable",
    "read_only",
    "system_param",
];
const QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET: usize = 180;
const QUERY_STATE_OWNER_LINE_BUDGET: usize = 450;

#[test]
fn query_state_stays_folder_backed_by_query_owner() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let legacy_file = query_root.join("query_state.rs");
    assert!(
        !legacy_file.exists(),
        "QueryState must stay folder-backed; do not recreate {}",
        relative_to_manifest(&legacy_file).display()
    );

    let owner_root = query_root.join("query_state");
    let actual_modules: BTreeSet<_> = std::fs::read_dir(&owner_root)
        .expect("read query_state owner directory")
        .map(|entry| {
            entry
                .expect("read query_state owner entry")
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".rs")
                .to_owned()
        })
        .collect();
    let expected_modules: BTreeSet<_> = EXPECTED_QUERY_STATE_MODULES
        .iter()
        .map(|module| (*module).to_owned())
        .collect();
    assert_eq!(
        actual_modules, expected_modules,
        "QueryState owner modules changed; update the architecture review before adding/removing query-state owners"
    );

    let query_mod = std::fs::read_to_string(query_root.join("mod.rs")).expect("read query mod");
    assert!(
        query_mod.contains("mod query_state;"),
        "query/mod.rs must keep QueryState behind the query_state owner module"
    );

    let root_path = owner_root.join("mod.rs");
    let root_text = std::fs::read_to_string(&root_path).expect("read query_state root");
    let root_non_empty_lines = root_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    assert!(
        root_non_empty_lines <= QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET,
        "query_state/mod.rs must stay a small state/cache owner; found {root_non_empty_lines} non-empty lines"
    );
    for forbidden in [
        "D: CachedQueryData",
        "D: QueryData,",
        "D: QueryMutData",
        "impl<D, F> SystemParam",
    ] {
        assert!(
            !root_text.contains(forbidden),
            "query_state/mod.rs must not own `{forbidden}` impl families"
        );
    }

    for module in EXPECTED_QUERY_STATE_MODULES {
        let module_path = owner_root.join(format!("{module}.rs"));
        assert!(
            module_path.exists(),
            "missing QueryState owner module {}",
            relative_to_manifest(&module_path).display()
        );
        let module_lines = std::fs::read_to_string(&module_path)
            .expect("read QueryState owner module")
            .lines()
            .count();
        assert!(
            module_lines <= QUERY_STATE_OWNER_LINE_BUDGET,
            "{} must split again before it becomes another ECS query hot spot; found {module_lines} lines",
            relative_to_manifest(&module_path).display()
        );
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

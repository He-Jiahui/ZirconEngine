use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_QUERY_STATE_MODULES: &[&str] = &[
    "cache",
    "cached_direct",
    "many_item_array",
    "mod",
    "mutable",
    "read_only",
    "read_only_cached",
    "stats",
    "system_param",
];
const QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET: usize = 180;
const QUERY_STATE_OWNER_LINE_BUDGET: usize = 450;

mod archetype_access;
mod cache_rebuild;
mod cached_iterators;
mod combinations;
mod mutable_iterators;
mod query_state_layout;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", relative_to_manifest(path).display()))
}

fn relative_to_manifest(path: &Path) -> PathBuf {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_path_buf()
}

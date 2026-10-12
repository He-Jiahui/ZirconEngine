//! 按所有权拆分查询结构守卫：原型访问、缓存刷新、迭代器和 QueryState 文件布局分别受检。
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_QUERY_STATE_MODULES: &[&str] = &[
    "archetype_plan",
    "cache",
    "cached_direct",
    "many_item_array",
    "mod",
    "mutable",
    "read_only",
    "read_only_cached",
    "state",
    "stats",
    "system_param",
];
const QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET: usize = 32;
const QUERY_STATE_OWNER_LINE_BUDGET: usize = 450;

#[path = "ecs_query_structure/archetype_access.rs"]
mod archetype_access;
#[path = "ecs_query_structure/cache_rebuild.rs"]
mod cache_rebuild;
#[path = "ecs_query_structure/cached_iterators.rs"]
mod cached_iterators;
#[path = "ecs_query_structure/combinations.rs"]
mod combinations;
#[path = "ecs_query_structure/mutable_iterators.rs"]
mod mutable_iterators;
#[path = "ecs_query_structure/query_state_layout.rs"]
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

use std::fs;
use std::path::{Path, PathBuf};

use super::super::support::{
    assert_contains_all, read_repo_text, read_runtime_15_naming_date_map,
    read_runtime_15_naming_status_map, read_runtime_15_naming_status_rows,
};

#[path = "hub/raw_text_policy.rs"]
mod raw_text_policy;

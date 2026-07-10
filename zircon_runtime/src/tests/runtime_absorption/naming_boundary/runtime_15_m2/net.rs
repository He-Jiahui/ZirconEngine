use std::path::Path;

use super::super::support::{
    assert_contains_all, read_repo_text, read_runtime_15_naming_date_map,
    read_runtime_15_naming_status_map, read_runtime_15_naming_status_rows,
};

#[path = "net/http1_client_policy.rs"]
mod http1_client_policy;

use std::fs;
use std::path::{Path, PathBuf};

use super::super::support::{assert_contains_all, read_repo_text};

#[path = "hub/raw_text_policy.rs"]
mod raw_text_policy;

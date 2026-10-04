//! 命名策略扫描需区分生产源码、测试夹具和已分类的历史名称。集中挂载下级测试；所有行为断言留在被挂载模块。
use std::fs;
use std::path::{Path, PathBuf};

use super::super::support::{assert_contains_all, read_repo_text};

#[path = "hub/raw_text_policy.rs"]
mod raw_text_policy;

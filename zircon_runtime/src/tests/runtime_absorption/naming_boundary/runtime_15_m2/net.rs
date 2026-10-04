//! 命名策略扫描需区分生产源码、测试夹具和已分类的历史名称。集中挂载下级测试；所有行为断言留在被挂载模块。
use std::path::Path;

use super::super::support::{assert_contains_all, read_repo_text};

#[path = "net/http1_client_policy.rs"]
mod http1_client_policy;

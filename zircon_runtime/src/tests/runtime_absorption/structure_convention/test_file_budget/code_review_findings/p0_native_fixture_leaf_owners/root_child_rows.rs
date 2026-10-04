//! 为原生夹具审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
use super::*;

pub(super) const P0_NATIVE_FIXTURE_ROOT_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "root_paths",
        P0_NATIVE_FIXTURE_ROOT_PATHS_CHILD,
        "P0_NATIVE_FIXTURE_ROOT_PATHS_CHILD",
    ),
    (
        "root_child_rows",
        P0_NATIVE_FIXTURE_ROOT_CHILD_ROWS_CHILD,
        "P0_NATIVE_FIXTURE_ROOT_CHILDREN",
    ),
    (
        "root_sources",
        P0_NATIVE_FIXTURE_ROOT_SOURCES_CHILD,
        "folder_backed_child_sources",
    ),
];

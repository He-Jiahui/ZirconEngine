//! 为类型化错误审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
use super::super::*;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "paths",
        TYPED_ERROR_SOURCE_INVENTORY_PATHS_CHILD,
        "const TYPED_ERROR_SOURCE_PATHS",
    ),
    (
        "reads",
        TYPED_ERROR_SOURCE_INVENTORY_READS_CHILD,
        "pub(in super::super) fn typed_error_children_source",
    ),
];

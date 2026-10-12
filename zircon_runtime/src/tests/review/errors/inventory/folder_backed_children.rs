//! 为类型化错误审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
use super::super::*;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[(
    "guard_body",
    TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_GUARD_BODY_CHILD,
    "pub(super) fn assert_typed_error_source_inventory_guard_is_folder_backed",
)];

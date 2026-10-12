//! 为类型化错误审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
use super::super::*;

pub(in super::super) const TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_CHILDREN: &[(
    &str,
    &str,
    &str,
)] = &[
    (
        "delegation_parent",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_DELEGATION_PARENT_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_delegation_is_child_backed",
    ),
    (
        "folder_backed_parent",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_FOLDER_BACKED_PARENT_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_delegation_folder_backed_is_child_owned",
    ),
    (
        "budgets",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_BUDGETS_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_delegation_folder_backed_ownership_child_budgets_are_current",
    ),
    (
        "route_ownership",
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_ROUTE_CHILD,
        "pub(super) fn assert_typed_error_source_inventory_delegation_folder_backed_ownership_is_child_backed",
    ),
];

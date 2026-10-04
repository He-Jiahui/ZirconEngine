//! 约束类型化错误审查的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_structure_delegates_source_inventory(
    sources: &TypedErrorSourceInventorySources,
) {
    assert_contains_all(
        "typed-error structure guard delegates source inventory to child owner",
        &sources.structure_child,
        &[
            "#[path = \"typed_error_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "source_inventory::typed_error_children_source",
            "source_inventory::assert_typed_error_line_budgets",
            "source_inventory::typed_error_review_guard_count",
        ],
    );
    assert!(
        !sources
            .structure_child
            .contains("const TYPED_ERROR_SOURCE_PATHS"),
        "typed_error_child_owners.rs should not retain the typed-error source inventory"
    );
    assert!(
        !sources.structure_child.contains("fn typed_error_sources()"),
        "typed_error_child_owners.rs should delegate typed-error source reads to source_inventory.rs"
    );
}

//! 保留类型化错误审查的历史辅助定义；当前模块入口未挂载此文件，文件存在本身不构成已执行的守卫覆盖。
use super::super::super::super::super::super::*;
use super::super::super::*;
use super::*;

// TODO: [CR-RUNTIME-TESTS-BUDGET-0004] 确认这些历史辅助文件应恢复挂载还是退役；当前父入口未连接它们，空清单循环也不提供覆盖证据。
pub(in super::super) fn assert_typed_error_source_inventory_child_inventory_status_current_child_budgets_are_current(
) {
    let mut budget_sources: Vec<(&'static str, String)> = vec![];

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

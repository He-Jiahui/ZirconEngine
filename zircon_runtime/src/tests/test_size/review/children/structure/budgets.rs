//! 检查类型化错误审查清单内源码的行数预算；注释和空行参与计数，未列入清单的文件仍需由递归预算守卫覆盖。
use super::*;

const STRUCTURE_ASSERTIONS_GUARD_CHILD_LINE_BUDGET: usize = 170;

pub(super) fn assert_structure_assertions_guard_child_budgets() {
    for (name, path, _) in STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILDREN {
        let line_count = read_runtime_src(path).lines().count();
        assert!(
            line_count <= STRUCTURE_ASSERTIONS_GUARD_CHILD_LINE_BUDGET,
            "{name} child {path} should stay under {STRUCTURE_ASSERTIONS_GUARD_CHILD_LINE_BUDGET} lines; got {line_count}"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_children_stay_budgeted(
) {
    assert_structure_assertions_guard_child_budgets();
}

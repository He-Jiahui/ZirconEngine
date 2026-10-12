//! 核对审查守卫汇总的父子职责、源码清单及迁移锚点；子串与计数约束保留结构快照，后续合法新增需同步复核预期。
use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_folder_backed_summary_children_are_child_owned() {
    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD);

    parent_absence::assert_folder_backed_summary_parent_keeps_child_details_out(&parent);
    direct_assertions::assert_folder_backed_direct_review_assertion_children_are_current();
    source_inventory_checks::assert_folder_backed_source_inventory_child_is_current(&parent);
    budgets::assert_folder_backed_summary_child_ownership_children_line_budgets_are_current();
}

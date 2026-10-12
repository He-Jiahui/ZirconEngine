//! 保护代码审查的子模块职责、检查入口与源码规模；这些约束读取检出文本，具体业务结果由被列出的行为测试另行验证。
#[path = "review/f8_child_owners.rs"]
mod f8_child_owners;
#[path = "review/folders.rs"]
mod folder_backed_summary;
#[path = "../../../test_size/review/late_api_cleanup_child_owners.rs"]
mod late_api_cleanup_child_owners;
#[path = "review/p0_child_owners.rs"]
mod p0_child_owners;
#[path = "review/fixtures.rs"]
mod p0_native_fixture_leaf_owners;
#[path = "../../../test_size/review/plugin_importer_dx_child_owners.rs"]
mod plugin_importer_dx_child_owners;
#[path = "review/children.rs"]
mod structure_guard_children;
#[path = "../../../structure/test_size/review/typed_error_child_owners.rs"]
mod typed_error_child_owners;

#[test]
fn runtime_15_code_review_findings_tests_are_folder_backed() {
    folder_backed_summary::assert_code_review_findings_tests_are_folder_backed();
}

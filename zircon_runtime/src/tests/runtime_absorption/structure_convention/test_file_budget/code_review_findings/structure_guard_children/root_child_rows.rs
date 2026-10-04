//! 为审查目录结构集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
use super::*;

pub(super) const STRUCTURE_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/delegation.rs",
        "runtime_15_code_review_findings_structure_guard_children_are_mounted",
    ),
    (
        "review_guard_groups",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/review_guard_groups.rs",
        "runtime_15_code_review_findings_structure_guard_review_groups_are_child_owned",
    ),
    (
        "plugin_importer",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs",
        "runtime_15_code_review_findings_structure_guard_plugin_importer_is_child_owned",
    ),
];

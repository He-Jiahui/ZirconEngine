//! 为插件导入接口审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
use super::*;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "child_ownership",
        PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "structure_assertions",
        PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
        "pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed",
    ),
];

pub(super) const FOLDER_BACKED_CHILD_PATHS: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/delegation.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/child_ownership.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/source_inventory.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/structure_assertions.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_owners/budgets.rs",
];

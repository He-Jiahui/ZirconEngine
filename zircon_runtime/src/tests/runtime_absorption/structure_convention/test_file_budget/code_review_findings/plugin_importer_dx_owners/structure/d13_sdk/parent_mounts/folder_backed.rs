use super::super::super::super::super::super::*;
use super::super::*;
use super::super::{budgets, review_children};
use super::*;

pub(super) fn assert_plugin_importer_d13_sdk_structure_assertions_are_child_owner() {
    let sources = plugin_importer_d13_sdk_structure_sources();
    let child_blob = plugin_importer_d13_sdk_structure_child_source_blob();
    let d13_sdk_child_tree = format!("{}\n{}", sources.d13_sdk_child, child_blob);

    assert_plugin_importer_d13_sdk_structure_parent_delegates(&sources);
    for moved_anchor in [
        "let plugin_importer_dx_d13 = read_runtime_src(",
        concat!(
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/",
            "d13_importer_sdk/manifest_parity.rs"
        ),
        concat!(
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/",
            "d13_importer_sdk/runtime_exports.rs"
        ),
        concat!("fn review_d13_importer_runtime_exports_", "use_sdk_macro"),
        concat!(
            "fn review_d13_importer_manifest_parity_guard_",
            "lives_in_sdk_builder"
        ),
    ] {
        assert!(
            !sources.structure_assertions_child.contains(moved_anchor),
            "plugin_importer_dx_owners/structure_assertions.rs should delegate D13 SDK structure assertion anchor `{moved_anchor}` to d13_sdk.rs"
        );
    }
    assert_contains_all(
        "plugin-importer D13 SDK structure assertions route owns focused child inventory",
        &d13_sdk_child_tree,
        &[
            "pub(super) fn assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
            PLUGIN_IMPORTER_D13_PATHS_CHILD,
            PLUGIN_IMPORTER_D13_SOURCES_CHILD,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD,
            PLUGIN_IMPORTER_D13_REVIEW_CHILDREN_CHILD,
            PLUGIN_IMPORTER_D13_BUDGETS_CHILD,
            "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner",
            PLUGIN_IMPORTER_D13_FOLDER_BACKED_GUARD,
        ],
    );
    assert_plugin_importer_d13_sdk_child_owners_are_folder_backed();
    budgets::assert_plugin_importer_d13_sdk_structure_assertions_children_line_budgets_are_current(
        &sources,
    );
    for (_, child_path, child_guard) in PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTION_CHILDREN {
        assert!(
            sources.d13_sdk_child.contains(child_path),
            "plugin-importer D13 SDK structure assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "plugin-importer D13 SDK child source blob should contain child guard {child_guard}"
        );
    }
}

pub(super) fn assert_plugin_importer_d13_sdk_structure_assertions_guard_is_folder_backed() {
    let sources = plugin_importer_d13_sdk_structure_sources();
    let child_blob = plugin_importer_d13_sdk_structure_child_source_blob();

    assert_plugin_importer_d13_sdk_structure_parent_delegates(&sources);
    assert_plugin_importer_d13_sdk_parent_mounts_review_children(&sources);
    review_children::assert_plugin_importer_d13_sdk_review_children_are_child_owned(&sources);
    budgets::assert_plugin_importer_d13_sdk_structure_assertions_children_line_budgets_are_current(
        &sources,
    );
    for (_, child_path, child_guard) in PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTION_CHILDREN {
        assert!(
            sources.d13_sdk_child.contains(child_path),
            "plugin-importer D13 SDK structure assertions parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "plugin-importer D13 SDK child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !sources
            .d13_sdk_child
            .contains("let plugin_importer_dx_d13 = read_runtime_src("),
        "d13_sdk.rs should delegate D13 SDK source reads to sources.rs"
    );
    assert_contains_all(
        "plugin-importer D13 SDK structure assertions parent records folder-backed status",
        &sources.d13_sdk_child,
        &[
            PLUGIN_IMPORTER_D13_FOLDER_BACKED_SLICE,
            PLUGIN_IMPORTER_D13_FOLDER_BACKED_STATUS,
            PLUGIN_IMPORTER_D13_FOLDER_BACKED_GUARD,
        ],
    );
}

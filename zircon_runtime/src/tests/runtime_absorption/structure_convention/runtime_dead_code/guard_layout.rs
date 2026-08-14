use super::super::assert_contains_all;
use super::{read_runtime_src, runtime_source_path, DEAD_CODE_ALLOW_ATTRIBUTE};

#[test]
fn runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed() {
    let root =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs");
    assert_contains_all(
        "runtime dead-code guard constant-backed forbidden attribute",
        &root,
        &[
            "const DEAD_CODE_ALLOW_ATTRIBUTE: &str = concat!(\"#[allow(\", \"dead_code\", \")]\");",
            "const DEAD_CODE_ALLOW_CALL_PREFIX: &str = concat!(\"allow(\", \"dead_code\");",
        ],
    );
}

#[test]
fn runtime_15_runtime_dead_code_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let root =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs");
    assert_contains_all(
        "structure convention parent runtime dead-code mount",
        &parent,
        &[
            "#[path = \"structure_convention/runtime_dead_code/mod.rs\"]",
            "mod runtime_dead_code;",
        ],
    );
    assert!(
        !runtime_source_path("tests/runtime_absorption/structure_convention/runtime_dead_code.rs")
            .exists(),
        "old flat runtime_dead_code.rs guard owner should be deleted after folder-backed cutover"
    );

    let parent_lines = parent.lines().count();
    assert!(
        parent_lines < 180,
        "structure_convention.rs should remain a thin aggregator after runtime dead-code split; got {parent_lines} lines"
    );
    let root_lines = root.lines().count();
    assert!(
        root_lines < 120,
        "runtime_dead_code/mod.rs should stay a thin support owner; got {root_lines} lines"
    );
}

#[test]
fn runtime_15_runtime_dead_code_guard_children_are_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let root =
        read_runtime_src("tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs");
    assert_contains_all(
        "runtime dead-code root mounts child owners",
        &root,
        &[
            "mod guard_layout;",
            "mod production_scan;",
            "mod runtime_owned;",
            "mod runtime_ui;",
            "mod script_host;",
            "mod status_anchor_cleanup;",
            "mod ui_text;",
        ],
    );
    assert!(
        !parent.contains("structure_convention/runtime_dead_code.rs"),
        "structure convention parent should not point at the retired flat runtime_dead_code.rs path"
    );
}

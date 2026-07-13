use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_status_mirrors_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/status_mirrors.rs",
    );
    let children = STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "structure-support status mirrors parent mounts children",
        &parent,
        &[
            "#[path = \"mirrors/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mirrors/paths.rs\"]",
            "mod paths;",
            "#[path = \"mirrors/row_maps.rs\"]",
            "mod row_maps;",
            "#[path = \"mirrors/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "runtime_15_structure_support_expected_slice_status_mirrors_are_current",
        "read_repo(\"docs/plans/zircon_runtime/runtime/index.md\")",
        STRUCTURE_SUPPORT_STATUS_MIRRORS_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "structure-support status mirrors parent should delegate `{moved_anchor}`"
        );
    }
    assert_contains_all(
        "structure-support status mirrors children",
        &children,
        &[
            STRUCTURE_SUPPORT_STATUS_MIRRORS_GUARD,
            "runtime_15_structure_support_expected_slice_status_rows_are_synced",
            "runtime_15_structure_support_expected_slice_status_docs_are_synced",
        ],
    );

    for (path, limit) in [
        (STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[0], 80usize),
        (STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[1], 30),
        (STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[2], 65),
        (STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[3], 95),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the structure-support status mirrors budget {limit}; got {line_count}"
        );
    }
}

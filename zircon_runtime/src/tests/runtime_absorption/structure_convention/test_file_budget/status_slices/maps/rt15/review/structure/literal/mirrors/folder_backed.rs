use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_literal_ownership_status_mirrors_are_folder_backed()
{
    let parent = read_literal_owner_source("literal/status_mirrors.rs");
    let children = LITERAL_STATUS_MIRROR_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "structure-support literal ownership status mirrors parent mounts children",
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
        "runtime_15_structure_support_expected_slice_literal_ownership_status_is_synced",
        "read_repo(\"docs/plans/zircon_runtime/runtime/index.md\")",
        LITERAL_STATUS_MIRRORS_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "structure-support literal ownership status mirrors parent should delegate `{moved_anchor}`"
        );
    }
    assert_contains_all(
        "structure-support literal ownership status mirrors children",
        &children,
        &[
            LITERAL_STATUS_MIRRORS_GUARD,
            "runtime_15_structure_support_expected_slice_literal_ownership_status_rows_are_synced",
            "runtime_15_structure_support_expected_slice_literal_ownership_status_docs_are_synced",
        ],
    );

    for (path, limit) in [
        (LITERAL_STATUS_MIRROR_CHILDREN[0], 80usize),
        (LITERAL_STATUS_MIRROR_CHILDREN[1], 30),
        (LITERAL_STATUS_MIRROR_CHILDREN[2], 70),
        (LITERAL_STATUS_MIRROR_CHILDREN[3], 95),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the structure-support literal status mirrors budget {limit}; got {line_count}"
        );
    }
}

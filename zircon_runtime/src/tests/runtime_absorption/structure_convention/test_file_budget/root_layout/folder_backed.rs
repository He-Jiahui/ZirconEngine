use super::*;

#[path = "folder_backed/assertions.rs"]
mod assertions;
#[path = "folder_backed/assertions_split.rs"]
mod assertions_split;
#[path = "folder_backed/guard_names.rs"]
mod guard_names;
#[path = "folder_backed/sources.rs"]
mod sources;

const ASSERTIONS_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs";
const ASSERTIONS_ASSET_CHILDREN_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions/asset_children.rs";
const ASSERTIONS_PARENT_MOUNTS_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions/parent_mounts.rs";
const ASSERTIONS_RENDER_CHILDREN_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions/render_children.rs";
const ASSERTIONS_RUNTIME_SCENE_CHILDREN_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions/runtime_scene_children.rs";
const ASSERTIONS_UI_CHILDREN_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions/ui_children.rs";
const ASSERTIONS_SPLIT_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions_split.rs";
const GUARD_NAMES_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs";
const SOURCES_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/sources.rs";

#[test]
fn runtime_15_test_file_budget_guard_is_folder_backed() {
    let sources = sources::read_guard_sources();
    let guards = guard_names::guard_names();

    assertions::assert_test_file_budget_root_is_folder_backed(&sources, &guards);
}

#[test]
fn runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed.rs",
    );
    let assertions = read_runtime_src(ASSERTIONS_OWNER);
    let assertions_asset_children = read_runtime_src(ASSERTIONS_ASSET_CHILDREN_OWNER);
    let assertions_parent_mounts = read_runtime_src(ASSERTIONS_PARENT_MOUNTS_OWNER);
    let assertions_render_children = read_runtime_src(ASSERTIONS_RENDER_CHILDREN_OWNER);
    let assertions_runtime_scene_children =
        read_runtime_src(ASSERTIONS_RUNTIME_SCENE_CHILDREN_OWNER);
    let assertions_ui_children = read_runtime_src(ASSERTIONS_UI_CHILDREN_OWNER);
    let assertions_split = read_runtime_src(ASSERTIONS_SPLIT_OWNER);
    let guard_names = read_runtime_src(GUARD_NAMES_OWNER);
    let sources = read_runtime_src(SOURCES_OWNER);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "folder-backed guard parent mounts support child owners",
        &parent,
        &[
            "#[path = \"folder_backed/assertions.rs\"]",
            "mod assertions;",
            "#[path = \"folder_backed/assertions_split.rs\"]",
            "mod assertions_split;",
            "#[path = \"folder_backed/guard_names.rs\"]",
            "mod guard_names;",
            "#[path = \"folder_backed/sources.rs\"]",
            "mod sources;",
            "fn runtime_15_test_file_budget_guard_is_folder_backed",
            "fn runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed",
        ],
    );
    for moved_anchor in [
        concat!("pub(super) struct ", "GuardSources"),
        concat!("pub(super) struct ", "GuardNames"),
        concat!(
            "pub(super) fn assert_test_file_budget_root_",
            "is_folder_backed"
        ),
        concat!("let asset_tests = ", "read_runtime_src("),
        concat!("let asset_pack_guard = ", "format!("),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "root_layout/folder_backed.rs should mount support child owners instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "folder-backed assertions route mounts focused assertion child owners",
        &assertions,
        &[
            "#[path = \"assertions/asset_children.rs\"]",
            "mod asset_children;",
            "#[path = \"assertions/parent_mounts.rs\"]",
            "mod parent_mounts;",
            "#[path = \"assertions/render_children.rs\"]",
            "mod render_children;",
            "#[path = \"assertions/runtime_scene_children.rs\"]",
            "mod runtime_scene_children;",
            "#[path = \"assertions/ui_children.rs\"]",
            "mod ui_children;",
            concat!(
                "pub(super) fn assert_test_file_budget_root_",
                "is_folder_backed"
            ),
            "parent_mounts::assert_parent_mounts_and_moved_guards",
            "asset_children::assert_asset_children",
            "runtime_scene_children::assert_runtime_scene_children",
            "render_children::assert_render_children",
            "ui_children::assert_ui_children",
        ],
    );
    for moved_anchor in [
        "test file budget parent mounts folder-backed guard owners",
        "asset pack test-budget child owns pack guard",
        "code review findings test-budget child owns findings guard",
        "UI shared core test-budget parent mounts shared-core child owners",
    ] {
        assert!(
            !assertions.contains(moved_anchor),
            "assertions.rs should route to focused children instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "folder-backed parent-mount assertions child owns root parent checks",
        &assertions_parent_mounts,
        &[
            "assert_parent_mounts_and_moved_guards",
            "test file budget parent mounts folder-backed guard owners",
            "test_file_budget/mod.rs should mount child guard owners",
        ],
    );
    assert_contains_all(
        "folder-backed asset assertions child owns asset checks",
        &assertions_asset_children,
        &[
            "assert_asset_children",
            "asset test-budget child owns child-owner mounts",
            "asset pack test-budget child owns pack guard",
            "asset scene test-budget child owns scene guard",
        ],
    );
    assert_contains_all(
        "folder-backed runtime-scene assertions child owns runtime and scene checks",
        &assertions_runtime_scene_children,
        &[
            "assert_runtime_scene_children",
            "code review findings test-budget child owns findings guard",
            "runtime diagnostics test-budget child owns diagnostics guard",
            "test-file budget module-layout child owns parent guard split",
        ],
    );
    assert_contains_all(
        "folder-backed render assertions child owns render checks",
        &assertions_render_children,
        &[
            "assert_render_children",
            "render product test-budget child owns camera-target guard",
            "shader prewarm manifest test-budget child owns manifest guard",
        ],
    );
    assert_contains_all(
        "folder-backed UI assertions child owns UI checks",
        &assertions_ui_children,
        &[
            "assert_ui_children",
            "UI shared core test-budget parent mounts shared-core child owners",
            "UI v2 asset test-budget child owns historical v2-asset guard",
        ],
    );
    assert_contains_all(
        "folder-backed guard-names child owns generated moved guard anchors",
        &guard_names,
        &[
            concat!("pub(super) struct ", "GuardNames"),
            "pub(super) fn guard_names",
            "asset_pack_guard",
            "shader_prewarm_manifest_guard",
        ],
    );
    assert_contains_all(
        "folder-backed sources child owns source reads",
        &sources,
        &[
            concat!("pub(super) struct ", "GuardSources"),
            "pub(super) fn read_guard_sources",
            "test_file_budget/asset_tests/pack.rs",
            "test_file_budget/ui_shared_core.rs",
        ],
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
            assertions.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/asset_children.rs",
            assertions_asset_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/parent_mounts.rs",
            assertions_parent_mounts.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/render_children.rs",
            assertions_render_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/runtime_scene_children.rs",
            assertions_runtime_scene_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/ui_children.rs",
            assertions_ui_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions_split.rs",
            assertions_split.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs",
            guard_names.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/sources.rs",
            sources.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused Runtime 15 test-support budget; got {line_count} lines"
        );
    }
}

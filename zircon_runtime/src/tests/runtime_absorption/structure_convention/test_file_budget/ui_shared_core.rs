use super::*;

#[path = "ui_shared_core/input_visibility.rs"]
mod input_visibility;
#[path = "ui_shared_core/layout_surface.rs"]
mod layout_surface;
#[path = "ui_shared_core/root.rs"]
mod root;
#[path = "ui_shared_core/scroll_mutation.rs"]
mod scroll_mutation;

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");
const ROOT_GUARD: &str = concat!("fn runtime_15_ui_shared_core_", "tests_are_folder_backed");
const LAYOUT_SURFACE_GUARD: &str = concat!(
    "fn runtime_15_ui_shared_core_layout_surface_",
    "children_are_folder_backed"
);
const INPUT_VISIBILITY_GUARD: &str = concat!(
    "fn runtime_15_ui_shared_core_input_visibility_",
    "children_are_folder_backed"
);
const SCROLL_MUTATION_GUARD: &str = concat!(
    "fn runtime_15_ui_shared_core_scroll_mutation_",
    "children_are_folder_backed"
);

#[test]
fn runtime_15_ui_shared_core_guard_child_owners_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core.rs",
    );
    let root = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core/root.rs",
    );
    let layout_surface = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core/layout_surface.rs",
    );
    let input_visibility = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core/input_visibility.rs",
    );
    let scroll_mutation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core/scroll_mutation.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "UI shared core guard parent mounts child owners",
        &parent,
        &[
            "mod input_visibility;",
            "mod layout_surface;",
            "mod root;",
            "mod scroll_mutation;",
            "runtime_15_ui_shared_core_guard_child_owners_are_folder_backed",
        ],
    );
    for moved_guard in [
        ROOT_GUARD,
        LAYOUT_SURFACE_GUARD,
        INPUT_VISIBILITY_GUARD,
        SCROLL_MUTATION_GUARD,
    ] {
        assert!(
            !parent.contains(moved_guard),
            "structure_convention/test_file_budget/ui_shared_core.rs should mount child owners instead of defining {moved_guard}"
        );
    }
    assert_contains_all(
        "UI shared core child guards preserve existing ownership checks",
        &format!("{root}\n{layout_surface}\n{input_visibility}\n{scroll_mutation}"),
        &[
            ROOT_GUARD,
            LAYOUT_SURFACE_GUARD,
            INPUT_VISIBILITY_GUARD,
            SCROLL_MUTATION_GUARD,
        ],
    );

    let test_count = [
        parent.as_str(),
        root.as_str(),
        layout_surface.as_str(),
        input_visibility.as_str(),
        scroll_mutation.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        test_count, 5,
        "UI shared-core guard parent plus children should preserve four existing guards plus the new layout guard"
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/ui_shared_core.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/ui_shared_core/root.rs",
            root.as_str(),
        ),
        (
            "structure_convention/test_file_budget/ui_shared_core/layout_surface.rs",
            layout_surface.as_str(),
        ),
        (
            "structure_convention/test_file_budget/ui_shared_core/input_visibility.rs",
            input_visibility.as_str(),
        ),
        (
            "structure_convention/test_file_budget/ui_shared_core/scroll_mutation.rs",
            scroll_mutation.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused guard budget; got {line_count} lines"
        );
    }
}

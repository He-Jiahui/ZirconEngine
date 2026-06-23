use super::*;

#[test]
fn runtime_15_ui_surface_frame_authority_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/surface_frame_authority.rs");
    let arranged_authority =
        read_runtime_src("ui/tests/surface_frame_authority/arranged_authority.rs");
    let taffy_flex = read_runtime_src("ui/tests/surface_frame_authority/taffy_flex.rs");
    let taffy_wrap_grid = read_runtime_src("ui/tests/surface_frame_authority/taffy_wrap_grid.rs");
    let zircon_fallback = read_runtime_src("ui/tests/surface_frame_authority/zircon_fallback.rs");

    assert_contains_all(
        "UI surface-frame authority parent mounts folder-backed children",
        &parent,
        &[
            "mod arranged_authority;",
            "mod taffy_flex;",
            "mod taffy_wrap_grid;",
            "mod zircon_fallback;",
            "fn overlapping_button_surface()",
            "fn taffy_flex_button_surface()",
            "fn taffy_grid_slot_button_surface()",
            "fn zircon_size_box_button_surface()",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/surface_frame_authority.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "surface_frame_render_hit_and_pointer_dispatch_share_arranged_authority",
        "taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch",
        "taffy_wrap_surface_frame_feeds_render_hit_and_pointer_dispatch",
        "zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI surface-frame authority test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI surface-frame authority arranged child owns arranged/focus contracts",
        &arranged_authority,
        &[
            "fn surface_frame_render_hit_and_pointer_dispatch_share_arranged_authority",
            "fn surface_frame_focus_path_uses_arranged_authority",
        ],
    );
    assert_contains_all(
        "UI surface-frame authority Taffy flex child owns flex contracts",
        &taffy_flex,
        &[
            "fn taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch",
            "fn taffy_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch",
            "fn taffy_vertical_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch",
            "fn taffy_flex_slot_policy_fallback_feeds_render_hit_and_pointer_dispatch",
        ],
    );
    assert_contains_all(
        "UI surface-frame authority Taffy wrap/grid child owns wrap and grid contracts",
        &taffy_wrap_grid,
        &[
            "fn taffy_wrap_surface_frame_feeds_render_hit_and_pointer_dispatch",
            "fn taffy_grid_slot_frame_policy_feeds_render_hit_and_pointer_dispatch",
        ],
    );
    assert_contains_all(
        "UI surface-frame authority Zircon fallback child owns SizeBox fallback contract",
        &zircon_fallback,
        &["fn zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch"],
    );

    let child_test_total = [
        arranged_authority.as_str(),
        taffy_flex.as_str(),
        taffy_wrap_grid.as_str(),
        zircon_fallback.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 9,
        "UI surface-frame authority children should preserve all 9 parent tests"
    );

    for (path, source) in [
        ("ui/tests/surface_frame_authority.rs", parent.as_str()),
        (
            "ui/tests/surface_frame_authority/arranged_authority.rs",
            arranged_authority.as_str(),
        ),
        (
            "ui/tests/surface_frame_authority/taffy_flex.rs",
            taffy_flex.as_str(),
        ),
        (
            "ui/tests/surface_frame_authority/taffy_wrap_grid.rs",
            taffy_wrap_grid.as_str(),
        ),
        (
            "ui/tests/surface_frame_authority/zircon_fallback.rs",
            zircon_fallback.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI surface-frame authority test folder split",
                "runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/surface_frame_authority.rs",
                "ui/tests/surface_frame_authority/taffy_flex.rs",
                "ui/tests/surface_frame_authority/taffy_wrap_grid.rs",
                "runtime_15_ui_surface_frame_authority_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI surface-frame authority test folder split",
            "runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/surface_frame_authority.rs",
            "ui/tests/surface_frame_authority/arranged_authority.rs",
            "runtime_15_ui_surface_frame_authority_tests_are_folder_backed",
        ],
    );
}

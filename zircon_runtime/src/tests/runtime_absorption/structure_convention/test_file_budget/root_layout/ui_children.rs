use super::*;

const UI_TEST_BUDGET_GUARDS: [(&str, &str, &str); 25] = [
    (
        "UI architecture test-budget child owns architecture guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_architecture.rs",
        "fn runtime_15_ui_architecture_tests_are_folder_backed",
    ),
    (
        "UI asset test-budget child owns asset guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_asset.rs",
        "fn runtime_15_ui_asset_tests_are_folder_backed",
    ),
    (
        "UI asset MUI X web style test-budget child owns asset MUI X web style guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_asset_mui_web_mui_x_style.rs",
        "fn runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed",
    ),
    (
        "UI asset MUI web style test-budget child owns asset MUI web style guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_asset_mui_web_style.rs",
        "fn runtime_15_ui_asset_mui_web_style_tests_are_folder_backed",
    ),
    (
        "UI boundary test-budget child owns boundary guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_boundary.rs",
        "fn runtime_15_ui_boundary_tests_are_folder_backed",
    ),
    (
        "UI accessibility test-budget child owns accessibility guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility.rs",
        "fn runtime_15_ui_accessibility_tests_are_folder_backed",
    ),
    (
        "UI accessibility widget-actions test-budget child owns accessibility widget-actions guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility_widget_actions.rs",
        "fn runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed",
    ),
    (
        "UI component catalog test-budget child owns component-catalog guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog.rs",
        "fn runtime_15_ui_component_catalog_tests_are_folder_backed",
    ),
    (
        "UI component catalog component-state test-budget child owns component-state guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state.rs",
        "fn runtime_15_ui_component_catalog_component_state_tests_are_folder_backed",
    ),
    (
        "UI component catalog component-state keyboard test-budget child owns keyboard guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state_keyboard.rs",
        "fn runtime_15_ui_component_catalog_component_state_keyboard_tests_are_folder_backed",
    ),
    (
        "UI component catalog Material foundation test-budget child owns Material foundation guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_material_foundation.rs",
        "fn runtime_15_ui_component_catalog_material_foundation_tests_are_folder_backed",
    ),
    (
        "UI event routing test-budget child owns event-routing guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_event_routing.rs",
        "fn runtime_15_ui_event_routing_tests_are_folder_backed",
    ),
    (
        "UI focus navigation test-budget child owns focus navigation guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_focus_navigation.rs",
        "fn runtime_15_ui_focus_navigation_tests_are_folder_backed",
    ),
    (
        "UI layout slots test-budget child owns layout-slots guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_layout_slots.rs",
        "fn runtime_15_ui_layout_slots_tests_are_folder_backed",
    ),
    (
        "UI material layout test-budget child owns material-layout guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_material_layout.rs",
        "fn runtime_15_ui_material_layout_tests_are_folder_backed",
    ),
    (
        "UI surface dirty-domain test-budget child owns dirty-domain guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_dirty_domains.rs",
        "fn runtime_15_ui_surface_dirty_domains_tests_are_folder_backed",
    ),
    (
        "UI surface-frame authority test-budget child owns surface-frame authority guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_frame_authority.rs",
        "fn runtime_15_ui_surface_frame_authority_tests_are_folder_backed",
    ),
    (
        "UI taffy layout pass test-budget child owns taffy layout guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_taffy_layout_pass.rs",
        "fn runtime_15_ui_taffy_layout_pass_tests_are_folder_backed",
    ),
    (
        "UI template test-budget child owns template guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_template.rs",
        "fn runtime_15_ui_template_tests_are_folder_backed",
    ),
    (
        "UI widget text input keyboard test-budget child owns keyboard guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_widget_text_input_keyboard.rs",
        "fn runtime_15_ui_widget_text_input_keyboard_tests_are_folder_backed",
    ),
    (
        "UI runtime input manager test-budget child owns input manager guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs",
        "fn runtime_15_ui_runtime_input_manager_tests_are_folder_backed",
    ),
    (
        "UI runtime input ownership test-budget child owns input ownership guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_ownership.rs",
        "fn runtime_15_ui_runtime_input_ownership_tests_are_folder_backed",
    ),
    (
        "UI runtime input reply routes test-budget child owns reply-routes aggregate guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs",
        "fn runtime_15_ui_runtime_input_reply_route_guard_child_owners_are_folder_backed",
    ),
    (
        "UI runtime window event ABI test-budget child owns ABI child guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_event_abi.rs",
        "fn runtime_15_ui_runtime_window_event_abi_children_are_folder_backed",
    ),
    (
        "UI runtime window input pump test-budget child owns input pump guard",
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_input_pump.rs",
        "fn runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed",
    ),
];

#[test]
fn runtime_15_test_file_budget_root_layout_ui_child_scan_is_child_owner() {
    let root_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs",
    );
    let parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/mod.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
    );

    assert_contains_all(
        "root-layout guard parent mounts UI child scan",
        &root_layout,
        &[
            "#[path = \"root_layout/ui_children.rs\"]",
            "mod ui_children;",
        ],
    );
    assert!(
        !root_layout.contains(
            "runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred"
        ),
        "root_layout.rs should delegate UI child status anchors to root_layout/ui_children.rs"
    );
    assert!(
        !root_layout
            .contains("runtime_15_test_file_budget_root_layout_ui_child_scan_is_child_owner"),
        "root_layout.rs should mount the UI child scan instead of defining it inline"
    );

    assert_contains_all(
        "test file budget parent keeps UI guard owner mounts",
        &parent,
        &[
            "mod ui_architecture;",
            "mod ui_asset;",
            "mod ui_asset_mui_web_mui_x_style;",
            "mod ui_asset_mui_web_style;",
            "mod ui_accessibility;",
            "mod ui_accessibility_widget_actions;",
            "mod ui_boundary;",
            "mod ui_component_catalog;",
            "mod ui_component_catalog_component_state;",
            "mod ui_component_catalog_component_state_keyboard;",
            "mod ui_component_catalog_material_foundation;",
            "mod ui_event_routing;",
            "mod ui_focus_navigation;",
            "mod ui_layout_slots;",
            "mod ui_material_layout;",
            "mod ui_runtime_input_manager;",
            "mod ui_surface_dirty_domains;",
            "mod ui_surface_frame_authority;",
            "mod ui_taffy_layout_pass;",
            "mod ui_template;",
            "mod ui_widget_text_input_keyboard;",
            "mod ui_runtime_input_ownership;",
            "mod ui_runtime_input_reply_routes;",
            "mod ui_runtime_window_event_abi;",
            "mod ui_runtime_window_input_pump;",
        ],
    );

    for (_, _, guard) in UI_TEST_BUDGET_GUARDS {
        assert!(
            !parent.contains(guard),
            "test_file_budget/mod.rs should mount UI child guard owners instead of defining {guard}"
        );
    }

    for (label, path, guard) in UI_TEST_BUDGET_GUARDS {
        let source = read_runtime_src(path);
        assert_contains_all(label, &source, &["use super::*;", guard]);
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 test file budget root-layout UI child split",
                "runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/root_layout/ui_children.rs",
                "runtime_15_test_file_budget_root_layout_ui_child_scan_is_child_owner",
            ],
        );
    }
}

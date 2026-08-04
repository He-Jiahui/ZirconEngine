use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_template_style_slot_contract_is_child_owner() {
    let parent = read_runtime_src("ui/template/asset/compiler/style_apply.rs");
    let slot_contract = read_runtime_src("ui/template/asset/compiler/style_apply/slot_contract.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "style-apply parent keeps style planning, selector application, and shared helpers",
        &parent,
        &[
            "mod slot_contract;",
            "pub(super) use slot_contract::{",
            "apply_mui_child_slot_props",
            "apply_mui_root_slot_props_to_node",
            "mui_slot_name",
            "pub(super) fn build_style_plan",
            "pub(super) fn apply_styles_to_tree",
            "pub(super) fn apply_mui_sx_to_node",
            "pub(super) fn append_mui_style_classes",
            "fn append_mui_component_specific_classes(",
            "fn selector_states(",
            "fn map_attribute_any(",
            "fn value_as_map(",
        ],
    );
    for moved_owner in [
        "fn apply_mui_root_slot_props_to_node(",
        "fn apply_mui_child_slot_props(",
        "fn apply_mui_slot_contract_to_child(",
        "fn append_mui_owner_slot_utility_classes(",
        "fn mui_slot_name(",
        "mui_slot_component",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "style_apply.rs should delegate slot-contract owner `{moved_owner}` to slot_contract.rs"
        );
    }
    assert_contains_all(
        "slot-contract child owns root/child slot props and owner slot utility class routing",
        &slot_contract,
        &[
            "pub(in super::super) fn apply_mui_root_slot_props_to_node",
            "pub(in super::super) fn apply_mui_child_slot_props",
            "fn apply_mui_slot_contract_to_child(",
            "fn append_mui_owner_slot_utility_classes(",
            "fn mui_slot_name(",
            "mui_display_surface_classes::append_skeleton_child_metadata",
            "mui_layout_classes::append_layout_slot_classes",
            "mui_x_classes::append_slot_classes",
            "mui_navigation_classes::append_tabs_slot_classes",
            "mui_slot_component",
        ],
    );

    for (path, source) in [
        ("ui/template/asset/compiler/style_apply.rs", parent.as_str()),
        (
            "ui/template/asset/compiler/style_apply/slot_contract.rs",
            slot_contract.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

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
                "Runtime 15 M4 UI template style slot-contract owner split",
                "runtime_15_ui_template_style_slot_contract_owner_split_static_passed_cargo_timeout_no_result",
                "ui/template/asset/compiler/style_apply.rs",
                "ui/template/asset/compiler/style_apply/slot_contract.rs",
                "runtime_15_ui_template_style_slot_contract_is_child_owner",
            ],
        );
    }
}

#[test]
fn runtime_15_ui_template_mui_x_data_grid_classes_are_child_owner() {
    let parent = read_runtime_src("ui/template/asset/compiler/style_apply/mui_x_classes.rs");
    let data_grid =
        read_runtime_src("ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "MUI X parent keeps component-family dispatch and shared helpers",
        &parent,
        &[
            "mod data_grid;",
            "\"DataGrid\" => data_grid::append_component_classes(node, prefix)",
            "data_grid::append_slot_classes(child, owner_attributes, slot_name)",
            "fn append_tree_view_classes(",
            "fn append_date_time_picker_classes(",
            "fn append_chart_classes(",
            "fn append_agent_chat_classes(",
            "fn array_attribute_any_non_empty(",
            "fn string_attribute_any_from_attributes(",
        ],
    );
    for moved_owner in [
        "fn append_data_grid_classes(",
        "fn append_column_header_slot_classes(",
        "fn append_row_slot_classes(",
        "fn append_cell_slot_classes(",
        "MuiDataGrid-columnHeader--sortable",
        "MuiDataGrid-cell--editing",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "mui_x_classes.rs should delegate DataGrid owner `{moved_owner}` to data_grid.rs"
        );
    }
    assert_contains_all(
        "DataGrid child owns root and slot utility classes",
        &data_grid,
        &[
            "pub(super) fn append_component_classes",
            "pub(super) fn append_slot_classes",
            "fn append_column_header_slot_classes(",
            "fn append_row_slot_classes(",
            "fn append_cell_slot_classes(",
            "MuiDataGrid-columnHeader--sortable",
            "MuiDataGrid-columnHeader--withRightBorder",
            "MuiDataGrid-cell--editing",
            "format!(\"{prefix}-hasViewportRange\")",
        ],
    );

    for (path, source) in [
        (
            "ui/template/asset/compiler/style_apply/mui_x_classes.rs",
            parent.as_str(),
        ),
        (
            "ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs",
            data_grid.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

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
                "Runtime 15 M4 UI template MUI X DataGrid class owner split",
                "runtime_15_ui_template_mui_x_data_grid_class_owner_split_static_passed_cargo_deferred",
                "ui/template/asset/compiler/style_apply/mui_x_classes.rs",
                "ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs",
                "runtime_15_ui_template_mui_x_data_grid_classes_are_child_owner",
            ],
        );
    }
}

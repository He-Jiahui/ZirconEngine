use super::*;

#[test]
fn runtime_15_ui_template_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/template.rs");
    let interaction_bindings = read_runtime_src("ui/tests/template/interaction_bindings.rs");
    let layout_compute = read_runtime_src("ui/tests/template/layout_compute.rs");
    let loader_instance_validation =
        read_runtime_src("ui/tests/template/loader_instance_validation.rs");
    let slot_contracts = read_runtime_src("ui/tests/template/slot_contracts.rs");
    let surface_containers = read_runtime_src("ui/tests/template/surface_containers.rs");

    assert_contains_all(
        "UI template parent mounts folder-backed children",
        &parent,
        &[
            "mod interaction_bindings;",
            "mod layout_compute;",
            "mod loader_instance_validation;",
            "mod slot_contracts;",
            "mod surface_containers;",
            "fn tree_from_root_toml(",
            "fn root_with_inline_node(",
            "fn only_root_node(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/template.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "template_loader_parses_component_slots_and_binding_refs_from_toml",
        "template_tree_builder_projects_template_instance_into_shared_ui_tree_with_metadata",
        "template_surface_builder_maps_known_container_components_into_shared_runtime_nodes",
        "template_tree_builder_preserves_parent_owned_slot_contracts",
        "template_surface_builder_computes_layout_from_template_contract_attributes",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI template test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI template loader child owns loader/instance validation contracts",
        &loader_instance_validation,
        &[
            "fn template_loader_parses_component_slots_and_binding_refs_from_toml",
            "fn template_instance_expands_composite_slots_and_preserves_stable_bindings",
            "fn template_validator_rejects_missing_required_slots",
        ],
    );
    assert_contains_all(
        "UI template interaction child owns interaction binding contracts",
        &interaction_bindings,
        &[
            "fn template_tree_builder_projects_template_instance_into_shared_ui_tree_with_metadata",
            "fn template_tree_builder_infers_scroll_binding_as_receive_input_only",
            "fn template_tree_builder_allows_explicit_focusable_input_metadata",
        ],
    );
    assert_contains_all(
        "UI template surface/container child owns surface and container contracts",
        &surface_containers,
        &[
            "fn template_surface_builder_maps_known_container_components_into_shared_runtime_nodes",
            "fn template_tree_builder_maps_layout_contract_attributes_into_shared_runtime_nodes",
            "fn template_tree_builder_parses_size_box_container_contract",
        ],
    );
    assert_contains_all(
        "UI template slot child owns slot contracts",
        &slot_contracts,
        &[
            "fn template_tree_builder_preserves_parent_owned_slot_contracts",
            "fn template_tree_builder_preserves_overlay_slot_z_order_contracts",
            "fn template_tree_builder_ignores_canvas_free_placement_on_space_slots",
        ],
    );
    assert_contains_all(
        "UI template layout compute child owns template layout contract",
        &layout_compute,
        &["fn template_surface_builder_computes_layout_from_template_contract_attributes"],
    );

    let child_test_total = [
        interaction_bindings.as_str(),
        layout_compute.as_str(),
        loader_instance_validation.as_str(),
        slot_contracts.as_str(),
        surface_containers.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 22,
        "UI template children should preserve all 22 parent tests"
    );

    for (path, source) in [
        ("ui/tests/template.rs", parent.as_str()),
        (
            "ui/tests/template/interaction_bindings.rs",
            interaction_bindings.as_str(),
        ),
        (
            "ui/tests/template/layout_compute.rs",
            layout_compute.as_str(),
        ),
        (
            "ui/tests/template/loader_instance_validation.rs",
            loader_instance_validation.as_str(),
        ),
        (
            "ui/tests/template/slot_contracts.rs",
            slot_contracts.as_str(),
        ),
        (
            "ui/tests/template/surface_containers.rs",
            surface_containers.as_str(),
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
                "Runtime 15 M3 UI template test folder split",
                "runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/template.rs",
                "ui/tests/template/interaction_bindings.rs",
                "ui/tests/template/slot_contracts.rs",
                "runtime_15_ui_template_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI template test folder split",
            "runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/template.rs",
            "ui/tests/template/interaction_bindings.rs",
            "runtime_15_ui_template_tests_are_folder_backed",
        ],
    );
}

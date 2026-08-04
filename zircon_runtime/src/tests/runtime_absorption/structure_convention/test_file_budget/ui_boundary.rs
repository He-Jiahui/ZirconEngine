use super::*;

#[test]
fn runtime_15_ui_boundary_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/boundary.rs");
    let asset_fixture_projection =
        read_runtime_src("ui/tests/boundary/asset_fixture_projection.rs");
    let binding_event_roots = read_runtime_src("ui/tests/boundary/binding_event_roots.rs");
    let layout_tree_surface = read_runtime_src("ui/tests/boundary/layout_tree_surface.rs");
    let template_namespace = read_runtime_src("ui/tests/boundary/template_namespace.rs");

    assert_contains_all(
        "UI boundary parent mounts folder-backed children",
        &parent,
        &[
            "mod asset_fixture_projection;",
            "mod binding_event_roots;",
            "mod layout_tree_surface;",
            "mod template_namespace;",
            "fn collect_ui_toml_files(",
            "fn format_paths(",
            "fn relative_path(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/boundary.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "template_legacy_adapter_is_removed_from_formal_namespace_surface",
        "template_asset_document_api_moves_under_template_namespace",
        "layout_solver_api_moves_under_layout_namespace",
        "surface_render_api_moves_under_surface_namespace",
        "binding_api_moves_under_binding_namespace",
        "event_ui_api_moves_under_event_ui_namespace",
        "runtime_fixture_assets_live_under_crate_assets",
        "zui_surface_projection_does_not_call_template_tree_builder",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI boundary test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI boundary template child owns template namespace contracts",
        &template_namespace,
        &[
            "fn template_legacy_adapter_is_removed_from_formal_namespace_surface",
            "fn template_asset_document_api_moves_under_template_namespace",
        ],
    );
    assert_contains_all(
        "UI boundary layout/tree/surface child owns structural contracts",
        &layout_tree_surface,
        &[
            "fn root_surface_avoids_wildcard_flatten_for_namespace_owned_domains",
            "fn layout_solver_api_moves_under_layout_namespace",
            "fn tree_specialist_api_moves_under_tree_namespace",
            "fn surface_render_api_moves_under_surface_namespace",
            "fn dispatch_api_moves_under_dispatch_namespace",
        ],
    );
    assert_contains_all(
        "UI boundary binding/event child owns root structure contracts",
        &binding_event_roots,
        &[
            "fn binding_api_moves_under_binding_namespace",
            "fn event_ui_api_moves_under_event_ui_namespace",
            "fn dispatch_root_stays_structural_after_folder_split",
            "fn surface_root_stays_structural_after_folder_split",
        ],
    );
    assert_contains_all(
        "UI boundary asset fixture child owns asset fixture contracts",
        &asset_fixture_projection,
        &[
            "fn runtime_ui_entry_assets_do_not_live_under_src",
            "fn runtime_fixture_assets_live_under_crate_assets",
            "fn runtime_ui_manager_loads_fixture_documents_from_asset_files",
            "fn zui_surface_projection_does_not_call_template_tree_builder",
        ],
    );

    let child_test_total = [
        asset_fixture_projection.as_str(),
        binding_event_roots.as_str(),
        layout_tree_surface.as_str(),
        template_namespace.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 32,
        "UI boundary children should preserve all 32 parent tests"
    );

    for (path, source) in [
        ("ui/tests/boundary.rs", parent.as_str()),
        (
            "ui/tests/boundary/asset_fixture_projection.rs",
            asset_fixture_projection.as_str(),
        ),
        (
            "ui/tests/boundary/binding_event_roots.rs",
            binding_event_roots.as_str(),
        ),
        (
            "ui/tests/boundary/layout_tree_surface.rs",
            layout_tree_surface.as_str(),
        ),
        (
            "ui/tests/boundary/template_namespace.rs",
            template_namespace.as_str(),
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
                "Runtime 15 M3 UI boundary test folder split",
                "runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/boundary.rs",
                "ui/tests/boundary/template_namespace.rs",
                "ui/tests/boundary/asset_fixture_projection.rs",
                "runtime_15_ui_boundary_tests_are_folder_backed",
            ],
        );
    }
}

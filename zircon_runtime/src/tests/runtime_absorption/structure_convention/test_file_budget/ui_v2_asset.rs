use super::*;

const STYLE_RUNTIME_STATUS_NAME: &str = "Runtime 15 M3 UI v2 style-runtime test folder split";
const STYLE_RUNTIME_STATUS_ID: &str =
    "runtime_15_ui_v2_style_runtime_tests_folder_split_static_passed_cargo_deferred";
const STYLE_RUNTIME_GUARD: &str = "runtime_15_ui_v2_style_runtime_tests_are_folder_backed";
const STYLE_RUNTIME_CHILDREN: &[(&str, &str)] = &[
    (
        "static_resolution",
        "ui/tests/v2_asset/style_runtime/static_resolution.rs",
    ),
    (
        "runtime_pseudo_state",
        "ui/tests/v2_asset/style_runtime/runtime_pseudo_state.rs",
    ),
    (
        "resolved_pseudo_state",
        "ui/tests/v2_asset/style_runtime/resolved_pseudo_state.rs",
    ),
    (
        "property_mutation",
        "ui/tests/v2_asset/style_runtime/property_mutation.rs",
    ),
    (
        "style_overrides",
        "ui/tests/v2_asset/style_runtime/style_overrides.rs",
    ),
];

#[test]
fn runtime_15_ui_v2_asset_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/v2_asset.rs");
    let asset_loading = read_runtime_src("ui/tests/v2_asset/asset_loading.rs");
    let style_runtime_parent = read_runtime_src("ui/tests/v2_asset/style_runtime.rs");
    let style_runtime_children = style_runtime_child_source_blob();
    let default_controls = read_runtime_src("ui/tests/v2_asset/default_controls.rs");
    let range_controls = read_runtime_src("ui/tests/v2_asset/range_controls.rs");
    let demo_and_builder = read_runtime_src("ui/tests/v2_asset/demo_and_builder.rs");
    let composite_components = read_runtime_src("ui/tests/v2_asset/composite_components.rs");
    let file_cache = read_runtime_src("ui/tests/v2_asset/file_cache.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "UI v2 asset parent test module mounts",
        &parent,
        &[
            "mod asset_loading;",
            "mod composite_components;",
            "mod default_controls;",
            "mod demo_and_builder;",
            "mod file_cache;",
            "mod range_controls;",
            "mod style_runtime;",
            "fn v2_document",
        ],
    );

    for moved_guard in [
        "fn ui_v2_parses_flat_view_asset",
        "fn ui_v2_style_specificity_and_pseudo_state_are_resolved",
        "fn ui_v2_surface_default_toggle_click_mutates_checked_and_restyles_runtime_pseudo_state",
        "fn ui_v2_surface_default_rangefield_click_sets_value_and_rebuilds_render_only",
        "fn material_demo_window_compiles_and_resolves_material_dark_states",
        "fn ui_v2_composite_component_patches_root_props_and_fills_slots",
        "fn ui_v2_file_cache_reuses_compiled_store_and_resolves_transitive_styles",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "ui/tests/v2_asset.rs should mount child test owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "UI v2 asset loading child owns loader contracts",
        &asset_loading,
        &[
            "fn ui_v2_parses_flat_view_asset",
            "fn ui_zui_loader_accepts_single_component_asset",
            "fn ui_v2_rejects_cycles_before_surface_build",
        ],
    );
    assert_contains_all(
        "UI v2 asset style runtime child owns cascade contracts",
        &style_runtime_children,
        &[
            "fn ui_v2_style_specificity_and_pseudo_state_are_resolved",
            "fn ui_v2_surface_property_mutation_updates_runtime_style_baseline_metadata",
            "fn ui_v2_inline_style_overrides_cascade_values_in_style_overrides",
        ],
    );
    assert_contains_all(
        "UI v2 asset default controls child owns stateful control contracts",
        &default_controls,
        &[
            "fn ui_v2_surface_default_toggle_click_mutates_checked_and_restyles_runtime_pseudo_state",
            "fn ui_v2_surface_default_combobox_click_toggles_popup_open_and_routes_typed_events",
        ],
    );
    assert_contains_all(
        "UI v2 asset range controls child owns range contracts",
        &range_controls,
        &[
            "fn ui_v2_surface_default_rangefield_click_sets_value_and_rebuilds_render_only",
            "fn ui_v2_surface_rangefield_keyboard_navigation_steps_value_render_only",
        ],
    );
    assert_contains_all(
        "UI v2 asset demo and builder child owns demo/build contracts",
        &demo_and_builder,
        &[
            "fn material_demo_window_compiles_and_resolves_material_dark_states",
            "fn ui_v2_surface_builder_infers_interaction_from_component_catalog",
            "fn ui_v2_virtual_list_window_uses_visible_range_and_overscan",
        ],
    );
    assert_contains_all(
        "UI v2 asset composite child owns composite contracts",
        &composite_components,
        &[
            "fn ui_v2_composite_component_patches_root_props_and_fills_slots",
            "fn ui_v2_imported_component_instance_style_patches_expanded_root",
        ],
    );
    assert_contains_all(
        "UI v2 asset file cache child owns cache contracts",
        &file_cache,
        &[
            "fn ui_v2_file_cache_reuses_compiled_store_and_resolves_transitive_styles",
            "fn ui_v2_file_cache_prefers_zui_asset_id_over_legacy_v2_document",
        ],
    );

    for (path, source) in [
        ("ui/tests/v2_asset.rs", parent.as_str()),
        ("ui/tests/v2_asset/asset_loading.rs", asset_loading.as_str()),
        (
            "ui/tests/v2_asset/style_runtime.rs",
            style_runtime_parent.as_str(),
        ),
        (
            "ui/tests/v2_asset/default_controls.rs",
            default_controls.as_str(),
        ),
        (
            "ui/tests/v2_asset/range_controls.rs",
            range_controls.as_str(),
        ),
        (
            "ui/tests/v2_asset/demo_and_builder.rs",
            demo_and_builder.as_str(),
        ),
        (
            "ui/tests/v2_asset/composite_components.rs",
            composite_components.as_str(),
        ),
        ("ui/tests/v2_asset/file_cache.rs", file_cache.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (_, path) in STYLE_RUNTIME_CHILDREN {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI v2 asset test folder split",
                "runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked",
                "ui/tests/v2_asset/style_runtime.rs",
                "ui/tests/v2_asset/file_cache.rs",
                "runtime_15_ui_v2_asset_tests_are_folder_backed",
            ],
        );
    }
}

#[test]
fn runtime_15_ui_v2_style_runtime_tests_are_folder_backed() {
    let route = read_runtime_src("ui/tests/v2_asset/style_runtime.rs");
    let child_sources = style_runtime_child_source_blob();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "UI v2 style runtime route mounts child test owners",
        &route,
        &[
            "mod static_resolution;",
            "mod runtime_pseudo_state;",
            "mod resolved_pseudo_state;",
            "mod property_mutation;",
            "mod style_overrides;",
        ],
    );
    for moved_test in [
        "fn ui_v2_style_specificity_and_pseudo_state_are_resolved",
        "fn ui_v2_surface_runtime_pseudo_state_restyles_from_retained_component_state",
        "fn ui_v2_resolved_pseudo_state_keeps_selection_identity_above_hover",
        "fn ui_v2_surface_property_mutation_updates_runtime_style_baseline_metadata",
        "fn ui_v2_inline_style_overrides_cascade_values_in_style_overrides",
    ] {
        assert!(
            !route.contains(moved_test),
            "ui/tests/v2_asset/style_runtime.rs should delegate moved test {moved_test}"
        );
        assert!(
            child_sources.contains(moved_test),
            "style_runtime child sources should retain moved test {moved_test}"
        );
    }
    for (_, path) in STYLE_RUNTIME_CHILDREN {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

fn style_runtime_child_source_blob() -> String {
    STYLE_RUNTIME_CHILDREN
        .iter()
        .map(|(_, path)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}

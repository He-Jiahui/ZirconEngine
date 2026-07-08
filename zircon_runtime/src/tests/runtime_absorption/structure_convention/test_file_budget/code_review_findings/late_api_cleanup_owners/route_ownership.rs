use super::*;

pub(super) fn assert_late_api_cleanup_child_owners_are_folder_backed() {
    let review_sources = read_late_api_cleanup_sources();
    let parent = review_sources.parent.as_str();

    assert_contains_all(
        "late API cleanup parent mounts focused child owners",
        parent,
        &[
            "#[path = \"late_api_cleanup/f11_shading_model_registry.rs\"]",
            "mod f11_shading_model_registry;",
            "#[path = \"late_api_cleanup/f15_editor_pane_data_conversion.rs\"]",
            "mod f15_editor_pane_data_conversion;",
            "#[path = \"late_api_cleanup/f17_entity_path_lookup.rs\"]",
            "mod f17_entity_path_lookup;",
            "#[path = \"late_api_cleanup/f18_asset_manager_resolution.rs\"]",
            "mod f18_asset_manager_resolution;",
            "#[path = \"late_api_cleanup/f19_scene_renderer_construction.rs\"]",
            "mod f19_scene_renderer_construction;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "late_api_cleanup.rs should only mount child review guard owners"
    );
    for child_owned_test in REVIEW_GUARDS {
        let child_owned_fn = format!("fn {child_owned_test}");
        assert!(
            !parent.contains(&child_owned_fn),
            "child-owned late API cleanup review guard `{child_owned_test}` should not return to late_api_cleanup.rs"
        );
    }

    assert_contains_all(
        "F11 shading-model child owns dead API cleanup review guard",
        &review_sources.f11_shading_model_registry,
        &[
            "fn review_f11_shading_model_registry_has_no_dead_plugin_registration_surface",
            "render_shading_model_registry_dead_api_removed_coremin_passed",
            "custom shading-model plugin registration remains a future Plan 08 surface",
        ],
    );
    assert_contains_all(
        "F15 editor pane data conversion child owns projection-owner review guard",
        &review_sources.f15_editor_pane_data_conversion,
        &[
            "fn review_f15_editor_pane_data_conversion_top_row_uses_projection_owners",
            "pane_data_conversion/mod.rs",
            "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "F17 entity path child owns get-verb review guard",
        &review_sources.f17_entity_path_lookup,
        &[
            "fn review_f17_entity_path_option_lookup_uses_get_verb",
            "get_entity_by_path",
            "f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "F18 asset manager child owns registered-handle review guard",
        &review_sources.f18_asset_manager_resolution,
        &[
            "fn review_f18_asset_manager_resolution_returns_registered_handle",
            "Result<Arc<AssetManagerHandle>, CoreError>",
            "runtime_10_asset_manager_resolution_handle_shape_coremin_check_passed",
        ],
    );
    assert_contains_all(
        "F19 scene renderer child owns construct-name review guard",
        &review_sources.f19_scene_renderer_construction,
        &[
            "fn review_f19_scene_renderer_construction_modules_use_construct_names",
            "scene_renderer_core_construct",
            "f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );

    let children = review_sources
        .all_sources()
        .iter()
        .map(|(_, source)| *source)
        .collect::<Vec<_>>()
        .join("\n");
    for guard in REVIEW_GUARDS {
        assert!(
            children.contains(&format!("fn {guard}")),
            "late API cleanup child owners should preserve {guard}"
        );
    }
    assert_eq!(
        late_api_cleanup_review_guard_count(),
        5,
        "late API cleanup child owners should preserve all five review guards"
    );
}

#[test]
fn runtime_15_late_api_cleanup_review_guards_are_child_owners() {
    assert_late_api_cleanup_child_owners_are_folder_backed();
}

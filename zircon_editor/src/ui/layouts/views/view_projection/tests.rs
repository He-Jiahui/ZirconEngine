use super::*;
use std::collections::BTreeMap;
use toml::Value;
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use super::materialization::component_owns_text_paint;

#[test]
fn view_projection_root_remains_a_structural_facade() {
    let root = include_str!("../view_projection.rs");

    assert!(root.lines().count() < 100);
    for behavior_marker in [
        "fn view_template_nodes_from_surface",
        "fn resolve_component_variant",
        "fn string_attribute",
        "struct ViewTemplateNodeMaterialization",
    ] {
        assert!(
            !root.contains(behavior_marker),
            "root facade must not own `{behavior_marker}`"
        );
    }
}

#[test]
fn view_template_projection_rejects_non_zui_asset_paths() {
    let text_overrides = BTreeMap::new();
    let error = build_view_template_nodes(
        "view.archived.project_overview",
        "/assets/ui/editor/project_overview.toml",
        &[],
        UiSize::new(640.0, 480.0),
        &text_overrides,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ViewTemplateProjectionError::NonV2AssetPath(path)
            if path == "/assets/ui/editor/project_overview.toml"
    ));
}

#[test]
fn zui_view_template_projection_uses_v2_surface_builder_without_legacy_fallback() {
    let text_overrides = BTreeMap::from([(
        "ProjectOverviewTitleText".to_string(),
        "V2 Project".to_string(),
    )]);

    let nodes = build_view_template_nodes(
        "view.v2.project_overview",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(320.0, 240.0),
        &text_overrides,
    )
    .unwrap();

    assert!(nodes
        .iter()
        .any(|node| node.control_id == "ProjectOverviewTitleText" && node.text == "V2 Project"));
    assert!(nodes.iter().any(|node| node.role == "Button"));
    assert!(store_cache::cached_store_count_for_tests() > 0);
}

#[test]
fn v2_view_template_projection_reuses_layout_and_reapplies_text_overrides() {
    projection_cache::clear_for_tests();
    let first_overrides = BTreeMap::from([
        (
            "ProjectOverviewTitleText".to_string(),
            "First title".to_string(),
        ),
        (
            "ProjectOverviewPathText".to_string(),
            "Stable path".to_string(),
        ),
    ]);
    let second_overrides = BTreeMap::from([
        (
            "ProjectOverviewTitleText".to_string(),
            "Second title".to_string(),
        ),
        (
            "ProjectOverviewPathText".to_string(),
            "Stable path".to_string(),
        ),
    ]);
    let size = UiSize::new(640.0, 480.0);

    let first = build_view_template_nodes(
        "view.v2.project_overview.cached",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &first_overrides,
    )
    .unwrap();
    let generation_after_first =
        projection_cache::surface_generation_for_tests("view.v2.project_overview.cached")
            .expect("cached surface generation");
    let mutation_count_after_first = projection_cache::property_mutation_count_for_tests();
    let rebuild_count_after_first = projection_cache::incremental_rebuild_count_for_tests();
    let second = build_view_template_nodes(
        "view.v2.project_overview.cached",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &second_overrides,
    )
    .unwrap();
    let generation_after_second =
        projection_cache::surface_generation_for_tests("view.v2.project_overview.cached")
            .expect("cached surface generation");
    let restored = build_view_template_nodes(
        "view.v2.project_overview.cached",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &BTreeMap::new(),
    )
    .unwrap();

    assert_eq!(projection_cache::materialization_count_for_tests(), 1);
    assert_eq!(mutation_count_after_first, 2);
    assert_eq!(rebuild_count_after_first, 1);
    assert_eq!(generation_after_second, generation_after_first + 1);
    assert_eq!(projection_cache::property_mutation_count_for_tests(), 5);
    assert_eq!(projection_cache::incremental_rebuild_count_for_tests(), 3);
    assert_eq!(
        projection_cache::surface_string_property_for_tests(
            "view.v2.project_overview.cached",
            "ProjectOverviewTitleText",
            "text",
        )
        .as_deref(),
        Some("Project")
    );
    assert_eq!(
        projection_cache::surface_string_property_for_tests(
            "view.v2.project_overview.cached",
            "ProjectOverviewPathText",
            "text",
        )
        .as_deref(),
        Some("")
    );
    assert_eq!(
        first
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.as_str()),
        Some("First title")
    );
    assert_eq!(
        second
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.as_str()),
        Some("Second title")
    );
    assert_eq!(
        restored
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.as_str()),
        Some("Project")
    );
}

#[test]
fn v2_view_template_projection_text_update_visits_only_indexed_geometry_commands() {
    projection_cache::clear_for_tests();
    let document_tree_id = "view.v2.project_overview.indexed_geometry";
    let size = UiSize::new(640.0, 480.0);
    let first_overrides = BTreeMap::from([(
        "ProjectOverviewTitleText".to_string(),
        "1111111111".to_string(),
    )]);
    let second_overrides = BTreeMap::from([(
        "ProjectOverviewTitleText".to_string(),
        "2222222222".to_string(),
    )]);

    let _ = build_view_template_nodes(
        document_tree_id,
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &first_overrides,
    )
    .unwrap();
    let render_command_count =
        projection_cache::render_command_count_for_tests(document_tree_id).unwrap();
    let command_visits_before = projection_cache::geometry_command_visit_count_for_tests();

    let updated = build_view_template_nodes(
        document_tree_id,
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &second_overrides,
    )
    .unwrap();
    let command_visits =
        projection_cache::geometry_command_visit_count_for_tests() - command_visits_before;

    assert_eq!(
        updated
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.as_str()),
        Some("2222222222")
    );
    assert!(render_command_count > 1);
    assert!(
        command_visits < render_command_count as u64,
        "a local text update visited {command_visits} of {render_command_count} render commands"
    );
}

#[test]
fn v2_view_template_projection_stable_update_does_not_mutate_or_rebuild_surface() {
    projection_cache::clear_for_tests();
    let size = UiSize::new(640.0, 480.0);
    let overrides = BTreeMap::from([(
        "ProjectOverviewTitleText".to_string(),
        "Stable title".to_string(),
    )]);

    let first = build_view_template_node_projection(
        "view.v2.project_overview.stable_surface",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &overrides,
    )
    .unwrap()
    .into_model();
    let generation =
        projection_cache::surface_generation_for_tests("view.v2.project_overview.stable_surface")
            .expect("cached surface generation");
    let property_mutations = projection_cache::property_mutation_count_for_tests();
    let incremental_rebuilds = projection_cache::incremental_rebuild_count_for_tests();
    let stable = build_view_template_node_projection(
        "view.v2.project_overview.stable_surface",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &overrides,
    )
    .unwrap()
    .into_model();

    assert!(first.shares_values_with(&stable));
    assert_eq!(
        projection_cache::surface_generation_for_tests("view.v2.project_overview.stable_surface",),
        Some(generation)
    );
    assert_eq!(
        projection_cache::property_mutation_count_for_tests(),
        property_mutations
    );
    assert_eq!(
        projection_cache::incremental_rebuild_count_for_tests(),
        incremental_rebuilds
    );
}

#[test]
fn v2_view_template_projection_size_change_reuses_surface_and_matches_full_build() {
    projection_cache::clear_for_tests();
    let overrides = BTreeMap::new();

    let _ = build_view_template_node_projection(
        "view.v2.project_overview.resized_surface",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(640.0, 480.0),
        &overrides,
    )
    .unwrap();
    let resized = build_view_template_node_projection(
        "view.v2.project_overview.resized_surface",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(800.0, 480.0),
        &overrides,
    )
    .unwrap()
    .into_model();

    assert_eq!(
        projection_cache::surface_materialization_count_for_tests(),
        1
    );
    assert_eq!(projection_cache::incremental_rebuild_count_for_tests(), 1);

    let full = build_view_template_node_projection(
        "view.v2.project_overview.resized_full_reference",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(800.0, 480.0),
        &overrides,
    )
    .unwrap()
    .into_model();

    assert_eq!(resized, full);
}

#[test]
fn v2_hierarchy_projection_size_change_uses_stable_projected_row_topology() {
    projection_cache::clear_for_tests();
    let overrides = BTreeMap::new();

    let _ = build_view_template_node_projection(
        "view.v2.hierarchy.resized_surface",
        "/assets/ui/editor/hierarchy.zui",
        &[],
        UiSize::new(280.0, 600.0),
        &overrides,
    )
    .unwrap();
    let resized = build_view_template_node_projection(
        "view.v2.hierarchy.resized_surface",
        "/assets/ui/editor/hierarchy.zui",
        &[],
        UiSize::new(280.0, 620.0),
        &overrides,
    )
    .unwrap()
    .into_model();

    assert_eq!(
        projection_cache::surface_materialization_count_for_tests(),
        1
    );
    assert_eq!(projection_cache::incremental_rebuild_count_for_tests(), 1);

    let full = build_view_template_node_projection(
        "view.v2.hierarchy.resized_full_reference",
        "/assets/ui/editor/hierarchy.zui",
        &[],
        UiSize::new(280.0, 620.0),
        &overrides,
    )
    .unwrap()
    .into_model();

    assert_eq!(resized, full);
}

#[test]
fn v2_view_template_projection_reuses_models_and_clones_only_changed_text_rows() {
    projection_cache::clear_for_tests();
    let size = UiSize::new(640.0, 480.0);
    let first_overrides = BTreeMap::from([
        (
            "ProjectOverviewTitleText".to_string(),
            "First title".to_string(),
        ),
        (
            "ProjectOverviewPathText".to_string(),
            "Stable path".to_string(),
        ),
    ]);
    let second_overrides = BTreeMap::from([
        (
            "ProjectOverviewTitleText".to_string(),
            "Second title".to_string(),
        ),
        (
            "ProjectOverviewPathText".to_string(),
            "Stable path".to_string(),
        ),
    ]);

    let first = build_view_template_node_projection(
        "view.v2.project_overview.retained",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &first_overrides,
    )
    .unwrap()
    .into_model();
    let clone_bytes_after_first = projection_cache::node_clone_owned_bytes_for_tests();
    let stable = build_view_template_node_projection(
        "view.v2.project_overview.retained",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &first_overrides,
    )
    .unwrap()
    .into_model();
    let clone_bytes_after_stable = projection_cache::node_clone_owned_bytes_for_tests();
    let changed = build_view_template_node_projection(
        "view.v2.project_overview.retained",
        "/assets/ui/editor/project_overview.zui",
        &[],
        size,
        &second_overrides,
    )
    .unwrap()
    .into_model();

    let title_row = first
        .iter()
        .position(|node| node.control_id == "ProjectOverviewTitleText")
        .expect("title row");
    let stable_override_row = first
        .iter()
        .position(|node| node.control_id == "ProjectOverviewPathText")
        .expect("stable override row");

    assert!(first.shares_values_with(&stable));
    assert!(!first.shares_values_with(&changed));
    assert!(!first.shares_row_with(&changed, title_row));
    assert!(first.shares_row_with(&changed, stable_override_row));
    assert_eq!(
        changed.get(title_row).map(|node| node.text.as_str()),
        Some("Second title")
    );
    assert_eq!(
        projection_cache::surface_materialization_count_for_tests(),
        1
    );
    assert_eq!(projection_cache::node_clone_count_for_tests(), 3);
    assert!(clone_bytes_after_stable > 0);
    assert_eq!(clone_bytes_after_stable, clone_bytes_after_first);
    assert!(projection_cache::node_clone_owned_bytes_for_tests() > clone_bytes_after_stable);
    assert_eq!(projection_cache::legacy_full_clone_count_for_tests(), 0);
}

#[test]
fn v2_view_template_projection_reuses_cached_store_for_identical_inputs() {
    let text_overrides = BTreeMap::new();
    store_cache::clear_for_tests();

    let first = build_view_template_nodes(
        "view.v2.project_overview.first",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(640.0, 480.0),
        &text_overrides,
    )
    .unwrap();
    let cache_len_after_first = store_cache::cached_store_count_for_tests();

    let second = build_view_template_nodes(
        "view.v2.project_overview.second",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(640.0, 480.0),
        &text_overrides,
    )
    .unwrap();
    let cache_len_after_second = store_cache::cached_store_count_for_tests();

    assert!(
        cache_len_after_first > 0,
        "first v2 projection should populate the store file cache"
    );
    assert_eq!(cache_len_after_second, cache_len_after_first);
    assert_eq!(first.len(), second.len());
    assert_eq!(
        first
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.clone()),
        second
            .iter()
            .find(|node| node.control_id == "ProjectOverviewTitleText")
            .map(|node| node.text.clone())
    );
}

#[test]
fn mui_feedback_metadata_projects_roles_variants_open_state_and_progress_percent() {
    let progress = metadata(
        "Progress",
        [
            ("variant", Value::String("circular".to_string())),
            ("value", Value::Float(68.0)),
        ],
    );
    assert_eq!(resolve_component_role(&progress.component), "progress");
    assert_eq!(resolve_component_variant(&progress), "circular");
    assert_eq!(
        resolve_node_value_percent(
            &progress,
            resolve_component_role(&progress.component),
            resolve_node_value_number(&progress),
        ),
        0.68
    );

    let backdrop = metadata(
        "Backdrop",
        [
            ("open", Value::Boolean(true)),
            ("invisible", Value::Boolean(true)),
        ],
    );
    assert_eq!(resolve_component_role(&backdrop.component), "backdrop");
    assert_eq!(resolve_component_variant(&backdrop), "invisible");
    assert!(resolve_node_popup_open(&backdrop));

    let skeleton = metadata(
        "Skeleton",
        [
            ("variant", Value::String("rounded".to_string())),
            ("animation", Value::String("wave".to_string())),
        ],
    );
    assert_eq!(resolve_component_role(&skeleton.component), "skeleton");
    assert_eq!(resolve_component_variant(&skeleton), "rounded wave");

    let fade = metadata(
        "Fade",
        [
            ("in", Value::Boolean(true)),
            ("transition_progress", Value::Float(0.5)),
        ],
    );
    let fade_role = resolve_component_role(&fade.component);
    assert_eq!(fade_role, "fade");
    assert_eq!(resolve_transition_kind(&fade, fade_role), "fade");
    assert!(resolve_transition_in(&fade, true, false));
    assert_eq!(resolve_transition_progress(&fade, "entering", true), 0.5);
    assert_eq!(default_transition_duration_ms("fade", true), 225);
    assert_eq!(
        default_transition_easing("fade", true),
        "cubic-bezier(0.4, 0, 0.2, 1)"
    );

    let slide = metadata("Slide", []);
    let slide_role = resolve_component_role(&slide.component);
    assert_eq!(slide_role, "slide");
    assert_eq!(resolve_transition_kind(&slide, slide_role), "slide");
    assert_eq!(default_transition_duration_ms("slide", false), 195);
    assert_eq!(
        default_transition_easing("slide", true),
        "cubic-bezier(0.0, 0, 0.2, 1)"
    );

    assert_eq!(resolve_component_role("Dialog"), "dialog");
    assert_eq!(resolve_component_role("Popover"), "popover");
    assert_eq!(resolve_component_role("Tooltip"), "tooltip");
    assert_eq!(resolve_component_role("ContextMenu"), "context-menu");
    assert_eq!(
        resolve_component_role("ContextActionMenu"),
        "context-action-menu"
    );
    assert_eq!(resolve_component_role("DropdownPopup"), "dropdown-popup");
    assert_eq!(resolve_component_role("Snackbar"), "snackbar");
    assert_eq!(resolve_component_role("Drawer"), "drawer");
}

#[test]
fn mui_text_field_metadata_projects_variant_state_tokens_for_native_painter() {
    let text_field = metadata(
        "TextField",
        [
            ("variant", Value::String("filled".to_string())),
            ("focused", Value::Boolean(true)),
            ("error", Value::Boolean(true)),
            ("size", Value::String("small".to_string())),
        ],
    );

    assert_eq!(resolve_component_role(&text_field.component), "input-field");
    assert_eq!(
        resolve_component_variant(&text_field),
        "filled focused error small"
    );

    let default_text_field = metadata("TextField", []);
    assert_eq!(resolve_component_variant(&default_text_field), "outlined");
}

#[test]
fn native_painter_components_own_generated_text_commands() {
    assert!(component_owns_text_paint(&metadata("Button", [])));
    assert!(component_owns_text_paint(&metadata(
        "IconButton",
        [("icon_placement", Value::String("icon_only".to_string()))],
    )));
    assert!(!component_owns_text_paint(&metadata(
        "IconButton",
        [("icon_placement", Value::String("leading".to_string()))],
    )));
    assert!(component_owns_text_paint(&metadata("EditableTable", [])));
    assert!(component_owns_text_paint(&metadata("InputField", [])));
    assert!(component_owns_text_paint(&metadata("NumberField", [])));
    assert!(component_owns_text_paint(&metadata("Table", [])));
    assert!(component_owns_text_paint(&metadata("TextField", [])));
}

#[test]
fn icon_only_icon_button_hides_its_paint_label() {
    assert!(icon_button_hides_label(&metadata(
        "IconButton",
        [("icon_placement", Value::String("icon_only".to_string()))],
    )));
    assert!(!icon_button_hides_label(&metadata(
        "IconButton",
        [("icon_placement", Value::String("leading".to_string()))],
    )));
}

#[test]
fn asset_browser_icon_only_actions_project_icons_without_paint_labels() {
    projection_cache::clear_for_tests();
    let text_overrides = BTreeMap::from([
        (
            "AssetBrowserViewModeListButton".to_string(),
            "List".to_string(),
        ),
        (
            "AssetBrowserViewModeThumbButton".to_string(),
            "Thumb".to_string(),
        ),
        ("SearchEdited".to_string(), "metal".to_string()),
    ]);
    let nodes = build_view_template_nodes(
        "view.v2.asset_browser.icon_actions",
        "/assets/ui/editor/asset_browser.zui",
        &[],
        UiSize::new(420.0, 320.0),
        &text_overrides,
    )
    .unwrap();

    for (control_id, icon_name) in [
        ("AssetBrowserViewModeListButton", "list-outline"),
        ("AssetBrowserViewModeThumbButton", "grid-outline"),
        (
            "LocateSelectedAsset",
            "editor_pages/asset_browser/navigation/search.svg",
        ),
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing asset browser action `{control_id}`"));
        assert_eq!(node.role.as_str(), "IconButton");
        assert_eq!(node.icon_name.as_str(), icon_name);
        assert!(
            node.text.is_empty(),
            "icon-only action `{control_id}` must not emit a duplicate text label"
        );
    }

    let search = nodes
        .iter()
        .find(|node| node.control_id == "SearchEdited")
        .expect("asset browser search field");
    assert_eq!(search.text.as_str(), "metal");
    assert_eq!(search.value_text.as_str(), "metal");
    assert_eq!(
        projection_cache::surface_string_property_for_tests(
            "view.v2.asset_browser.icon_actions",
            "SearchEdited",
            "value",
        )
        .as_deref(),
        Some("metal")
    );
    assert_eq!(
        projection_cache::surface_string_property_for_tests(
            "view.v2.asset_browser.icon_actions",
            "SearchEdited",
            "placeholder",
        )
        .as_deref(),
        Some("Search")
    );
}

#[test]
fn empty_search_override_keeps_placeholder_out_of_the_surface_value() {
    projection_cache::clear_for_tests();
    let text_overrides = BTreeMap::from([("SearchEdited".to_string(), String::new())]);
    let nodes = build_view_template_nodes(
        "view.v2.asset_browser.empty_search",
        "/assets/ui/editor/asset_browser.zui",
        &[],
        UiSize::new(420.0, 320.0),
        &text_overrides,
    )
    .unwrap();

    let search = nodes
        .iter()
        .find(|node| node.control_id == "SearchEdited")
        .expect("asset browser search field");
    assert_eq!(search.text.as_str(), "Search");
    assert!(search.value_text.is_empty());
    assert_eq!(
        projection_cache::surface_string_property_for_tests(
            "view.v2.asset_browser.empty_search",
            "SearchEdited",
            "value",
        )
        .as_deref(),
        Some("")
    );
}

#[test]
fn number_field_binding_preserves_the_authored_numeric_value_kind() {
    let metadata = metadata(
        "NumberField",
        [
            ("value", Value::Float(42.0)),
            ("value_text", Value::String("42".to_string())),
            ("placeholder", Value::String("0".to_string())),
        ],
    );
    let binding = text_binding_for_metadata(
        zircon_runtime_interface::ui::event_ui::UiNodeId::new(7),
        &metadata,
        "number-field",
    );

    assert_eq!(
        binding.requested_mutations(Some("12.5")),
        vec![
            (
                "value_text".to_string(),
                UiValue::String("12.5".to_string())
            ),
            ("value".to_string(), UiValue::Float(12.5)),
        ]
    );
    assert_eq!(
        binding.requested_mutations(Some("12.")),
        vec![("value_text".to_string(), UiValue::String("12.".to_string()))]
    );
    assert_eq!(
        binding.requested_mutations(Some("abc")),
        vec![("value_text".to_string(), UiValue::String("abc".to_string()))]
    );
    assert_eq!(
        binding.requested_mutations(None),
        vec![
            ("value_text".to_string(), UiValue::String("42".to_string())),
            ("value".to_string(), UiValue::Float(42.0)),
        ]
    );
}

#[test]
fn number_field_without_value_text_keeps_valid_values_typed_and_invalid_edits_visible() {
    let metadata = metadata("NumberField", [("value", Value::Float(42.0))]);
    let binding = text_binding_for_metadata(
        zircon_runtime_interface::ui::event_ui::UiNodeId::new(8),
        &metadata,
        "number-field",
    );

    assert_eq!(
        binding.requested_mutations(Some("12.5")),
        vec![("value".to_string(), UiValue::Float(12.5))]
    );
    assert_eq!(
        binding.requested_mutations(Some("12.")),
        vec![("value".to_string(), UiValue::String("12.".to_string()))]
    );
    assert_eq!(
        binding.requested_mutations(None),
        vec![("value".to_string(), UiValue::Float(42.0))]
    );
}

fn metadata<const N: usize>(
    component: &str,
    attributes: [(&str, Value); N],
) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        attributes: attributes
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
        ..UiTemplateNodeMetadata::default()
    }
}

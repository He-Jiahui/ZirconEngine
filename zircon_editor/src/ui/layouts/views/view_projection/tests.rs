use super::*;
use std::collections::BTreeMap;
use toml::Value;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

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
    assert!(
        view_v2_store_file_cache()
            .lock()
            .expect("v2 cache mutex should not be poisoned")
            .len()
            > 0
    );
}

#[test]
fn v2_view_template_projection_reuses_cached_store_for_identical_inputs() {
    let text_overrides = BTreeMap::new();
    let cache = view_v2_store_file_cache();
    cache
        .lock()
        .expect("v2 cache mutex should not be poisoned")
        .clear();

    let first = build_view_template_nodes(
        "view.v2.project_overview.first",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(640.0, 480.0),
        &text_overrides,
    )
    .unwrap();
    let cache_len_after_first = cache
        .lock()
        .expect("v2 cache mutex should not be poisoned")
        .len();

    let second = build_view_template_nodes(
        "view.v2.project_overview.second",
        "/assets/ui/editor/project_overview.zui",
        &[],
        UiSize::new(640.0, 480.0),
        &text_overrides,
    )
    .unwrap();
    let cache_len_after_second = cache
        .lock()
        .expect("v2 cache mutex should not be poisoned")
        .len();

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
    assert!(component_owns_text_paint(&metadata("EditableTable", [])));
    assert!(component_owns_text_paint(&metadata("InputField", [])));
    assert!(component_owns_text_paint(&metadata("NumberField", [])));
    assert!(component_owns_text_paint(&metadata("Table", [])));
    assert!(component_owns_text_paint(&metadata("TextField", [])));
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

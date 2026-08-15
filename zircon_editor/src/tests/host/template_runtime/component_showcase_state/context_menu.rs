use super::support::apply_showcase_binding;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostValue, UiComponentShowcaseDemoEventInput,
};

#[test]
fn showcase_context_action_menu_selects_clean_action_labels_from_menu_metadata() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ContextActionMenuChanged",
        UiComponentShowcaseDemoEventInput::SelectOption {
            option_id: "Duplicate||Ctrl+D".to_string(),
            selected: true,
        },
    );

    assert_eq!(
        runtime
            .showcase_demo_state()
            .value_text("ContextActionMenuDemo", "value")
            .as_deref(),
        Some("Duplicate"),
        "ContextActionMenu should store the selected action label, not the encoded menu-row metadata"
    );
}

#[test]
fn showcase_context_action_menu_opens_at_retained_pointer_anchor() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ContextActionMenuOpenAt",
        UiComponentShowcaseDemoEventInput::OpenPopupAt { x: 212.0, y: 96.0 },
    );

    let projection = runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let menu = host_projection
        .node_by_control_id("ContextActionMenuDemo")
        .expect("ContextActionMenuDemo");
    assert!(menu.popup_open);
    assert!(menu.has_popup_anchor);
    assert_eq!(menu.popup_anchor_x, 212.0);
    assert_eq!(menu.popup_anchor_y, 96.0);
    assert_eq!(
        menu.properties.get("popup_anchor_x"),
        Some(&RetainedUiHostValue::Float(212.0))
    );
    assert_eq!(
        menu.properties.get("popup_anchor_y"),
        Some(&RetainedUiHostValue::Float(96.0))
    );
}

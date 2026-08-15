use super::support::apply_showcase_binding;
use crate::ui::template_runtime::{EditorUiHostRuntime, UiComponentShowcaseDemoEventInput};

#[test]
fn showcase_demo_state_projects_collection_children_and_control_flags() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowClicked",
        UiComponentShowcaseDemoEventInput::None,
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowHovered",
        UiComponentShowcaseDemoEventInput::Hover(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowPressed",
        UiComponentShowcaseDemoEventInput::Press(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/NumberFieldDragBegin",
        UiComponentShowcaseDemoEventInput::None,
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldDropHovered",
        UiComponentShowcaseDemoEventInput::DropHover(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldActiveDragTarget",
        UiComponentShowcaseDemoEventInput::ActiveDragTarget(true),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ContextActionMenuOpenPopup",
        UiComponentShowcaseDemoEventInput::None,
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

    let list_row = host_projection
        .node_by_control_id("ListRowDemo")
        .expect("ListRowDemo should be projected");
    assert!(list_row.focused, "ListRow should retain focus state");
    assert_eq!(
        list_row.selection_state.as_deref(),
        Some("focused"),
        "focused row state should be represented as a selection-state token"
    );
    assert!(list_row.hovered, "ListRow should retain hover state");
    assert!(list_row.pressed, "ListRow should retain press state");

    assert!(
        host_projection
            .node_by_control_id("NumberFieldDemo")
            .is_some_and(|node| node.dragging),
        "NumberField BeginDrag should be retained and projected"
    );
    assert!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .is_some_and(|node| node.drop_hovered),
        "AssetField DropHover should be retained and projected"
    );
    assert!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .is_some_and(|node| node.active_drag_target),
        "AssetField ActiveDragTarget should be retained and projected"
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowHovered",
        UiComponentShowcaseDemoEventInput::Hover(false),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ListRowPressed",
        UiComponentShowcaseDemoEventInput::Press(false),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldDropHovered",
        UiComponentShowcaseDemoEventInput::DropHover(false),
    );
    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/AssetFieldActiveDragTarget",
        UiComponentShowcaseDemoEventInput::ActiveDragTarget(false),
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

    let list_row = host_projection
        .node_by_control_id("ListRowDemo")
        .expect("ListRowDemo should be projected after transient flags clear");
    assert!(
        !list_row.hovered,
        "ListRow Hover(false) should override the authored showcase hover prop"
    );
    assert!(
        !list_row.pressed,
        "ListRow Press(false) should clear retained press state"
    );
    let asset_field = host_projection
        .node_by_control_id("AssetFieldDemo")
        .expect("AssetFieldDemo should be projected after transient flags clear");
    assert!(
        !asset_field.drop_hovered,
        "AssetField DropHover(false) should override the authored showcase drop-hover prop"
    );
    assert!(
        !asset_field.active_drag_target,
        "AssetField ActiveDragTarget(false) should override the authored showcase active target prop"
    );

    assert_eq!(
        host_projection
            .node_by_control_id("ArrayFieldDemo")
            .expect("ArrayFieldDemo")
            .collection_items,
        vec![
            "#0 UiComponentRef = Label".to_string(),
            "#1 UiComponentRef = NumberField".to_string(),
            "#2 UiComponentRef = AssetField".to_string(),
        ],
        "ArrayField should project generated child rows from its element schema"
    );
    assert_eq!(
        host_projection
            .node_by_control_id("MapFieldDemo")
            .expect("MapFieldDemo")
            .collection_items,
        vec![
            "speed: String -> UiValue = 1".to_string(),
            "visible: String -> UiValue = true".to_string(),
        ],
        "MapField should project generated key/value child rows from its typed schema"
    );

    let menu = host_projection
        .node_by_control_id("ContextActionMenuDemo")
        .expect("ContextActionMenuDemo");
    assert!(menu.popup_open);
    assert_eq!(
        menu.menu_items,
        vec![
            "Inspect|checked,focused|Ctrl+I".to_string(),
            "---".to_string(),
            "Duplicate|hovered,pressed|Ctrl+D".to_string(),
            "Delete|disabled|Del".to_string(),
        ],
        "ContextActionMenu should project menu-row metadata beyond a flat option label"
    );
}

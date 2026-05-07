use crate::ui::workbench::reflection::{
    WorkbenchWidgetReflectorError, WorkbenchWidgetReflectorModel,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{
        UiNodeId, UiNodePath, UiReflectedProperty, UiReflectedPropertySource, UiReflectorNode,
        UiReflectorSnapshot, UiStateFlags, UiTreeId, UiWidgetLifecycleState,
    },
    tree::{UiDirtyFlags, UiVisibility},
};

#[test]
fn workbench_reflection_widget_reflector_projects_tree_rows_and_selected_node_details() {
    let snapshot = reflector_snapshot();
    let mut model = WorkbenchWidgetReflectorModel::new(snapshot);

    let rows = model.rows();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].node_id, UiNodeId::new(1));
    assert_eq!(rows[0].depth, 0);
    assert_eq!(rows[1].node_id, UiNodeId::new(2));
    assert_eq!(rows[1].depth, 1);
    assert_eq!(rows[1].lifecycle, UiWidgetLifecycleState::Interactive);
    assert!(rows[1].dirty);
    assert!(rows[1].focused);

    model.set_selected_node(UiNodeId::new(2)).unwrap();
    let selected = model.selected().expect("selected node details");
    assert_eq!(selected.node.display_name, "Run Button");
    assert_eq!(selected.node.lifecycle, UiWidgetLifecycleState::Interactive);
    assert_eq!(selected.properties.len(), 1);
    assert_eq!(selected.properties[0].name, "text");
    assert_eq!(
        selected.properties[0].resolved_value,
        UiValue::String("Run".to_string())
    );
    assert_eq!(
        model.export_snapshot().tree_id,
        UiTreeId::new("editor.widget.reflector")
    );
}

#[test]
fn workbench_reflection_widget_reflector_rejects_missing_selection_without_losing_current_selection(
) {
    let snapshot = reflector_snapshot();
    let mut model = WorkbenchWidgetReflectorModel::new(snapshot);

    model.set_selected_node(UiNodeId::new(1)).unwrap();
    let result = model.set_selected_node(UiNodeId::new(99));

    assert_eq!(
        result,
        Err(WorkbenchWidgetReflectorError::MissingNode(UiNodeId::new(
            99
        )))
    );
    assert_eq!(model.selected_node_id(), Some(UiNodeId::new(1)));
    assert_eq!(model.selected().unwrap().node.display_name, "Root");
}

fn reflector_snapshot() -> UiReflectorSnapshot {
    let mut root = UiReflectorNode::new(
        UiNodeId::new(1),
        UiNodePath::new("root"),
        "RootPanel",
        "Root",
    );
    root.children = vec![UiNodeId::new(2)];
    root.lifecycle = UiWidgetLifecycleState::Visible;
    root.visibility = UiVisibility::Visible;
    root.effective_visibility = UiVisibility::Visible;
    root.state_flags = visible_enabled_flags();

    let mut button = UiReflectorNode::new(
        UiNodeId::new(2),
        UiNodePath::new("root/run"),
        "MaterialButton",
        "Run Button",
    )
    .with_property(UiReflectedProperty::new(
        "text",
        UiReflectedPropertySource::Authored,
        UiValue::String("Run".to_string()),
    ));
    button.parent = Some(UiNodeId::new(1));
    button.lifecycle = UiWidgetLifecycleState::Interactive;
    button.visibility = UiVisibility::Visible;
    button.effective_visibility = UiVisibility::Visible;
    button.state_flags = visible_enabled_flags();
    button.focused = true;
    button.dirty = UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    };

    let mut snapshot = UiReflectorSnapshot::new(
        UiTreeId::new("editor.widget.reflector"),
        vec![UiNodeId::new(1)],
        vec![root, button],
    );
    snapshot.focused = Some(UiNodeId::new(2));
    snapshot
}

fn visible_enabled_flags() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

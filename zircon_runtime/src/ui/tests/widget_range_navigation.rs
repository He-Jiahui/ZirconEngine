use crate::ui::{
    dispatch::UiNavigationDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::UiNavigationEventKind,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn range_home_and_end_navigation_use_authored_min_max_aliases() {
    let mut surface = range_surface();
    let node_id = UiNodeId::new(2);
    surface.focus_node(node_id).unwrap();
    surface.clear_dirty_flags();

    let end = surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::End,
        )
        .unwrap();
    assert_eq!(end.handled_by, Some(node_id));
    assert_eq!(end.focus_changed_to, None);
    assert_range_value(&surface, 100.0);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);

    surface.clear_dirty_flags();
    let home = surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Home,
        )
        .unwrap();
    assert_eq!(home.handled_by, Some(node_id));
    assert_eq!(home.focus_changed_to, None);
    assert_range_value(&surface, 0.0);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
}

fn range_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.range.navigation"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/range"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeMeter".to_string(),
                    attributes: toml::from_str(
                        "amount = 50.0\nlow = 0.0\nhigh = 100.0\nquantum = 5.0",
                    )
                    .unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Range,
                        value_property: Some("amount".to_string()),
                        min_property: Some("low".to_string()),
                        max_property: Some("high".to_string()),
                        step_property: Some("quantum".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn assert_range_value(surface: &UiSurface, expected: f64) {
    let value = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes["amount"]
        .as_float()
        .unwrap();
    assert!(
        (value - expected).abs() < f64::EPSILON,
        "expected range value {expected}, got {value}"
    );
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        ..UiStateFlags::default()
    }
}

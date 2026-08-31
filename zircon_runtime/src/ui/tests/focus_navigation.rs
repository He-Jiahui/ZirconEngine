use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiPointerId,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    focus::{UiFocusChangeReason, UiFocusVisibleReason, UiFocusedInputKind},
    layout::{UiFrame, UiSize},
    navigation::{
        UiDirectionalNavigation, UiDirectionalNavigationTarget, UiNavigationBoundary,
        UiNavigationContract, UiNavigationGroup, UiNavigationGroupId, UiTabIndex,
    },
    surface::UiNavigationEventKind,
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

mod focus_state;
mod modal_popup;
mod property_mutation;
mod tab_directional;

fn focus_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.focus.m2"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(2, "first", 0.0, 0.0).with_focus_contract({
                let mut focus = zircon_runtime_interface::ui::focus::UiFocusContract::default();
                focus.focusable = true;
                focus.autofocus = true;
                focus
            }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(id(1), focus_node(3, "second", 90.0, 0.0))
        .unwrap();
    surface.rebuild();
    surface
}

fn mui_modal_surface(
    disable_auto_focus: bool,
    disable_enforce_focus: bool,
    disable_restore_focus: bool,
) -> UiSurface {
    mui_modal_component_surface(
        "Modal",
        "open",
        disable_auto_focus,
        disable_enforce_focus,
        disable_restore_focus,
    )
}

fn mui_modal_component_surface(
    component: &str,
    open_property: &str,
    disable_auto_focus: bool,
    disable_enforce_focus: bool,
    disable_restore_focus: bool,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.mui.modal.focus"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(id(1), focus_node(2, "outside", 0.0, 0.0))
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/modal"))
                .with_frame(UiFrame::new(0.0, 40.0, 120.0, 72.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: [
                        (open_property.to_string(), toml::Value::Boolean(false)),
                        (
                            "disable_auto_focus".to_string(),
                            toml::Value::Boolean(disable_auto_focus),
                        ),
                        (
                            "disable_enforce_focus".to_string(),
                            toml::Value::Boolean(disable_enforce_focus),
                        ),
                        (
                            "disable_restore_focus".to_string(),
                            toml::Value::Boolean(disable_restore_focus),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    widget: UiWidgetContract {
                        open_property: Some(open_property.to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(id(3), focus_node(4, "modal/first", 0.0, 48.0))
        .unwrap();
    surface
        .tree
        .insert_child(id(3), focus_node(5, "modal/second", 56.0, 48.0))
        .unwrap();
    surface.rebuild();
    surface
}

fn popup_focus_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.popup.focus.loop"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(id(1), focus_node(2, "outside", 0.0, 0.0))
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/popup"))
                .with_frame(UiFrame::new(0.0, 40.0, 120.0, 72.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(id(3), focus_node(4, "popup/first", 0.0, 48.0))
        .unwrap();
    surface
        .tree
        .insert_child(id(3), focus_node(5, "popup/second", 56.0, 48.0))
        .unwrap();
    surface.rebuild();
    surface
}

fn generic_modal_group_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.generic.modal.focus"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(id(1), focus_node(2, "outside", 0.0, 0.0))
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/drawer"))
                .with_frame(UiFrame::new(0.0, 40.0, 120.0, 72.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                })
                .with_navigation_contract(UiNavigationContract {
                    group: Some(UiNavigationGroup {
                        group_id: UiNavigationGroupId::new("drawer"),
                        root: Some(id(3)),
                        modal: true,
                        wrap: true,
                        ..Default::default()
                    }),
                    boundary: UiNavigationBoundary::Trap,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Drawer".to_string(),
                    attributes: [("open".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        open_property: Some("open".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(id(3), focus_node(4, "drawer/first", 0.0, 48.0))
        .unwrap();
    surface
        .tree
        .insert_child(id(3), focus_node(5, "drawer/second", 56.0, 48.0))
        .unwrap();
    surface.rebuild();
    surface
}

fn stacked_generic_modal_group_surface() -> UiSurface {
    let mut surface = generic_modal_group_surface();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(8), UiNodePath::new("root/top_drawer"))
                .with_frame(UiFrame::new(120.0, 40.0, 56.0, 72.0))
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..Default::default()
                })
                .with_navigation_contract(UiNavigationContract {
                    group: Some(UiNavigationGroup {
                        group_id: UiNavigationGroupId::new("top_drawer"),
                        root: Some(id(8)),
                        modal: true,
                        wrap: true,
                        ..Default::default()
                    }),
                    boundary: UiNavigationBoundary::Trap,
                    ..Default::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Drawer".to_string(),
                    attributes: [("open".to_string(), toml::Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        open_property: Some("open".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(id(8), focus_node(9, "top_drawer/first", 128.0, 48.0))
        .unwrap();
    surface.rebuild();
    surface
}

fn navigation_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.navigation.m3"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(2, "two", 0.0, 0.0).with_navigation_contract({
                let mut navigation = navigation_contract(2, 20);
                navigation.directional = Some(UiDirectionalNavigation {
                    right: UiDirectionalNavigationTarget::Node(id(5)),
                    ..Default::default()
                });
                navigation
            }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(3, "three", 40.0, 0.0).with_navigation_contract(navigation_contract(1, 10)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(5, "modal_a", 0.0, 50.0).with_navigation_contract({
                let mut navigation = navigation_contract(1, 0);
                navigation.group = Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("dialog"),
                    root: Some(id(5)),
                    modal: true,
                    wrap: true,
                    order: 0,
                    ..Default::default()
                });
                navigation.directional = Some(UiDirectionalNavigation {
                    left: UiDirectionalNavigationTarget::Blocked,
                    right: UiDirectionalNavigationTarget::Node(id(6)),
                    ..Default::default()
                });
                navigation
            }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(6, "modal_b", 40.0, 50.0).with_navigation_contract({
                let mut navigation = navigation_contract(2, 0);
                navigation.group = Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("dialog"),
                    parent: None,
                    root: Some(id(5)),
                    modal: true,
                    wrap: true,
                    order: 0,
                });
                navigation
            }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn non_modal_group_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.navigation.groups"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(2, "root_a", 0.0, 0.0).with_navigation_contract(navigation_contract(2, 0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(3, "root_b", 40.0, 0.0).with_navigation_contract(navigation_contract(1, 0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(5, "tools", 80.0, 0.0).with_navigation_contract({
                let mut navigation = navigation_contract(1, 10);
                navigation.group = Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("tools"),
                    root: Some(id(1)),
                    modal: false,
                    wrap: true,
                    order: 10,
                    ..Default::default()
                });
                navigation
            }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn root_node() -> UiTreeNode {
    UiTreeNode::new(id(1), UiNodePath::new("root")).with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0))
}

fn focus_node(id_value: u64, path: &str, x: f32, y: f32) -> UiTreeNode {
    let metadata = zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata {
        component: "TextField".to_string(),
        control_id: Some(path.to_string()),
        attributes: [
            ("editable_text".to_string(), toml::Value::Boolean(true)),
            ("value".to_string(), toml::Value::String(String::new())),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    UiTreeNode::new(id(id_value), UiNodePath::new(format!("root/{path}")))
        .with_frame(UiFrame::new(x, y, 32.0, 24.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            ..Default::default()
        })
        .with_template_metadata(metadata)
}

fn navigation_contract(order: i32, group_order: i32) -> UiNavigationContract {
    UiNavigationContract {
        tab_index: Some(UiTabIndex::new(order)),
        group: Some(UiNavigationGroup {
            group_id: UiNavigationGroupId::new("root"),
            root: Some(id(1)),
            modal: false,
            wrap: true,
            order: group_order,
            ..Default::default()
        }),
        directional: None,
        boundary: UiNavigationBoundary::Escape,
    }
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn assert_render_only_dirty(dirty: UiDirtyFlags) {
    assert!(dirty.render);
    assert!(!dirty.layout);
    assert!(!dirty.hit_test);
    assert!(!dirty.style);
    assert!(!dirty.text);
    assert!(!dirty.input);
    assert!(!dirty.visible_range);
}

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

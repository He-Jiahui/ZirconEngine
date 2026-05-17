use crate::ui::{dispatch::UiPointerDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityContract,
    },
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiInputEvent, UiInputEventMetadata,
        UiPointerEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiAxis, UiContainerKind, UiFrame, UiScrollState, UiScrollableBoxConfig},
    surface::{UiPointerButton, UiPointerEventKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: false,
        ..UiStateFlags::default()
    }
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: true,
        ..UiStateFlags::default()
    }
}

fn scrollbar_metadata(scroll_target: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Scrollbar".to_string(),
        widget: UiWidgetContract {
            behavior: UiWidgetBehavior::Scrollbar,
            scroll_target: Some(scroll_target.to_string()),
            scroll_axis: Some(UiAxis::Vertical),
            min_thumb_extent: Some(16.0),
            ..UiWidgetContract::default()
        },
        ..UiTemplateNodeMetadata::default()
    }
}

fn thumb_metadata() -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "ScrollbarThumb".to_string(),
        widget: UiWidgetContract {
            behavior: UiWidgetBehavior::ScrollbarThumb,
            ..UiWidgetContract::default()
        },
        ..UiTemplateNodeMetadata::default()
    }
}

fn scrollable_node() -> UiTreeNode {
    UiTreeNode::new(id(2), UiNodePath::new("root/scroll"))
        .with_frame(UiFrame::new(0.0, 0.0, 100.0, 100.0))
        .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: UiAxis::Vertical,
            ..UiScrollableBoxConfig::default()
        }))
        .with_scroll_state(UiScrollState {
            offset: 0.0,
            viewport_extent: 100.0,
            content_extent: 300.0,
        })
        .with_state_flags(focusable_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "ScrollableBox".to_string(),
            a11y: UiAccessibilityContract {
                name: Some("Results".to_string()),
                ..UiAccessibilityContract::default()
            },
            ..UiTemplateNodeMetadata::default()
        })
}

fn scrollbar_surface(include_thumb: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.scrollbar"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0)),
    );
    surface.tree.insert_child(id(1), scrollable_node()).unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/scrollbar"))
                .with_frame(UiFrame::new(120.0, 0.0, 12.0, 100.0))
                .with_state_flags(pointer_state())
                .with_template_metadata(scrollbar_metadata("#2")),
        )
        .unwrap();
    if include_thumb {
        surface
            .tree
            .insert_child(
                id(3),
                UiTreeNode::new(id(4), UiNodePath::new("root/scrollbar/thumb"))
                    .with_frame(UiFrame::new(120.0, 60.0, 12.0, 20.0))
                    .with_state_flags(pointer_state())
                    .with_template_metadata(thumb_metadata()),
            )
            .unwrap();
    }
    surface.rebuild();
    surface
}

fn click_primary(surface: &mut UiSurface, x: f32, y: f32) {
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(
                UiPointerEventKind::Down,
                zircon_runtime_interface::ui::layout::UiPoint::new(x, y),
            )
            .with_button(UiPointerButton::Primary),
        )
        .unwrap();
}

#[test]
fn scrollbar_track_click_pages_target_scroll_container() {
    let mut surface = scrollbar_surface(false);

    click_primary(&mut surface, 126.0, 80.0);
    let result = surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(
                UiPointerEventKind::Up,
                zircon_runtime_interface::ui::layout::UiPoint::new(126.0, 80.0),
            )
            .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(id(3)));
    assert!(result.diagnostics.scroll_defaulted);
    assert!(result.component_events.is_empty());
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        100.0
    );
}

#[test]
fn scrollbar_thumb_click_does_not_page_scroll_container() {
    let mut surface = scrollbar_surface(true);

    click_primary(&mut surface, 126.0, 70.0);
    let result = surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(
                UiPointerEventKind::Up,
                zircon_runtime_interface::ui::layout::UiPoint::new(126.0, 70.0),
            )
            .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_ne!(result.handled_by, Some(id(3)));
    assert!(!result.diagnostics.scroll_defaulted);
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        0.0
    );
}

#[test]
fn scrollbar_is_headless_in_accessibility_unless_authored_explicitly() {
    let mut surface = scrollbar_surface(false);

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(3)).is_none());

    let metadata = surface
        .tree
        .node_mut(id(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    metadata.a11y = UiAccessibilityContract {
        role: UiA11yRole::Scrollbar,
        name: Some("Results scroll bar".to_string()),
        ..UiAccessibilityContract::default()
    };

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(3))
        .expect("explicitly-authored scrollbar a11y node is retained");

    assert_eq!(node.role, UiA11yRole::Scrollbar);
    assert!(node.actions.is_empty());
}

#[test]
fn accessibility_scroll_to_mutates_scrollable_container_offset() {
    let mut surface = scrollbar_surface(false);

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &crate::ui::dispatch::UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request: UiAccessibilityActionRequest {
                    target: id(2),
                    action: UiAccessibilityAction::ScrollTo,
                    numeric_value: Some(64.0),
                    ..UiAccessibilityActionRequest::default()
                },
            }),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.scroll_to")
    );
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        64.0
    );
}

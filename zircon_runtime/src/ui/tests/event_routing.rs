use crate::ui::template::{UiTemplateInstance, UiTemplateLoader, UiTemplateSurfaceBuilder};
use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchDisposition, UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchReply,
    UiFocusEffectReason, UiImeDeleteSurrounding, UiImeInputEvent, UiImeInputEventKind,
    UiInputEvent, UiInputEventMetadata, UiInputMethodRequest, UiInputMethodRequestKind,
    UiInputMethodSurroundingText, UiInputRoutePolicy, UiInputSequence, UiInputTimestamp,
    UiKeyboardInputEvent, UiKeyboardInputState, UiNavigationRequestPolicy, UiPointerCaptureReason,
    UiPointerComponentEventReason, UiPointerDispatchEffect, UiPointerEvent, UiPointerId,
    UiPointerInputEvent, UiPointerLockPolicy, UiPointerSource, UiPreciseScrollDelta,
    UiTextInputEvent,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentEventKind, UiValue},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAxis, UiContainerKind, UiFrame, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
    },
    surface::{
        UiHitTestQuery, UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
        UiVirtualPointerPosition,
    },
    template::UiBindingRef,
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
};

mod component_events;
mod dispatch_effects;
mod pointer_state;
mod shared_input;

#[test]
fn pointer_dispatch_does_not_clone_an_unused_hover_path() {
    let source = include_str!("../surface/surface/event_routing.rs");

    assert!(
        !source.contains("_hover_before_dispatch"),
        "pointer dispatch must not clone the hovered path before routing when that copy is unused"
    );
}

#[test]
fn default_table_sort_borrows_common_scalar_text() {
    let source = include_str!("../surface/surface/default_interactions/table/columns.rs");

    assert!(
        source.contains("left.and_then(borrowed_sort_text)"),
        "default table sorting should compare common string-like values without allocating display text"
    );
}

fn capture_pointer_for_test(surface: &mut UiSurface, pointer_id: UiPointerId, owner: UiNodeId) {
    surface.focus.captured = Some(owner);
    surface.input.set_pointer_capture_for_id(pointer_id, owner);
}

fn assert_no_pointer_capture(surface: &UiSurface) {
    assert_eq!(surface.input.active_pointer_capture(), None);
}

fn button_surface() -> UiSurface {
    button_surface_with_metadata(None)
}

fn bound_button_surface(bindings: Vec<UiBindingRef>) -> UiSurface {
    button_surface_with_metadata(Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("MaterialButton".to_string()),
        bindings,
        ..Default::default()
    }))
}

fn two_button_surface(
    first_metadata: Option<UiTemplateNodeMetadata>,
    second_metadata: Option<UiTemplateNodeMetadata>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    let mut first = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/first"))
        .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state());
    if let Some(metadata) = first_metadata {
        first = first.with_template_metadata(metadata);
    }
    surface.tree.insert_child(UiNodeId::new(1), first).unwrap();
    let mut second = UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/second"))
        .with_frame(UiFrame::new(10.0, 50.0, 80.0, 30.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state());
    if let Some(metadata) = second_metadata {
        second = second.with_template_metadata(metadata);
    }
    surface.tree.insert_child(UiNodeId::new(1), second).unwrap();
    surface.rebuild();
    surface
}

fn scrollable_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            },
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 0,
                    }),
                }))
                .with_scroll_state(UiScrollState::default())
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(20 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
                .with_state_flags(pointer_state()),
            )
            .unwrap();
    }
    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();
    surface
}

fn nested_scrollable_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/outer"))
                .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: 0.0,
                    viewport_extent: 100.0,
                    content_extent: 200.0,
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/outer/inner"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 80.0))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: 0.0,
                    viewport_extent: 80.0,
                    content_extent: 80.0,
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn button_surface_with_metadata(template_metadata: Option<UiTemplateNodeMetadata>) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    let mut button = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
        .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state());
    if let Some(template_metadata) = template_metadata {
        button = button.with_template_metadata(template_metadata);
    }
    surface.tree.insert_child(UiNodeId::new(1), button).unwrap();
    surface.rebuild();
    surface
}

fn editable_text_surface(value: &str, caret_offset: usize) -> UiSurface {
    button_surface_with_metadata(Some(UiTemplateNodeMetadata {
        component: "TextField".to_string(),
        control_id: Some("EditableText".to_string()),
        bindings: vec![
            binding("EditableText/Change", UiEventKind::Change),
            binding("EditableText/Submit", UiEventKind::Submit),
        ],
        attributes: toml::from_str(&format!(
            r#"
value = "{}"
caret_offset = {}
editable_text = true
"#,
            value, caret_offset
        ))
        .unwrap(),
        ..Default::default()
    }))
}

fn editable_attr_string(surface: &UiSurface, key: &str) -> String {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
}

fn editable_attr_usize(surface: &UiSurface, key: &str) -> usize {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| value.as_integer())
        .unwrap_or_default() as usize
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn template_surface_from_root_toml(root: String) -> UiSurface {
    let document =
        UiTemplateLoader::load_toml_str(&format!("version = 1\n\n[root]\n{root}")).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();
    UiTemplateSurfaceBuilder::build_surface(UiTreeId::new("runtime.ui.events"), &instance).unwrap()
}

fn root_with_inline_node(node: &str) -> String {
    format!("template = \"Root\"\n\n[components.Root]\nroot = {node}")
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: input_metadata(),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "A".to_string(),
        text: Some("a".to_string()),
    })
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
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

fn stretch_constraint(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

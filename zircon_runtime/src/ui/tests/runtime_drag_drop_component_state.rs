use std::collections::BTreeMap;

use crate::ui::surface::UiSurface;
use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentFlags,
    dispatch::{
        UiDispatchEffect, UiDispatchReply, UiDragDropEffectKind, UiDragSessionId, UiInputEvent,
        UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
        UiKeyboardInputState, UiPointerId,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiInputPolicy, UiTreeNode},
    v2::{
        UiV2AssetDocument, UiV2AssetHeader, UiV2AssetKind, UiV2ChildMount, UiV2NodeDefinition,
        UiV2Root, UiV2StyleDeclarationBlock, UiV2StyleRule, UiV2StyleSheet,
        UI_V2_ASSET_SCHEMA_VERSION,
    },
};

use crate::ui::v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder};

#[test]
fn drag_drop_effects_project_source_and_target_component_flags() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);

    let begin = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Begin,
            id(2),
            pointer_id,
            Some(session_id),
        )),
    );

    assert!(begin.rejected_effects.is_empty());
    assert_drag_flags(&surface, id(2), true, true, true);
    assert!(surface.tree.nodes[&id(2)].dirty.render);

    let update = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            id(3),
            pointer_id,
            Some(session_id),
        )),
    );

    assert!(update.rejected_effects.is_empty());
    assert_drag_flags(&surface, id(2), true, false, false);
    assert_drag_flags(&surface, id(3), false, true, true);

    let complete = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Complete,
            id(3),
            pointer_id,
            Some(session_id),
        )),
    );

    assert!(complete.rejected_effects.is_empty());
    assert_eq!(surface.input.drag_drop, None);
    assert_drag_flags(&surface, id(2), false, false, false);
    assert_drag_flags(&surface, id(3), false, false, false);
}

#[test]
fn rejected_drag_drop_update_preserves_existing_component_flags() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);

    let begin = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Begin,
            id(2),
            pointer_id,
            Some(session_id),
        )),
    );
    assert!(begin.rejected_effects.is_empty());

    let stale_session = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            id(3),
            pointer_id,
            Some(UiDragSessionId::new(99)),
        )),
    );

    assert_eq!(stale_session.rejected_effects.len(), 1);
    assert_eq!(
        stale_session.rejected_effects[0].reason,
        "drag session owner mismatch"
    );
    assert_eq!(
        surface.input.drag_drop.as_ref().map(|drag| drag.target),
        Some(id(2))
    );
    assert_drag_flags(&surface, id(2), true, true, true);
    assert_drag_flags(&surface, id(3), false, false, false);
}

#[test]
fn drag_drop_effects_project_retained_state_to_v2_style_and_generic_painter() {
    let mut surface = drag_drop_v2_surface();
    let source = node_id_by_control_id(&surface, "DragSourcePanel");
    let target = node_id_by_control_id(&surface, "DropTargetPanel");
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);

    assert_eq!(
        runtime_attr(&surface, source, "background_color"),
        Some("#101010")
    );
    assert_eq!(
        runtime_attr(&surface, target, "background_color"),
        Some("#101010")
    );
    assert!(runtime_bool_attr(&surface, source, "dragging").is_none());
    assert_eq!(
        generic_surface(&surface, source).style.painter_state,
        UiPainterResolvedState::Normal
    );

    let begin = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Begin,
            source,
            pointer_id,
            Some(session_id),
        )),
    );
    assert!(begin.rejected_effects.is_empty());

    let update = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            target,
            pointer_id,
            Some(session_id),
        )),
    );
    assert!(update.rejected_effects.is_empty());

    assert_drag_flags(&surface, source, true, false, false);
    assert_drag_flags(&surface, target, false, true, true);
    assert_eq!(runtime_bool_attr(&surface, source, "dragging"), Some(true));
    assert!(runtime_bool_attr(&surface, source, "drop_hovered").is_none());
    assert!(runtime_bool_attr(&surface, source, "active_drag_target").is_none());
    assert_eq!(
        runtime_attr(&surface, source, "background_color"),
        Some("#3030a0")
    );
    assert_eq!(
        runtime_bool_attr(&surface, target, "drop_hovered"),
        Some(true)
    );
    assert_eq!(
        runtime_bool_attr(&surface, target, "active_drag_target"),
        Some(true)
    );
    assert!(runtime_bool_attr(&surface, target, "dragging").is_none());
    assert_eq!(
        runtime_attr(&surface, target, "background_color"),
        Some("#205020")
    );
    assert_eq!(
        runtime_attr(&surface, target, "border_color"),
        Some("#90ffb0")
    );

    let source_dirty = surface.tree.nodes.get(&source).unwrap().dirty;
    assert!(source_dirty.render);
    assert!(!source_dirty.style);
    assert!(!source_dirty.text);
    let target_dirty = surface.tree.nodes.get(&target).unwrap().dirty;
    assert!(target_dirty.render);
    assert!(!target_dirty.style);
    assert!(!target_dirty.text);

    surface.rebuild();
    let source_command = generic_surface(&surface, source);
    assert_eq!(
        source_command.style.painter_state,
        UiPainterResolvedState::Dragging
    );
    assert_eq!(
        source_command.style.background_color.as_deref(),
        Some("#3030a0")
    );
    let target_command = generic_surface(&surface, target);
    assert_eq!(
        target_command.style.painter_state,
        UiPainterResolvedState::DropHovered
    );
    assert_eq!(
        target_command.style.background_color.as_deref(),
        Some("#205020")
    );
    assert_eq!(
        target_command.style.border_color.as_deref(),
        Some("#90ffb0")
    );

    surface.clear_dirty_flags();
    let complete = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Complete,
            target,
            pointer_id,
            Some(session_id),
        )),
    );
    assert!(complete.rejected_effects.is_empty());
    assert_drag_flags(&surface, source, false, false, false);
    assert_drag_flags(&surface, target, false, false, false);
    assert!(runtime_bool_attr(&surface, source, "dragging").is_none());
    assert!(runtime_bool_attr(&surface, target, "drop_hovered").is_none());
    assert!(runtime_bool_attr(&surface, target, "active_drag_target").is_none());
    assert_eq!(
        runtime_attr(&surface, source, "background_color"),
        Some("#101010")
    );
    assert_eq!(
        runtime_attr(&surface, target, "background_color"),
        Some("#101010")
    );
    assert!(runtime_attr(&surface, target, "border_color").is_none());

    surface.rebuild();
    assert_eq!(
        generic_surface(&surface, source).style.painter_state,
        UiPainterResolvedState::Normal
    );
    assert_eq!(
        generic_surface(&surface, target).style.painter_state,
        UiPainterResolvedState::Normal
    );
}

fn two_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.drag.component_state"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/first"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/second"))
                .with_frame(UiFrame::new(10.0, 50.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn drag_drop_v2_surface() -> UiSurface {
    let mut document = v2_document("asset://ui/tests/runtime_drag_drop.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "VerticalBox".to_string(),
            control_id: Some("DragDropRoot".to_string()),
            layout: Some(fixed_size_layout(140.0, 76.0)),
            children: vec![child_mount("source"), child_mount("target")],
            ..Default::default()
        },
    );
    document.nodes.insert(
        "source".to_string(),
        UiV2NodeDefinition {
            component: "Panel".to_string(),
            control_id: Some("DragSourcePanel".to_string()),
            classes: vec!["drag-surface".to_string(), "drag-source".to_string()],
            props: interactive_props(),
            layout: Some(fixed_size_layout(120.0, 28.0)),
            ..Default::default()
        },
    );
    document.nodes.insert(
        "target".to_string(),
        UiV2NodeDefinition {
            component: "Panel".to_string(),
            control_id: Some("DropTargetPanel".to_string()),
            classes: vec!["drag-surface".to_string(), "drop-target".to_string()],
            props: interactive_props(),
            layout: Some(fixed_size_layout(120.0, 28.0)),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_drag_drop_material".to_string(),
        rules: vec![
            style_rule("Panel.drag-surface", [("background_color", "#101010")]),
            style_rule(
                "Panel.drag-source:dragging",
                [("background_color", "#3030a0")],
            ),
            style_rule(
                "Panel.drop-target:drop_hovered",
                [("background_color", "#205020")],
            ),
            style_rule(
                "Panel.drop-target:active_drag_target",
                [("border_color", "#90ffb0")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_drag_drop"),
        &document,
        &compiled,
    )
    .unwrap();
    surface.rebuild();
    surface.clear_dirty_flags();
    surface
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

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn v2_document(asset_id: &str, root: &str) -> UiV2AssetDocument {
    UiV2AssetDocument {
        asset: UiV2AssetHeader {
            kind: UiV2AssetKind::View,
            id: asset_id.to_string(),
            version: UI_V2_ASSET_SCHEMA_VERSION,
            display_name: String::new(),
        },
        root: Some(UiV2Root {
            node: root.to_string(),
        }),
        imports: Default::default(),
        tokens: BTreeMap::new(),
        nodes: BTreeMap::new(),
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

fn child_mount(node: &str) -> UiV2ChildMount {
    UiV2ChildMount {
        node: node.to_string(),
        slot: BTreeMap::new(),
    }
}

fn interactive_props() -> BTreeMap<String, Value> {
    BTreeMap::from([("input_interactive".to_string(), Value::Boolean(true))])
}

fn style_rule<'a, const N: usize>(
    selector: &str,
    values: [(&'a str, &'a str); N],
) -> UiV2StyleRule {
    UiV2StyleRule {
        id: None,
        selector: selector.to_string(),
        set: UiV2StyleDeclarationBlock {
            self_values: values
                .into_iter()
                .map(|(key, value)| (key.to_string(), Value::String(value.to_string())))
                .collect(),
            slot: BTreeMap::new(),
        },
    }
}

fn fixed_size_layout(width: f64, height: f64) -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "width".to_string(),
            Value::Table(fixed_axis_constraint(width)),
        ),
        (
            "height".to_string(),
            Value::Table(fixed_axis_constraint(height)),
        ),
    ])
}

fn fixed_axis_constraint(value: f64) -> toml::map::Map<String, Value> {
    toml::map::Map::from_iter([
        ("min".to_string(), Value::Float(value)),
        ("preferred".to_string(), Value::Float(value)),
        ("max".to_string(), Value::Float(value)),
        ("stretch".to_string(), Value::String("Fixed".to_string())),
    ])
}

fn node_id_by_control_id(surface: &UiSurface, control_id: &str) -> UiNodeId {
    surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should be projected"))
        .node_id
}

fn runtime_attr<'a>(surface: &'a UiSurface, node_id: UiNodeId, key: &str) -> Option<&'a str> {
    surface
        .tree
        .nodes
        .get(&node_id)?
        .template_metadata
        .as_ref()?
        .attributes
        .get(key)
        .and_then(Value::as_str)
}

fn runtime_bool_attr(surface: &UiSurface, node_id: UiNodeId, key: &str) -> Option<bool> {
    surface
        .tree
        .nodes
        .get(&node_id)?
        .template_metadata
        .as_ref()?
        .attributes
        .get(key)
        .and_then(Value::as_bool)
}

fn generic_surface(surface: &UiSurface, node_id: UiNodeId) -> &UiRenderCommand {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.style.painter_family == UiPainterFamily::Generic
        })
        .unwrap()
}

fn drag_effect(
    kind: UiDragDropEffectKind,
    target: UiNodeId,
    pointer_id: UiPointerId,
    session_id: Option<UiDragSessionId>,
) -> UiDispatchEffect {
    UiDispatchEffect::DragDrop {
        kind,
        target,
        pointer_id,
        session_id,
        point: Some(UiPoint::new(20.0, 20.0)),
        payload: None,
    }
}

fn input_state() -> UiStateFlags {
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

fn assert_drag_flags(
    surface: &UiSurface,
    node_id: UiNodeId,
    dragging: bool,
    drop_hovered: bool,
    active_drag_target: bool,
) {
    let flags = surface
        .component_state(node_id)
        .map(|state| state.flags.clone())
        .unwrap_or_default();
    assert_eq!(
        flags,
        UiComponentFlags {
            dragging,
            drop_hovered,
            active_drag_target,
            ..UiComponentFlags::default()
        }
    );
}

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

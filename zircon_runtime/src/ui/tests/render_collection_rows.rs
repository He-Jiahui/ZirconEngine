use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_collection_row_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.collection_rows"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 360.0, 180.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "ListRow",
        UiFrame::new(12.0, 16.0, 180.0, 28.0),
        r##"
label = "Health Regen"
selected = true
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "TreeRow",
        UiFrame::new(12.0, 52.0, 240.0, 24.0),
        r##"
label = "LevelRoot"
selected = true
expanded = true
tree_depth = 1.0
icon = "box"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "TableRow",
        UiFrame::new(12.0, 88.0, 240.0, 28.0),
        r##"
label = "Asset_01"
selected = true
background_color = "#0d4149"
"##,
        UiStateFlags {
            enabled: true,
            visible: true,
            pressed: true,
            ..UiStateFlags::default()
        },
    );
    insert_control(
        &mut surface,
        UiNodeId::new(5),
        "TreeRow",
        UiFrame::new(12.0, 124.0, 240.0, 24.0),
        r##"
label = "RuntimeFolder"
expanded = false
icon = "folder"
"##,
        visible_state(),
    );
    assert!(surface
        .component_states
        .set_expanded(UiNodeId::new(5), true));
    surface
        .mark_component_state_render_dirty(UiNodeId::new(5))
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::ListRow
            && command.style.painter_state == UiPainterResolvedState::Selected
            && command.style.background_color.as_deref() == Some("#0d4149")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Health Regen")
            && command.style.painter_family == UiPainterFamily::ListRow
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.text.as_deref() == Some("Health Regen")
            })
            .count(),
        1
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-down".to_string()))
            && command.style.painter_family == UiPainterFamily::TreeRow
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(3)
                    && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("box".to_string()))
            })
            .count(),
        1
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("LevelRoot")
            && command.style.painter_family == UiPainterFamily::TreeRow
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::TableRow
            && command.style.painter_state == UiPainterResolvedState::Pressed
            && command.style.background_color.as_deref() == Some("#0d4149")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Asset_01")
            && command.style.painter_family == UiPainterFamily::TableRow
            && command.style.painter_state == UiPainterResolvedState::Pressed
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(5)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-down".to_string()))
            && command.style.painter_family == UiPainterFamily::TreeRow
            && command.style.painter_state == UiPainterResolvedState::Normal
    }));
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(5)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("chevron-right".to_string()))
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(4) && command.text.as_deref() == Some("Asset_01")
            })
            .count(),
        1
    );
}

fn insert_control(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new(format!("root/{component}")))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_slider_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.sliders"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/slider"))
                .with_frame(UiFrame::new(8.0, 12.0, 240.0, 30.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RangeField".to_string(),
                    attributes: toml::from_str(
                        r##"
label = "Value"
value = 75.0
min = 0.0
max = 100.0
value_text = "0.75"
tick_count = 5
thumb_size = 11.0
track_color = "#364046"
value_color = "#35c7d0"
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Value")
            && command.frame == UiFrame::new(16.0, 20.4, 50.0, 13.200001)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 25.5, 108.0, 3.0)
            && command.style.background_color.as_deref() == Some("#364046")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 25.5, 81.0, 3.0)
            && command.style.background_color.as_deref() == Some("#35c7d0")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.kind == UiRenderCommandKind::Quad
                    && command.frame.y == 33.5
                    && command.frame.width == 1.0
                    && command.frame.height == 4.0
            })
            .count(),
        5
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(153.5, 21.5, 11.0, 11.0)
            && command.style.background_color.as_deref() == Some("#c9f2f6")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("0.75")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Value")
            })
            .count(),
        1
    );
}

#[test]
fn render_extract_uses_shared_selector_for_slider_drop_hover_halo() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.sliders.selector"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 64.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/slider"))
                .with_frame(UiFrame::new(8.0, 12.0, 160.0, 30.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RangeField".to_string(),
                    attributes: toml::from_str(
                        r##"
value_percent = 0.5
drop_hovered = true
"##,
                    )
                    .unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.background_color.as_deref() == Some("#35c7d03a")
            && command.frame.width == 20.0
            && command.frame.height == 20.0
    }));
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_expands_tabs_and_segmented_control_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.segmented_controls"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 120.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "SegmentedControl",
        UiFrame::new(12.0, 8.0, 150.0, 48.0),
        r##"
label = "Mode"
value = "center"
options = ["left", "center", "right"]
selected_underline_height = 1.0
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Tab",
        UiFrame::new(12.0, 68.0, 132.0, 32.0),
        r##"
text = "UI Components"
checked = true
selected = true
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Mode")
            && command.frame == UiFrame::new(12.0, 8.0, 150.0, 14.0)
            && command.style.foreground_color.as_deref() == Some("#a1acb2")
            && command.style.painter_family == UiPainterFamily::Tab
            && command.style.painter_state == UiPainterResolvedState::Normal
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 26.0, 150.0, 30.0)
            && command.style.background_color.as_deref() == Some("#1d2327")
            && command.style.border_color.as_deref() == Some("#323a41")
            && command.style.corner_radius == 5.0
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 28.0, 46.0, 26.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.is_none()
            && command.style.border_width == 0.0
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 53.0, 46.0, 1.0)
            && command.style.background_color.as_deref() == Some("#2aa6b8")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Center")
            && command.frame == UiFrame::new(70.0, 31.0, 34.0, 20.0)
            && command.style.foreground_color.as_deref() == Some("#e6f1f4")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Mode")
            })
            .count(),
        1
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Center")
            })
            .count(),
        1
    );

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 98.0, 132.0, 2.0)
            && command.style.background_color.as_deref() == Some("#2aa6b8")
            && command.style.painter_family == UiPainterFamily::Tab
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("UI Components")
            && command.frame == UiFrame::new(24.0, 76.8, 108.0, 14.400001)
            && command.style.foreground_color.as_deref() == Some("#e6f1f4")
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(3)
                    && command.text.as_deref() == Some("UI Components")
            })
            .count(),
        1
    );
}

#[test]
fn render_extract_segmented_defaults_to_slate_underline_selected_indicator() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.segmented_controls.selected_defaults",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 190.0, 56.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "SegmentedControl",
        UiFrame::new(12.0, 8.0, 150.0, 30.0),
        r##"
value = "center"
options = ["left", "center", "right"]
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 10.0, 46.0, 26.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.is_none()
            && command.style.border_width == 0.0
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 34.0, 46.0, 2.0)
            && command.style.background_color.as_deref() == Some("#2aa6b8")
    }));
}

#[test]
fn render_extract_loading_tabs_and_segmented_controls_use_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.segmented_controls.loading",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 280.0, 128.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "SegmentedControl",
        UiFrame::new(12.0, 8.0, 150.0, 48.0),
        r##"
label = "Mode"
value = "center"
options = ["left", "center", "right"]
selected = true
checked = true
loading = true
hovered = true
focused = true
pressed = true
background_color = "#1d2327"
border_color = "#4b626d"
selected_background_color = "#0f6574"
selected_border_color = "#35c7d0"
selected_underline_color = "#35c7d0"
selected_underline_height = 1.0
foreground_color = "#8fa3ac"
selected_foreground_color = "#e6f1f4"
label_color = "#a1acb2"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Tab",
        UiFrame::new(12.0, 72.0, 132.0, 32.0),
        r##"
text = "UI Components"
checked = true
selected = true
loading = true
hovered = true
pressed = true
background_color = "#1d2327"
selected_underline_color = "#35c7d0"
selected_foreground_color = "#e6f1f4"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Mode")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#58656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 26.0, 150.0, 30.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#191d22")
            && command.style.border_color.as_deref() == Some("#334852")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 28.0, 46.0, 26.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#191d22")
            && command.style.border_color.is_none()
            && command.style.border_width == 0.0
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 53.0, 46.0, 1.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#58656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Center")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#58656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 72.0, 132.0, 32.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#191d22")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 102.0, 132.0, 2.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#58656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("UI Components")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#58656c")
    }));
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

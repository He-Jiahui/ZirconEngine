use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
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

#[test]
fn render_extract_expands_range_slider_dual_thumb_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.range-sliders"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/range-slider"))
                .with_frame(UiFrame::new(8.0, 12.0, 260.0, 46.0))
                .with_state_flags(visible_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RangeSlider".to_string(),
                    attributes: toml::from_str(
                        r##"
label_text = "Range"
range_min_percent = 0.2
value_percent = 0.8
value_text = "0.80"
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
            && command.text.as_deref() == Some("Range")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 33.5, 128.0, 3.0)
            && command.style.background_color.as_deref() == Some("#364046")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && frame_approx(command.frame, 103.6, 33.5, 76.8, 3.0)
            && command.style.background_color.as_deref() == Some("#35c7d0")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.kind == UiRenderCommandKind::Quad
                    && command.frame.width == 11.0
                    && command.frame.height == 11.0
                    && command.style.background_color.as_deref() == Some("#c9f2f6")
            })
            .count(),
        2
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("0.20")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("0.80")
    }));
}

#[test]
fn render_extract_loading_slider_uses_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.sliders.loading"));
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
                .with_state_flags(UiStateFlags {
                    pressed: true,
                    ..visible_state()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RangeField".to_string(),
                    attributes: toml::from_str(
                        r##"
label = "Value"
value_percent = 0.75
value_text = "0.75"
tick_count = 5
pressed = true
focused = true
drop_hovered = true
loading = true
validation_level = "warning"
track_color = "#364046"
value_color = "#35c7d0"
thumb_color = "#c9f2f6"
thumb_outline_color = "#35c7d0"
label_color = "#aebdc4"
foreground_color = "#d9fbff"
state_layer_color = "#35c7d03a"
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
            && command.frame == UiFrame::new(78.0, 25.5, 108.0, 3.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#262d32")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 25.5, 81.0, 3.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(153.5, 21.5, 11.0, 11.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#59656c")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(196.0, 15.0, 44.0, 24.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Value")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("0.75")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#59656c")
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
                    && command.style.background_color.as_deref() == Some("#343f47")
            })
            .count(),
        5
    );
    assert!(!commands.iter().any(|command| {
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

fn frame_approx(actual: UiFrame, x: f32, y: f32, width: f32, height: f32) -> bool {
    (actual.x - x).abs() < 0.01
        && (actual.y - y).abs() < 0.01
        && (actual.width - width).abs() < 0.01
        && (actual.height - height).abs() < 0.01
}

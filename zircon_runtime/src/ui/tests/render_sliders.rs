use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
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
            && command.frame == UiFrame::new(78.0, 25.0, 108.0, 4.0)
            && command.style.background_color.as_deref() == Some("#11161a")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 25.0, 81.0, 4.0)
            && command.style.background_color.as_deref() == Some("#414b54")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.kind == UiRenderCommandKind::Quad
                    && command.frame.y == 33.0
                    && command.frame.width == 1.0
                    && command.frame.height == 4.0
            })
            .count(),
        5
    );
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(155.0, 23.0, 8.0, 8.0)
            && command.style.background_color.as_deref() == Some("#e8ecee")
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
            && command.style.background_color.as_deref() == Some("#d8e3e71a")
            && command.frame.width == 16.0
            && command.frame.height == 16.0
    }));
}

#[test]
fn render_extract_slider_keeps_focused_value_border_neutral_with_focus_halo() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.sliders.focused_value_border",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 104.0))
            .with_state_flags(visible_state()),
    );
    insert_slider(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 12.0, 240.0, 30.0),
        r##"
label = "Focus"
value_percent = 0.5
value_text = "0.50"
"##,
        visible_state(),
    );
    insert_slider(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(8.0, 52.0, 240.0, 30.0),
        r##"
label = "Press"
value_percent = 0.5
value_text = "0.50"
"##,
        pressed_state(),
    );
    set_focused(&mut surface, UiNodeId::new(2));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let focused_halo = slider_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(124.0, 19.0, 16.0, 16.0),
    );
    assert_eq!(
        focused_halo.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_halo.style.background_color.as_deref(),
        Some("#d8e3e71a")
    );

    let focused_value = slider_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(196.0, 15.0, 44.0, 24.0),
    );
    assert_eq!(
        focused_value.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(focused_value.style.border_color.as_deref(), Some("#2d3940"));

    let pressed_halo = slider_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(124.0, 59.0, 16.0, 16.0),
    );
    assert_eq!(
        pressed_halo.style.painter_state,
        UiPainterResolvedState::Pressed
    );
    assert_eq!(
        pressed_halo.style.background_color.as_deref(),
        Some("#d8e3e71a")
    );

    let pressed_value = slider_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(196.0, 55.0, 44.0, 24.0),
    );
    assert_eq!(
        pressed_value.style.painter_state,
        UiPainterResolvedState::Pressed
    );
    assert_eq!(pressed_value.style.border_color.as_deref(), Some("#414b54"));
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
            && command.frame == UiFrame::new(78.0, 33.0, 128.0, 4.0)
            && command.style.background_color.as_deref() == Some("#11161a")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && frame_approx(command.frame, 103.6, 33.0, 76.8, 4.0)
            && command.style.background_color.as_deref() == Some("#414b54")
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.kind == UiRenderCommandKind::Quad
                    && command.frame.width == 8.0
                    && command.frame.height == 8.0
                    && command.style.background_color.as_deref() == Some("#e8ecee")
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
track_color = "#2a3338"
value_color = "#414b54"
thumb_color = "#e8ecee"
thumb_outline_color = "#2d3940"
label_color = "#aebdc4"
foreground_color = "#d9fbff"
state_layer_color = "#d8e3e71a"
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
            && command.frame == UiFrame::new(78.0, 25.0, 108.0, 4.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#262d32")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 25.0, 81.0, 4.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(155.0, 23.0, 8.0, 8.0)
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
                    && command.frame.y == 33.0
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
            && command.style.background_color.as_deref() == Some("#d8e3e71a")
            && command.frame.width == 16.0
            && command.frame.height == 16.0
    }));
}

fn insert_slider(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new("root/slider"))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RangeField".to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn slider_quad(
    commands: &[UiRenderCommand],
    node_id: UiNodeId,
    frame: UiFrame,
) -> &UiRenderCommand {
    commands
        .iter()
        .find(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.frame == frame
        })
        .expect("expected slider quad")
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

fn pressed_state() -> UiStateFlags {
    UiStateFlags {
        pressed: true,
        ..visible_state()
    }
}

fn set_focused(surface: &mut UiSurface, node_id: UiNodeId) {
    assert!(surface.component_states.set_focused(node_id, true));
}

fn frame_approx(actual: UiFrame, x: f32, y: f32, width: f32, height: f32) -> bool {
    (actual.x - x).abs() < 0.01
        && (actual.y - y).abs() < 0.01
        && (actual.width - width).abs() < 0.01
        && (actual.height - height).abs() < 0.01
}

use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    design_tokens::EditorTypographyTokens,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, MAX_UI_SLIDER_TICK_COUNT},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn slider_rendering_classifies_before_visual_resolution_and_uses_shared_tokens() {
    let source = include_str!("../surface/render/sliders.rs");
    let classification = source
        .find("if !is_slider(metadata)")
        .expect("slider renderer should classify the component");
    let visual = source
        .find("let visual = SliderVisual::resolve")
        .expect("slider renderer should resolve the visual model");

    assert!(
        classification < visual,
        "non-slider nodes should exit before visual resolution"
    );
    for required_hook in [
        "EditorDesignTokens",
        "EditorTypographyTokens",
        "style_overrides",
        "default_slider_visual",
        "ui_slider_tick_count_for_track",
        "mod state_colors",
    ] {
        assert!(
            source.contains(required_hook),
            "slider renderer should retain {required_hook}"
        );
    }
}

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
            && command.frame == UiFrame::new(16.0, 19.0, 50.0, 16.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 25.0, 108.0, 4.0)
            && command.style.background_color.as_deref() == Some("#111416")
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
            && command.style.background_color.as_deref() == Some("#e8ecee20")
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
        Some("#e8ecee20")
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
    assert_eq!(focused_value.style.border_color.as_deref(), Some("#323a41"));

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
        Some("#e8ecee20")
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
            && command.style.background_color.as_deref() == Some("#111416")
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
            && command.style.background_color.as_deref() == Some("#22272b")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(78.0, 25.0, 81.0, 4.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(155.0, 23.0, 8.0, 8.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#656f76")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(196.0, 15.0, 44.0, 24.0)
            && command.style.painter_family == UiPainterFamily::Slider
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Value")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("0.75")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
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
                    && command.style.background_color.as_deref() == Some("#2c3237")
            })
            .count(),
        5
    );
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.background_color.as_deref() == Some("#e8ecee20")
            && command.frame.width == 16.0
            && command.frame.height == 16.0
    }));
}

#[test]
fn render_extract_sliders_prioritize_valid_style_overrides_and_reject_invalid_values() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.sliders.overrides"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 104.0))
            .with_state_flags(visible_state()),
    );
    insert_slider_with_style_overrides(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 12.0, 240.0, 30.0),
        r##"
label = "Gain"
value_percent = 0.5
track_color = "#10161a"
value_color = "#243238"
thumb_color = "#d6e2e5"
"##,
        r##"
track_color = "#254c5a"
value_color = "#4c9dab"
thumb_color = "#eef8fa"
thumb_outline_color = "#4c9dab"
label_color = "#b5c5ca"
font_size = 12.0
line_height_ratio = 1.5
track_height = 6.0
thumb_size = 10.0
"##,
        visible_state(),
    );
    insert_slider_with_style_overrides(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(8.0, 56.0, 240.0, 30.0),
        r##"
label = "Fallback"
value_percent = 0.5
"##,
        r##"
track_color = "not-a-color"
track_height = 0.0
thumb_size = -2.0
font_size = 0.0
line_height_ratio = 0.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let overridden_track = slider_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(78.0, 24.0, 108.0, 6.0),
    );
    assert_eq!(
        overridden_track.style.background_color.as_deref(),
        Some("#254c5a")
    );
    let overridden_fill = slider_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(78.0, 24.0, 54.0, 6.0),
    );
    assert_eq!(
        overridden_fill.style.background_color.as_deref(),
        Some("#4c9dab")
    );
    let overridden_thumb = slider_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(127.0, 22.0, 10.0, 10.0),
    );
    assert_eq!(
        overridden_thumb.style.background_color.as_deref(),
        Some("#eef8fa")
    );
    assert_eq!(
        overridden_thumb.style.border_color.as_deref(),
        Some("#4c9dab")
    );
    let overridden_label = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Gain")
        })
        .expect("overridden slider should render its label");
    assert_eq!(
        overridden_label.style.foreground_color.as_deref(),
        Some("#b5c5ca")
    );
    assert_eq!(overridden_label.style.font_size, 12.0);
    assert_eq!(overridden_label.style.line_height, 18.0);

    let fallback_track = slider_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(78.0, 69.0, 108.0, 4.0),
    );
    assert_eq!(
        fallback_track.style.background_color.as_deref(),
        Some("#111416")
    );
    let fallback_label = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(3) && command.text.as_deref() == Some("Fallback")
        })
        .expect("fallback slider should render its label");
    assert_eq!(
        fallback_label.style.font_size,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
    assert_eq!(
        fallback_label.style.line_height,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
            * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO
    );
}

#[test]
fn runtime_slider_tick_commands_are_capped_by_shared_budget_and_track_columns() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.slider-tick-budget"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 720.0, 112.0))
            .with_state_flags(visible_state()),
    );
    insert_slider(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(8.0, 12.0, 600.0, 30.0),
        "value_percent = 0.5\ntick_count = 10000",
        visible_state(),
    );
    insert_slider(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(8.0, 52.0, 80.0, 30.0),
        "value_percent = 0.5\ntick_count = 10000",
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert_eq!(
        slider_tick_command_count(commands, UiNodeId::new(2), 33.0),
        MAX_UI_SLIDER_TICK_COUNT
    );
    assert_eq!(
        slider_tick_command_count(commands, UiNodeId::new(3), 73.0),
        64,
        "a 64px track cannot produce more than 64 distinct tick columns"
    );
}

fn insert_slider(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    insert_slider_with_style_overrides(surface, node_id, frame, attributes, "", state_flags);
}

fn insert_slider_with_style_overrides(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    attributes: &str,
    style_overrides: &str,
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
                    style_overrides: toml::from_str(style_overrides).unwrap(),
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

fn slider_tick_command_count(
    commands: &[UiRenderCommand],
    node_id: UiNodeId,
    tick_y: f32,
) -> usize {
    commands
        .iter()
        .filter(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.frame.y == tick_y
                && command.frame.width == 1.0
                && command.frame.height == 4.0
        })
        .count()
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

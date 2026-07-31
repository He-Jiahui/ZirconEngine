use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    design_tokens::EditorTypographyTokens,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn segmented_rendering_classifies_before_state_and_avoids_selected_lowercase_copy() {
    let source = include_str!("../surface/render/segmented_controls.rs");
    let kind = source
        .find("let Some(kind) = control_kind(metadata)")
        .expect("segmented rendering should classify the component");
    let state = source
        .find("let state = SegmentedRenderState::resolve")
        .expect("segmented rendering should resolve painter state");

    assert!(
        kind < state,
        "non-segmented nodes should exit before state resolution"
    );
    assert!(
        !source.contains("to_ascii_lowercase"),
        "selected segment matching should compare borrowed text without allocation"
    );
    for required_token_hook in [
        "EditorDesignTokens",
        "EditorTypographyTokens",
        "style_overrides",
        "default_segmented_visual",
    ] {
        assert!(
            source.contains(required_token_hook),
            "segmented renderer should resolve {required_token_hook} through the design-token contract"
        );
    }
}

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
            && command.style.foreground_color.as_deref() == Some("#a4aeb4")
            && command.style.painter_family == UiPainterFamily::Tab
            && command.style.painter_state == UiPainterResolvedState::Normal
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 26.0, 150.0, 30.0)
            && command.style.background_color.as_deref() == Some("#1b1f23")
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
            && command.style.background_color.as_deref() == Some("#3cc7d6")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Center")
            && command.frame == UiFrame::new(70.0, 31.0, 34.0, 20.0)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
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
            && command.style.background_color.as_deref() == Some("#3cc7d6")
            && command.style.painter_family == UiPainterFamily::Tab
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("UI Components")
            && command.frame == UiFrame::new(24.0, 76.8, 108.0, 14.400001)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
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
fn render_extract_segmented_defaults_to_accent_token_selected_indicator() {
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
            && command.style.background_color.as_deref() == Some("#3cc7d6")
    }));
}

#[test]
fn render_extract_segmented_and_tab_keep_focused_surface_neutral_until_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.segmented_controls.focused_surface_neutral",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 136.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "SegmentedControl",
        UiFrame::new(12.0, 8.0, 150.0, 30.0),
        r##"
options = ["left", "right"]
background_color = "#10161a"
border_color = "#323a41"
hover_background_color = "#2a3036"
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "SegmentedControl",
        UiFrame::new(12.0, 44.0, 150.0, 30.0),
        r##"
options = ["left", "right"]
background_color = "#10161a"
border_color = "#323a41"
hover_background_color = "#2a3036"
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Tab",
        UiFrame::new(12.0, 84.0, 120.0, 28.0),
        r##"
text = "Details"
background_color = "#10161a"
hover_background_color = "#2a3036"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(5),
        "Tab",
        UiFrame::new(140.0, 84.0, 120.0, 28.0),
        r##"
text = "Preview"
background_color = "#10161a"
hover_background_color = "#2a3036"
"##,
        visible_state(),
    );
    set_focused(&mut surface, UiNodeId::new(2));
    set_focused(&mut surface, UiNodeId::new(3));
    set_hovered(&mut surface, UiNodeId::new(3));
    set_focused(&mut surface, UiNodeId::new(4));
    set_focused(&mut surface, UiNodeId::new(5));
    set_hovered(&mut surface, UiNodeId::new(5));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let focused_segment = surface_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(12.0, 8.0, 150.0, 30.0),
    );
    assert_eq!(
        focused_segment.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_segment.style.background_color.as_deref(),
        Some("#10161a")
    );
    assert_eq!(
        focused_segment.style.border_color.as_deref(),
        Some("#35c7d0")
    );

    let hovered_segment = surface_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(12.0, 44.0, 150.0, 30.0),
    );
    assert_eq!(
        hovered_segment.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        hovered_segment.style.background_color.as_deref(),
        Some("#2a3036")
    );
    assert_eq!(
        hovered_segment.style.border_color.as_deref(),
        Some("#35c7d0")
    );

    let focused_tab = surface_quad(
        commands,
        UiNodeId::new(4),
        UiFrame::new(12.0, 84.0, 120.0, 28.0),
    );
    assert_eq!(
        focused_tab.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_tab.style.background_color.as_deref(),
        Some("#10161a")
    );

    let hovered_tab = surface_quad(
        commands,
        UiNodeId::new(5),
        UiFrame::new(140.0, 84.0, 120.0, 28.0),
    );
    assert_eq!(
        hovered_tab.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        hovered_tab.style.background_color.as_deref(),
        Some("#2a3036")
    );
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
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 26.0, 150.0, 30.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 28.0, 46.0, 26.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.is_none()
            && command.style.border_width == 0.0
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(64.0, 53.0, 46.0, 1.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Center")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 72.0, 132.0, 32.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 102.0, 132.0, 2.0)
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("UI Components")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
}

#[test]
fn render_extract_segmented_controls_prioritize_valid_style_overrides_and_reject_invalid_values() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.segmented_controls.overrides",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 80.0))
            .with_state_flags(visible_state()),
    );
    insert_control_with_style_overrides(
        &mut surface,
        UiNodeId::new(2),
        "SegmentedControl",
        UiFrame::new(12.0, 12.0, 128.0, 32.0),
        r##"
value = "right"
options = ["left", "right"]
background_color = "#10161a"
border_color = "#243238"
selected_background_color = "#173942"
selected_foreground_color = "#d6e2e5"
"##,
        r##"
background_color = "#254c5a"
border_color = "#4c9dab"
selected_background_color = "#173942"
selected_border_color = "#315f6d"
selected_border_width = 1.0
selected_underline_color = "#4c9dab"
selected_foreground_color = "#eef8fa"
corner_radius = 7.0
font_size = 12.0
line_height_ratio = 1.5
"##,
        visible_state(),
    );
    insert_control_with_style_overrides(
        &mut surface,
        UiNodeId::new(3),
        "SegmentedControl",
        UiFrame::new(156.0, 12.0, 128.0, 32.0),
        r##"
value = "left"
options = ["left", "right"]
"##,
        r##"
background_color = "not-a-color"
border_width = -1.0
corner_radius = -4.0
font_size = 0.0
line_height_ratio = 0.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let overridden_surface = surface_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(12.0, 12.0, 128.0, 32.0),
    );
    assert_eq!(
        overridden_surface.style.background_color.as_deref(),
        Some("#254c5a")
    );
    assert_eq!(
        overridden_surface.style.border_color.as_deref(),
        Some("#4c9dab")
    );
    assert_eq!(overridden_surface.style.corner_radius, 7.0);
    let overridden_selection = surface_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(78.0, 14.0, 60.0, 28.0),
    );
    assert_eq!(
        overridden_selection.style.background_color.as_deref(),
        Some("#173942")
    );
    assert_eq!(
        overridden_selection.style.border_color.as_deref(),
        Some("#315f6d")
    );
    assert_eq!(overridden_selection.style.border_width, 1.0);
    let overridden_text = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Right")
        })
        .expect("overridden segmented control should render its selected option");
    assert_eq!(
        overridden_text.style.foreground_color.as_deref(),
        Some("#eef8fa")
    );
    assert_eq!(overridden_text.style.font_size, 12.0);
    assert_eq!(overridden_text.style.line_height, 18.0);

    let fallback_surface = surface_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(156.0, 12.0, 128.0, 32.0),
    );
    assert_eq!(
        fallback_surface.style.background_color.as_deref(),
        Some("#1b1f23")
    );
    assert_eq!(
        fallback_surface.style.border_color.as_deref(),
        Some("#323a41")
    );
    assert_eq!(fallback_surface.style.border_width, 1.0);
    assert_eq!(fallback_surface.style.corner_radius, 5.0);
    let fallback_text = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(3) && command.text.as_deref() == Some("Left")
        })
        .expect("fallback segmented control should render its selected option");
    assert_eq!(
        fallback_text.style.font_size,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
    assert_eq!(
        fallback_text.style.line_height,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
            * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO
    );
}

fn surface_quad(
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
        .expect("expected segmented control surface quad")
}

fn insert_control(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    component: &str,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    insert_control_with_style_overrides(
        surface,
        node_id,
        component,
        frame,
        attributes,
        "",
        state_flags,
    );
}

fn insert_control_with_style_overrides(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    component: &str,
    frame: UiFrame,
    attributes: &str,
    style_overrides: &str,
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
                    style_overrides: toml::from_str(style_overrides).unwrap(),
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

fn set_focused(surface: &mut UiSurface, node_id: UiNodeId) {
    assert!(surface.component_states.set_focused(node_id, true));
}

fn set_hovered(surface: &mut UiSurface, node_id: UiNodeId) {
    assert!(surface.component_states.set_hovered(node_id, true));
}

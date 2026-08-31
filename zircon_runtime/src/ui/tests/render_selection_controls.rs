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
fn selection_control_rendering_uses_central_tokens_and_validated_overrides() {
    let source = include_str!("../surface/render/selection_controls.rs");
    let owner_source = [
        source,
        include_str!("../surface/render/selection_controls/checkbox.rs"),
        include_str!("../surface/render/selection_controls/commands.rs"),
        include_str!("../surface/render/selection_controls/geometry.rs"),
        include_str!("../surface/render/selection_controls/metadata.rs"),
        include_str!("../surface/render/selection_controls/radio.rs"),
        include_str!("../surface/render/selection_controls/state.rs"),
        include_str!("../surface/render/selection_controls/style.rs"),
        include_str!("../surface/render/selection_controls/toggle.rs"),
    ]
    .join("\n");
    for needle in [
        "EditorDesignTokens",
        "EditorTypographyTokens",
        "SelectionVisual",
        "style_overrides",
        "parse_css_color",
        "value_as_f32",
    ] {
        assert!(
            owner_source.contains(needle),
            "missing selection renderer feature: {needle}"
        );
    }
    for legacy in [
        "const MARK_INSET_X",
        "const LABEL_FONT_SIZE",
        "const SURFACE_SELECTED",
    ] {
        assert!(
            !owner_source.contains(legacy),
            "legacy selection visual remains: {legacy}"
        );
    }
    let kind = source
        .find("let Some(kind) = selection_control_kind(metadata)")
        .expect("selection rendering should classify the component");
    let state = source
        .find("let state = SelectionRenderState::resolve")
        .expect("selection rendering should resolve painter state");
    assert!(kind < state, "classification should precede state folding");
    for module in [
        "mod checkbox;",
        "mod commands;",
        "mod geometry;",
        "mod metadata;",
        "mod radio;",
        "mod state;",
        "mod style;",
        "mod toggle;",
    ] {
        assert!(source.contains(module), "root should mount {module}");
    }
    assert!(
        source.lines().count() <= 90,
        "selection render root should remain a declarative dispatch owner"
    );
    assert_eq!(
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
            * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
        16.0
    );
}

#[test]
fn render_extract_expands_selection_control_indicators() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.selection_controls"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 140.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 120.0, 28.0),
        r##"
text = "Checkbox"
checked = true
layout_spacing = 9.0
layout_icon_size = 16.0
background_color = "#10161a"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Radio",
        UiFrame::new(8.0, 44.0, 120.0, 28.0),
        r##"
text = "Radio option"
checked = true
dot_size = 7.0
dot_color = "#43d8e2"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Toggle",
        UiFrame::new(8.0, 80.0, 96.0, 28.0),
        r##"
text = "Switch"
checked = true
track_width = 34.0
track_height = 18.0
thumb_size = 12.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 14.0, 16.0, 16.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(26.0, 18.0, 3.0, 8.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Checkbox")
            && command.frame.x >= 43.0
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Checkbox")
            })
            .count(),
        1
    );

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(22.5, 54.5, 7.0, 7.0)
            && command.style.background_color.as_deref() == Some("#3cc7d6")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Radio option")
    }));

    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(62.0, 85.0, 34.0, 18.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(82.0, 88.0, 12.0, 12.0)
            && command.style.background_color.as_deref() == Some("#e8ecee")
    }));
}

#[test]
fn render_extract_uses_shared_selector_for_pressed_checked_selection_border() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.selection_controls.selector",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 64.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 120.0, 28.0),
        r##"
text = "Pressed"
checked = true
"##,
        UiStateFlags {
            pressed: true,
            ..visible_state()
        },
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 14.0, 16.0, 16.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
    }));
}

#[test]
fn render_extract_checked_selection_controls_keep_active_visuals_when_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.selection_controls.checked_hover",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 120.0, 28.0),
        r##"
text = "Checked hot"
checked = true
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Toggle",
        UiFrame::new(8.0, 44.0, 110.0, 28.0),
        r##"
text = "Toggle hot"
checked = true
"##,
        visible_state(),
    );
    assert!(surface.component_states.set_hovered(UiNodeId::new(2), true));
    assert!(surface
        .component_states
        .set_drop_hovered(UiNodeId::new(2), true));
    assert!(surface.component_states.set_hovered(UiNodeId::new(3), true));
    assert!(surface
        .component_states
        .set_drop_hovered(UiNodeId::new(3), true));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 14.0, 16.0, 16.0)
            && command.style.painter_family == UiPainterFamily::Checkbox
            && command.style.painter_state == UiPainterResolvedState::Checked
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(76.0, 49.0, 34.0, 18.0)
            && command.style.painter_family == UiPainterFamily::Toggle
            && command.style.painter_state == UiPainterResolvedState::Checked
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
    }));
}

#[test]
fn render_extract_selection_controls_keep_focused_surface_neutral_until_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.selection_controls.focused_surface_neutral",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 128.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Toggle",
        UiFrame::new(8.0, 8.0, 110.0, 28.0),
        r##"
text = "Focus"
background_color = "#10161a"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Toggle",
        UiFrame::new(8.0, 44.0, 110.0, 28.0),
        r##"
text = "Hover"
background_color = "#10161a"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Checkbox",
        UiFrame::new(8.0, 80.0, 120.0, 28.0),
        r##"
text = "Checkbox"
background_color = "#10161a"
border_color = "#323a41"
"##,
        visible_state(),
    );
    set_focused(&mut surface, UiNodeId::new(2));
    set_focused(&mut surface, UiNodeId::new(3));
    set_hovered(&mut surface, UiNodeId::new(3));
    set_focused(&mut surface, UiNodeId::new(4));

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let focused_toggle = control_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(76.0, 13.0, 34.0, 18.0),
    );
    assert_eq!(
        focused_toggle.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_toggle.style.background_color.as_deref(),
        Some("#10161a")
    );
    assert_eq!(
        focused_toggle.style.border_color.as_deref(),
        Some("#3cc7d6")
    );

    let hovered_toggle = control_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(76.0, 49.0, 34.0, 18.0),
    );
    assert_eq!(
        hovered_toggle.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        hovered_toggle.style.background_color.as_deref(),
        Some("#1a2429")
    );
    assert_eq!(
        hovered_toggle.style.border_color.as_deref(),
        Some("#3cc7d6")
    );

    let focused_checkbox = control_quad(
        commands,
        UiNodeId::new(4),
        UiFrame::new(18.0, 86.0, 16.0, 16.0),
    );
    assert_eq!(
        focused_checkbox.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_checkbox.style.background_color.as_deref(),
        Some("#10161a")
    );
    assert_eq!(
        focused_checkbox.style.border_color.as_deref(),
        Some("#3cc7d6")
    );
}

#[test]
fn render_extract_loading_selection_controls_use_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.selection_controls.loading",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 132.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 150.0, 28.0),
        r##"
text = "Checkbox"
checked = true
selected = true
pressed = true
hovered = true
loading = true
background_color = "#10161a"
border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Radio",
        UiFrame::new(8.0, 44.0, 150.0, 28.0),
        r##"
text = "Radio"
checked = true
pressed = true
hovered = true
loading = true
dot_color = "#43d8e2"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "Toggle",
        UiFrame::new(8.0, 80.0, 110.0, 28.0),
        r##"
text = "Toggle"
checked = true
pressed = true
hovered = true
loading = true
background_color = "#209fa8"
border_color = "#35c7d0"
foreground_color = "#ffffff"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 14.0, 16.0, 16.0)
            && command.style.painter_family == UiPainterFamily::Checkbox
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(26.0, 18.0, 3.0, 8.0)
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(18.0, 50.0, 16.0, 16.0)
            && command.style.painter_family == UiPainterFamily::Radio
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(22.5, 54.5, 7.0, 7.0)
            && command.style.painter_family == UiPainterFamily::Radio
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#59656c")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(76.0, 85.0, 34.0, 18.0)
            && command.style.painter_family == UiPainterFamily::Toggle
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(4)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(96.0, 88.0, 12.0, 12.0)
            && command.style.painter_family == UiPainterFamily::Toggle
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#59656c")
    }));
}

#[test]
fn render_extract_selection_controls_prioritize_valid_style_overrides_and_reject_invalid_values() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.selection_controls.overrides",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 280.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_control_with_style_overrides(
        &mut surface,
        UiNodeId::new(2),
        "Checkbox",
        UiFrame::new(8.0, 8.0, 120.0, 28.0),
        r##"
text = "Override"
background_color = "#10161a"
border_color = "#243238"
label_color = "#d6e2e5"
"##,
        r##"
background_color = "#254c5a"
border_color = "#4c9dab"
label_color = "#eef8fa"
font_size = 12.0
line_height_ratio = 1.5
"##,
        visible_state(),
    );
    insert_control_with_style_overrides(
        &mut surface,
        UiNodeId::new(3),
        "Checkbox",
        UiFrame::new(148.0, 8.0, 120.0, 28.0),
        r##"
text = "Fallback"
"##,
        r##"
background_color = "not-a-color"
border_width = -1.0
layout_icon_size = 0.0
font_size = 0.0
line_height_ratio = 0.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let overridden_mark = control_quad(
        commands,
        UiNodeId::new(2),
        UiFrame::new(18.0, 14.0, 16.0, 16.0),
    );
    assert_eq!(
        overridden_mark.style.background_color.as_deref(),
        Some("#254c5a")
    );
    assert_eq!(
        overridden_mark.style.border_color.as_deref(),
        Some("#4c9dab")
    );
    let overridden_label = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2) && command.text.is_some())
        .expect("overridden checkbox should render a label");
    assert_eq!(
        overridden_label.style.foreground_color.as_deref(),
        Some("#eef8fa")
    );
    assert_eq!(overridden_label.style.font_size, 12.0);
    assert_eq!(overridden_label.style.line_height, 18.0);

    let fallback_mark = control_quad(
        commands,
        UiNodeId::new(3),
        UiFrame::new(158.0, 14.0, 16.0, 16.0),
    );
    assert_eq!(
        fallback_mark.style.background_color.as_deref(),
        Some("#0f1316")
    );
    assert_eq!(fallback_mark.style.border_color.as_deref(), Some("#414b54"));
    assert_eq!(fallback_mark.style.border_width, 1.0);
    let fallback_label = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(3) && command.text.is_some())
        .expect("fallback checkbox should render a label");
    assert_eq!(
        fallback_label.style.font_size,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
}

fn control_quad(
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
        .expect("expected selection control quad")
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

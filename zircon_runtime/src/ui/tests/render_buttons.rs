use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    design_tokens::EditorTypographyTokens,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn button_rendering_resolves_variant_once_without_joining_lowercase_text() {
    let root_source = include_str!("../surface/render/buttons.rs");
    let owner_source = concat!(
        include_str!("../surface/render/buttons.rs"),
        include_str!("../surface/render/buttons/button.rs"),
        include_str!("../surface/render/buttons/commands.rs"),
        include_str!("../surface/render/buttons/icon_button.rs"),
        include_str!("../surface/render/buttons/metadata.rs"),
        include_str!("../surface/render/buttons/state.rs"),
        include_str!("../surface/render/buttons/style.rs"),
    );

    assert_eq!(
        owner_source.matches("button_kind(metadata)").count(),
        1,
        "button kind should be resolved once into render state"
    );
    assert!(
        !owner_source.contains(".join(\" \")") && !owner_source.contains("to_ascii_lowercase"),
        "button kind classification should not allocate joined lowercase strings"
    );
    for child in [
        "mod button;",
        "mod commands;",
        "mod icon_button;",
        "mod metadata;",
        "mod state;",
        "mod style;",
    ] {
        assert!(root_source.contains(child), "missing button owner {child}");
    }
    assert!(
        root_source.lines().count() <= 80,
        "button routing owner should stay compact"
    );
    assert!(owner_source.contains("EditorDesignTokens"));
    assert!(owner_source.contains("EditorTypographyTokens"));
    assert!(owner_source.contains("style_overrides"));
    assert!(owner_source.contains("parse_css_color"));
    assert!(owner_source.contains("value_as_f32"));
    assert!(!owner_source.contains("const PRIMARY_SURFACE"));
    assert!(!owner_source.contains("const DEFAULT_FONT_SIZE"));
}

#[test]
fn render_extract_expands_button_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.buttons"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 120.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Button",
        UiFrame::new(12.0, 16.0, 132.0, 30.0),
        r##"
text = "Compile"
icon = "play"
button_color = "primary"
layout_padding_left = 12.0
layout_padding_right = 12.0
layout_spacing = 7.0
layout_icon_size = 16.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 16.0, 132.0, 30.0)
            && command.style.background_color.as_deref() == Some("#17434d")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
            && command.style.painter_family == UiPainterFamily::Button
            && command.style.painter_state == UiPainterResolvedState::Normal
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("play".to_string()))
            && command.frame == UiFrame::new(24.0, 23.0, 16.0, 16.0)
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Compile")
            && command.frame
                == UiFrame::new(
                    47.0,
                    23.0,
                    85.0,
                    EditorTypographyTokens::WORKBENCH_BODY_SIZE
                        * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
                )
            && command.style.foreground_color.as_deref() == Some("#e8ecee")
            && command.style.font_size == EditorTypographyTokens::WORKBENCH_BODY_SIZE
    }));
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("Compile")
            })
            .count(),
        1
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| {
                command.node_id == UiNodeId::new(2)
                    && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("play".to_string()))
            })
            .count(),
        1
    );
}

#[test]
fn render_extract_expands_icon_button_state_surface() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.icon_buttons"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 80.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "IconButton",
        UiFrame::new(16.0, 20.0, 40.0, 40.0),
        r##"
icon = "transform"
label = "Move"
selected = true
layout_icon_size = 18.0
corner_radius = 6.0
"##,
        UiStateFlags {
            hoverable: true,
            clickable: true,
            focusable: true,
            ..visible_state()
        },
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(16.0, 20.0, 40.0, 40.0)
            && command.style.background_color.as_deref() == Some("#173942")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
            && command.style.corner_radius == 6.0
            && command.style.painter_family == UiPainterFamily::IconButton
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("transform".to_string()))
            && command.frame == UiFrame::new(27.0, 31.0, 18.0, 18.0)
            && command.style.foreground_color.as_deref() == Some("#3cc7d6")
            && command.style.painter_state == UiPainterResolvedState::Selected
    }));
    assert!(
        commands
            .iter()
            .all(|command| command.node_id != UiNodeId::new(2) || command.text.is_none()),
        "icon-only buttons should not render accessibility labels as visible text"
    );
}

#[test]
fn render_extract_button_and_icon_button_keep_focused_surface_neutral_until_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.buttons.focused_neutral"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 360.0, 132.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Button",
        UiFrame::new(12.0, 12.0, 132.0, 30.0),
        r##"
text = "Compile"
button_color = "primary"
focused = true
background_color = "#10161a"
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "Button",
        UiFrame::new(12.0, 56.0, 132.0, 30.0),
        r##"
text = "Compile"
button_color = "primary"
focused = true
hovered = true
background_color = "#10161a"
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(4),
        "IconButton",
        UiFrame::new(176.0, 12.0, 40.0, 40.0),
        r##"
icon = "transform"
focused = true
background_color = "#1f2529"
hover_background_color = "#20282d"
focus_border_color = "#35c7d0"
icon_color = "#a4aeb4"
selected_icon_color = "#2aa6b8"
layout_icon_size = 18.0
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(5),
        "IconButton",
        UiFrame::new(176.0, 64.0, 40.0, 40.0),
        r##"
icon = "transform"
focused = true
hovered = true
background_color = "#1f2529"
hover_background_color = "#20282d"
focus_border_color = "#35c7d0"
icon_color = "#a4aeb4"
selected_icon_color = "#2aa6b8"
layout_icon_size = 18.0
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(6),
        "ToggleButton",
        UiFrame::new(236.0, 12.0, 96.0, 30.0),
        r##"
text = "Snap"
selected = true
focus_border_color = "#35c7d0"
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let focused_button = control_surface(commands, UiNodeId::new(2), UiPainterFamily::Button);
    assert_eq!(
        focused_button.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_button.style.background_color.as_deref(),
        Some("#10161a")
    );
    assert_eq!(
        focused_button.style.border_color.as_deref(),
        Some("#35c7d0")
    );
    assert!(!commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.background_color.as_deref() == Some("#263d43")
    }));

    let focused_hovered_button =
        control_surface(commands, UiNodeId::new(3), UiPainterFamily::Button);
    assert_eq!(
        focused_hovered_button.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_hovered_button.style.background_color.as_deref(),
        Some("#173942")
    );
    assert_eq!(
        focused_hovered_button.style.border_color.as_deref(),
        Some("#35c7d0")
    );

    let focused_icon = control_surface(commands, UiNodeId::new(4), UiPainterFamily::IconButton);
    assert_eq!(
        focused_icon.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_icon.style.background_color.as_deref(),
        Some("#1f2529")
    );
    assert_eq!(focused_icon.style.border_color.as_deref(), Some("#35c7d0"));
    assert_eq!(
        control_icon(commands, UiNodeId::new(4))
            .style
            .foreground_color
            .as_deref(),
        Some("#a4aeb4")
    );

    let focused_hovered_icon =
        control_surface(commands, UiNodeId::new(5), UiPainterFamily::IconButton);
    assert_eq!(
        focused_hovered_icon.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        focused_hovered_icon.style.background_color.as_deref(),
        Some("#2a3036")
    );
    assert_eq!(
        control_icon(commands, UiNodeId::new(5))
            .style
            .foreground_color
            .as_deref(),
        Some("#a4aeb4")
    );

    let selected_toggle = control_surface(commands, UiNodeId::new(6), UiPainterFamily::Button);
    assert_eq!(
        selected_toggle.style.painter_state,
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        selected_toggle.style.background_color.as_deref(),
        Some("#2a3036")
    );
    assert_eq!(
        selected_toggle.style.border_color.as_deref(),
        Some("#35c7d0")
    );
}

#[test]
fn render_extract_loading_button_and_icon_button_use_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.buttons.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 120.0))
            .with_state_flags(visible_state()),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(2),
        "Button",
        UiFrame::new(12.0, 16.0, 132.0, 30.0),
        r##"
text = "Compile"
icon = "play"
button_color = "primary"
loading = true
hovered = true
focused = true
pressed = true
background_color = "#32b8c5"
border_color = "#249aa6"
foreground_color = "#08181b"
layout_padding_left = 12.0
layout_padding_right = 12.0
layout_spacing = 7.0
layout_icon_size = 16.0
"##,
        visible_state(),
    );
    insert_control(
        &mut surface,
        UiNodeId::new(3),
        "IconButton",
        UiFrame::new(160.0, 16.0, 40.0, 40.0),
        r##"
icon = "trash"
selected = true
checked = true
loading = true
hovered = true
focused = true
pressed = true
background_color = "#14373c"
border_color = "#35c7d0"
icon_color = "#ef7066"
selected_icon_color = "#35c7d0"
layout_icon_size = 18.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::Button
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("play".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Compile")
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::IconButton
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#2c3237")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Image
            && command.image.as_ref() == Some(&UiVisualAssetRef::Icon("trash".to_string()))
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.foreground_color.as_deref() == Some("#656f76")
    }));
}

#[test]
fn render_extract_buttons_prioritize_valid_style_overrides_and_reject_invalid_values() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.buttons.overrides"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 120.0))
            .with_state_flags(visible_state()),
    );
    insert_control_with_style_overrides(
        &mut surface,
        UiNodeId::new(2),
        "Button",
        UiFrame::new(12.0, 12.0, 128.0, 32.0),
        r##"
text = "Apply"
button_color = "primary"
background_color = "#10161a"
border_color = "#243238"
foreground_color = "#d6e2e5"
"##,
        r##"
background_color = "#254c5a"
border_color = "#4c9dab"
foreground_color = "#eef8fa"
layout_padding_left = 20.0
font_size = 10.0
line_height_ratio = 1.5
"##,
        visible_state(),
    );
    insert_control_with_style_overrides(
        &mut surface,
        UiNodeId::new(3),
        "Button",
        UiFrame::new(156.0, 12.0, 128.0, 32.0),
        r##"
text = "Fallback"
button_color = "primary"
"##,
        r##"
background_color = "not-a-color"
border_width = -1.0
layout_padding_left = -4.0
font_size = 0.0
line_height_ratio = 0.0
"##,
        visible_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let overridden_surface = control_surface(commands, UiNodeId::new(2), UiPainterFamily::Button);
    assert_eq!(
        overridden_surface.style.background_color.as_deref(),
        Some("#254c5a")
    );
    assert_eq!(
        overridden_surface.style.border_color.as_deref(),
        Some("#4c9dab")
    );
    assert_eq!(
        commands
            .iter()
            .find(|command| command.node_id == UiNodeId::new(2) && command.text.is_some())
            .expect("overridden button text should be rendered")
            .style
            .foreground_color
            .as_deref(),
        Some("#eef8fa")
    );
    let overridden_text = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2) && command.text.is_some())
        .expect("overridden button text should be rendered");
    assert_eq!(overridden_text.frame.x, 32.0);
    assert_eq!(overridden_text.style.font_size, 10.0);
    assert_eq!(overridden_text.style.line_height, 15.0);

    let fallback_surface = control_surface(commands, UiNodeId::new(3), UiPainterFamily::Button);
    assert_eq!(
        fallback_surface.style.background_color.as_deref(),
        Some("#17434d")
    );
    assert_eq!(fallback_surface.style.border_width, 1.0);
    let fallback_text = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(3) && command.text.is_some())
        .expect("fallback button text should be rendered");
    assert_eq!(fallback_text.frame.x, 168.0);
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

fn control_surface(
    commands: &[UiRenderCommand],
    node_id: UiNodeId,
    family: UiPainterFamily,
) -> &UiRenderCommand {
    commands
        .iter()
        .find(|command| {
            command.node_id == node_id
                && command.kind == UiRenderCommandKind::Quad
                && command.style.painter_family == family
        })
        .expect("control surface command should be rendered")
}

fn control_icon(commands: &[UiRenderCommand], node_id: UiNodeId) -> &UiRenderCommand {
    commands
        .iter()
        .find(|command| command.node_id == node_id && command.kind == UiRenderCommandKind::Image)
        .expect("control icon command should be rendered")
}

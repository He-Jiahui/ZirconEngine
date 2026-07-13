use super::*;

#[test]
fn render_extract_carries_visual_contract_fields_for_visible_nodes() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 120.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/launch"))
                .with_frame(UiFrame::new(12.0, 8.0, 96.0, 32.0))
                .with_z_index(7)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "IconButton".to_string(),
                    control_id: Some("LaunchButton".to_string()),
                    classes: vec!["primary".to_string()],
                    attributes: toml::from_str(
                        r##"
text = "Launch"
icon = "rocket-outline"
opacity = 0.75
font = "res://fonts/default.font.toml"
font_family = "Fira Mono"
font_size = 18.0
line_height = 24.0
text_align = "center"
wrap = "word"
text_render_mode = "sdf"

[background]
color = "#112233"

[foreground]
color = "#ddeeff"

[border]
color = "#334455"
width = 2.0
radius = 6.0
"##,
                    )
                    .unwrap(),
                    slot_attributes: Default::default(),
                    style_overrides: Default::default(),
                    style_tokens: Default::default(),
                    bindings: Vec::new(),
                    ..Default::default()
                }),
        )
        .unwrap();
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 80.0, 28.0));

    surface.rebuild();

    let root_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(1))
        .unwrap();
    assert_eq!(root_command.kind, UiRenderCommandKind::Group);
    assert_eq!(root_command.style, UiResolvedStyle::default());
    assert_eq!(root_command.text.as_deref(), None);
    assert_eq!(root_command.image, None);
    assert_eq!(root_command.opacity, 1.0);

    let launch_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(launch_command.kind, UiRenderCommandKind::Quad);
    assert_eq!(
        launch_command.clip_frame,
        Some(UiFrame::new(0.0, 0.0, 80.0, 28.0))
    );
    assert_eq!(launch_command.z_index, 7);
    assert_eq!(launch_command.text, None);
    assert_eq!(launch_command.image, None);
    assert_eq!(launch_command.opacity, 0.75);
    assert_eq!(
        launch_command.style,
        UiResolvedStyle {
            background_color: Some("#112233".to_string()),
            foreground_color: Some("#ddeeff".to_string()),
            border_color: Some("#334455".to_string()),
            border_width: 2.0,
            corner_radius: 6.0,
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Fira Mono".to_string()),
            font_size: 18.0,
            line_height: 24.0,
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::Word,
            text_direction: Default::default(),
            text_overflow: Default::default(),
            rich_text_format: zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain,
            text_render_mode: UiTextRenderMode::Sdf,
            painter_family: UiPainterFamily::IconButton,
            painter_state: UiPainterResolvedState::Normal,
            ..UiResolvedStyle::default()
        }
    );
    let launch_icon = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.image == Some(UiVisualAssetRef::Icon("rocket-outline".to_string()))
        })
        .expect("IconButton painter should emit a dedicated icon command");
    assert!(launch_icon.z_index > launch_command.z_index);
}

#[test]
fn render_extract_accepts_flat_style_color_aliases() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 90.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/workbench-button"))
                .with_frame(UiFrame::new(12.0, 12.0, 112.0, 32.0))
                .with_state_flags(pointer_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("WorkbenchPrimaryButton".to_string()),
                    classes: vec![
                        "workbench-control-button".to_string(),
                        "workbench-primary-button".to_string(),
                    ],
                    attributes: toml::from_str(
                        r##"
label = "Primary"
background_color = "#12383d"
foreground_color = "#e8edf2"
border_color = "#35c7d0"
border_width = 1.0
radius = 8.0
"##,
                    )
                    .unwrap(),
                    slot_attributes: Default::default(),
                    style_overrides: Default::default(),
                    style_tokens: Default::default(),
                    bindings: Vec::new(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap();
    assert_eq!(command.kind, UiRenderCommandKind::Quad);
    assert_eq!(command.text, None);
    assert_eq!(
        command.style,
        UiResolvedStyle {
            background_color: Some("#12383d".to_string()),
            foreground_color: Some("#e8edf2".to_string()),
            border_color: Some("#35c7d0".to_string()),
            border_width: 1.0,
            corner_radius: 8.0,
            painter_family: UiPainterFamily::Button,
            painter_state: UiPainterResolvedState::Normal,
            ..UiResolvedStyle::default()
        }
    );
    assert!(surface.render_extract.list.commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Text
            && command.text.as_deref() == Some("Primary")
    }));
}

#[test]
fn render_extract_uses_label_when_schema_text_default_is_placeholder() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            }),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/locate"))
                .with_frame(UiFrame::new(8.0, 8.0, 124.0, 32.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    pressed: false,
                    checked: false,
                    dirty: false,
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Button".to_string(),
                    control_id: Some("LocateSelectedAsset".to_string()),
                    classes: Vec::new(),
                    attributes: toml::from_str(
                        r#"
text = "Button"
label = "Locate In Assets"
"#,
                    )
                    .unwrap(),
                    slot_attributes: Default::default(),
                    style_overrides: Default::default(),
                    style_tokens: Default::default(),
                    bindings: Vec::new(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let locate_text_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.kind == UiRenderCommandKind::Text
                && command.text.as_deref() == Some("Locate In Assets")
        })
        .unwrap();
    assert_eq!(
        locate_text_command.text.as_deref(),
        Some("Locate In Assets")
    );
}

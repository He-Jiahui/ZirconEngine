use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::UiRenderCommandKind,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn render_extract_expands_text_field_primitives() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.text_fields"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 120.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/search"))
                .with_frame(UiFrame::new(12.0, 16.0, 180.0, 30.0))
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "InputField".to_string(),
                    attributes: toml::from_str(
                        r##"
content = "Health Regen"
placeholder = "Filter..."
focused = true
selection_anchor = 0
selection_focus = 6
caret_offset = 6
layout_padding_left = 10.0
layout_padding_right = 8.0
layout_padding_top = 4.0
layout_padding_bottom = 4.0
font_size = 11.0
line_height = 13.2
background_color = "#10161a"
border_color = "#323f47"
focus_border_color = "#35c7d0"
foreground_color = "#c5d0d5"
"##,
                    )
                    .unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.frame == UiFrame::new(12.0, 16.0, 180.0, 30.0)
            && command.style.background_color.as_deref() == Some("#10161a")
            && command.style.border_color.as_deref() == Some("#35c7d0")
            && command.style.painter_family == UiPainterFamily::TextField
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));

    let text = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.kind == UiRenderCommandKind::Text
                && command.text.as_deref() == Some("Health Regen")
        })
        .expect("focused input should render its value through a component text command");
    assert_eq!(text.frame, UiFrame::new(22.0, 20.0, 162.0, 22.0));
    assert_eq!(text.clip_frame, Some(UiFrame::new(22.0, 20.0, 162.0, 22.0)));
    assert_eq!(text.style.foreground_color.as_deref(), Some("#c5d0d5"));
    assert_eq!(text.style.painter_family, UiPainterFamily::TextField);
    assert_eq!(text.style.painter_state, UiPainterResolvedState::Focused);

    let editable = text
        .text_layout
        .as_ref()
        .and_then(|layout| layout.editable.as_ref())
        .expect("focused input text layout should carry editable state");
    assert_eq!(editable.text, "Health Regen");
    assert_eq!(editable.caret.offset, 6);
    assert_eq!(editable.selection.as_ref().unwrap().range().start, 0);
    assert_eq!(editable.selection.as_ref().unwrap().range().end, 6);
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
}

#[test]
fn render_extract_expands_text_field_placeholder_without_unfocused_caret() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.text_fields.placeholder"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/filter"))
                .with_frame(UiFrame::new(8.0, 12.0, 150.0, 28.0))
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    attributes: toml::from_str(
                        r##"
value = ""
placeholder = "Search assets"
layout_padding_left = 9.0
layout_padding_right = 9.0
layout_padding_top = 4.0
layout_padding_bottom = 4.0
font_size = 11.0
line_height = 13.2
"##,
                    )
                    .unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let text = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.kind == UiRenderCommandKind::Text
                && command.text.as_deref() == Some("Search assets")
        })
        .expect("empty text field should render placeholder text");
    assert_eq!(text.style.foreground_color.as_deref(), Some("#68747b"));
    assert_eq!(text.style.painter_state, UiPainterResolvedState::Normal);
    assert!(
        text.text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref())
            .is_none(),
        "unfocused placeholder paint should not expose caret or selection decorations"
    );
}

#[test]
fn render_extract_expands_search_field_query_value() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.text_fields.search"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/search"))
                .with_frame(UiFrame::new(10.0, 12.0, 190.0, 32.0))
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "SearchField".to_string(),
                    attributes: toml::from_str(
                        r##"
query = "Player"
placeholder = "Search scene"
focused = true
caret_offset = 6
layout_padding_left = 28.0
layout_padding_right = 24.0
layout_padding_top = 4.0
layout_padding_bottom = 4.0
"##,
                    )
                    .unwrap(),
                    widget: UiWidgetContract {
                        value_property: Some("query".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::TextField
            && command.style.painter_state == UiPainterResolvedState::Focused
    }));

    let text = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.kind == UiRenderCommandKind::Text
                && command.text.as_deref() == Some("Player")
        })
        .expect("search field should render its query through the configured value property");
    assert_eq!(text.style.painter_family, UiPainterFamily::TextField);
    assert_eq!(text.frame, UiFrame::new(38.0, 16.0, 138.0, 24.0));

    let editable = text
        .text_layout
        .as_ref()
        .and_then(|layout| layout.editable.as_ref())
        .expect("focused search field should carry editable state");
    assert_eq!(editable.text, "Player");
    assert_eq!(editable.caret.offset, 6);
}

#[test]
fn render_extract_loading_text_field_uses_unavailable_visuals() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.text_fields.loading"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 96.0))
            .with_state_flags(visible_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/search"))
                .with_frame(UiFrame::new(12.0, 16.0, 180.0, 30.0))
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "InputField".to_string(),
                    attributes: toml::from_str(
                        r##"
content = "Health Regen"
placeholder = "Filter..."
loading = true
hovered = true
focused = true
pressed = true
selection_anchor = 0
selection_focus = 6
caret_offset = 6
layout_padding_left = 10.0
layout_padding_right = 8.0
layout_padding_top = 4.0
layout_padding_bottom = 4.0
background_color = "#10161a"
border_color = "#323f47"
focus_border_color = "#35c7d0"
foreground_color = "#c5d0d5"
"##,
                    )
                    .unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_family == UiPainterFamily::TextField
            && command.style.painter_state == UiPainterResolvedState::Loading
            && command.style.background_color.as_deref() == Some("#252c31")
            && command.style.border_color.as_deref() == Some("#343f47")
    }));

    let text = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2)
                && command.kind == UiRenderCommandKind::Text
                && command.text.as_deref() == Some("Health Regen")
        })
        .expect("loading input should still render its value through component text");
    assert_eq!(text.style.painter_state, UiPainterResolvedState::Loading);
    assert_eq!(text.style.foreground_color.as_deref(), Some("#59656c"));
    assert!(
        text.text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref())
            .is_none(),
        "loading text field paint should not expose focused editable decorations"
    );
}

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        focusable: true,
        hoverable: true,
        clickable: true,
        ..visible_state()
    }
}

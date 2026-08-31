use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    design_tokens::EditorTypographyTokens,
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiPaintPayload, UiRenderCommandKind},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn text_field_rendering_uses_central_tokens_and_validated_overrides() {
    let source = include_str!("../surface/render/text_fields.rs");

    for needle in [
        "EditorDesignTokens",
        "EditorTypographyTokens",
        "TextFieldVisual",
        "style_overrides",
        "parse_css_color",
        "value_as_f32",
    ] {
        assert!(
            source.contains(needle),
            "missing token renderer feature: {needle}"
        );
    }
    for legacy in [
        "const DEFAULT_FONT_SIZE",
        "const SURFACE_IDLE",
        "const BORDER_FOCUS",
    ] {
        assert!(
            !source.contains(legacy),
            "text field renderer must not retain local palette constant: {legacy}"
        );
    }
}

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

    let quad = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.kind == UiRenderCommandKind::Quad
        })
        .expect("empty text field should render a recessed input surface");
    assert_eq!(quad.style.background_color.as_deref(), Some("#0f1316"));
    assert_eq!(quad.style.border_color.as_deref(), Some("#262d33"));

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
    assert_eq!(text.style.foreground_color.as_deref(), Some("#a4aeb4"));
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
fn password_input_kind_publishes_only_masked_text_through_command_paint_and_artifact() {
    let source = "a\u{0301}\u{4e2d}\u{1f600}";
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.text_fields.secure"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_text_field(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(12.0, 16.0, 180.0, 30.0),
        r##"
content = 'á中😀'
input_kind = 'password'
focused = true
caret_offset = 10
selection_anchor = 0
selection_focus = 10
composition_start = 0
composition_end = 3
composition_text = 'á'
composition_restore_text = 'á'
"##,
        focusable_state(),
    );

    surface.rebuild();

    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.kind == UiRenderCommandKind::Text
        })
        .expect("secure input must publish one text command");
    assert_eq!(command.text.as_deref(), Some("\u{2022}\u{2022}\u{2022}"));
    assert_ne!(command.text.as_deref(), Some(source));
    assert_eq!(
        command.style.rich_text_format,
        zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
    );
    let editable = command
        .text_layout
        .as_ref()
        .and_then(|layout| layout.editable.as_ref())
        .expect("focused secure input keeps editable geometry");
    assert_eq!(editable.text, "\u{2022}\u{2022}\u{2022}");
    assert_eq!(editable.caret.offset, source.len());
    assert!(editable.composition.is_none());
    let artifact = command
        .text_layout
        .as_ref()
        .and_then(|layout| layout.rich_text_artifact.as_ref())
        .and_then(crate::text::resolve_resolved_text_glyph_artifact)
        .expect("secure command must publish a display-only glyph artifact");
    assert_eq!(artifact.source_text.as_ref(), "\u{2022}\u{2022}\u{2022}");
    assert_ne!(artifact.source_text.as_ref(), source);
    assert_eq!(
        surface.text_measure_cache.frame_layout_report().entry_count,
        1,
        "supported secure text layout must publish through the surface-owned layout cache"
    );
    let paint = command
        .to_transient_paint_elements(0)
        .into_iter()
        .find_map(|element| match element.payload {
            UiPaintPayload::Text { text } => Some(text),
            _ => None,
        })
        .expect("secure command must produce text paint");
    assert_eq!(paint.source_text, "\u{2022}\u{2022}\u{2022}");
    assert!(paint.composition.is_none());
    assert!(paint.runs.iter().all(|run| run.text != source));
}

#[test]
fn secure_multiline_text_edit_fails_closed_without_publishing_raw_text() {
    let source = "top secret";
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.text_fields.secure-text-edit",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_text_field_with_component(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(12.0, 16.0, 180.0, 48.0),
        "TextEdit",
        r##"
content = "top secret"
secure = true
focused = true
caret_offset = 10
selection_anchor = 0
selection_focus = 10
"##,
        focusable_state(),
    );

    surface.rebuild();

    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.kind == UiRenderCommandKind::Text
        })
        .expect("secure TextEdit must publish one safe text command");
    assert_eq!(
        command.text.as_deref(),
        Some("\u{2022}".repeat(source.len()).as_str())
    );
    assert_ne!(command.text.as_deref(), Some(source));
    let layout = command
        .text_layout
        .as_ref()
        .expect("secure TextEdit must carry a safe empty layout");
    assert!(layout.lines.is_empty());
    assert!(layout.rich_text_artifact.is_none());
    let editable = layout
        .editable
        .as_ref()
        .expect("focused secure TextEdit keeps sanitized editable state");
    assert_eq!(editable.text, "\u{2022}".repeat(source.len()));
    assert!(editable.composition.is_none());
}

#[test]
fn secure_text_field_with_multiline_attribute_fails_closed_outside_shared_text_caches() {
    let source = "top secret";
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.text_fields.secure-multiline-attribute",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_text_field(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(12.0, 16.0, 180.0, 48.0),
        r##"
content = "top secret"
secure = true
multiline = true
focused = true
caret_offset = 10
selection_anchor = 0
selection_focus = 10
"##,
        focusable_state(),
    );

    surface.rebuild();

    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.kind == UiRenderCommandKind::Text
        })
        .expect("secure multiline TextField must publish one safe text command");
    assert_eq!(
        command.text.as_deref(),
        Some("\u{2022}".repeat(source.len()).as_str())
    );
    assert_ne!(command.text.as_deref(), Some(source));
    let layout = command
        .text_layout
        .as_ref()
        .expect("secure multiline TextField must carry a safe empty layout");
    assert!(layout.lines.is_empty());
    assert!(layout.rich_text_artifact.is_none());
    let editable = layout
        .editable
        .as_ref()
        .expect("focused secure multiline TextField keeps sanitized editable state");
    assert_eq!(editable.text, "\u{2022}".repeat(source.len()));
    assert!(editable.composition.is_none());
    assert_eq!(
        surface.text_measure_cache.frame_layout_report().entry_count,
        0,
        "secure multiline layout must not enter the shared persistent cache"
    );
    assert_eq!(
        surface
            .text_measure_cache
            .frame_layout_report()
            .insert_count,
        0,
        "secure multiline layout must not publish a shared cache entry"
    );
    assert_eq!(
        surface
            .text_measure_cache
            .frame_shape_prewarm_report()
            .requested_count,
        0,
        "secure multiline text must not enter the render-command prewarm batch"
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
            && command.style.background_color.as_deref() == Some("#0f1316")
            && command.style.border_color.as_deref() == Some("#3cc7d6")
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
fn render_extract_text_field_keeps_focused_surface_when_hovered() {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.render.text_fields.focused_hovered",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_text_field(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(12.0, 12.0, 180.0, 30.0),
        r##"
content = "Focused"
focused = true
hovered = true
background_color = "#10161a"
hover_background_color = "#1b1f23"
focus_border_color = "#35c7d0"
hover_border_color = "#323a41"
"##,
        focusable_state(),
    );
    insert_text_field(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(12.0, 52.0, 180.0, 30.0),
        r##"
content = "Hovered"
hovered = true
background_color = "#10161a"
hover_background_color = "#1b1f23"
border_color = "#262d33"
hover_border_color = "#323a41"
"##,
        focusable_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(2)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_state == UiPainterResolvedState::Focused
            && command.style.background_color.as_deref() == Some("#10161a")
            && command.style.border_color.as_deref() == Some("#35c7d0")
    }));
    assert!(commands.iter().any(|command| {
        command.node_id == UiNodeId::new(3)
            && command.kind == UiRenderCommandKind::Quad
            && command.style.painter_state == UiPainterResolvedState::Hovered
            && command.style.background_color.as_deref() == Some("#22272b")
            && command.style.border_color.as_deref() == Some("#323a41")
    }));
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
            && command.style.background_color.as_deref() == Some("#1b1f23")
            && command.style.border_color.as_deref() == Some("#2c3237")
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
    assert_eq!(text.style.foreground_color.as_deref(), Some("#656f76"));
    assert!(
        text.text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref())
            .is_none(),
        "loading text field paint should not expose focused editable decorations"
    );
}

#[test]
fn render_extract_text_fields_prioritize_valid_style_overrides_and_reject_invalid_values() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.render.text_fields.overrides"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 96.0))
            .with_state_flags(visible_state()),
    );
    insert_text_field_with_style_overrides(
        &mut surface,
        UiNodeId::new(2),
        UiFrame::new(12.0, 12.0, 128.0, 32.0),
        r##"
content = "Override"
background_color = "#10161a"
border_color = "#243238"
foreground_color = "#d6e2e5"
"##,
        r##"
background_color = "#254c5a"
border_color = "#4c9dab"
foreground_color = "#eef8fa"
layout_padding_left = 16.0
font_size = 12.0
line_height_ratio = 1.5
"##,
        focusable_state(),
    );
    insert_text_field_with_style_overrides(
        &mut surface,
        UiNodeId::new(3),
        UiFrame::new(156.0, 12.0, 128.0, 32.0),
        r##"
content = "Fallback"
"##,
        r##"
background_color = "not-a-color"
border_width = -1.0
layout_padding_left = -4.0
font_size = 0.0
line_height_ratio = 0.0
"##,
        focusable_state(),
    );

    surface.rebuild();

    let commands = &surface.render_extract.list.commands;
    let overridden_surface = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.kind == UiRenderCommandKind::Quad
        })
        .expect("overridden input should render a surface");
    assert_eq!(
        overridden_surface.style.background_color.as_deref(),
        Some("#254c5a")
    );
    assert_eq!(
        overridden_surface.style.border_color.as_deref(),
        Some("#4c9dab")
    );
    let overridden_text = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2) && command.text.is_some())
        .expect("overridden input should render text");
    assert_eq!(overridden_text.frame.x, 28.0);
    assert_eq!(
        overridden_text.style.foreground_color.as_deref(),
        Some("#eef8fa")
    );
    assert_eq!(overridden_text.style.font_size, 12.0);
    assert_eq!(overridden_text.style.line_height, 18.0);

    let fallback_surface = commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(3) && command.kind == UiRenderCommandKind::Quad
        })
        .expect("fallback input should render a surface");
    assert_eq!(
        fallback_surface.style.background_color.as_deref(),
        Some("#0f1316")
    );
    assert_eq!(
        fallback_surface.style.border_color.as_deref(),
        Some("#262d33")
    );
    assert_eq!(fallback_surface.style.border_width, 1.0);
    let fallback_text = commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(3) && command.text.is_some())
        .expect("fallback input should render text");
    assert_eq!(fallback_text.frame.x, 164.0);
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

fn insert_text_field(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    insert_text_field_with_component(
        surface,
        node_id,
        frame,
        "InputField",
        attributes,
        state_flags,
    );
}

fn insert_text_field_with_component(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    component: &str,
    attributes: &str,
    state_flags: UiStateFlags,
) {
    insert_text_field_with_style_overrides_and_component(
        surface,
        node_id,
        frame,
        component,
        attributes,
        "",
        state_flags,
    );
}

fn insert_text_field_with_style_overrides(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    attributes: &str,
    style_overrides: &str,
    state_flags: UiStateFlags,
) {
    insert_text_field_with_style_overrides_and_component(
        surface,
        node_id,
        frame,
        "InputField",
        attributes,
        style_overrides,
        state_flags,
    );
}

#[allow(clippy::too_many_arguments)]
fn insert_text_field_with_style_overrides_and_component(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    frame: UiFrame,
    component: &str,
    attributes: &str,
    style_overrides: &str,
    state_flags: UiStateFlags,
) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(node_id, UiNodePath::new("root/text-field"))
                .with_frame(frame)
                .with_state_flags(state_flags)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    style_overrides: toml::from_str(style_overrides).unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
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

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        focusable: true,
        hoverable: true,
        clickable: true,
        ..visible_state()
    }
}

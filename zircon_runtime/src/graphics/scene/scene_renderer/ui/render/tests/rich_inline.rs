use super::*;

#[test]
fn screen_space_ui_rich_inline_reuses_the_command_parse() {
    let source = include_str!("../rich_text.rs");
    let parse_call = ["parse_rich", "_text("].concat();

    assert_eq!(source.matches(&parse_call).count(), 1);
}

#[test]
fn screen_space_ui_plan_places_html_inline_image_without_placeholder_glyph() {
    let markup = "before<img src=\"res://icons/star.png\" width=\"16\" height=\"24\">after";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::Html,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 220.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(8),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(260, 100),
    );

    assert!(
        plan.vertices.is_empty(),
        "inline images must not fall back to a solid-color placeholder quad"
    );
    assert_eq!(plan.images.len(), 1);
    assert_eq!(
        plan.images[0].texture,
        crate::core::resource::ResourceId::from_stable_label("res://icons/star.png")
    );
    assert!((plan.images[0].frame.width - 16.0).abs() < 0.01);
    assert!((plan.images[0].frame.height - 24.0).abs() < 0.01);
    assert!(plan.images[0].frame.x > frame.x);
    assert_eq!(plan.native_texts.len(), 2);
    assert_eq!(plan.native_texts[0].text, "before");
    assert_eq!(plan.native_texts[1].text, "after");
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| !batch.text.contains('\u{fffc}'))
    );
}

#[test]
fn screen_space_ui_plan_renders_bbcode_icon_as_glyph_batch() {
    let markup = "before[icon=★|Zircon Icons]after";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 18.0,
        line_height: 22.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::BbCode,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 240.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-icon"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(12),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(280, 100),
    );

    let icon = plan
        .native_texts
        .iter()
        .find(|batch| batch.text == "★")
        .expect("inline icon glyph batch");
    assert_eq!(icon.font_family.as_deref(), Some("Zircon Icons"));
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| !batch.text.contains('\u{fffc}'))
    );
    assert!(plan.images.is_empty());
    assert!(plan.vertices.is_empty());
}

#[test]
fn screen_space_ui_plan_keeps_inline_image_retained_by_ellipsis() {
    let markup = "a<img src=\"res://icons/star.png\" width=\"16\" height=\"24\"> trailing";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 24.0,
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::Ellipsis,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::Html,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 34.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    assert!(layout.lines[0].ellipsized);

    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-ellipsis"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(10),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(100, 80),
    );

    assert_eq!(plan.images.len(), 1);
    assert!((plan.images[0].frame.width - 16.0).abs() < 0.01);
    assert!(
        plan.native_texts
            .iter()
            .all(|batch| !batch.text.contains('\u{fffc}'))
    );
}

#[test]
fn screen_space_ui_plan_places_rtl_inline_image_at_visual_run_offset() {
    let markup = "אב <img src=\"res://icons/star.png\" width=\"16\" height=\"24\"> גד";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        text_direction: UiTextDirection::RightToLeft,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::Html,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 220.0, 40.0);
    let layout = layout_text(markup, &style, frame, None);
    let line = &layout.lines[0];
    let inline_run = line
        .runs
        .iter()
        .find(|run| run.text == "\u{fffc}")
        .expect("resolved visual inline run");
    let inline_visual_index = line.text[..inline_run.visual_range.start]
        .graphemes(true)
        .count();
    let expected_x = line.frame.x
        + line
            .glyph_advances
            .iter()
            .take(inline_visual_index)
            .sum::<f32>();
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-rtl"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(9),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(260, 100),
    );

    assert_eq!(plan.images.len(), 1);
    assert!((plan.images[0].frame.x - expected_x).abs() < 0.01);
    assert!((plan.images[0].frame.width - 16.0).abs() < 0.01);
}

#[test]
fn screen_space_ui_plan_places_vertical_rl_inline_image_at_main_axis_offset() {
    let markup = "甲<img src=\"res://icons/star.png\" width=\"18\" height=\"24\">乙";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Glyph,
        text_writing_mode: UiTextWritingMode::VerticalRl,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::Html,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 48.0, 35.0);
    let layout = layout_text(markup, &style, frame, None);
    let line = &layout.lines[0];
    let inline_run = line
        .runs
        .iter()
        .find(|run| run.text == "\u{fffc}")
        .expect("resolved vertical inline run");
    let inline_visual_index = line.text[..inline_run.visual_range.start]
        .graphemes(true)
        .count();
    let expected_y = line.frame.y
        + line
            .glyph_advances
            .iter()
            .take(inline_visual_index)
            .sum::<f32>();
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-inline-vertical"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(11),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(100, 100),
    );

    assert_eq!(plan.images.len(), 1);
    assert!((plan.images[0].frame.y - expected_y).abs() < 0.01);
    assert!((plan.images[0].frame.width - 18.0).abs() < 0.01);
    assert!((plan.images[0].frame.height - 24.0).abs() < 0.01);
}

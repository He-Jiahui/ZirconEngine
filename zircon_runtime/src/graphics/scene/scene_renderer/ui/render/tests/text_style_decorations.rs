use super::*;
use zircon_runtime_interface::ui::surface::UiTextDecorations;

#[test]
fn render_text_plain_decorations_project_color_and_resolved_baseline() {
    let text = "Plain decorations";
    let frame = UiFrame::new(10.0, 20.0, 180.0, 32.0);
    let style = UiResolvedStyle {
        foreground_color: Some("#ccddee".to_string()),
        font_size: 20.0,
        line_height: 24.0,
        text_render_mode: UiTextRenderMode::Native,
        text_decorations: UiTextDecorations {
            underline: true,
            strikethrough: true,
            underline_color: Some("#ff000080".to_string()),
            strikethrough_color: Some("#00ff00".to_string()),
        },
        ..UiResolvedStyle::default()
    };
    let layout = layout_text(text, &style, frame, None);
    let expected_baseline = layout.lines[0].frame.y + layout.lines[0].baseline;
    let plan = decoration_plan(text, style, layout, frame, 0.5);

    assert_eq!(plan.native_texts.len(), 1);
    let batch = &plan.native_texts[0];
    assert!(batch.text_decorations.underline);
    assert!(batch.text_decorations.strikethrough);
    assert_eq!(
        batch.text_decorations.underline_color[0..3],
        [1.0, 0.0, 0.0]
    );
    assert!((batch.text_decorations.underline_color[3] - (128.0 / 255.0) * 0.5).abs() < 0.0001);
    assert_eq!(
        batch.text_decorations.strikethrough_color,
        [0.0, 1.0, 0.0, 0.5]
    );
    assert_eq!(batch.text_decoration_baseline, Some(expected_baseline));
}

#[test]
fn render_text_rich_underline_and_strike_project_without_fixed_one_pixel_quads() {
    let markup = "<u>Under</u><s>Strike</s>";
    let frame = UiFrame::new(10.0, 20.0, 180.0, 32.0);
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 20.0,
        line_height: 24.0,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::Html,
        ..UiResolvedStyle::default()
    };
    let layout = layout_text(markup, &style, frame, None);
    let plan = decoration_plan(markup, style, layout, frame, 1.0);

    let underline = plan
        .native_texts
        .iter()
        .find(|batch| batch.text == "Under")
        .expect("underline rich run");
    let strikeout = plan
        .native_texts
        .iter()
        .find(|batch| batch.text == "Strike")
        .expect("strikeout rich run");
    assert!(underline.text_decorations.underline);
    assert!(!underline.text_decorations.strikethrough);
    assert!(!strikeout.text_decorations.underline);
    assert!(strikeout.text_decorations.strikethrough);
    assert!(
        plan.vertices.is_empty(),
        "rich style decorations must be emitted by the face-metric text path, not fixed UI quads"
    );
}

fn decoration_plan(
    text: &str,
    style: UiResolvedStyle,
    layout: UiResolvedTextLayout,
    frame: UiFrame,
    opacity: f32,
) -> PlannedScreenSpaceUi {
    plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.text-decorations"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(42),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(text.to_string()),
                    image: None,
                    opacity,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(240, 100),
    )
}

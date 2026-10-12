use super::*;

#[test]
fn empty_failed_rich_paint_projection_cannot_fall_through_to_plain_layout_batches() {
    let markup = "Alpha **Beta**";
    let style = UiResolvedStyle {
        font_size: 12.0,
        line_height: 16.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::MarkdownInlineV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 180.0, 24.0);
    let mut layout = layout_text(markup, &style, frame, None);
    assert!(layout.lines[0].runs.len() >= 2);
    layout.lines[0].runs[1].visual_range.end = layout.lines[0].text.len() + 1;
    let command = UiRenderCommand {
        node_id: UiNodeId::new(618),
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            background_color: Some("#112233".to_owned()),
            ..style
        },
        text_layout: Some(layout),
        text: Some(markup.to_owned()),
        image: None,
        opacity: 1.0,
    };
    let paint_elements = command.to_transient_paint_elements(0);
    let text_paint = paint_elements
        .iter()
        .find_map(|element| match &element.payload {
            zircon_runtime_interface::ui::surface::UiPaintPayload::Text { text } => Some(text),
            _ => None,
        })
        .expect("text paint payload");
    assert!(text_paint.runs.is_empty());

    let viewport = UiFrame::new(0.0, 0.0, 220.0, 64.0);
    let route_tree_id = Arc::<str>::from("runtime.ui.rich-empty-failed-projection");
    let backgrounds = ScreenSpaceUiBackgroundTracker::default();
    let mut plan = PlannedScreenSpaceUi::default();
    let rejected = plan_command_batches(
        &command,
        &paint_elements,
        &route_tree_id,
        command.node_id,
        viewport,
        1.0,
        &backgrounds,
        &mut plan,
    );

    assert!(rejected);
    assert!(plan.auto_texts.is_empty());
    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert!(plan.images.is_empty());
    assert_eq!(plan.vertices.len(), 6);
    assert_eq!(
        plan.resolved_glyph_artifact_routes,
        ScreenSpaceUiResolvedGlyphArtifactRouteReport {
            incomplete_artifact_count: 1,
            rejected_command_count: 1,
            ..ScreenSpaceUiResolvedGlyphArtifactRouteReport::default()
        }
    );
}

#[test]
fn rich_presentation_reuses_layout_style_admission_for_invalid_overrides() {
    let frame = UiFrame::new(10.0, 20.0, 40.0, 18.0);
    let command = UiRenderCommand {
        node_id: UiNodeId::new(619),
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            font_family: Some("Zircon Base".to_owned()),
            font_size: 12.0,
            line_height: 18.0,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some("A".to_owned()),
        image: None,
        opacity: 1.0,
    };
    let run = zircon_runtime_interface::ui::surface::UiTextPaintRun {
        kind: UiTextRunKind::Plain,
        text: "A".to_owned(),
        source_range: UiTextRange { start: 0, end: 1 },
        visual_range: UiTextRange { start: 0, end: 1 },
        frame,
        color: None,
        font: None,
        font_family: Some("Zircon Base".to_owned()),
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 12.0,
        line_height: 18.0,
        style: Default::default(),
    };
    let rich_run = crate::text::StyledRun {
        byte_range: (0, 1),
        style: crate::text::StyleOverride {
            font_size: Some(0.0),
            family: Some(crate::text::FontFamilyName::from("")),
            ..crate::text::StyleOverride::default()
        },
        inline: None,
        link: None,
    };
    let mut plan = PlannedScreenSpaceUi::default();

    let presentation = rich_text::prepare_text_run(
        &command,
        &run,
        Some(&rich_run),
        UiFrame::new(0.0, 0.0, 100.0, 100.0),
        [1.0; 4],
        &mut plan,
    );

    assert_eq!(presentation.font_size, 12.0);
    assert_eq!(presentation.line_height, 18.0);
    assert_eq!(presentation.font_family.as_deref(), Some("Zircon Base"));
}

use super::*;
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommandKind, UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextDirection,
};

#[test]
fn text_decoration_colors_preserve_text_fallback_opacity() {
    let resolved = resolve_text_decorations(
        &UiTextDecorations {
            underline: true,
            underline_color: Some("#ff000080".to_string()),
            ..UiTextDecorations::default()
        },
        [0.2, 0.3, 0.4, 0.25],
        0.5,
    );

    assert_eq!(resolved.underline_color[0..3], [1.0, 0.0, 0.0]);
    assert!((resolved.underline_color[3] - (128.0 / 255.0) * 0.5).abs() < 0.0001);
    assert_eq!(resolved.strikethrough_color, [0.2, 0.3, 0.4, 0.25]);
}

#[test]
fn resolved_baseline_rejects_non_finite_layout_coordinates() {
    let mut command = UiRenderCommand {
        node_id: UiNodeId::new(1),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(8.0, 12.0, 40.0, 24.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: Some(UiResolvedTextLayout {
            lines: vec![UiResolvedTextLine {
                text: "A".to_string(),
                placement_frame: UiFrame::default(),
                frame: UiFrame::new(8.0, 12.0, 40.0, 24.0),
                source_range: UiTextRange { start: 0, end: 1 },
                visual_range: UiTextRange { start: 0, end: 1 },
                measured_width: 12.0,
                glyph_advances: vec![12.0],
                baseline: 16.0,
                direction: UiTextDirection::LeftToRight,
                runs: Vec::new(),
                ellipsized: false,
            }],
            ..UiResolvedTextLayout::default()
        }),
        text: Some("A".to_string()),
        image: None,
        opacity: 1.0,
    };

    assert_eq!(
        resolved_text_decoration_baseline(&command, None, UiTextWritingMode::HorizontalTb),
        Some(28.0)
    );
    command.text_layout.as_mut().expect("resolved layout").lines[0].baseline = f32::NAN;
    assert_eq!(
        resolved_text_decoration_baseline(&command, None, UiTextWritingMode::HorizontalTb),
        None
    );
    command.text_layout.as_mut().expect("resolved layout").lines[0].baseline = f32::INFINITY;
    assert_eq!(
        resolved_text_decoration_baseline(&command, None, UiTextWritingMode::VerticalRl),
        None
    );
}

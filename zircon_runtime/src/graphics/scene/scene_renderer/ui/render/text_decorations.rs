use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiTextDecorations, UiTextRange, UiTextWritingMode,
};

use super::color::parse_hex_color;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiTextDecorations {
    pub(in crate::graphics::scene::scene_renderer::ui) underline: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) strikethrough: bool,
    pub(in crate::graphics::scene::scene_renderer::ui) underline_color: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::ui) strikethrough_color: [f32; 4],
}

pub(super) fn resolve_text_decorations(
    decorations: &UiTextDecorations,
    fallback_color: [f32; 4],
    opacity: f32,
) -> ScreenSpaceUiTextDecorations {
    ScreenSpaceUiTextDecorations {
        underline: decorations.underline,
        strikethrough: decorations.strikethrough,
        underline_color: resolved_decoration_color(
            decorations.underline_color.as_deref(),
            fallback_color,
            opacity,
        ),
        strikethrough_color: resolved_decoration_color(
            decorations.strikethrough_color.as_deref(),
            fallback_color,
            opacity,
        ),
    }
}

pub(super) fn resolved_text_decoration_baseline(
    command: &UiRenderCommand,
    source_range: Option<UiTextRange>,
    writing_mode: UiTextWritingMode,
) -> Option<f32> {
    let layout = command.text_layout.as_ref()?;
    let line = source_range
        .and_then(|range| {
            layout.lines.iter().find(|line| {
                line.source_range.start <= range.start && range.end <= line.source_range.end
            })
        })
        .or_else(|| layout.lines.first())?;
    let baseline = if matches!(writing_mode, UiTextWritingMode::VerticalRl) {
        line.frame.x + line.baseline
    } else {
        line.frame.y + line.baseline
    };
    baseline.is_finite().then_some(baseline)
}

fn resolved_decoration_color(authored: Option<&str>, fallback: [f32; 4], opacity: f32) -> [f32; 4] {
    authored
        .and_then(|color| parse_hex_color(color, opacity))
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::event_ui::UiNodeId;
    use zircon_runtime_interface::ui::surface::{
        UiRenderCommandKind, UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine,
        UiTextDirection,
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
}

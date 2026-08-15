use std::sync::Arc;

use crate::text::{text_style, ShapedGlyphRun, SharedTextLayoutSession, TextRange};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRange};

pub fn shape_text_line(text: &str, style: &UiResolvedStyle) -> ShapedGlyphRun {
    crate::profile_scope!("runtime", "text.surface", "shape_text_line");
    let mut session = SharedTextLayoutSession::new();
    Arc::unwrap_or_clone(session.shape_horizontal_line(
        text,
        &text_style(style),
        style.text_direction.into(),
        TextRange {
            start: 0,
            end: text.len(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRange};

    use super::shape_text_line;
    use crate::ui::surface::measure_text_source_range_width;

    #[cfg(feature = "profiling")]
    #[test]
    fn shape_text_line_emits_one_surface_profile_span() {
        let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
        let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
        config.session_id = "surface-text-shape-profile".to_owned();
        config.max_spans = 2;
        assert!(crate::core::diagnostics::profiling::start_capture(config).active);

        let _ = shape_text_line("Profile", &UiResolvedStyle::default());

        let snapshot = crate::core::diagnostics::profiling::snapshot();
        assert!(
            !crate::core::diagnostics::profiling::reset_capture().active,
            "surface shape profiling capture must reset before another test starts"
        );
        assert_eq!(
            snapshot
                .spans
                .iter()
                .filter(|span| span.category == "text.surface" && span.name == "shape_text_line")
                .count(),
            1
        );
    }

    #[test]
    fn shape_text_line_exposes_surface_shaped_run_without_font_family_requirement() {
        let style = UiResolvedStyle {
            font_size: 13.0,
            line_height: 16.0,
            ..UiResolvedStyle::default()
        };

        let run = shape_text_line("Preview", &style);
        let line = run.lines.first().expect("shaped line");

        assert_eq!(run.source_range.start, 0);
        assert_eq!(run.source_range.end, "Preview".len());
        assert!(!line.glyphs.is_empty());
        assert!(line.measured_width.is_finite());
        assert!(line.measured_width > 0.0);
    }

    #[test]
    fn measure_text_source_range_width_exposes_shaped_absolute_source_ranges() {
        let style = UiResolvedStyle {
            font_size: 13.0,
            line_height: 16.0,
            ..UiResolvedStyle::default()
        };
        let text = "xxWi";
        let prefix_end = "xx".len();
        let w_end = prefix_end + "W".len();
        let run = shape_text_line(text, &style);
        let line_width = run.lines.first().expect("shaped line").measured_width;

        let full = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: 0,
                end: text.len(),
            },
        );
        let prefix = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: 0,
                end: prefix_end,
            },
        );
        let w = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: prefix_end,
                end: w_end,
            },
        );
        let i = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: w_end,
                end: text.len(),
            },
        );
        let wi = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: prefix_end,
                end: text.len(),
            },
        );

        assert!((full - line_width).abs() < 0.1);
        assert!((prefix + wi - full).abs() < 0.1);
        assert!((w + i - wi).abs() < 0.1);
        assert_eq!(
            measure_text_source_range_width(
                text,
                &style,
                UiTextRange {
                    start: text.len() + 4,
                    end: text.len() + 8,
                },
            ),
            0.0
        );
        assert_eq!(
            measure_text_source_range_width(
                text,
                &style,
                UiTextRange {
                    start: w_end,
                    end: prefix_end,
                },
            ),
            0.0
        );
    }
}

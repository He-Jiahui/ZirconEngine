use crate::core::framework::text::TextDirection;
use crate::text::{BackendShapeRequest, ShapedGlyphRotation, VerticalMode};
use crate::text::{TextRange, TextStyle};

use super::projection::vertical_backend_direction;
use super::{apply_vertical_layout, apply_vertical_layout_with_native_metrics};
use crate::text::shaping::{TextShapeRunProvider, VerticalTextShapeRunProvider};
use std::sync::Arc;

use super::backend::VerticalBackendDirection;

#[cfg(target_os = "windows")]
use super::backend::shape_vertical_run;

#[test]
fn text_vertical_projection_advances_cluster_heads_on_y() {
    let style = TextStyle {
        font_size: 20.0,
        line_height: 24.0,
        ..TextStyle::default()
    };
    let text = "本A。";
    let request = BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 4,
            end: 4 + text.len(),
        },
        VerticalMode::Mixed,
    );
    let mut shaped = vertical_fixture(text, request.source_range);

    apply_vertical_layout(&mut shaped, request, None);

    let glyphs = &shaped.lines[0].glyphs;
    assert_eq!(glyphs[0].rotation, ShapedGlyphRotation::None);
    assert_eq!(glyphs[1].rotation, ShapedGlyphRotation::Cw90);
    assert_eq!(glyphs[2].rotation, ShapedGlyphRotation::None);
    assert_eq!(glyphs[0].y, 0.0);
    assert_eq!(glyphs[1].y, 20.0);
    assert_eq!(glyphs[2].y, 31.0);
    assert_eq!(shaped.measured_height, 51.0);
    assert_eq!(shaped.measured_width, 20.0);
}

#[test]
fn text_vertical_upright_cluster_prefers_native_vertical_advance() {
    let style = TextStyle {
        font_size: 20.0,
        line_height: 24.0,
        ..TextStyle::default()
    };
    let text = "本";
    let request = BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
        VerticalMode::Mixed,
    );
    let mut shaped = vertical_fixture(text, request.source_range);

    apply_vertical_layout_with_native_metrics(&mut shaped, request, |_, _, _| Some(37.0));

    assert_eq!(shaped.lines[0].glyphs[0].advance, 37.0);
    assert_eq!(shaped.lines[0].measured_width, 37.0);
    assert_eq!(shaped.measured_height, 37.0);
}

#[test]
fn text_vertical_bidi_direction_maps_ltr_to_ttb_and_rtl_to_btt() {
    assert_eq!(
        vertical_backend_direction(TextDirection::LeftToRight),
        VerticalBackendDirection::TopToBottom
    );
    assert_eq!(
        vertical_backend_direction(TextDirection::RightToLeft),
        VerticalBackendDirection::BottomToTop
    );
}

#[test]
fn text_vertical_database_has_no_per_glyph_compatibility_helper() {
    let source = include_str!("../../font/vertical_metrics.rs");

    assert!(!source.contains("vertical_glyph_advance_px"));
}

#[cfg(target_os = "windows")]
#[test]
fn text_vertical_cjk_uses_backend_face_vmtx_advance() {
    use crate::text::font::shared_font_database_snapshot;
    use crate::text::shaping::shape_text;

    let style = TextStyle {
        font_family: Some("Microsoft YaHei UI".to_string()),
        font_size: 20.0,
        line_height: 24.0,
        language: Some("zh-Hans".to_string()),
        ..TextStyle::default()
    };
    let text = "本";
    let shaped = shape_text(BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
        VerticalMode::Mixed,
    ));
    let glyph = shaped.lines[0].glyphs.first().expect("shaped CJK glyph");
    let face = glyph.font_id.expect("actual backend face ID");
    let (_, database) = shared_font_database_snapshot();
    let native_advance = database
        .vertical_metrics(face, style.font_size)
        .and_then(|metrics| metrics.glyph_advance_px(glyph.glyph_id))
        .expect("Microsoft YaHei UI vmtx metrics");

    assert!((glyph.advance - native_advance).abs() < 0.01);
    assert!(native_advance > 0.0);
}

#[cfg(target_os = "windows")]
#[test]
fn text_vertical_backend_shapes_ttb_and_btt_with_signed_y_advances() {
    use crate::text::font::shared_font_database_snapshot;
    use crate::text::shaping::shape_text;

    let style = TextStyle {
        font_family: Some("Microsoft YaHei UI".to_string()),
        font_size: 20.0,
        line_height: 24.0,
        language: Some("zh-Hans".to_string()),
        ..TextStyle::default()
    };
    let text = "布局";
    let horizontal = shape_text(BackendShapeRequest::horizontal(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    ));
    let face = horizontal.lines[0].glyphs[0]
        .font_id
        .expect("actual Microsoft YaHei UI backend face");
    let (_, database) = shared_font_database_snapshot();

    let ttb = shape_vertical_run(
        &database,
        face,
        text,
        VerticalBackendDirection::TopToBottom,
        style.language.as_deref(),
        &[],
        true,
        style.font_weight,
        style.font_size,
    )
    .expect("TTB rustybuzz shaping");
    let btt = shape_vertical_run(
        &database,
        face,
        text,
        VerticalBackendDirection::BottomToTop,
        style.language.as_deref(),
        &[],
        true,
        style.font_weight,
        style.font_size,
    )
    .expect("BTT rustybuzz shaping");

    assert_eq!(ttb.glyphs.len(), 2);
    assert_eq!(btt.glyphs.len(), 2);
    assert!(ttb.glyphs.iter().all(|glyph| glyph.y_advance < 0.0));
    assert!(btt.glyphs.iter().all(|glyph| glyph.y_advance < 0.0));
    assert!(ttb.glyphs[0].source_offset < ttb.glyphs[1].source_offset);
    assert!(btt.glyphs[0].source_offset > btt.glyphs[1].source_offset);
}

#[cfg(target_os = "windows")]
#[test]
fn text_vertical_shape_path_consumes_ttb_backend_glyphs() {
    use crate::text::font::shared_font_database_snapshot;
    use crate::text::shaping::shape_text;

    let style = TextStyle {
        font_family: Some("Microsoft YaHei UI".to_string()),
        font_size: 20.0,
        line_height: 24.0,
        language: Some("zh-Hans".to_string()),
        ..TextStyle::default()
    };
    let text = "布局";
    let shaped = shape_text(BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
        VerticalMode::Mixed,
    ));
    let glyphs = &shaped.lines[0].glyphs;
    let face = glyphs[0].font_id.expect("actual YaHei backend face");
    let (_, database) = shared_font_database_snapshot();
    let backend = shape_vertical_run(
        &database,
        face,
        text,
        VerticalBackendDirection::TopToBottom,
        style.language.as_deref(),
        &[],
        true,
        style.font_weight,
        style.font_size,
    )
    .expect("TTB rustybuzz shaping");

    assert_eq!(glyphs.len(), backend.glyphs.len());
    assert_eq!(
        glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        backend
            .glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>()
    );
    for (glyph, backend_glyph) in glyphs.iter().zip(&backend.glyphs) {
        assert!((glyph.advance - backend_glyph.y_advance.abs()).abs() < 0.01);
    }
}

#[cfg(target_os = "windows")]
#[test]
fn text_vertical_rtl_shape_path_consumes_btt_cluster_order() {
    use crate::text::shaping::shape_text;

    let style = TextStyle {
        font_family: Some("Segoe UI".to_string()),
        font_size: 20.0,
        line_height: 24.0,
        language: Some("he".to_string()),
        ..TextStyle::default()
    };
    let text = "אב";
    let shaped = shape_text(BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::RightToLeft,
        TextRange {
            start: 0,
            end: text.len(),
        },
        VerticalMode::Upright,
    ));
    let glyphs = &shaped.lines[0].glyphs;

    assert_eq!(glyphs.len(), 2);
    assert!(glyphs[0].source_range.start > glyphs[1].source_range.start);
    assert!(glyphs.iter().all(|glyph| glyph.advance > 0.0));
}

#[test]
fn vertical_provider_routes_measurement_requests_to_vertical_shape_path() {
    #[derive(Default)]
    struct RecordingProvider {
        horizontal_calls: usize,
        vertical_calls: usize,
    }

    impl TextShapeRunProvider for RecordingProvider {
        fn shape_horizontal_line_with_kerning(
            &mut self,
            text: &str,
            _style: &TextStyle,
            _direction: TextDirection,
            source_range: TextRange,
            _include_kerning: bool,
        ) -> Arc<crate::text::ShapedGlyphRun> {
            self.horizontal_calls += 1;
            Arc::new(vertical_fixture(text, source_range))
        }

        fn shape_vertical_line_with_kerning(
            &mut self,
            text: &str,
            _style: &TextStyle,
            _direction: TextDirection,
            source_range: TextRange,
            _vertical_mode: VerticalMode,
            _include_kerning: bool,
        ) -> Arc<crate::text::ShapedGlyphRun> {
            self.vertical_calls += 1;
            Arc::new(vertical_fixture(text, source_range))
        }
    }

    let style = TextStyle::default();
    let mut recording = RecordingProvider::default();
    let mut vertical = VerticalTextShapeRunProvider::new(&mut recording, VerticalMode::Mixed);
    let _ = vertical.shape_horizontal_line_with_kerning(
        "本A。",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 7 },
        true,
    );

    assert_eq!(recording.horizontal_calls, 0);
    assert_eq!(recording.vertical_calls, 1);
}

fn vertical_fixture(text: &str, source_range: TextRange) -> crate::text::ShapedGlyphRun {
    use crate::text::{
        ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRun, ShapedGlyphScript, ShapedTextLine,
        TextOrientation,
    };

    let mut local_start = 0_usize;
    let advances = [18.0, 11.0, 8.0];
    let glyphs = text
        .chars()
        .zip(advances)
        .enumerate()
        .map(|(index, (character, advance))| {
            let end = local_start + character.len_utf8();
            let glyph = ShapedGlyph {
                glyph_id: index as u32 + 1,
                font_id: None,
                font_instance_id: None,
                source_range: TextRange {
                    start: source_range.start + local_start,
                    end: source_range.start + end,
                },
                visual_range: TextRange {
                    start: local_start,
                    end,
                },
                advance,
                x: advances[..index].iter().sum(),
                y: 0.0,
                offset_x: 0.0,
                offset_y: 0.0,
                direction: TextDirection::LeftToRight,
                bidi_level: 0,
                cluster_flags: ShapedGlyphClusterFlags {
                    cluster_start: true,
                    ..ShapedGlyphClusterFlags::default()
                },
                rotation: ShapedGlyphRotation::None,
                script: ShapedGlyphScript::default(),
            };
            local_start = end;
            glyph
        })
        .collect::<Vec<_>>();

    ShapedGlyphRun {
        source_text: text.to_string(),
        source_range,
        direction: TextDirection::LeftToRight,
        orientation: TextOrientation::Vertical,
        vertical_mode: VerticalMode::Mixed,
        include_kerning: true,
        measured_width: advances.into_iter().sum(),
        measured_height: 24.0,
        lines: vec![ShapedTextLine {
            line_index: 0,
            text: text.to_string(),
            source_range,
            visual_range: TextRange {
                start: 0,
                end: text.len(),
            },
            measured_width: advances.into_iter().sum(),
            baseline: 16.0,
            line_height: 24.0,
            glyphs,
        }],
    }
}

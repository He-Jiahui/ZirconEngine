use super::*;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLine, UiResolvedTextRun, UiTextRenderMode, UiTextRunKind,
};

#[test]
fn layout_cache_heap_estimate_includes_owned_line_run_and_advance_storage() {
    let line_text = "owned-line".to_string();
    let run_text = "owned-run".to_string();
    let layout = UiResolvedTextLayout {
        lines: vec![UiResolvedTextLine {
            text: line_text.clone(),
            placement_frame: UiFrame::default(),
            frame: UiFrame::new(0.0, 0.0, 120.0, 20.0),
            source_range: UiTextRange {
                start: 0,
                end: line_text.len(),
            },
            visual_range: UiTextRange {
                start: 0,
                end: line_text.len(),
            },
            measured_width: 80.0,
            glyph_advances: vec![8.0; 4],
            baseline: 14.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: run_text.clone(),
                source_range: UiTextRange {
                    start: 0,
                    end: run_text.len(),
                },
                visual_range: UiTextRange {
                    start: 0,
                    end: run_text.len(),
                },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    };
    let resolution = UiTextLayoutResolution {
        layout,
        size: UiSize::new(120.0, 20.0),
        first_baseline: 14.0,
        source_hash: EphemeralCacheHash::from_hashable("owned-line"),
    };

    assert!(
        resolution.estimated_cache_heap_bytes()
            >= line_text.len() + run_text.len() + 4 * std::mem::size_of::<f32>()
    );
}

#[test]
fn style_key_heap_estimate_includes_owned_font_and_language_strings() {
    let key = UiTextStyleKey::from_style(&UiResolvedStyle {
        font_family: Some("Zircon Sans".to_string()),
        language: Some("zh-Hans-CN".to_string()),
        ..UiResolvedStyle::default()
    });

    assert_eq!(
        key.estimated_heap_bytes(),
        "Zircon Sans".len() + "zh-Hans-CN".len()
    );
}

#[test]
fn style_key_encodes_clamp_overflow_float_bits() {
    let mut style = UiResolvedStyle {
        text_overflow: UiTextOverflow::ClampFontSize {
            min_px: 8.0,
            max_px: 18.0,
        },
        ..UiResolvedStyle::default()
    };
    let key = UiTextStyleKey::from_style(&style);

    assert_eq!(key, UiTextStyleKey::from_style(&style));

    style.text_overflow = UiTextOverflow::ClampFontSize {
        min_px: 8.0,
        max_px: 19.0,
    };
    assert_ne!(key, UiTextStyleKey::from_style(&style));
}

#[test]
fn style_key_encodes_tab_size_bits() {
    let mut style = UiResolvedStyle {
        tab_size: 4.0,
        ..UiResolvedStyle::default()
    };
    let key = UiTextStyleKey::from_style(&style);

    style.tab_size = 6.0;

    assert_ne!(key, UiTextStyleKey::from_style(&style));
}

#[test]
fn style_key_encodes_font_weight() {
    let mut style = UiResolvedStyle {
        font_weight: 400,
        ..UiResolvedStyle::default()
    };
    let key = UiTextStyleKey::from_style(&style);

    style.font_weight = 600;

    assert_ne!(key, UiTextStyleKey::from_style(&style));
}

#[test]
fn preedit_request_disables_viewport_layout() {
    let style = UiResolvedStyle::default();
    let preedit = UiPreeditSpan {
        range: UiTextRange { start: 0, end: 0 },
        text: "x".to_string(),
    };
    let viewport = UiTextViewport::new(40.0, 20.0, 2).expect("finite viewport");
    let request =
        UiTextLayoutRequest::new("source", &style, UiFrame::new(0.0, 0.0, 120.0, 80.0), None)
            .with_viewport(viewport)
            .with_preedit(&preedit);

    assert_eq!(request.viewport, Some(viewport));
    assert_eq!(request.layout_viewport(), None);
}

#[test]
fn viewport_derives_a_document_local_offset_from_absolute_frames() {
    let viewport = UiTextViewport::from_document_and_clip(
        UiFrame::new(20.0, -180.0, 240.0, 1_600.0),
        UiFrame::new(20.0, 60.0, 240.0, 80.0),
    )
    .expect("finite document and clip frames");

    assert_eq!(viewport.offset_y, 240.0);
    assert_eq!(viewport.extent_y, 80.0);
    assert_eq!(viewport.overscan_screens, 2);
}

#[test]
fn style_key_normalizes_and_separates_run_language() {
    let mut style = UiResolvedStyle {
        language: Some(" ZH-hans ".to_string()),
        ..UiResolvedStyle::default()
    };
    let simplified = UiTextStyleKey::from_style(&style);

    style.language = Some("zh-HANS".to_string());
    assert_eq!(simplified, UiTextStyleKey::from_style(&style));

    style.language = Some("ja".to_string());
    assert_ne!(simplified, UiTextStyleKey::from_style(&style));
}

#[test]
fn style_key_encodes_text_writing_mode() {
    let mut style = UiResolvedStyle {
        text_writing_mode: UiTextWritingMode::HorizontalTb,
        ..UiResolvedStyle::default()
    };
    let key = UiTextStyleKey::from_style(&style);

    style.text_writing_mode = UiTextWritingMode::VerticalRl;

    assert_ne!(key, UiTextStyleKey::from_style(&style));
}

#[test]
fn style_key_ignores_text_render_mode() {
    let mut style = UiResolvedStyle {
        text_render_mode: UiTextRenderMode::Native,
        ..UiResolvedStyle::default()
    };
    let native = UiTextStyleKey::from_style(&style);

    style.text_render_mode = UiTextRenderMode::Sdf;

    assert_eq!(native, UiTextStyleKey::from_style(&style));
}

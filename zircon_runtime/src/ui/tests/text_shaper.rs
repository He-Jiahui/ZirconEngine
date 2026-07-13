use crate::ui::text::{
    layout_text, measure_text_size,
    shaper::{
        UiSharedTextShaper, UiTextBackendIntent, UiTextShapeRequest, UiTextShaper,
        UiTextShaperSelection, UiTextShaperStack,
    },
};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        resolve_ui_text_render_mode, UiResolvedStyle, UiTextOverflow, UiTextRenderMode, UiTextWrap,
    },
};

#[test]
fn shared_text_shaper_matches_public_layout_entrypoint() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);
    let frame = UiFrame::new(0.0, 0.0, ellipsis_width_for_test(&style), 12.0);
    let request = UiTextShapeRequest::new("a\u{0301}bc", &style, frame, None);

    let shaper_layout = UiTextShaperStack::default().shape_text(&request);
    let public_layout = layout_text("a\u{0301}bc", &style, frame, None);

    assert_eq!(shaper_layout, public_layout);
    assert!(shaper_layout.lines[0].ellipsized);
    assert_eq!(shaper_layout.lines[0].text, "a\u{0301}…");
}

#[test]
fn shared_text_shaper_matches_public_measurement_entrypoint() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let shaper_size = UiTextShaperStack::default().measure_text("a\u{0301}b", &style);
    let narrow = UiTextShaperStack::default().measure_text("iii", &style);
    let wide = UiTextShaperStack::default().measure_text("WWW", &style);

    assert_eq!(shaper_size, measure_text_size("a\u{0301}b", &style));
    assert!(wide.width > narrow.width);
    assert_eq!(shaper_size.height, 12.0);
}

#[test]
fn text_shaper_stack_records_render_mode_backend_intent() {
    let auto_style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let native_style = UiResolvedStyle {
        text_render_mode: UiTextRenderMode::Native,
        ..auto_style.clone()
    };
    let sdf_style = UiResolvedStyle {
        text_render_mode: UiTextRenderMode::Sdf,
        ..auto_style
    };
    let stack = UiTextShaperStack::default();

    assert_eq!(
        stack.selection_for_style(&native_style),
        UiTextShaperSelection {
            requested_mode: UiTextRenderMode::Native,
            effective_mode: UiTextRenderMode::Native,
            intended_backend: UiTextBackendIntent::NativeGlyphon,
            active_backend: UiTextBackendIntent::SharedTextService,
            fallback_reason: None,
        }
    );
    assert_eq!(
        stack.selection_for_style(&sdf_style),
        UiTextShaperSelection {
            requested_mode: UiTextRenderMode::Sdf,
            effective_mode: UiTextRenderMode::Sdf,
            intended_backend: UiTextBackendIntent::SdfAtlas,
            active_backend: UiTextBackendIntent::SharedTextService,
            fallback_reason: None,
        }
    );
    assert_eq!(
        stack.selection_for_style(&UiResolvedStyle {
            text_render_mode: UiTextRenderMode::Auto,
            ..native_style
        }),
        UiTextShaperSelection {
            requested_mode: UiTextRenderMode::Auto,
            effective_mode: UiTextRenderMode::Native,
            intended_backend: UiTextBackendIntent::NativeGlyphon,
            active_backend: UiTextBackendIntent::SharedTextService,
            fallback_reason: None,
        }
    );
}

#[test]
fn text_render_mode_resolver_matches_runtime_font_asset_policy() {
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Auto, None),
        UiTextRenderMode::Native
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Auto)),
        UiTextRenderMode::Native
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Sdf)),
        UiTextRenderMode::Sdf
    );
    assert_eq!(
        resolve_ui_text_render_mode(UiTextRenderMode::Native, Some(UiTextRenderMode::Sdf)),
        UiTextRenderMode::Native
    );
}

#[test]
fn text_shaper_stack_records_auto_font_default_backend_intent() {
    let selection = UiTextShaperSelection::for_render_mode_with_font_default(
        UiTextRenderMode::Auto,
        Some(UiTextRenderMode::Sdf),
    );

    assert_eq!(
        selection,
        UiTextShaperSelection {
            requested_mode: UiTextRenderMode::Auto,
            effective_mode: UiTextRenderMode::Sdf,
            intended_backend: UiTextBackendIntent::SdfAtlas,
            active_backend: UiTextBackendIntent::SharedTextService,
            fallback_reason: None,
        }
    );
}

#[test]
fn text_shaper_stack_uses_shared_text_service_for_font_backends() {
    let style = UiResolvedStyle {
        text_render_mode: UiTextRenderMode::Native,
        ..test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis)
    };
    let frame = UiFrame::new(0.0, 0.0, ellipsis_width_for_test(&style), 12.0);
    let request = UiTextShapeRequest::new("a\u{0301}bc", &style, frame, None);
    let stack = UiTextShaperStack::default();

    assert_eq!(
        stack.shape_text(&request),
        UiSharedTextShaper.shape_text(&request)
    );
    assert_eq!(
        stack.measure_text("a\u{0301}b", &style),
        UiSharedTextShaper.measure_text("a\u{0301}b", &style)
    );
}

fn test_style(wrap: UiTextWrap, overflow: UiTextOverflow) -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap,
        text_overflow: overflow,
        ..UiResolvedStyle::default()
    }
}

fn ellipsis_width_for_test(style: &UiResolvedStyle) -> f32 {
    let minimum = measure_text_size("a\u{0301}…", style).width + 0.1;
    let maximum = measure_text_size("a\u{0301}b…", style).width - 0.1;
    minimum
        .min(maximum)
        .max(measure_text_size("…", style).width)
}

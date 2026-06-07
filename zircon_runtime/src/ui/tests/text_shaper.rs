use crate::ui::text::{
    layout_text, measure_text_size,
    shaper::{
        resolve_text_render_mode, UiHeuristicTextShaper, UiTextBackendIntent, UiTextShapeRequest,
        UiTextShaper, UiTextShaperSelection, UiTextShaperStack,
    },
};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedStyle, UiTextOverflow, UiTextRenderMode, UiTextWrap},
};

#[test]
fn heuristic_text_shaper_matches_public_layout_entrypoint() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);
    let frame = UiFrame::new(0.0, 0.0, 10.0, 12.0);
    let request = UiTextShapeRequest::new("a\u{0301}bc", &style, frame, None);

    let shaper_layout = UiTextShaperStack::default().shape_text(&request);
    let public_layout = layout_text("a\u{0301}bc", &style, frame, None);

    assert_eq!(shaper_layout, public_layout);
    assert!(shaper_layout.lines[0].ellipsized);
    assert_eq!(shaper_layout.lines[0].text, "a\u{0301}…");
}

#[test]
fn heuristic_text_shaper_matches_public_measurement_entrypoint() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let shaper_size = UiTextShaperStack::default().measure_text("a\u{0301}b", &style);

    assert_eq!(shaper_size, measure_text_size("a\u{0301}b", &style));
    assert_eq!(shaper_size, UiSize::new(10.0, 12.0));
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
            active_backend: UiTextBackendIntent::Heuristic,
            fallback_reason: Some("glyphon native text backend is not connected to layout yet"),
        }
    );
    assert_eq!(
        stack.selection_for_style(&sdf_style),
        UiTextShaperSelection {
            requested_mode: UiTextRenderMode::Sdf,
            effective_mode: UiTextRenderMode::Sdf,
            intended_backend: UiTextBackendIntent::SdfAtlas,
            active_backend: UiTextBackendIntent::Heuristic,
            fallback_reason: Some("SDF atlas text backend is not connected to layout yet"),
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
            active_backend: UiTextBackendIntent::Heuristic,
            fallback_reason: Some("glyphon native text backend is not connected to layout yet"),
        }
    );
}

#[test]
fn text_render_mode_resolver_matches_runtime_font_asset_policy() {
    assert_eq!(
        resolve_text_render_mode(UiTextRenderMode::Auto, None),
        UiTextRenderMode::Native
    );
    assert_eq!(
        resolve_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Auto)),
        UiTextRenderMode::Native
    );
    assert_eq!(
        resolve_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Sdf)),
        UiTextRenderMode::Sdf
    );
    assert_eq!(
        resolve_text_render_mode(UiTextRenderMode::Native, Some(UiTextRenderMode::Sdf)),
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
            active_backend: UiTextBackendIntent::Heuristic,
            fallback_reason: Some("SDF atlas text backend is not connected to layout yet"),
        }
    );
}

#[test]
fn text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land() {
    let style = UiResolvedStyle {
        text_render_mode: UiTextRenderMode::Native,
        ..test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis)
    };
    let frame = UiFrame::new(0.0, 0.0, 10.0, 12.0);
    let request = UiTextShapeRequest::new("a\u{0301}bc", &style, frame, None);
    let stack = UiTextShaperStack::default();

    assert_eq!(
        stack.shape_text(&request),
        UiHeuristicTextShaper.shape_text(&request)
    );
    assert_eq!(
        stack.measure_text("a\u{0301}b", &style),
        UiHeuristicTextShaper.measure_text("a\u{0301}b", &style)
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

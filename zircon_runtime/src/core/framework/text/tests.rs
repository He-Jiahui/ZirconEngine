use super::{
    TextDirection, TextFontFaceHandle, TextFontRequest, TextLayoutError, TextLayoutMetrics,
    TextLayoutService, TextRenderMode, TextShapeRequest, TextShapeResult,
};

struct RecordingTextLayoutService;

impl TextLayoutService for RecordingTextLayoutService {
    fn resolve_render_mode(&self, request: &TextFontRequest<'_>) -> TextRenderMode {
        match request.render_mode {
            TextRenderMode::Auto => TextRenderMode::Native,
            mode => mode,
        }
    }

    fn resolve_direction(&self, text: &str, requested: TextDirection) -> TextDirection {
        match requested {
            TextDirection::Auto if text.starts_with('\u{5e8}') => TextDirection::RightToLeft,
            TextDirection::Auto => TextDirection::LeftToRight,
            direction => direction,
        }
    }

    fn shape(&self, request: TextShapeRequest<'_>) -> Result<TextShapeResult, TextLayoutError> {
        if !request.font.size.is_finite() || request.font.size <= 0.0 {
            return Err(TextLayoutError::InvalidFontSize);
        }
        Ok(TextShapeResult {
            runs: Vec::new(),
            metrics: TextLayoutMetrics {
                width: request.text.len() as f32,
                height: request.font.size,
                ..TextLayoutMetrics::default()
            },
            resolved_direction: self.resolve_direction(request.text, request.direction),
        })
    }
}

#[test]
fn text_layout_contract_resolves_mode_direction_and_metrics_without_domain_types() {
    let service = RecordingTextLayoutService;
    let font = TextFontRequest::default();
    let request = TextShapeRequest::new("hello", font);

    assert_eq!(service.resolve_render_mode(&font), TextRenderMode::Native);
    let result = service
        .shape(request)
        .expect("neutral request should shape");
    assert_eq!(result.resolved_direction, TextDirection::LeftToRight);
    assert_eq!(result.metrics.width, 5.0);
}

#[test]
fn text_layout_contract_returns_typed_error_for_invalid_font_size() {
    let service = RecordingTextLayoutService;
    let font = TextFontRequest {
        size: 0.0,
        ..TextFontRequest::default()
    };

    assert_eq!(
        service.shape(TextShapeRequest::new("invalid", font)),
        Err(TextLayoutError::InvalidFontSize)
    );
}

#[test]
fn text_font_face_handle_distinguishes_reused_slot_generations() {
    let initial = TextFontFaceHandle::new(7, 1);
    let reloaded = TextFontFaceHandle::new(7, 2);

    assert_ne!(initial, reloaded);
    assert_eq!(initial.index, reloaded.index);
}

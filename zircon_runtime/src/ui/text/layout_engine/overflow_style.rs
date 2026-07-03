use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextOverflow};

const MIN_TEXT_FONT_SIZE: f32 = 1.0;
const FIT_SEARCH_STEPS: usize = 8;
const FIT_WIDTH_EPSILON: f32 = 0.5;

pub(super) fn resolve(
    text: &str,
    style: &UiResolvedStyle,
    max_width: f32,
    measure: fn(&str, &UiResolvedStyle) -> UiSize,
) -> UiResolvedStyle {
    match style.text_overflow {
        UiTextOverflow::ShrinkToFit => fit_text_style(
            text,
            style,
            max_width,
            MIN_TEXT_FONT_SIZE,
            style.font_size.max(MIN_TEXT_FONT_SIZE),
            measure,
        ),
        UiTextOverflow::ClampFontSize { min_px, max_px } => {
            let (min_font_size, max_font_size) = normalized_font_size_bounds(min_px, max_px);
            let style = clamp_requested_style(style, min_font_size, max_font_size);
            fit_text_style(
                text,
                &style,
                max_width,
                min_font_size,
                max_font_size,
                measure,
            )
        }
        _ => style.clone(),
    }
}

fn normalized_font_size_bounds(min_px: f32, max_px: f32) -> (f32, f32) {
    let min_font_size = min_px.max(MIN_TEXT_FONT_SIZE);
    let max_font_size = max_px.max(min_font_size);
    (min_font_size, max_font_size)
}

fn clamp_requested_style(
    style: &UiResolvedStyle,
    min_font_size: f32,
    max_font_size: f32,
) -> UiResolvedStyle {
    let requested_font_size = style.font_size.max(MIN_TEXT_FONT_SIZE);
    let clamped_font_size = requested_font_size.clamp(min_font_size, max_font_size);
    if (clamped_font_size - requested_font_size).abs() <= f32::EPSILON {
        return style.clone();
    }

    let scale = clamped_font_size / requested_font_size;
    scaled_text_style(style, scale, min_font_size, max_font_size)
}

fn fit_text_style(
    text: &str,
    style: &UiResolvedStyle,
    max_width: f32,
    min_font_size: f32,
    max_font_size: f32,
    measure: fn(&str, &UiResolvedStyle) -> UiSize,
) -> UiResolvedStyle {
    let max_width = max_width.max(0.0);
    if text.is_empty() || max_width <= 0.0 {
        return style.clone();
    }

    let requested_font_size = style
        .font_size
        .max(MIN_TEXT_FONT_SIZE)
        .clamp(min_font_size, max_font_size);
    let max_style = style_with_font_size(style, requested_font_size, min_font_size, max_font_size);
    let natural_width = measure(text, &max_style).width;
    if natural_width <= max_width + FIT_WIDTH_EPSILON || natural_width <= 0.0 {
        return max_style;
    }

    let min_scale = (min_font_size / requested_font_size).min(1.0);
    let mut low = min_scale;
    let mut high = 1.0;
    let mut best = min_scale;

    for _ in 0..FIT_SEARCH_STEPS {
        let scale = (low + high) * 0.5;
        let candidate = scaled_text_style(&max_style, scale, min_font_size, max_font_size);
        let candidate_width = measure(text, &candidate).width;
        if candidate_width <= max_width + FIT_WIDTH_EPSILON {
            best = scale;
            low = scale;
        } else {
            high = scale;
        }
    }

    scaled_text_style(&max_style, best, min_font_size, max_font_size)
}

fn style_with_font_size(
    style: &UiResolvedStyle,
    font_size: f32,
    min_font_size: f32,
    max_font_size: f32,
) -> UiResolvedStyle {
    let requested_font_size = style.font_size.max(MIN_TEXT_FONT_SIZE);
    let scale = font_size.clamp(min_font_size, max_font_size) / requested_font_size;
    scaled_text_style(style, scale, min_font_size, max_font_size)
}

fn scaled_text_style(
    style: &UiResolvedStyle,
    scale: f32,
    min_font_size: f32,
    max_font_size: f32,
) -> UiResolvedStyle {
    let mut scaled = style.clone();
    scaled.font_size =
        (style.font_size.max(MIN_TEXT_FONT_SIZE) * scale).clamp(min_font_size, max_font_size);
    scaled.line_height = (style.line_height * scale).max(scaled.font_size);
    scaled
}

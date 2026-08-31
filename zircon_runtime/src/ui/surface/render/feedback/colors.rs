use std::{borrow::Cow, sync::OnceLock};

use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens, style::UiRgbaColor, tree::UiTemplateNodeMetadata,
};

use super::{color_attribute, state::FeedbackRenderState};

#[derive(Clone, Debug)]
struct FeedbackPalette {
    tooltip_surface: String,
    tooltip_border: String,
    tooltip_title: String,
    tooltip_body: String,
    tooltip_icon: String,
    alert_info_surface: String,
    alert_info_border: String,
    alert_info_mark: String,
    alert_success_surface: String,
    alert_success_border: String,
    alert_success_mark: String,
    alert_warning_surface: String,
    alert_warning_border: String,
    alert_warning_mark: String,
    alert_error_surface: String,
    alert_error_border: String,
    alert_error_mark: String,
    toast_surface: String,
    toast_surface_hover: String,
    toast_surface_pressed: String,
    toast_border: String,
    toast_text: String,
    toast_action: String,
    disabled_surface: String,
    disabled_border: String,
    disabled_text: String,
    focus_border: String,
}

fn feedback_palette() -> &'static FeedbackPalette {
    static PALETTE: OnceLock<FeedbackPalette> = OnceLock::new();
    PALETTE.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let palette = &tokens.palette;
        FeedbackPalette {
            tooltip_surface: css_color(palette.popup),
            tooltip_border: css_color(palette.separator_soft),
            tooltip_title: css_color(palette.text_primary),
            tooltip_body: css_color(palette.text_secondary),
            tooltip_icon: css_color(palette.accent),
            alert_info_surface: css_color(palette.info_container),
            alert_info_border: css_color(palette.info),
            alert_info_mark: css_color(palette.info),
            alert_success_surface: css_color(palette.success_container),
            alert_success_border: css_color(palette.success),
            alert_success_mark: css_color(palette.success),
            alert_warning_surface: css_color(palette.warning_container),
            alert_warning_border: css_color(palette.warning),
            alert_warning_mark: css_color(palette.warning),
            alert_error_surface: css_color(palette.error_container),
            alert_error_border: css_color(palette.error),
            alert_error_mark: css_color(palette.error),
            toast_surface: css_color(palette.accent_soft),
            toast_surface_hover: css_color(palette.surface_hover),
            toast_surface_pressed: css_color(palette.surface[3]),
            toast_border: css_color(palette.separator_soft),
            toast_text: css_color(palette.text_primary),
            toast_action: css_color(palette.accent),
            disabled_surface: css_color(palette.surface_disabled),
            disabled_border: css_color(palette.border_disabled),
            disabled_text: css_color(palette.text_disabled),
            focus_border: css_color(palette.accent),
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AlertTone {
    Info,
    Success,
    Warning,
    Error,
}

pub(super) fn alert_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_surface)
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_surface(tone))
    } else if state.pointer_hot() {
        color_attribute(metadata, "hover_background_color")
            .or_else(|| color_attribute(metadata, "background_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_surface(tone))
    } else {
        color_attribute(metadata, "background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_surface(tone))
    }
}

pub(super) fn alert_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_border)
    } else if state.pressed() {
        color_attribute(metadata, "focus_border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_border(tone))
    }
}

pub(super) fn alert_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

pub(super) fn alert_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "icon_color")
            .or_else(|| color_attribute(metadata, "label_color"))
            .or_else(|| color_attribute(metadata, "mark_color"))
            .or_else(|| color_attribute(metadata, "status_mark_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_tone_mark(tone))
    }
}

pub(super) fn alert_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
    tone: AlertTone,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| alert_text_color(metadata, state, tone))
    }
}

fn alert_tone_surface<'a>(tone: AlertTone) -> Cow<'a, str> {
    let palette = feedback_palette();
    Cow::Borrowed(match tone {
        AlertTone::Info => &palette.alert_info_surface,
        AlertTone::Success => &palette.alert_success_surface,
        AlertTone::Warning => &palette.alert_warning_surface,
        AlertTone::Error => &palette.alert_error_surface,
    })
}

fn alert_tone_border<'a>(tone: AlertTone) -> Cow<'a, str> {
    let palette = feedback_palette();
    Cow::Borrowed(match tone {
        AlertTone::Info => &palette.alert_info_border,
        AlertTone::Success => &palette.alert_success_border,
        AlertTone::Warning => &palette.alert_warning_border,
        AlertTone::Error => &palette.alert_error_border,
    })
}

fn alert_tone_mark<'a>(tone: AlertTone) -> Cow<'a, str> {
    let palette = feedback_palette();
    Cow::Borrowed(match tone {
        AlertTone::Info => &palette.alert_info_mark,
        AlertTone::Success => &palette.alert_success_mark,
        AlertTone::Warning => &palette.alert_warning_mark,
        AlertTone::Error => &palette.alert_error_mark,
    })
}

pub(super) fn tooltip_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_surface)
    } else {
        color_attribute(metadata, "background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_surface))
    }
}

pub(super) fn tooltip_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_border)
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_border))
    }
}

pub(super) fn tooltip_title_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_title))
    }
}

pub(super) fn tooltip_body_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "body_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_body))
    }
}

pub(super) fn tooltip_icon_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "icon_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "icon_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().tooltip_icon))
    }
}

pub(super) fn toast_surface_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_surface)
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_surface_pressed))
    } else if state.pointer_hot() {
        color_attribute(metadata, "hover_background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_surface_hover))
    } else {
        color_attribute(metadata, "background_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_surface))
    }
}

pub(super) fn toast_border_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_border)
    } else if state.focused() || state.pressed() {
        color_attribute(metadata, "focus_border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().focus_border))
    } else {
        color_attribute(metadata, "border_color")
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_border))
    }
}

pub(super) fn toast_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "text_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_text))
    }
}

pub(super) fn toast_mark_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "label_color")
            .or_else(|| color_attribute(metadata, "mark_color"))
            .or_else(|| color_attribute(metadata, "status_mark_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_action))
    }
}

pub(super) fn toast_action_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &FeedbackRenderState,
) -> Cow<'a, str> {
    if state.unavailable() {
        Cow::Borrowed(&feedback_palette().disabled_text)
    } else {
        color_attribute(metadata, "action_color")
            .or_else(|| color_attribute(metadata, "value_color"))
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Borrowed(&feedback_palette().toast_action))
    }
}

fn css_color(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    let mut value = String::with_capacity(if alpha == u8::MAX { 7 } else { 9 });
    value.push('#');
    push_hex_byte(&mut value, red);
    push_hex_byte(&mut value, green);
    push_hex_byte(&mut value, blue);
    if alpha != u8::MAX {
        push_hex_byte(&mut value, alpha);
    }
    value
}

fn push_hex_byte(target: &mut String, value: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    target.push(HEX[(value >> 4) as usize] as char);
    target.push(HEX[(value & 0x0f) as usize] as char);
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime_interface::ui::style::UiRgbaColor;

    use super::css_color;

    #[test]
    fn optimization_batch_20260831gr_runtime573_css_color_preserves_hex_forms() {
        assert_eq!(css_color(UiRgbaColor::from_u8(0, 16, 255, 255)), "#0010ff");
        assert_eq!(css_color(UiRgbaColor::from_u8(1, 35, 69, 127)), "#0123457f");
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260831gr_runtime573_css_color_single_buffer_benchmark() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 250_000;
        let colors = [
            UiRgbaColor::from_u8(0, 16, 255, 255),
            UiRgbaColor::from_u8(1, 35, 69, 127),
            UiRgbaColor::from_u8(240, 128, 7, 255),
            UiRgbaColor::from_u8(255, 0, 16, 64),
        ];
        let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut checksum = 0usize;
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let (elapsed, value) = measure(ITERATIONS, &colors, legacy_css_color);
                legacy_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, &colors, css_color);
                optimized_ns.push(elapsed);
                checksum ^= value;
            } else {
                let (elapsed, value) = measure(ITERATIONS, &colors, css_color);
                optimized_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, &colors, legacy_css_color);
                legacy_ns.push(elapsed);
                checksum ^= value;
            }
        }
        let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
        let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "single-buffer CSS color P95 must be at least 15% below format/insert: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "RUNTIME573_CSS_COLOR_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
            join_samples(&legacy_ns),
            join_samples(&optimized_ns),
        );

        fn measure(
            iterations: usize,
            colors: &[UiRgbaColor],
            operation: fn(UiRgbaColor) -> String,
        ) -> (u128, usize) {
            let started = Instant::now();
            let mut checksum = 0usize;
            for index in 0..iterations {
                checksum =
                    checksum.wrapping_add(operation(black_box(colors[index % colors.len()])).len());
            }
            (started.elapsed().as_nanos(), black_box(checksum))
        }

        fn legacy_css_color(color: UiRgbaColor) -> String {
            let [red, green, blue, alpha] = color.to_u8();
            let mut value = if alpha == u8::MAX {
                format!("{red:02x}{green:02x}{blue:02x}")
            } else {
                format!("{red:02x}{green:02x}{blue:02x}{alpha:02x}")
            };
            value.insert(0, '#');
            value
        }

        fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
            let mut ordered = samples.to_vec();
            ordered.sort_unstable();
            let rank = (ordered.len() * percentile).div_ceil(100).max(1);
            ordered[rank - 1]
        }

        fn join_samples(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }
    }
}

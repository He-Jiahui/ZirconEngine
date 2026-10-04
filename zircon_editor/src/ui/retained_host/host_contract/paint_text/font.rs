#[cfg(test)]
use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use zircon_runtime::ui::surface::{current_resolved_text_font_generation, measure_text_size};
#[cfg(test)]
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextOverflow, UiTextRunPaintStyle, UiTextWrap,
};

use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_text_preferences, HostTextPreferences,
};

mod metrics;

use self::metrics::{
    default_runtime_line_height, empty_runtime_text_width, measured_text_width,
    resolved_runtime_font_size, resolved_runtime_line_height, should_measure_runtime_text,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum HostTextFontFace {
    Ui,
    UiStrong,
    Mono,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostTextFontRequest {
    pub face: HostTextFontFace,
    pub family: String,
    pub weight: u16,
}

pub(in crate::ui::retained_host::host_contract) fn font_face_for_paint_style(
    style: UiTextRunPaintStyle,
) -> HostTextFontFace {
    if style.code {
        HostTextFontFace::Mono
    } else if style.strong {
        HostTextFontFace::UiStrong
    } else {
        HostTextFontFace::Ui
    }
}

pub(in crate::ui::retained_host::host_contract) fn runtime_font_family_for_face(
    face: HostTextFontFace,
) -> Arc<str> {
    Arc::from(font_request_for_face(face).family)
}

pub(in crate::ui::retained_host::host_contract) fn font_request_for_face(
    face: HostTextFontFace,
) -> HostTextFontRequest {
    font_request_for_face_with_preferences(face, &current_host_text_preferences())
}

pub(in crate::ui::retained_host::host_contract) fn font_request_for_face_with_preferences(
    face: HostTextFontFace,
    preferences: &HostTextPreferences,
) -> HostTextFontRequest {
    match face {
        HostTextFontFace::Ui => HostTextFontRequest {
            face,
            family: preferences.ui_family.clone(),
            weight: preferences.ui_weight,
        },
        HostTextFontFace::UiStrong => HostTextFontRequest {
            face,
            family: preferences.ui_strong_family.clone(),
            weight: preferences.strong_weight,
        },
        HostTextFontFace::Mono => HostTextFontRequest {
            face,
            family: preferences.code_family.clone(),
            weight: preferences.code_weight,
        },
    }
}

pub(crate) fn measure_runtime_text_width(text: &str, font_size: f32) -> f32 {
    measure_runtime_text_width_with_style(text, font_size, UiTextRunPaintStyle::default())
}

pub(crate) fn runtime_text_metrics_generation() -> [u64; 3] {
    [
        runtime_font_request_generation(HostTextFontFace::Ui),
        runtime_font_request_generation(HostTextFontFace::UiStrong),
        runtime_font_request_generation(HostTextFontFace::Mono),
    ]
}

pub(in crate::ui::retained_host::host_contract) fn runtime_font_request_generation(
    face: HostTextFontFace,
) -> u64 {
    let request = font_request_for_face(face);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    current_resolved_text_font_generation().hash(&mut hasher);
    request.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn measure_runtime_text_width_with_style(
    text: &str,
    font_size: f32,
    style: UiTextRunPaintStyle,
) -> f32 {
    if !should_measure_runtime_text(text, font_size) {
        return empty_runtime_text_width();
    }

    let font_face = font_face_for_paint_style(style);
    let style = runtime_text_style_for_face(
        font_face,
        font_size,
        default_runtime_line_height(font_size),
        UiTextWrap::None,
        UiTextOverflow::Clip,
    );
    measured_text_width(measure_text_size(text, &style).width)
}

pub(in crate::ui::retained_host::host_contract) fn runtime_text_style_for_face(
    face: HostTextFontFace,
    font_size: f32,
    line_height: f32,
    wrap: UiTextWrap,
    text_overflow: UiTextOverflow,
) -> UiResolvedStyle {
    let request = font_request_for_face(face);
    let font_size = resolved_runtime_font_size(font_size);
    let line_height = resolved_runtime_line_height(font_size, line_height);
    UiResolvedStyle {
        font_family: Some(request.family),
        font_weight: request.weight,
        font_size,
        line_height,
        wrap,
        text_overflow,
        ..UiResolvedStyle::default()
    }
}

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract) struct HostRuntimeTextFace {
    pub(in crate::ui::retained_host::host_contract) family: Arc<str>,
    pub(in crate::ui::retained_host::host_contract) weight: u16,
}

pub(in crate::ui::retained_host::host_contract) struct HostRuntimeTextFaces {
    ui: HostRuntimeTextFace,
    ui_strong: HostRuntimeTextFace,
    mono: HostRuntimeTextFace,
}

impl HostRuntimeTextFaces {
    pub(in crate::ui::retained_host::host_contract) fn face(
        &self,
        face: HostTextFontFace,
    ) -> &HostRuntimeTextFace {
        match face {
            HostTextFontFace::Ui => &self.ui,
            HostTextFontFace::UiStrong => &self.ui_strong,
            HostTextFontFace::Mono => &self.mono,
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn capture_runtime_text_faces(
) -> HostRuntimeTextFaces {
    #[cfg(test)]
    RUNTIME_TEXT_FACE_CAPTURE_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    HostRuntimeTextFaces {
        ui: runtime_text_face(HostTextFontFace::Ui),
        ui_strong: runtime_text_face(HostTextFontFace::UiStrong),
        mono: runtime_text_face(HostTextFontFace::Mono),
    }
}

fn runtime_text_face(face: HostTextFontFace) -> HostRuntimeTextFace {
    let request = font_request_for_face(face);
    HostRuntimeTextFace {
        family: Arc::from(request.family),
        weight: request.weight,
    }
}

#[cfg(test)]
thread_local! {
    static RUNTIME_TEXT_FACE_CAPTURE_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn take_runtime_text_face_capture_count() -> usize {
    RUNTIME_TEXT_FACE_CAPTURE_COUNT.with(|count| count.replace(0))
}

#[cfg(test)]
#[path = "font/tests/cases.rs"]
mod tests;

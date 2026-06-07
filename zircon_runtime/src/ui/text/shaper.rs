use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedStyle, UiResolvedTextLayout, UiTextRenderMode},
};

use super::layout_engine::{
    layout_text as heuristic_layout_text, measure_text_size as heuristic_measure_text_size,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct UiTextShapeRequest<'a> {
    pub text: &'a str,
    pub style: &'a UiResolvedStyle,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
}

impl<'a> UiTextShapeRequest<'a> {
    pub(crate) const fn new(
        text: &'a str,
        style: &'a UiResolvedStyle,
        frame: UiFrame,
        clip_frame: Option<UiFrame>,
    ) -> Self {
        Self {
            text,
            style,
            frame,
            clip_frame,
        }
    }
}

pub(crate) trait UiTextShaper {
    fn shape_text(&self, request: &UiTextShapeRequest<'_>) -> UiResolvedTextLayout;
    fn measure_text(&self, text: &str, style: &UiResolvedStyle) -> UiSize;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiTextBackendIntent {
    Heuristic,
    NativeGlyphon,
    SdfAtlas,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiTextShaperSelection {
    pub requested_mode: UiTextRenderMode,
    pub effective_mode: UiTextRenderMode,
    pub intended_backend: UiTextBackendIntent,
    pub active_backend: UiTextBackendIntent,
    pub fallback_reason: Option<&'static str>,
}

impl UiTextShaperSelection {
    pub(crate) fn for_style(style: &UiResolvedStyle) -> Self {
        Self::for_render_mode_with_font_default(style.text_render_mode, None)
    }

    pub(crate) const fn for_render_mode_with_font_default(
        requested_mode: UiTextRenderMode,
        font_render_mode: Option<UiTextRenderMode>,
    ) -> Self {
        let effective_mode = resolve_text_render_mode(requested_mode, font_render_mode);
        let intended_backend = backend_intent_for_render_mode(effective_mode);
        let active_backend = active_layout_backend_for_intent(intended_backend);
        Self {
            requested_mode,
            effective_mode,
            intended_backend,
            active_backend,
            fallback_reason: fallback_reason_for_backend(intended_backend, active_backend),
        }
    }
}

pub(crate) const fn resolve_text_render_mode(
    requested_mode: UiTextRenderMode,
    font_render_mode: Option<UiTextRenderMode>,
) -> UiTextRenderMode {
    match requested_mode {
        UiTextRenderMode::Native => UiTextRenderMode::Native,
        UiTextRenderMode::Sdf => UiTextRenderMode::Sdf,
        UiTextRenderMode::Auto => match font_render_mode {
            Some(UiTextRenderMode::Native) => UiTextRenderMode::Native,
            Some(UiTextRenderMode::Sdf) => UiTextRenderMode::Sdf,
            Some(UiTextRenderMode::Auto) | None => UiTextRenderMode::Native,
        },
    }
}

const fn backend_intent_for_render_mode(render_mode: UiTextRenderMode) -> UiTextBackendIntent {
    match render_mode {
        UiTextRenderMode::Auto | UiTextRenderMode::Native => UiTextBackendIntent::NativeGlyphon,
        UiTextRenderMode::Sdf => UiTextBackendIntent::SdfAtlas,
    }
}

const fn active_layout_backend_for_intent(intent: UiTextBackendIntent) -> UiTextBackendIntent {
    match intent {
        UiTextBackendIntent::Heuristic => UiTextBackendIntent::Heuristic,
        UiTextBackendIntent::NativeGlyphon | UiTextBackendIntent::SdfAtlas => {
            UiTextBackendIntent::Heuristic
        }
    }
}

const fn fallback_reason_for_backend(
    intended_backend: UiTextBackendIntent,
    active_backend: UiTextBackendIntent,
) -> Option<&'static str> {
    match (intended_backend, active_backend) {
        (UiTextBackendIntent::Heuristic, UiTextBackendIntent::Heuristic)
        | (UiTextBackendIntent::NativeGlyphon, UiTextBackendIntent::NativeGlyphon)
        | (UiTextBackendIntent::SdfAtlas, UiTextBackendIntent::SdfAtlas) => None,
        (UiTextBackendIntent::NativeGlyphon, _) => {
            Some("glyphon native text backend is not connected to layout yet")
        }
        (UiTextBackendIntent::SdfAtlas, _) => {
            Some("SDF atlas text backend is not connected to layout yet")
        }
        (UiTextBackendIntent::Heuristic, _) => None,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiHeuristicTextShaper;

impl UiTextShaper for UiHeuristicTextShaper {
    fn shape_text(&self, request: &UiTextShapeRequest<'_>) -> UiResolvedTextLayout {
        heuristic_layout_text(
            request.text,
            request.style,
            request.frame,
            request.clip_frame,
        )
    }

    fn measure_text(&self, text: &str, style: &UiResolvedStyle) -> UiSize {
        heuristic_measure_text_size(text, style)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiTextShaperStack {
    heuristic: UiHeuristicTextShaper,
}

impl UiTextShaperStack {
    pub(crate) const fn new() -> Self {
        Self {
            heuristic: UiHeuristicTextShaper,
        }
    }

    pub(crate) fn selection_for_style(&self, style: &UiResolvedStyle) -> UiTextShaperSelection {
        UiTextShaperSelection::for_style(style)
    }
}

impl UiTextShaper for UiTextShaperStack {
    fn shape_text(&self, request: &UiTextShapeRequest<'_>) -> UiResolvedTextLayout {
        let selection = self.selection_for_style(request.style);

        debug_assert_eq!(selection.requested_mode, request.style.text_render_mode);
        debug_assert_eq!(
            selection.fallback_reason.is_some(),
            selection.intended_backend != selection.active_backend
        );

        match selection.active_backend {
            UiTextBackendIntent::Heuristic => self.heuristic.shape_text(request),
            UiTextBackendIntent::NativeGlyphon | UiTextBackendIntent::SdfAtlas => {
                self.heuristic.shape_text(request)
            }
        }
    }

    fn measure_text(&self, text: &str, style: &UiResolvedStyle) -> UiSize {
        let selection = self.selection_for_style(style);

        debug_assert_eq!(
            selection.fallback_reason.is_some(),
            selection.intended_backend != selection.active_backend
        );

        match selection.active_backend {
            UiTextBackendIntent::Heuristic => self.heuristic.measure_text(text, style),
            UiTextBackendIntent::NativeGlyphon | UiTextBackendIntent::SdfAtlas => {
                self.heuristic.measure_text(text, style)
            }
        }
    }
}

pub fn layout_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> UiResolvedTextLayout {
    UiTextShaperStack::new().shape_text(&UiTextShapeRequest::new(text, style, frame, clip_frame))
}

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    UiTextShaperStack::new().measure_text(text, style)
}

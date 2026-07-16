use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{
        resolve_ui_text_render_mode, UiResolvedStyle, UiResolvedTextLayout, UiTextRange,
        UiTextRenderMode,
    },
};

use crate::text::SharedTextLayoutSession;

use super::layout_engine::{
    layout_text as shared_layout_text,
    layout_text_with_provider as shared_layout_text_with_provider,
    measure_text_size as shared_measure_text_size,
    measure_text_size_with_provider as shared_measure_text_size_with_provider,
    measure_text_source_range_width as shared_measure_text_source_range_width,
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
    fn measure_text_source_range_width(
        &self,
        text: &str,
        style: &UiResolvedStyle,
        range: UiTextRange,
    ) -> f32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiTextBackendIntent {
    SharedTextService,
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
        let effective_mode = resolve_ui_text_render_mode(requested_mode, font_render_mode);
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

const fn backend_intent_for_render_mode(render_mode: UiTextRenderMode) -> UiTextBackendIntent {
    match render_mode {
        UiTextRenderMode::Auto | UiTextRenderMode::Native => UiTextBackendIntent::NativeGlyphon,
        UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf => {
            UiTextBackendIntent::SdfAtlas
        }
    }
}

const fn active_layout_backend_for_intent(intent: UiTextBackendIntent) -> UiTextBackendIntent {
    match intent {
        UiTextBackendIntent::SharedTextService
        | UiTextBackendIntent::NativeGlyphon
        | UiTextBackendIntent::SdfAtlas => UiTextBackendIntent::SharedTextService,
    }
}

const fn fallback_reason_for_backend(
    _intended_backend: UiTextBackendIntent,
    _active_backend: UiTextBackendIntent,
) -> Option<&'static str> {
    None
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiSharedTextShaper;

impl UiTextShaper for UiSharedTextShaper {
    fn shape_text(&self, request: &UiTextShapeRequest<'_>) -> UiResolvedTextLayout {
        shared_layout_text(
            request.text,
            request.style,
            request.frame,
            request.clip_frame,
        )
    }

    fn measure_text(&self, text: &str, style: &UiResolvedStyle) -> UiSize {
        shared_measure_text_size(text, style)
    }

    fn measure_text_source_range_width(
        &self,
        text: &str,
        style: &UiResolvedStyle,
        range: UiTextRange,
    ) -> f32 {
        shared_measure_text_source_range_width(text, style, range)
    }
}

impl UiSharedTextShaper {
    fn shape_text_with_provider(
        &self,
        request: &UiTextShapeRequest<'_>,
        provider: &mut SharedTextLayoutSession,
    ) -> UiResolvedTextLayout {
        shared_layout_text_with_provider(
            request.text,
            request.style,
            request.frame,
            request.clip_frame,
            provider,
        )
    }

    fn measure_text_with_provider(
        &self,
        text: &str,
        style: &UiResolvedStyle,
        provider: &mut SharedTextLayoutSession,
    ) -> UiSize {
        shared_measure_text_size_with_provider(text, style, provider)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiTextShaperStack {
    shared: UiSharedTextShaper,
}

impl UiTextShaperStack {
    pub(crate) const fn new() -> Self {
        Self {
            shared: UiSharedTextShaper,
        }
    }

    pub(crate) fn selection_for_style(&self, style: &UiResolvedStyle) -> UiTextShaperSelection {
        UiTextShaperSelection::for_style(style)
    }

    pub(crate) fn shape_text_with_provider(
        &self,
        request: &UiTextShapeRequest<'_>,
        provider: &mut SharedTextLayoutSession,
    ) -> UiResolvedTextLayout {
        let selection = self.selection_for_style(request.style);

        debug_assert_eq!(selection.requested_mode, request.style.text_render_mode);
        debug_assert!(selection.fallback_reason.is_none());

        match selection.active_backend {
            UiTextBackendIntent::SharedTextService
            | UiTextBackendIntent::NativeGlyphon
            | UiTextBackendIntent::SdfAtlas => {
                self.shared.shape_text_with_provider(request, provider)
            }
        }
    }

    pub(crate) fn measure_text_with_provider(
        &self,
        text: &str,
        style: &UiResolvedStyle,
        provider: &mut SharedTextLayoutSession,
    ) -> UiSize {
        let selection = self.selection_for_style(style);

        debug_assert!(selection.fallback_reason.is_none());

        match selection.active_backend {
            UiTextBackendIntent::SharedTextService
            | UiTextBackendIntent::NativeGlyphon
            | UiTextBackendIntent::SdfAtlas => self
                .shared
                .measure_text_with_provider(text, style, provider),
        }
    }
}

impl UiTextShaper for UiTextShaperStack {
    fn shape_text(&self, request: &UiTextShapeRequest<'_>) -> UiResolvedTextLayout {
        let selection = self.selection_for_style(request.style);

        debug_assert_eq!(selection.requested_mode, request.style.text_render_mode);
        debug_assert!(selection.fallback_reason.is_none());

        match selection.active_backend {
            UiTextBackendIntent::SharedTextService
            | UiTextBackendIntent::NativeGlyphon
            | UiTextBackendIntent::SdfAtlas => self.shared.shape_text(request),
        }
    }

    fn measure_text(&self, text: &str, style: &UiResolvedStyle) -> UiSize {
        let selection = self.selection_for_style(style);

        debug_assert!(selection.fallback_reason.is_none());

        match selection.active_backend {
            UiTextBackendIntent::SharedTextService
            | UiTextBackendIntent::NativeGlyphon
            | UiTextBackendIntent::SdfAtlas => self.shared.measure_text(text, style),
        }
    }

    fn measure_text_source_range_width(
        &self,
        text: &str,
        style: &UiResolvedStyle,
        range: UiTextRange,
    ) -> f32 {
        let selection = self.selection_for_style(style);

        debug_assert!(selection.fallback_reason.is_none());

        match selection.active_backend {
            UiTextBackendIntent::SharedTextService
            | UiTextBackendIntent::NativeGlyphon
            | UiTextBackendIntent::SdfAtlas => self
                .shared
                .measure_text_source_range_width(text, style, range),
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

pub(crate) fn layout_text_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    UiTextShaperStack::new().shape_text_with_provider(
        &UiTextShapeRequest::new(text, style, frame, clip_frame),
        provider,
    )
}

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    UiTextShaperStack::new().measure_text(text, style)
}

pub(crate) fn measure_text_size_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> UiSize {
    UiTextShaperStack::new().measure_text_with_provider(text, style, provider)
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
) -> f32 {
    UiTextShaperStack::new().measure_text_source_range_width(text, style, range)
}

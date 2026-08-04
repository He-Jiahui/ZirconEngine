use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedStyle, UiResolvedTextLayout, UiTextRange},
};

use crate::text::{SharedTextLayoutSession, TextDocumentKey};

use super::layout_engine::{
    layout_text as shared_layout_text,
    layout_text_with_provider as shared_layout_text_with_provider,
    layout_text_with_provider_and_viewport as shared_layout_text_with_provider_and_viewport,
    layout_text_with_viewport as shared_layout_text_with_viewport,
    measure_text_size as shared_measure_text_size,
    measure_text_size_with_provider as shared_measure_text_size_with_provider,
    measure_text_source_range_width as shared_measure_text_source_range_width,
    measure_unwrapped_text_height as shared_measure_unwrapped_text_height,
    measure_unwrapped_text_height_with_provider as shared_measure_unwrapped_text_height_with_provider,
};
use super::resolved_layout::UiTextViewport;

#[derive(Clone, Copy, Debug)]
pub(crate) struct UiTextShapeRequest<'a> {
    pub text: &'a str,
    pub style: &'a UiResolvedStyle,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub viewport: Option<UiTextViewport>,
    pub document_key: Option<TextDocumentKey>,
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
            viewport: None,
            document_key: None,
        }
    }

    pub(crate) const fn with_viewport(mut self, viewport: UiTextViewport) -> Self {
        self.viewport = Some(viewport);
        self
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

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiSharedTextShaper;

impl UiTextShaper for UiSharedTextShaper {
    fn shape_text(&self, request: &UiTextShapeRequest<'_>) -> UiResolvedTextLayout {
        match request.viewport {
            Some(viewport) => shared_layout_text_with_viewport(
                request.text,
                request.style,
                request.frame,
                request.clip_frame,
                viewport,
            ),
            None => shared_layout_text(
                request.text,
                request.style,
                request.frame,
                request.clip_frame,
            ),
        }
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
        match request.viewport {
            Some(viewport) => shared_layout_text_with_provider_and_viewport(
                request.text,
                request.style,
                request.frame,
                request.clip_frame,
                viewport,
                request.document_key,
                provider,
            ),
            None => shared_layout_text_with_provider(
                request.text,
                request.style,
                request.frame,
                request.clip_frame,
                provider,
            ),
        }
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

    pub(crate) fn shape_text_with_provider(
        &self,
        request: &UiTextShapeRequest<'_>,
        provider: &mut SharedTextLayoutSession,
    ) -> UiResolvedTextLayout {
        self.shared.shape_text_with_provider(request, provider)
    }

    pub(crate) fn measure_text_with_provider(
        &self,
        text: &str,
        style: &UiResolvedStyle,
        provider: &mut SharedTextLayoutSession,
    ) -> UiSize {
        self.shared
            .measure_text_with_provider(text, style, provider)
    }
}

impl UiTextShaper for UiTextShaperStack {
    fn shape_text(&self, request: &UiTextShapeRequest<'_>) -> UiResolvedTextLayout {
        self.shared.shape_text(request)
    }

    fn measure_text(&self, text: &str, style: &UiResolvedStyle) -> UiSize {
        self.shared.measure_text(text, style)
    }

    fn measure_text_source_range_width(
        &self,
        text: &str,
        style: &UiResolvedStyle,
        range: UiTextRange,
    ) -> f32 {
        self.shared
            .measure_text_source_range_width(text, style, range)
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

pub(crate) fn layout_text_with_viewport(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: UiTextViewport,
) -> UiResolvedTextLayout {
    UiTextShaperStack::new().shape_text(
        &UiTextShapeRequest::new(text, style, frame, clip_frame).with_viewport(viewport),
    )
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

pub(crate) fn layout_text_with_provider_and_viewport(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: UiTextViewport,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    let mut request =
        UiTextShapeRequest::new(text, style, frame, clip_frame).with_viewport(viewport);
    request.document_key = document_key;
    UiTextShaperStack::new().shape_text_with_provider(&request, provider)
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

pub(crate) fn measure_unwrapped_text_height(text: &str, style: &UiResolvedStyle) -> Option<f32> {
    shared_measure_unwrapped_text_height(text, style)
}

pub(crate) fn measure_unwrapped_text_height_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> Option<f32> {
    shared_measure_unwrapped_text_height_with_provider(text, style, provider)
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
) -> f32 {
    UiTextShaperStack::new().measure_text_source_range_width(text, style, range)
}

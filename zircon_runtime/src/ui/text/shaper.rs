use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedStyle, UiResolvedTextLayout},
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

pub fn layout_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> UiResolvedTextLayout {
    UiHeuristicTextShaper.shape_text(&UiTextShapeRequest::new(text, style, frame, clip_frame))
}

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    UiHeuristicTextShaper.measure_text(text, style)
}

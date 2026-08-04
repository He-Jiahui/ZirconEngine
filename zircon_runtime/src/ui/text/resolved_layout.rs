use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiRichTextFormat, UiTextAlign, UiTextDirection,
        UiTextOverflow, UiTextRange, UiTextWrap, UiTextWritingMode, normalize_ui_text_language_tag,
    },
};

use crate::text::{SharedTextLayoutSession, TextDocumentKey};

use super::shaper::{
    layout_text, layout_text_with_provider, layout_text_with_provider_and_viewport,
    layout_text_with_viewport,
};
use super::{
    layout_engine::layout_parsed_text_with_provider_and_viewport, rich_text::UiParsedText,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextLayoutResolution {
    pub layout: UiResolvedTextLayout,
    pub size: UiSize,
    pub first_baseline: f32,
    pub source_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextStyleKey {
    pub font_family: Option<String>,
    pub language: Option<String>,
    pub font_weight: u16,
    pub font_size_bits: u32,
    pub line_height_bits: u32,
    pub tab_size_bits: u32,
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub text_direction: UiTextDirection,
    pub text_writing_mode: UiTextWritingMode,
    pub text_overflow: UiTextOverflowKey,
    pub rich_text_format: UiRichTextFormat,
}

impl Hash for UiTextStyleKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_family.hash(state);
        self.language.hash(state);
        self.font_weight.hash(state);
        self.font_size_bits.hash(state);
        self.line_height_bits.hash(state);
        self.tab_size_bits.hash(state);
        std::mem::discriminant(&self.text_align).hash(state);
        std::mem::discriminant(&self.wrap).hash(state);
        std::mem::discriminant(&self.text_direction).hash(state);
        std::mem::discriminant(&self.text_writing_mode).hash(state);
        self.text_overflow.hash(state);
        std::mem::discriminant(&self.rich_text_format).hash(state);
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum UiTextOverflowKey {
    Clip,
    Ellipsis,
    EllipsisWord,
    EllipsisStart,
    EllipsisMiddle,
    ShrinkToFit,
    ClampFontSize { min_px_bits: u32, max_px_bits: u32 },
}

impl From<UiTextOverflow> for UiTextOverflowKey {
    fn from(overflow: UiTextOverflow) -> Self {
        match overflow {
            UiTextOverflow::Clip => Self::Clip,
            UiTextOverflow::Ellipsis => Self::Ellipsis,
            UiTextOverflow::EllipsisWord => Self::EllipsisWord,
            UiTextOverflow::EllipsisStart => Self::EllipsisStart,
            UiTextOverflow::EllipsisMiddle => Self::EllipsisMiddle,
            UiTextOverflow::ShrinkToFit => Self::ShrinkToFit,
            UiTextOverflow::ClampFontSize { min_px, max_px } => Self::ClampFontSize {
                min_px_bits: min_px.to_bits(),
                max_px_bits: max_px.to_bits(),
            },
        }
    }
}

impl UiTextStyleKey {
    pub(crate) fn from_style(style: &UiResolvedStyle) -> Self {
        Self {
            font_family: style.font_family.clone().or_else(|| style.font.clone()),
            language: normalize_ui_text_language_tag(style.language.as_deref()),
            font_weight: style.font_weight,
            font_size_bits: style.font_size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            tab_size_bits: style.tab_size.to_bits(),
            text_align: style.text_align,
            wrap: style.wrap,
            text_direction: style.text_direction,
            text_writing_mode: style.text_writing_mode,
            text_overflow: UiTextOverflowKey::from(style.text_overflow),
            rich_text_format: style.rich_text_format,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiPreeditSpan {
    pub range: UiTextRange,
    pub text: String,
}

/// A document-local viewport for bounded plain-text layout.
///
/// This is deliberately separate from render clipping: the offset identifies the rows that
/// must be shaped, while the clip still controls what is emitted to the renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiTextViewport {
    pub(crate) offset_y: f32,
    pub(crate) extent_y: f32,
    pub(crate) overscan_screens: usize,
}

impl UiTextViewport {
    pub(crate) const DEFAULT_OVERSCAN_SCREENS: usize = 2;

    pub(crate) fn new(offset_y: f32, extent_y: f32, overscan_screens: usize) -> Option<Self> {
        (offset_y.is_finite() && extent_y.is_finite() && extent_y > 0.0).then_some(Self {
            offset_y: offset_y.max(0.0),
            extent_y,
            overscan_screens,
        })
    }

    pub(crate) fn from_document_and_clip(
        document_frame: UiFrame,
        clip_frame: UiFrame,
    ) -> Option<Self> {
        Self::new(
            clip_frame.y - document_frame.y,
            clip_frame.height,
            Self::DEFAULT_OVERSCAN_SCREENS,
        )
    }

    pub(crate) fn cache_key(self) -> (u32, u32, usize) {
        (
            self.offset_y.to_bits(),
            self.extent_y.to_bits(),
            self.overscan_screens,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct UiTextLayoutRequest<'a> {
    pub text: &'a str,
    pub style: &'a UiResolvedStyle,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub preedit: Option<&'a UiPreeditSpan>,
    pub viewport: Option<UiTextViewport>,
    pub document_key: Option<TextDocumentKey>,
}

impl<'a> UiTextLayoutRequest<'a> {
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
            preedit: None,
            viewport: None,
            document_key: None,
        }
    }

    pub(crate) const fn with_preedit(mut self, preedit: &'a UiPreeditSpan) -> Self {
        self.preedit = Some(preedit);
        self
    }

    pub(crate) const fn with_viewport(mut self, viewport: UiTextViewport) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub(crate) const fn with_document_key(mut self, document_key: TextDocumentKey) -> Self {
        self.document_key = Some(document_key);
        self
    }

    pub(crate) const fn layout_viewport(&self) -> Option<UiTextViewport> {
        if self.preedit.is_some() {
            None
        } else {
            self.viewport
        }
    }

    pub(crate) fn style_key(&self) -> UiTextStyleKey {
        UiTextStyleKey::from_style(self.style)
    }

    pub(crate) fn source_hash(&self) -> u64 {
        if self.preedit.is_none() {
            if let Some(document_key) = self.document_key {
                return document_key.fingerprint();
            }
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.text.hash(&mut hasher);
        if let Some(preedit) = self.preedit {
            preedit.range.start.hash(&mut hasher);
            preedit.range.end.hash(&mut hasher);
            preedit.text.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub(crate) const fn has_stable_viewport_document(&self) -> bool {
        self.document_key.is_some() && self.preedit.is_none() && self.viewport.is_some()
    }

    pub(crate) fn resolved_text(&self) -> Cow<'_, str> {
        let Some(preedit) = self.preedit else {
            return Cow::Borrowed(self.text);
        };

        let mut text = self.text.to_string();
        let start = preedit.range.start.min(text.len());
        let end = preedit.range.end.min(text.len()).max(start);
        if text.is_char_boundary(start) && text.is_char_boundary(end) {
            text.replace_range(start..end, &preedit.text);
        }
        Cow::Owned(text)
    }
}

pub(crate) fn resolve_text_layout(request: &UiTextLayoutRequest<'_>) -> UiTextLayoutResolution {
    resolve_text_layout_inner(request, |resolved_text| match request.layout_viewport() {
        Some(viewport) => layout_text_with_viewport(
            resolved_text,
            request.style,
            request.frame,
            request.clip_frame,
            viewport,
        ),
        None => layout_text(
            resolved_text,
            request.style,
            request.frame,
            request.clip_frame,
        ),
    })
}

pub(crate) fn resolve_text_layout_with_provider(
    request: &UiTextLayoutRequest<'_>,
    provider: &mut SharedTextLayoutSession,
) -> UiTextLayoutResolution {
    resolve_text_layout_inner(request, |resolved_text| match request.layout_viewport() {
        Some(viewport) => layout_text_with_provider_and_viewport(
            resolved_text,
            request.style,
            request.frame,
            request.clip_frame,
            viewport,
            request.document_key,
            provider,
        ),
        None => layout_text_with_provider(
            resolved_text,
            request.style,
            request.frame,
            request.clip_frame,
            provider,
        ),
    })
}

pub(crate) fn resolve_text_layout_with_provider_and_parsed(
    request: &UiTextLayoutRequest<'_>,
    parsed: &UiParsedText,
    provider: &mut SharedTextLayoutSession,
) -> UiTextLayoutResolution {
    let layout = layout_parsed_text_with_provider_and_viewport(
        parsed,
        request.style,
        request.frame,
        request.clip_frame,
        request.layout_viewport(),
        request.document_key,
        provider,
    );
    resolution_from_layout(request, layout)
}

fn resolve_text_layout_inner(
    request: &UiTextLayoutRequest<'_>,
    layout: impl FnOnce(&str) -> UiResolvedTextLayout,
) -> UiTextLayoutResolution {
    let resolved_text = request.resolved_text();
    let layout = layout(resolved_text.as_ref());
    resolution_from_layout(request, layout)
}

fn resolution_from_layout(
    request: &UiTextLayoutRequest<'_>,
    layout: UiResolvedTextLayout,
) -> UiTextLayoutResolution {
    let size = UiSize::new(layout.measured_width, layout.measured_height);
    let first_baseline = layout
        .lines
        .first()
        .map(|line| line.baseline)
        .unwrap_or_default();

    UiTextLayoutResolution {
        layout,
        size,
        first_baseline,
        source_hash: request.source_hash(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::UiTextRenderMode;

    #[test]
    fn style_key_encodes_clamp_overflow_float_bits() {
        let mut style = UiResolvedStyle {
            text_overflow: UiTextOverflow::ClampFontSize {
                min_px: 8.0,
                max_px: 18.0,
            },
            ..UiResolvedStyle::default()
        };
        let key = UiTextStyleKey::from_style(&style);

        assert_eq!(key, UiTextStyleKey::from_style(&style));

        style.text_overflow = UiTextOverflow::ClampFontSize {
            min_px: 8.0,
            max_px: 19.0,
        };
        assert_ne!(key, UiTextStyleKey::from_style(&style));
    }

    #[test]
    fn style_key_encodes_tab_size_bits() {
        let mut style = UiResolvedStyle {
            tab_size: 4.0,
            ..UiResolvedStyle::default()
        };
        let key = UiTextStyleKey::from_style(&style);

        style.tab_size = 6.0;

        assert_ne!(key, UiTextStyleKey::from_style(&style));
    }

    #[test]
    fn style_key_encodes_font_weight() {
        let mut style = UiResolvedStyle {
            font_weight: 400,
            ..UiResolvedStyle::default()
        };
        let key = UiTextStyleKey::from_style(&style);

        style.font_weight = 600;

        assert_ne!(key, UiTextStyleKey::from_style(&style));
    }

    #[test]
    fn preedit_request_disables_viewport_layout() {
        let style = UiResolvedStyle::default();
        let preedit = UiPreeditSpan {
            range: UiTextRange { start: 0, end: 0 },
            text: "x".to_string(),
        };
        let viewport = UiTextViewport::new(40.0, 20.0, 2).expect("finite viewport");
        let request =
            UiTextLayoutRequest::new("source", &style, UiFrame::new(0.0, 0.0, 120.0, 80.0), None)
                .with_viewport(viewport)
                .with_preedit(&preedit);

        assert_eq!(request.viewport, Some(viewport));
        assert_eq!(request.layout_viewport(), None);
    }

    #[test]
    fn viewport_derives_a_document_local_offset_from_absolute_frames() {
        let viewport = UiTextViewport::from_document_and_clip(
            UiFrame::new(20.0, -180.0, 240.0, 1_600.0),
            UiFrame::new(20.0, 60.0, 240.0, 80.0),
        )
        .expect("finite document and clip frames");

        assert_eq!(viewport.offset_y, 240.0);
        assert_eq!(viewport.extent_y, 80.0);
        assert_eq!(viewport.overscan_screens, 2);
    }

    #[test]
    fn style_key_normalizes_and_separates_run_language() {
        let mut style = UiResolvedStyle {
            language: Some(" ZH-hans ".to_string()),
            ..UiResolvedStyle::default()
        };
        let simplified = UiTextStyleKey::from_style(&style);

        style.language = Some("zh-HANS".to_string());
        assert_eq!(simplified, UiTextStyleKey::from_style(&style));

        style.language = Some("ja".to_string());
        assert_ne!(simplified, UiTextStyleKey::from_style(&style));
    }

    #[test]
    fn style_key_encodes_text_writing_mode() {
        let mut style = UiResolvedStyle {
            text_writing_mode: UiTextWritingMode::HorizontalTb,
            ..UiResolvedStyle::default()
        };
        let key = UiTextStyleKey::from_style(&style);

        style.text_writing_mode = UiTextWritingMode::VerticalRl;

        assert_ne!(key, UiTextStyleKey::from_style(&style));
    }

    #[test]
    fn style_key_ignores_text_render_mode() {
        let mut style = UiResolvedStyle {
            text_render_mode: UiTextRenderMode::Native,
            ..UiResolvedStyle::default()
        };
        let native = UiTextStyleKey::from_style(&style);

        style.text_render_mode = UiTextRenderMode::Sdf;

        assert_eq!(native, UiTextStyleKey::from_style(&style));
    }
}

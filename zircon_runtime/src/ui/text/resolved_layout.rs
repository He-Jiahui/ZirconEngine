use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiRichTextFormat, UiTextAlign, UiTextDirection,
        UiTextOverflow, UiTextRange, UiTextWrap, UiTextWritingMode,
    },
};

use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::{
    text_language_cache_identity, EphemeralCacheHash, SharedTextLayoutSession, TextDocumentKey,
};

use super::shaper::{
    layout_text, layout_text_with_provider, layout_text_with_provider_and_viewport,
    layout_text_with_viewport,
};
use super::{
    layout_engine::{
        layout_parsed_text_with_provider_and_viewport,
        layout_parsed_text_with_provider_and_viewport_outcome,
    },
    rich_text::{parse_source_text_with_provider, UiParsedText},
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextLayoutResolution {
    pub layout: UiResolvedTextLayout,
    pub size: UiSize,
    pub first_baseline: f32,
    pub source_hash: EphemeralCacheHash,
}

impl UiTextLayoutResolution {
    /// Heap bytes owned directly by the serializable layout DTO.
    ///
    /// The process-local artifact handle may share compiled/glyph allocations with other caches and
    /// is deliberately excluded until those owners publish a non-duplicating residency receipt.
    pub(crate) fn estimated_cache_heap_bytes(&self) -> usize {
        estimated_layout_heap_bytes(&self.layout)
    }
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
            language: text_language_cache_identity(style.language.as_deref()),
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

    pub(crate) fn estimated_heap_bytes(&self) -> usize {
        self.font_family
            .as_ref()
            .map_or(0, String::len)
            .saturating_add(self.language.as_ref().map_or(0, String::len))
    }
}

fn estimated_layout_heap_bytes(layout: &UiResolvedTextLayout) -> usize {
    let mut bytes = layout
        .lines
        .len()
        .saturating_mul(std::mem::size_of::<
            zircon_runtime_interface::ui::surface::UiResolvedTextLine,
        >())
        .saturating_add(layout.boxes.len().saturating_mul(std::mem::size_of::<
            zircon_runtime_interface::ui::surface::UiResolvedTextBox,
        >()));
    for line in &layout.lines {
        bytes = bytes
            .saturating_add(line.text.len())
            .saturating_add(
                line.glyph_advances
                    .len()
                    .saturating_mul(std::mem::size_of::<f32>()),
            )
            .saturating_add(line.runs.len().saturating_mul(std::mem::size_of::<
                zircon_runtime_interface::ui::surface::UiResolvedTextRun,
            >()));
        for run in &line.runs {
            bytes = bytes.saturating_add(run.text.len());
        }
    }
    if let Some(editable) = layout.editable.as_ref() {
        bytes = bytes.saturating_add(editable.text.len());
        if let Some(composition) = editable.composition.as_ref() {
            bytes =
                bytes
                    .saturating_add(composition.text.len())
                    .saturating_add(composition.restore_text.as_ref().map_or(0, String::len))
                    .saturating_add(composition.preedit_clauses.len().saturating_mul(
                        std::mem::size_of::<
                            zircon_runtime_interface::ui::surface::UiTextPreeditClause,
                        >(),
                    ));
        }
    }
    bytes
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

    pub(crate) fn source_hash(&self) -> EphemeralCacheHash {
        if self.preedit.is_none() {
            if let Some(document_key) = self.document_key {
                return document_key.ephemeral_hash();
            }
        }
        let mut hasher = crate::text::EphemeralCacheHasher::new();
        hasher.write(self.text);
        if let Some(preedit) = self.preedit {
            hasher.write(&preedit.range.start);
            hasher.write(&preedit.range.end);
            hasher.write(&preedit.text);
        }
        hasher.finish()
    }

    pub(crate) fn supports_viewport_virtualized_plain_layout(&self) -> bool {
        self.document_key.is_some()
            && self.preedit.is_none()
            && self.viewport.is_some()
            && matches!(self.style.rich_text_format, UiRichTextFormat::Plain)
            && matches!(
                self.style.text_writing_mode,
                UiTextWritingMode::HorizontalTb
            )
            && matches!(self.style.wrap, UiTextWrap::None)
            && matches!(self.style.text_overflow, UiTextOverflow::Clip)
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

/// Produces a cache-admissible UI layout only after every shaping and layout stage is Ready.
pub(crate) fn resolve_text_layout_with_provider_outcome(
    request: &UiTextLayoutRequest<'_>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiTextLayoutResolution> {
    let resolved_text = request.resolved_text();
    let parsed = match parse_source_text_with_provider(
        resolved_text.as_ref(),
        request.style.rich_text_format.into(),
        provider,
    ) {
        Ok(parsed) => parsed,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    resolve_text_layout_with_provider_and_parsed_outcome(request, &parsed, provider)
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

pub(crate) fn resolve_text_layout_with_provider_and_parsed_outcome(
    request: &UiTextLayoutRequest<'_>,
    parsed: &UiParsedText,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiTextLayoutResolution> {
    layout_parsed_text_with_provider_and_viewport_outcome(
        parsed,
        request.style,
        request.frame,
        request.clip_frame,
        request.layout_viewport(),
        request.document_key,
        provider,
    )
    .map(|layout| resolution_from_layout(request, layout))
}

fn resolve_text_layout_inner(
    request: &UiTextLayoutRequest<'_>,
    layout: impl FnOnce(&str) -> UiResolvedTextLayout,
) -> UiTextLayoutResolution {
    let resolved_text = request.resolved_text();
    let layout = layout(resolved_text.as_ref());
    resolution_from_layout(request, layout)
}

pub(crate) fn resolution_from_layout(
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
#[path = "tests/resolved_layout.rs"]
mod tests;

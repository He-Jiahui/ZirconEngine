use std::hash::{Hash, Hasher};

use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiTextAlign, UiTextDirection, UiTextOverflow,
        UiTextRange, UiTextRenderMode, UiTextWrap, UiTextWritingMode,
    },
};

use crate::graphics::text::shaping::TextShapeRunProvider;

use super::shaper::{layout_text, layout_text_with_provider};

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
    pub font_weight: u16,
    pub font_size_bits: u32,
    pub line_height_bits: u32,
    pub tab_size_bits: u32,
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub text_direction: UiTextDirection,
    pub text_writing_mode: UiTextWritingMode,
    pub text_overflow: UiTextOverflowKey,
    pub rich_text: bool,
    pub text_render_mode: UiTextRenderMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            font_weight: style.font_weight,
            font_size_bits: style.font_size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            tab_size_bits: style.tab_size.to_bits(),
            text_align: style.text_align,
            wrap: style.wrap,
            text_direction: style.text_direction,
            text_writing_mode: style.text_writing_mode,
            text_overflow: UiTextOverflowKey::from(style.text_overflow),
            rich_text: style.rich_text,
            text_render_mode: style.text_render_mode,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiPreeditSpan {
    pub range: UiTextRange,
    pub text: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct UiTextLayoutRequest<'a> {
    pub text: &'a str,
    pub style: &'a UiResolvedStyle,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub preedit: Option<&'a UiPreeditSpan>,
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
        }
    }

    pub(crate) const fn with_preedit(mut self, preedit: &'a UiPreeditSpan) -> Self {
        self.preedit = Some(preedit);
        self
    }

    pub(crate) fn style_key(&self) -> UiTextStyleKey {
        UiTextStyleKey::from_style(self.style)
    }

    pub(crate) fn source_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.text.hash(&mut hasher);
        if let Some(preedit) = self.preedit {
            preedit.range.start.hash(&mut hasher);
            preedit.range.end.hash(&mut hasher);
            preedit.text.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub(crate) fn resolved_text(&self) -> String {
        let Some(preedit) = self.preedit else {
            return self.text.to_string();
        };

        let mut text = self.text.to_string();
        let start = preedit.range.start.min(text.len());
        let end = preedit.range.end.min(text.len()).max(start);
        if text.is_char_boundary(start) && text.is_char_boundary(end) {
            text.replace_range(start..end, &preedit.text);
        }
        text
    }
}

pub(crate) fn resolve_text_layout(request: &UiTextLayoutRequest<'_>) -> UiTextLayoutResolution {
    resolve_text_layout_inner(request, |resolved_text| {
        layout_text(
            resolved_text,
            request.style,
            request.frame,
            request.clip_frame,
        )
    })
}

pub(crate) fn resolve_text_layout_with_provider<P>(
    request: &UiTextLayoutRequest<'_>,
    provider: &mut P,
) -> UiTextLayoutResolution
where
    P: TextShapeRunProvider + ?Sized,
{
    resolve_text_layout_inner(request, |resolved_text| {
        layout_text_with_provider(
            resolved_text,
            request.style,
            request.frame,
            request.clip_frame,
            provider,
        )
    })
}

fn resolve_text_layout_inner(
    request: &UiTextLayoutRequest<'_>,
    layout: impl FnOnce(&str) -> UiResolvedTextLayout,
) -> UiTextLayoutResolution {
    let resolved_text = request.resolved_text();
    let layout = layout(&resolved_text);
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
    fn style_key_encodes_text_writing_mode() {
        let mut style = UiResolvedStyle {
            text_writing_mode: UiTextWritingMode::HorizontalTb,
            ..UiResolvedStyle::default()
        };
        let key = UiTextStyleKey::from_style(&style);

        style.text_writing_mode = UiTextWritingMode::VerticalRl;

        assert_ne!(key, UiTextStyleKey::from_style(&style));
    }
}

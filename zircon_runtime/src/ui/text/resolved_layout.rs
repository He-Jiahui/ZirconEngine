use std::hash::{Hash, Hasher};

use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiTextAlign, UiTextDirection, UiTextOverflow,
        UiTextRange, UiTextRenderMode, UiTextWrap,
    },
};

use super::shaper::layout_text;

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
    pub font_size_bits: u32,
    pub line_height_bits: u32,
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub text_direction: UiTextDirection,
    pub text_overflow: UiTextOverflow,
    pub rich_text: bool,
    pub text_render_mode: UiTextRenderMode,
}

impl UiTextStyleKey {
    pub(crate) fn from_style(style: &UiResolvedStyle) -> Self {
        Self {
            font_family: style.font_family.clone().or_else(|| style.font.clone()),
            font_size_bits: style.font_size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            text_align: style.text_align,
            wrap: style.wrap,
            text_direction: style.text_direction,
            text_overflow: style.text_overflow,
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
    let resolved_text = request.resolved_text();
    let layout = layout_text(
        &resolved_text,
        request.style,
        request.frame,
        request.clip_frame,
    );
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

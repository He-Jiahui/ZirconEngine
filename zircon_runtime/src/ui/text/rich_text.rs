use crate::core::framework::render::{
    InlineObjectRef, LinkRef, ParagraphOverride, RichParseResult, RichTextFormat, StyleOverride,
};
use crate::graphics::text::rich::parse_rich_text;
use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

mod link_hit;

pub(crate) use link_hit::link_at_layout_point;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextSourceRun {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
    pub style: StyleOverride,
    pub inline: Option<InlineObjectRef>,
    pub link: Option<LinkRef>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiParsedText {
    pub text: String,
    pub runs: Vec<UiTextSourceRun>,
    pub paragraphs: Vec<((u32, u32), ParagraphOverride)>,
    pub rich: RichParseResult,
}

pub(crate) fn parse_source_text(text: &str, format: RichTextFormat) -> UiParsedText {
    let parsed = parse_rich_text(text, format);
    let runs = parsed
        .runs
        .iter()
        .filter_map(|run| {
            let start = usize::try_from(run.byte_range.0).ok()?;
            let end = usize::try_from(run.byte_range.1).ok()?;
            let text = parsed.text.get(start..end)?.to_string();
            (!text.is_empty()).then_some(UiTextSourceRun {
                kind: ui_run_kind(&run.style, run.link.as_ref()),
                text,
                source_range: UiTextRange { start, end },
                style: run.style.clone(),
                inline: run.inline.clone(),
                link: run.link.clone(),
            })
        })
        .collect();
    UiParsedText {
        text: parsed.text.clone(),
        runs,
        paragraphs: parsed.paragraphs.clone(),
        rich: parsed,
    }
}

fn ui_run_kind(style: &StyleOverride, link: Option<&LinkRef>) -> UiTextRunKind {
    if link.is_some() {
        UiTextRunKind::Link
    } else if style.code == Some(true) {
        UiTextRunKind::Code
    } else if style.weight.is_some_and(|weight| weight >= 600) {
        UiTextRunKind::Strong
    } else if style.italic == Some(true) {
        UiTextRunKind::Emphasis
    } else {
        UiTextRunKind::Plain
    }
}

#[cfg(test)]
mod tests;

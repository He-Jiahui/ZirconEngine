use crate::core::framework::render::{RichTextFormat, StyleOverride};
use crate::graphics::text::rich::parse_rich_text;
use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiTextSourceRun {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiParsedText {
    pub text: String,
    pub runs: Vec<UiTextSourceRun>,
}

pub(crate) fn parse_source_text(text: &str, rich_text: bool) -> UiParsedText {
    if !rich_text {
        return UiParsedText {
            text: text.to_string(),
            runs: (!text.is_empty())
                .then(|| UiTextSourceRun {
                    kind: UiTextRunKind::Plain,
                    text: text.to_string(),
                    source_range: UiTextRange {
                        start: 0,
                        end: text.len(),
                    },
                })
                .into_iter()
                .collect(),
        };
    }

    let parsed = parse_rich_text(text, RichTextFormat::Markdown);
    let runs = parsed
        .runs
        .into_iter()
        .filter_map(|run| {
            let start = usize::try_from(run.byte_range.0).ok()?;
            let end = usize::try_from(run.byte_range.1).ok()?;
            let text = parsed.text.get(start..end)?.to_string();
            (!text.is_empty()).then_some(UiTextSourceRun {
                kind: ui_run_kind(&run.style),
                text,
                source_range: UiTextRange { start, end },
            })
        })
        .collect();
    UiParsedText {
        text: parsed.text,
        runs,
    }
}

fn ui_run_kind(style: &StyleOverride) -> UiTextRunKind {
    if style.code == Some(true) {
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

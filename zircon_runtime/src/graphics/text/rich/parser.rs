use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::render::{RichParseResult, RichTextFormat, StyleOverride, StyledRun};

use super::bbcode::{token_at, BbCodeToken};
use super::decorator::DecoratorRegistry;

pub(super) fn parse(
    markup: &str,
    format: RichTextFormat,
    decorators: &DecoratorRegistry,
) -> RichParseResult {
    match format {
        RichTextFormat::Plain | RichTextFormat::Html => plain(markup),
        RichTextFormat::BbCode => parse_bbcode(markup, decorators),
        RichTextFormat::Markdown => parse_markdown(markup),
    }
}

fn plain(text: &str) -> RichParseResult {
    let mut result = RichParseResult {
        text: text.to_string(),
        ..RichParseResult::default()
    };
    if !text.is_empty() {
        result
            .runs
            .push(styled_run(0, text.len(), StyleOverride::default()));
    }
    result
}

fn parse_bbcode(markup: &str, decorators: &DecoratorRegistry) -> RichParseResult {
    let mut result = RichParseResult::default();
    let mut active_tags: Vec<(String, StyleOverride)> = Vec::new();
    let mut index = 0;
    let mut text_start = 0;

    while index < markup.len() {
        let remaining = &markup[index..];
        let Some((token_len, token)) = token_at(remaining) else {
            index += next_char_len(remaining);
            continue;
        };

        append_text(
            &mut result,
            &markup[text_start..index],
            active_tags
                .last()
                .map(|(_, style)| style)
                .cloned()
                .unwrap_or_default(),
        );
        match token {
            BbCodeToken::Open { name, value } => {
                let mut style = active_tags
                    .last()
                    .map(|(_, style)| style)
                    .cloned()
                    .unwrap_or_default();
                if decorators.apply(&name, value.as_deref(), &mut style) {
                    active_tags.push((name, style));
                }
            }
            BbCodeToken::Close { name } => {
                if let Some(position) = active_tags
                    .iter()
                    .rposition(|(active_name, _)| active_name == &name)
                {
                    active_tags.truncate(position);
                }
            }
        }
        index += token_len;
        text_start = index;
    }

    append_text(
        &mut result,
        &markup[text_start..],
        active_tags
            .last()
            .map(|(_, style)| style)
            .cloned()
            .unwrap_or_default(),
    );
    result.runs = align_runs_to_graphemes(&result.text, &result.runs);
    result
}

fn parse_markdown(markup: &str) -> RichParseResult {
    let mut result = RichParseResult::default();
    let mut index = 0;
    let mut text_start = 0;
    while index < markup.len() {
        let remaining = &markup[index..];
        let Some((open, close, style)) = markdown_marker(remaining) else {
            index += next_char_len(remaining);
            continue;
        };
        let Some(close_offset) = remaining[open.len()..].find(close) else {
            index += open.len();
            continue;
        };
        append_text(
            &mut result,
            &markup[text_start..index],
            StyleOverride::default(),
        );
        let content_start = index + open.len();
        let content_end = content_start + close_offset;
        append_text(&mut result, &markup[content_start..content_end], style);
        index = content_end + close.len();
        text_start = index;
    }
    append_text(&mut result, &markup[text_start..], StyleOverride::default());
    result.runs = align_runs_to_graphemes(&result.text, &result.runs);
    result
}

fn markdown_marker(input: &str) -> Option<(&'static str, &'static str, StyleOverride)> {
    if input.starts_with("**") {
        Some((
            "**",
            "**",
            StyleOverride {
                weight: Some(700),
                ..StyleOverride::default()
            },
        ))
    } else if input.starts_with('*') {
        Some((
            "*",
            "*",
            StyleOverride {
                italic: Some(true),
                ..StyleOverride::default()
            },
        ))
    } else if input.starts_with('`') {
        Some((
            "`",
            "`",
            StyleOverride {
                code: Some(true),
                ..StyleOverride::default()
            },
        ))
    } else {
        None
    }
}

fn append_text(result: &mut RichParseResult, text: &str, style: StyleOverride) {
    if text.is_empty() {
        return;
    }
    let start = result.text.len();
    result.text.push_str(text);
    push_or_merge_run(
        &mut result.runs,
        styled_run(start, result.text.len(), style),
    );
}

fn align_runs_to_graphemes(text: &str, runs: &[StyledRun]) -> Vec<StyledRun> {
    let mut aligned = Vec::new();
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        let style = runs
            .iter()
            .find(|run| range_contains(run.byte_range, start))
            .map(|run| run.style.clone())
            .unwrap_or_default();
        push_or_merge_run(&mut aligned, styled_run(start, end, style));
    }
    aligned
}

fn range_contains(range: (u32, u32), offset: usize) -> bool {
    usize::try_from(range.0).unwrap_or(usize::MAX) <= offset
        && offset < usize::try_from(range.1).unwrap_or(0)
}

fn styled_run(start: usize, end: usize, style: StyleOverride) -> StyledRun {
    StyledRun {
        byte_range: (
            u32::try_from(start).unwrap_or(u32::MAX),
            u32::try_from(end).unwrap_or(u32::MAX),
        ),
        style,
        inline: None,
        link: None,
    }
}

fn push_or_merge_run(runs: &mut Vec<StyledRun>, run: StyledRun) {
    if let Some(previous) = runs.last_mut() {
        if previous.byte_range.1 == run.byte_range.0
            && previous.style == run.style
            && previous.inline == run.inline
            && previous.link == run.link
        {
            previous.byte_range.1 = run.byte_range.1;
            return;
        }
    }
    runs.push(run);
}

fn next_char_len(input: &str) -> usize {
    input.chars().next().map(char::len_utf8).unwrap_or(1)
}

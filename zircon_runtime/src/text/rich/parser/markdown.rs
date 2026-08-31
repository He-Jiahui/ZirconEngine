use super::*;

pub(super) fn parse_markdown(
    markup: &str,
    budget: RichParseBudget,
    content_trust: RichTextContentTrust,
) -> Result<RichParseResult, RichTextParseError> {
    let mut result = RichParseBuilder::new(budget, content_trust);
    let bold_close_frontier = ClosingDelimiterFrontier::new(markup, "**");
    let italic_close_frontier = ClosingDelimiterFrontier::new(markup, "*");
    let code_close_frontier = ClosingDelimiterFrontier::new(markup, "`");
    let mut index = 0;
    let mut text_start = 0;
    while index < markup.len() {
        let remaining = &markup[index..];
        let Some((open, close, style)) = markdown_marker(remaining) else {
            index += next_char_len(remaining);
            continue;
        };
        let close_frontier = match close {
            "**" => bold_close_frontier,
            "*" => italic_close_frontier,
            "`" => code_close_frontier,
            _ => unreachable!("markdown_marker returned an unsupported closer"),
        };
        if !close_frontier.has_close_at_or_after(index + open.len()) {
            index += open.len();
            continue;
        }
        let Some(close_offset) = remaining[open.len()..].find(close) else {
            index += open.len();
            continue;
        };
        result.admit_tokens(2)?;
        append_source_text(
            &mut result,
            &markup[text_start..index],
            text_start,
            StyleOverride::default(),
        )?;
        let content_start = index + open.len();
        let content_end = content_start + close_offset;
        append_source_text(
            &mut result,
            &markup[content_start..content_end],
            content_start,
            style,
        )?;
        index = content_end + close.len();
        text_start = index;
    }
    append_source_text(
        &mut result,
        &markup[text_start..],
        text_start,
        StyleOverride::default(),
    )?;
    result.runs = align_runs_to_graphemes_bounded(&result.text, &result.runs, budget.max_runs)?;
    result.finish()
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

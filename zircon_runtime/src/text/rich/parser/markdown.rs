use super::*;

/// MarkdownInlineV1 仅接纳粗体、斜体与行内代码标记；未配对标记保留字面内容，不承诺块级 Markdown 语义。
pub(super) fn parse_markdown(
    markup: &str,
    budget: RichParseBudget,
    content_trust: RichTextContentTrust,
) -> Result<RichParseResult, RichTextParseError> {
    let mut result = RichParseBuilder::new(budget, content_trust);
    let close_frontiers = markdown_closing_frontiers(markup);
    let mut index = 0;
    let mut text_start = 0;
    while index < markup.len() {
        let remaining = &markup[index..];
        if let Some(length) = markdown_escape_len(remaining) {
            index += length;
            continue;
        }
        let Some((open, close, style)) = markdown_marker(remaining) else {
            index += next_char_len(remaining);
            continue;
        };
        let last_close = close_frontiers[match close {
            "**" => 0,
            "*" => 1,
            "`" => 2,
            _ => unreachable!("markdown_marker returned an unsupported closer"),
        }];
        if !last_close.is_some_and(|offset| offset >= index + open.len()) {
            index += open.len();
            continue;
        }
        let Some(content_end) = find_markdown_close(markup, index + open.len(), close) else {
            index += open.len();
            continue;
        };
        result.admit_tokens(2)?;
        append_markdown_source_text(
            &mut result,
            &markup[text_start..index],
            text_start,
            StyleOverride::default(),
        )?;
        let content_start = index + open.len();
        append_markdown_source_text(
            &mut result,
            &markup[content_start..content_end],
            content_start,
            style,
        )?;
        index = content_end + close.len();
        text_start = index;
    }
    append_markdown_source_text(
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

fn markdown_escape_len(input: &str) -> Option<usize> {
    (input.as_bytes().first() == Some(&b'\\')
        && input.as_bytes().get(1).is_some_and(|byte| {
            matches!(
                *byte,
                b'\\' | b'*' | b'_' | b'[' | b']' | b'(' | b')' | b'\x60'
            )
        }))
    .then_some(2)
}

// One linear pass records real closers; escaped suffixes cannot trigger repeated failed searches.
fn markdown_closing_frontiers(markup: &str) -> [Option<usize>; 3] {
    let mut last = [None; 3];
    let mut index = 0;
    while index < markup.len() {
        let remaining = &markup[index..];
        if let Some(length) = markdown_escape_len(remaining) {
            index += length;
            continue;
        }
        if remaining.starts_with("**") {
            last[0] = Some(index);
        }
        if remaining.starts_with('*') {
            last[1] = Some(index);
        }
        if remaining.as_bytes().first() == Some(&b'\x60') {
            last[2] = Some(index);
        }
        index += next_char_len(remaining);
    }
    last
}

fn find_markdown_close(markup: &str, mut index: usize, close: &str) -> Option<usize> {
    while index < markup.len() {
        let remaining = &markup[index..];
        if let Some(length) = markdown_escape_len(remaining) {
            index += length;
        } else if remaining.starts_with(close) {
            return Some(index);
        } else {
            index += next_char_len(remaining);
        }
    }
    None
}

// Decode only this format's ASCII escapes while retaining every original source offset.
fn append_markdown_source_text(
    result: &mut RichParseBuilder,
    text: &str,
    source_start: usize,
    style: StyleOverride,
) -> Result<(), RichTextParseError> {
    let mut index = 0;
    let mut text_start = 0;
    while index < text.len() {
        let remaining = &text[index..];
        if let Some(length) = markdown_escape_len(remaining) {
            append_source_text(
                result,
                &text[text_start..index],
                source_start + text_start,
                style.clone(),
            )?;
            if length > result.budget.max_token_bytes {
                return Err(RichTextParseError::TokenByteBudgetExceeded {
                    attempted_bytes: length,
                    max_bytes: result.budget.max_token_bytes,
                });
            }
            result.admit_tokens(1)?;
            append_source_text(
                result,
                &text[index + 1..index + length],
                source_start + index + 1,
                style.clone(),
            )?;
            index += length;
            text_start = index;
        } else {
            index += next_char_len(remaining);
        }
    }
    append_source_text(
        result,
        &text[text_start..],
        source_start + text_start,
        style,
    )
}

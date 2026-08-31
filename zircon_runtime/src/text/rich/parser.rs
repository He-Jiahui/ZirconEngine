use crate::text::{
    InlineObjectRef, LinkRef, ParagraphOverride, RichListItem, RichParseResult, RichTextFormat,
    StyleOverride, StyledRun, TextAlign,
};

use super::admission::checked_artifact_index;
use super::bbcode::{BbCodeToken, literal_tag_text, token_at};
use super::bbcode_blocks::{BbCodeBlockState, BlockClose, BlockOpen};
use super::bbcode_table::BbCodeTableState;
use super::decorator::{DecoratorRegistry, RichTextDecoration};
use super::emoji_shortcode::EmojiShortcodeRegistry;
use super::html_subset;
use super::{RichParseBudget, RichTextContentTrust, RichTextParseError};

mod active_tags;
mod bidi_diagnostics;
mod builder;
mod html;
mod html_diagnostics;
mod markdown;
mod run_alignment;

use active_tags::{ActiveTag, ActiveTagClose, ActiveTagStack, current_link, current_style};
use bidi_diagnostics::{
    push_literal_bidi_control_diagnostic, push_source_bidi_control_diagnostics,
};
use builder::RichParseBuilder;
use run_alignment::{align_runs_to_graphemes, align_runs_to_graphemes_bounded};

#[derive(Clone, Copy, Debug)]
struct ClosingDelimiterFrontier {
    last_close_start: Option<usize>,
}

impl ClosingDelimiterFrontier {
    fn new(markup: &str, close: &str) -> Self {
        Self {
            last_close_start: markup.rfind(close),
        }
    }

    fn has_close_at_or_after(self, offset: usize) -> bool {
        self.last_close_start
            .is_some_and(|last_close_start| last_close_start >= offset)
    }
}

fn repeated_opening_delimiter_skip(input: &str, delimiter: u8) -> usize {
    input
        .bytes()
        .take_while(|byte| *byte == delimiter)
        .count()
        .saturating_sub(1)
}

pub(super) fn parse(
    markup: &str,
    format: RichTextFormat,
    decorators: &DecoratorRegistry,
    emoji_shortcodes: &EmojiShortcodeRegistry,
    budget: RichParseBudget,
    content_trust: RichTextContentTrust,
) -> Result<RichParseResult, RichTextParseError> {
    match format {
        RichTextFormat::Plain => plain(markup, budget, content_trust),
        RichTextFormat::BbCodeV1 => {
            parse_bbcode(markup, decorators, emoji_shortcodes, budget, content_trust)
        }
        RichTextFormat::HtmlSubsetV1 => html::parse_html(markup, budget, content_trust),
        RichTextFormat::MarkdownInlineV1 => markdown::parse_markdown(markup, budget, content_trust),
    }
}

fn markup_source_range(start: usize, token_len: usize) -> Result<(u32, u32), RichTextParseError> {
    let end = start.checked_add(token_len).unwrap_or(usize::MAX);
    Ok((
        checked_artifact_index("rich diagnostic source start", start)?,
        checked_artifact_index("rich diagnostic source end", end)?,
    ))
}

fn plain(
    text: &str,
    budget: RichParseBudget,
    content_trust: RichTextContentTrust,
) -> Result<RichParseResult, RichTextParseError> {
    let mut result = RichParseBuilder::new(budget, content_trust);
    append_source_text(&mut result, text, 0, StyleOverride::default())?;
    result.finish()
}

fn parse_bbcode(
    markup: &str,
    decorators: &DecoratorRegistry,
    emoji_shortcodes: &EmojiShortcodeRegistry,
    budget: RichParseBudget,
    content_trust: RichTextContentTrust,
) -> Result<RichParseResult, RichTextParseError> {
    let mut result = RichParseBuilder::new(budget, content_trust);
    let mut active_tags = ActiveTagStack::new(budget.max_active_tag_depth);
    let mut active_paragraphs: Vec<ActiveParagraph> = Vec::new();
    let mut block_state = BbCodeBlockState::new(budget.max_block_depth);
    let mut table_state = BbCodeTableState::new(budget.max_table_cells, budget.max_table_depth);
    let mut pending_block_break = false;
    let close_frontier = ClosingDelimiterFrontier::new(markup, "]");
    let mut index = 0;
    let mut text_start = 0;

    while index < markup.len() {
        let remaining = &markup[index..];
        let repeated_opener_skip = repeated_opening_delimiter_skip(remaining, b'[');
        if repeated_opener_skip > 0 {
            index += repeated_opener_skip;
            continue;
        }
        if remaining.starts_with('[') && !close_frontier.has_close_at_or_after(index + 1) {
            break;
        }
        let Some((token_len, token)) = token_at(remaining, budget.tokenizer_budget())? else {
            index += next_char_len(remaining);
            continue;
        };
        result.admit_tokens(1)?;
        let token_source_range = markup_source_range(index, token_len)?;

        append_bbcode_text(
            &mut result,
            &markup[text_start..index],
            text_start,
            current_style(&active_tags),
            current_link(&active_tags),
            emoji_shortcodes,
            &mut pending_block_break,
        )?;
        match token {
            BbCodeToken::Open {
                name,
                value,
                attributes,
            } => {
                let mut style = current_style(&active_tags);
                if name == "table" {
                    ensure_block_boundary(
                        &mut result,
                        &mut pending_block_break,
                        style,
                        current_link(&active_tags),
                    );
                    table_state.open_table(value.as_deref(), result.current_offset())?;
                } else if name == "cell" {
                    table_state.open_cell(
                        value.as_deref(),
                        &attributes,
                        result.current_offset(),
                    )?;
                } else if let Some(block) = block_state.open(
                    &name,
                    value.as_deref(),
                    &attributes,
                    active_paragraphs.len(),
                )? {
                    ensure_block_boundary(
                        &mut result,
                        &mut pending_block_break,
                        style.clone(),
                        current_link(&active_tags),
                    );
                    if let BlockOpen::Paragraph {
                        name,
                        mut paragraph,
                        list_item,
                    } = block
                    {
                        let start = result.current_offset();
                        if let Some(list_item) = list_item {
                            append_text_with_metadata(
                                &mut result,
                                &list_item.prefix,
                                style,
                                None,
                                current_link(&active_tags),
                            );
                            paragraph.list_item = Some(RichListItem {
                                kind: list_item.kind,
                                level: list_item.level,
                                marker_range: (start, result.current_offset()),
                            });
                        }
                        if result.admit_active_paragraph_depth(
                            block_state
                                .depth()
                                .saturating_add(active_paragraphs.len())
                                .saturating_add(1),
                        ) {
                            active_paragraphs.push(ActiveParagraph {
                                name,
                                start,
                                paragraph: Some(paragraph),
                            });
                        }
                    }
                } else if let Some(literal) = literal_tag_text(&name) {
                    push_literal_bidi_control_diagnostic(&mut result, literal, token_source_range)?;
                    ensure_pending_block_break(
                        &mut result,
                        &mut pending_block_break,
                        style.clone(),
                        current_link(&active_tags),
                    );
                    append_text_with_metadata(
                        &mut result,
                        literal,
                        style,
                        None,
                        current_link(&active_tags),
                    );
                } else if name == "img" {
                    if let Some(inline) =
                        html_subset::bbcode_inline_image(value.as_deref(), &attributes)
                    {
                        ensure_pending_block_break(
                            &mut result,
                            &mut pending_block_break,
                            style.clone(),
                            current_link(&active_tags),
                        );
                        append_inline_object(
                            &mut result,
                            style,
                            current_link(&active_tags),
                            inline,
                        );
                    }
                } else if name == "url" {
                    if let Some(link) =
                        html_subset::bbcode_link(value.as_deref(), &attributes, &mut style)
                    {
                        active_tags.push(ActiveTag {
                            name,
                            style,
                            link: Some(link),
                            source_range: token_source_range,
                        })?;
                    }
                } else if let Some(align) = bbcode_paragraph_align(&name) {
                    ensure_block_boundary(
                        &mut result,
                        &mut pending_block_break,
                        style,
                        current_link(&active_tags),
                    );
                    let paragraph = active_paragraphs.is_empty().then_some(ParagraphOverride {
                        align: Some(align),
                        ..ParagraphOverride::default()
                    });
                    if result.admit_active_paragraph_depth(
                        block_state
                            .depth()
                            .saturating_add(active_paragraphs.len())
                            .saturating_add(1),
                    ) {
                        active_paragraphs.push(ActiveParagraph {
                            name,
                            start: result.current_offset(),
                            paragraph,
                        });
                    }
                } else {
                    let mut decoration = RichTextDecoration {
                        style,
                        inline: None,
                        link: current_link(&active_tags),
                    };
                    if decorators.apply(
                        &name,
                        value.as_deref(),
                        &mut decoration,
                        budget.max_decorator_metadata_bytes_per_call,
                    )? {
                        if let Some(inline) = decoration.inline {
                            ensure_pending_block_break(
                                &mut result,
                                &mut pending_block_break,
                                decoration.style.clone(),
                                decoration.link.clone(),
                            );
                            append_inline_object(
                                &mut result,
                                decoration.style,
                                decoration.link,
                                inline,
                            );
                        } else {
                            active_tags.push(ActiveTag {
                                name,
                                style: decoration.style,
                                link: decoration.link,
                                source_range: token_source_range,
                            })?;
                        }
                    }
                }
            }
            BbCodeToken::Close { name } => {
                if name == "cell" {
                    table_state.close_cell(result.current_offset());
                } else if name == "table" {
                    if let Some(table) = table_state.close_table(result.current_offset()) {
                        result.push_table(table);
                        pending_block_break = true;
                    }
                } else if let Some(block) = block_state.close(&name) {
                    if let BlockClose::Paragraph { name } = block {
                        close_paragraph_override(&mut result, &name, &mut active_paragraphs);
                    }
                    pending_block_break = true;
                } else {
                    let closes_paragraph = bbcode_paragraph_align(&name).is_some();
                    close_paragraph_override(&mut result, &name, &mut active_paragraphs);
                    if closes_paragraph {
                        pending_block_break = true;
                    }
                    active_tags.close(&name);
                }
            }
        }
        index += token_len;
        text_start = index;
    }

    append_bbcode_text(
        &mut result,
        &markup[text_start..],
        text_start,
        current_style(&active_tags),
        current_link(&active_tags),
        emoji_shortcodes,
        &mut pending_block_break,
    )?;
    result.runs = align_runs_to_graphemes_bounded(&result.text, &result.runs, budget.max_runs)?;
    close_open_paragraph_overrides(&mut result, active_paragraphs);
    let text_end = result.current_offset();
    for table in table_state.finish(text_end) {
        result.push_table(table);
    }
    result.tables.sort_by(|left, right| {
        left.byte_range
            .0
            .cmp(&right.byte_range.0)
            .then_with(|| right.byte_range.1.cmp(&left.byte_range.1))
            .then_with(|| left.depth.cmp(&right.depth))
    });
    result.paragraphs.sort_by(|left, right| {
        left.0
            .0
            .cmp(&right.0.0)
            .then_with(|| right.0.1.cmp(&left.0.1))
    });
    result.finish()
}

fn append_bbcode_text(
    result: &mut RichParseBuilder,
    text: &str,
    source_start: usize,
    style: StyleOverride,
    link: Option<LinkRef>,
    emoji_shortcodes: &EmojiShortcodeRegistry,
    pending_block_break: &mut bool,
) -> Result<(), RichTextParseError> {
    push_source_bidi_control_diagnostics(result, text, source_start)?;
    let text = match emoji_shortcodes.expand(
        text,
        result.text.len(),
        result.budget.admitted_output_bytes(),
    ) {
        Ok(text) => text,
        Err(error) => {
            result.error.get_or_insert(error);
            return Ok(());
        }
    };
    if text.is_empty() {
        return Ok(());
    }
    ensure_pending_block_break(result, pending_block_break, style.clone(), link.clone());
    append_text_with_metadata(result, text.as_ref(), style, None, link);
    Ok(())
}

fn ensure_block_boundary(
    result: &mut RichParseBuilder,
    pending_block_break: &mut bool,
    style: StyleOverride,
    link: Option<LinkRef>,
) {
    if result.text.is_empty() {
        *pending_block_break = false;
        return;
    }
    *pending_block_break = true;
    ensure_pending_block_break(result, pending_block_break, style, link);
}

fn ensure_pending_block_break(
    result: &mut RichParseBuilder,
    pending_block_break: &mut bool,
    style: StyleOverride,
    link: Option<LinkRef>,
) {
    if !std::mem::take(pending_block_break) || result.text.is_empty() || result.text.ends_with('\n')
    {
        return;
    }
    append_text_with_metadata(result, "\n", style, None, link);
}

#[derive(Clone, Debug)]
struct ActiveParagraph {
    name: String,
    start: u32,
    paragraph: Option<ParagraphOverride>,
}

fn bbcode_paragraph_align(name: &str) -> Option<TextAlign> {
    match name {
        "left" => Some(TextAlign::Left),
        "center" => Some(TextAlign::Center),
        "right" => Some(TextAlign::Right),
        "fill" => Some(TextAlign::Justify),
        _ => None,
    }
}

fn close_paragraph_override(
    result: &mut RichParseBuilder,
    name: &str,
    active_paragraphs: &mut Vec<ActiveParagraph>,
) {
    let Some(position) = active_paragraphs
        .iter()
        .rposition(|active| active.name == name)
    else {
        return;
    };
    let end = result.current_offset();
    for active in active_paragraphs.drain(position..) {
        if let Some(paragraph) = active.paragraph {
            result.push_paragraph((active.start, end), paragraph);
        }
    }
}

fn close_open_paragraph_overrides(
    result: &mut RichParseBuilder,
    active_paragraphs: Vec<ActiveParagraph>,
) {
    let end = result.current_offset();
    for active in active_paragraphs {
        if let Some(paragraph) = active.paragraph {
            result.push_paragraph((active.start, end), paragraph);
        }
    }
}

fn append_source_text(
    result: &mut RichParseBuilder,
    text: &str,
    source_start: usize,
    style: StyleOverride,
) -> Result<(), RichTextParseError> {
    append_source_text_with_metadata(result, text, source_start, style, None, None)
}

fn append_source_text_with_metadata(
    result: &mut RichParseBuilder,
    text: &str,
    source_start: usize,
    style: StyleOverride,
    inline: Option<InlineObjectRef>,
    link: Option<LinkRef>,
) -> Result<(), RichTextParseError> {
    push_source_bidi_control_diagnostics(result, text, source_start)?;
    append_text_with_metadata(result, text, style, inline, link);
    Ok(())
}

fn append_text_with_metadata(
    result: &mut RichParseBuilder,
    text: &str,
    style: StyleOverride,
    inline: Option<InlineObjectRef>,
    link: Option<LinkRef>,
) {
    if text.is_empty() {
        return;
    }
    let Some((start, end)) = result.admit_append(text.len()) else {
        return;
    };
    let mut run = styled_run(start, end, style);
    run.inline = inline;
    run.link = link;
    if !result.admit_run(&run) {
        return;
    }
    result.text.push_str(text);
    result.push_run(run);
}

fn append_inline_object(
    result: &mut RichParseBuilder,
    style: StyleOverride,
    link: Option<LinkRef>,
    inline: InlineObjectRef,
) {
    append_text_with_metadata(
        result,
        super::INLINE_OBJECT_REPLACEMENT,
        style,
        Some(inline),
        link,
    );
}

fn range_contains(range: (u32, u32), offset: usize) -> bool {
    range.0 as usize <= offset && offset < range.1 as usize
}

fn styled_run(start: u32, end: u32, style: StyleOverride) -> StyledRun {
    StyledRun {
        byte_range: (start, end),
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

#[cfg(test)]
mod performance_tests;

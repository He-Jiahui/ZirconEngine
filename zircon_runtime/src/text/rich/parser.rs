use std::collections::HashMap;

use unicode_segmentation::UnicodeSegmentation;

use crate::text::TextAlign;
use crate::text::{
    InlineObjectRef, LinkRef, ParagraphOverride, RichParseResult, RichTable, RichTextFormat,
    StyleOverride, StyledRun,
};

use super::bbcode::{BbCodeToken, literal_tag_text, token_at};
use super::bbcode_blocks::{BbCodeBlockState, BlockClose, BlockOpen};
use super::bbcode_table::BbCodeTableState;
use super::decorator::{DecoratorRegistry, RichTextDecoration};
use super::emoji_shortcode::EmojiShortcodeRegistry;
use super::html_subset::{self, HtmlToken};

#[derive(Default)]
struct RichParseBuilder {
    text: String,
    runs: Vec<StyledRun>,
    paragraphs: Vec<((u32, u32), ParagraphOverride)>,
    tables: Vec<RichTable>,
}

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

impl RichParseBuilder {
    fn finish(self) -> RichParseResult {
        RichParseResult {
            text: self.text.into(),
            runs: self.runs,
            paragraphs: self.paragraphs,
            tables: self.tables,
        }
    }
}

pub(super) fn parse(
    markup: &str,
    format: RichTextFormat,
    decorators: &DecoratorRegistry,
    emoji_shortcodes: &EmojiShortcodeRegistry,
) -> RichParseResult {
    match format {
        RichTextFormat::Plain => plain(markup),
        RichTextFormat::BbCode => parse_bbcode(markup, decorators, emoji_shortcodes),
        RichTextFormat::Html => parse_html(markup),
        RichTextFormat::Markdown => parse_markdown(markup),
    }
}

fn parse_html(markup: &str) -> RichParseResult {
    let mut result = RichParseBuilder::default();
    let mut active_tags = ActiveTagStack::default();
    let close_frontier = ClosingDelimiterFrontier::new(markup, ">");
    let mut index = 0;
    let mut text_start = 0;

    while index < markup.len() {
        let remaining = &markup[index..];
        let repeated_opener_skip = repeated_opening_delimiter_skip(remaining, b'<');
        if repeated_opener_skip > 0 {
            index += repeated_opener_skip;
            continue;
        }
        if remaining.starts_with('<') && !close_frontier.has_close_at_or_after(index + 1) {
            break;
        }
        let Some((token_len, token)) = html_subset::token_at(remaining) else {
            index += next_char_len(remaining);
            continue;
        };
        append_html_text(
            &mut result,
            &markup[text_start..index],
            current_style(&active_tags),
            current_link(&active_tags),
        );
        match token {
            HtmlToken::Open { name, .. } if name == "br" => {
                append_text_with_metadata(
                    &mut result,
                    "\n",
                    current_style(&active_tags),
                    None,
                    current_link(&active_tags),
                );
            }
            HtmlToken::Open {
                name, attributes, ..
            } if name == "img" => {
                if let Some(inline) = html_subset::inline_image(&attributes) {
                    append_inline_object(
                        &mut result,
                        current_style(&active_tags),
                        current_link(&active_tags),
                        inline,
                    );
                }
            }
            HtmlToken::Open {
                name,
                attributes,
                self_closing,
            } if name == "a" => {
                let mut style = current_style(&active_tags);
                if let Some(link) = html_subset::link(&attributes, &mut style) {
                    if !self_closing {
                        active_tags.push(ActiveTag {
                            name,
                            style,
                            link: Some(link),
                        });
                    }
                }
            }
            HtmlToken::Open {
                name,
                attributes,
                self_closing,
            } if html_subset::is_style_tag(&name) => {
                let mut style = current_style(&active_tags);
                if html_subset::apply_style_tag(&name, &attributes, &mut style) && !self_closing {
                    let link = current_link(&active_tags);
                    active_tags.push(ActiveTag { name, style, link });
                }
            }
            HtmlToken::Close { name } if html_subset::is_style_tag(&name) || name == "a" => {
                active_tags.close(&name);
            }
            HtmlToken::Open { .. } | HtmlToken::Close { .. } | HtmlToken::Ignored => {}
        }
        index += token_len;
        text_start = index;
    }
    append_html_text(
        &mut result,
        &markup[text_start..],
        current_style(&active_tags),
        current_link(&active_tags),
    );
    result.runs = align_runs_to_graphemes(&result.text, &result.runs);
    result.finish()
}

fn append_html_text(
    result: &mut RichParseBuilder,
    text: &str,
    style: StyleOverride,
    link: Option<LinkRef>,
) {
    append_text_with_metadata(
        result,
        html_subset::decode_entities(text).as_ref(),
        style,
        None,
        link,
    );
}

#[derive(Clone, Debug)]
struct ActiveTag {
    name: String,
    style: StyleOverride,
    link: Option<LinkRef>,
}

const ACTIVE_TAG_INDEX_THRESHOLD: usize = 32;

#[derive(Default)]
struct ActiveTagStack {
    tags: Vec<ActiveTag>,
    positions: Option<HashMap<String, Vec<usize>>>,
}

impl ActiveTagStack {
    fn push(&mut self, tag: ActiveTag) {
        let position = self.tags.len();
        if let Some(positions) = self.positions.as_mut() {
            positions
                .entry(tag.name.clone())
                .or_default()
                .push(position);
        }
        self.tags.push(tag);
        if self.positions.is_none() && self.tags.len() > ACTIVE_TAG_INDEX_THRESHOLD {
            self.rebuild_positions();
        }
    }

    fn close(&mut self, name: &str) {
        let position = if let Some(positions) = self.positions.as_ref() {
            positions
                .get(name)
                .and_then(|positions| positions.last())
                .copied()
        } else {
            self.tags.iter().rposition(|active| active.name == name)
        };
        let Some(position) = position else {
            return;
        };

        while self.tags.len() > position {
            let removed_position = self.tags.len() - 1;
            let removed = self.tags.pop().expect("active tag length checked");
            let Some(positions) = self.positions.as_mut() else {
                continue;
            };
            let remove_name = {
                let name_positions = positions
                    .get_mut(&removed.name)
                    .expect("indexed active tag must have a position");
                debug_assert_eq!(name_positions.pop(), Some(removed_position));
                name_positions.is_empty()
            };
            if remove_name {
                positions.remove(&removed.name);
            }
        }
    }

    fn last(&self) -> Option<&ActiveTag> {
        self.tags.last()
    }

    fn rebuild_positions(&mut self) {
        let mut positions: HashMap<String, Vec<usize>> = HashMap::new();
        for (position, active) in self.tags.iter().enumerate() {
            positions
                .entry(active.name.clone())
                .or_default()
                .push(position);
        }
        self.positions = Some(positions);
    }
}

fn current_style(active_tags: &ActiveTagStack) -> StyleOverride {
    active_tags
        .last()
        .map(|active| &active.style)
        .cloned()
        .unwrap_or_default()
}

fn current_link(active_tags: &ActiveTagStack) -> Option<LinkRef> {
    active_tags.last().and_then(|active| active.link.clone())
}

fn plain(text: &str) -> RichParseResult {
    let mut result = RichParseBuilder {
        text: text.to_string(),
        ..RichParseBuilder::default()
    };
    if !text.is_empty() {
        result
            .runs
            .push(styled_run(0, text.len(), StyleOverride::default()));
    }
    result.finish()
}

fn parse_bbcode(
    markup: &str,
    decorators: &DecoratorRegistry,
    emoji_shortcodes: &EmojiShortcodeRegistry,
) -> RichParseResult {
    let mut result = RichParseBuilder::default();
    let mut active_tags = ActiveTagStack::default();
    let mut active_paragraphs: Vec<ActiveParagraph> = Vec::new();
    let mut block_state = BbCodeBlockState::default();
    let mut table_state = BbCodeTableState::default();
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
        let Some((token_len, token)) = token_at(remaining) else {
            index += next_char_len(remaining);
            continue;
        };

        append_bbcode_text(
            &mut result,
            &markup[text_start..index],
            current_style(&active_tags),
            current_link(&active_tags),
            emoji_shortcodes,
            &mut pending_block_break,
        );
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
                    table_state.open_table(
                        value.as_deref(),
                        u32::try_from(result.text.len()).unwrap_or(u32::MAX),
                    );
                } else if name == "cell" {
                    table_state.open_cell(
                        value.as_deref(),
                        &attributes,
                        u32::try_from(result.text.len()).unwrap_or(u32::MAX),
                    );
                } else if let Some(block) = block_state.open(&name, value.as_deref(), &attributes) {
                    ensure_block_boundary(
                        &mut result,
                        &mut pending_block_break,
                        style.clone(),
                        current_link(&active_tags),
                    );
                    if let BlockOpen::Paragraph {
                        name,
                        mut paragraph,
                        prefix,
                    } = block
                    {
                        let start = u32::try_from(result.text.len()).unwrap_or(u32::MAX);
                        if let Some(prefix) = prefix {
                            append_text_with_metadata(
                                &mut result,
                                &prefix,
                                style,
                                None,
                                current_link(&active_tags),
                            );
                            paragraph.list_prefix =
                                Some((start, u32::try_from(result.text.len()).unwrap_or(u32::MAX)));
                        }
                        active_paragraphs.push(ActiveParagraph {
                            name,
                            start,
                            paragraph: Some(paragraph),
                        });
                    }
                } else if let Some(literal) = literal_tag_text(&name) {
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
                        value.as_deref().and_then(html_subset::bbcode_inline_image)
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
                    if let Some(link) = value
                        .as_deref()
                        .and_then(|href| html_subset::bbcode_link(href, &mut style))
                    {
                        active_tags.push(ActiveTag {
                            name,
                            style,
                            link: Some(link),
                        });
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
                    active_paragraphs.push(ActiveParagraph {
                        name,
                        start: u32::try_from(result.text.len()).unwrap_or(u32::MAX),
                        paragraph,
                    });
                } else {
                    let mut decoration = RichTextDecoration {
                        style,
                        inline: None,
                        link: current_link(&active_tags),
                    };
                    if decorators.apply(&name, value.as_deref(), &mut decoration) {
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
                            });
                        }
                    }
                }
            }
            BbCodeToken::Close { name } => {
                if name == "cell" {
                    table_state.close_cell(u32::try_from(result.text.len()).unwrap_or(u32::MAX));
                } else if name == "table" {
                    if let Some(table) = table_state
                        .close_table(u32::try_from(result.text.len()).unwrap_or(u32::MAX))
                    {
                        result.tables.push(table);
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
        current_style(&active_tags),
        current_link(&active_tags),
        emoji_shortcodes,
        &mut pending_block_break,
    );
    result.runs = align_runs_to_graphemes(&result.text, &result.runs);
    close_open_paragraph_overrides(&mut result, active_paragraphs);
    result
        .tables
        .extend(table_state.finish(u32::try_from(result.text.len()).unwrap_or(u32::MAX)));
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
    style: StyleOverride,
    link: Option<LinkRef>,
    emoji_shortcodes: &EmojiShortcodeRegistry,
    pending_block_break: &mut bool,
) {
    let text = emoji_shortcodes.expand(text);
    if text.is_empty() {
        return;
    }
    ensure_pending_block_break(result, pending_block_break, style.clone(), link.clone());
    append_text_with_metadata(result, text.as_ref(), style, None, link);
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
    let end = u32::try_from(result.text.len()).unwrap_or(u32::MAX);
    for active in active_paragraphs.drain(position..) {
        if let Some(paragraph) = active.paragraph {
            result.paragraphs.push(((active.start, end), paragraph));
        }
    }
}

fn close_open_paragraph_overrides(
    result: &mut RichParseBuilder,
    active_paragraphs: Vec<ActiveParagraph>,
) {
    let end = u32::try_from(result.text.len()).unwrap_or(u32::MAX);
    for active in active_paragraphs {
        if let Some(paragraph) = active.paragraph {
            result.paragraphs.push(((active.start, end), paragraph));
        }
    }
}

fn parse_markdown(markup: &str) -> RichParseResult {
    let mut result = RichParseBuilder::default();
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

fn append_text(result: &mut RichParseBuilder, text: &str, style: StyleOverride) {
    append_text_with_metadata(result, text, style, None, None);
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
    let start = result.text.len();
    result.text.push_str(text);
    let mut run = styled_run(start, result.text.len(), style);
    run.inline = inline;
    run.link = link;
    push_or_merge_run(&mut result.runs, run);
}

fn append_inline_object(
    result: &mut RichParseBuilder,
    style: StyleOverride,
    link: Option<LinkRef>,
    inline: InlineObjectRef,
) {
    append_text_with_metadata(result, "\u{fffc}", style, Some(inline), link);
}

fn align_runs_to_graphemes(text: &str, runs: &[StyledRun]) -> Vec<StyledRun> {
    let mut aligned = Vec::new();
    let mut run_index = 0;
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        while run_index < runs.len()
            && usize::try_from(runs[run_index].byte_range.1).unwrap_or(0) <= start
        {
            run_index += 1;
        }
        let source = runs
            .get(run_index)
            .filter(|run| range_contains(run.byte_range, start))
            .cloned()
            .unwrap_or_default();
        let mut run = styled_run(start, end, source.style);
        run.inline = source.inline;
        run.link = source.link;
        push_or_merge_run(&mut aligned, run);
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

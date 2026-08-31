//! Tokenizer and projection helpers for the deliberately bounded HTML V1 subset.

use std::{borrow::Cow, sync::Arc};

use zircon_runtime_interface::resource::ResourceId;
use zircon_runtime_interface::ui::text::UiRichLinkTarget;

use crate::core::math::{Vec2, Vec4};
use crate::text::{FontFamilyName, InlineBaseline, InlineObjectRef, LinkRef, StyleOverride};

use super::admission::{RichTextParseError, RichTokenizerBudget};
use super::bbcode::{attribute_value as bbcode_attribute_value, parse_hex_color};
use super::resource_admission::controlled_resource_locator;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HtmlAttribute {
    pub(super) name: String,
    pub(super) value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HtmlToken {
    Open {
        name: String,
        attributes: Vec<HtmlAttribute>,
        self_closing: bool,
        issues: HtmlTokenIssues,
    },
    Close {
        name: String,
    },
    Malformed {
        issues: HtmlTokenIssues,
    },
    Ignored,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HtmlTokenIssues {
    pub(super) unsupported_attribute: bool,
    pub(super) malformed_attribute: bool,
    pub(super) malformed_tag: bool,
    pub(super) unterminated_quoted_attribute: bool,
}

impl HtmlToken {
    pub(super) fn issues(&self) -> HtmlTokenIssues {
        match self {
            Self::Open { issues, .. } => *issues,
            Self::Malformed { issues } => *issues,
            Self::Close { .. } | Self::Ignored => HtmlTokenIssues::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HtmlEntityIssues {
    pub(super) malformed_entity: bool,
    pub(super) unrecognized_entity: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HtmlAttributeApplicationIssues {
    pub(super) invalid_attribute_value: bool,
    pub(super) unsupported_style_property: bool,
}

pub(super) fn token_at(
    input: &str,
    tokenizer_budget: RichTokenizerBudget,
) -> Result<Option<(usize, HtmlToken)>, RichTextParseError> {
    if !input.starts_with('<') {
        return Ok(None);
    }
    let Some(close) = tag_close_offset(input) else {
        return Ok(None);
    };
    let (close, unterminated_quoted_attribute) = match close {
        TagCloseOffset::Closed(close) => (close, false),
        TagCloseOffset::UnterminatedQuote(close) => (close, true),
    };
    tokenizer_budget.admit_token_bytes(close.checked_add(1).unwrap_or(usize::MAX))?;
    if unterminated_quoted_attribute {
        return Ok(Some((
            close + 1,
            HtmlToken::Malformed {
                issues: HtmlTokenIssues {
                    unterminated_quoted_attribute: true,
                    ..HtmlTokenIssues::default()
                },
            },
        )));
    }
    let mut body = input[1..close].trim();
    if body.is_empty() {
        return Ok(Some((close + 1, malformed_tag())));
    }
    if body.starts_with('!') || body.starts_with('?') {
        return Ok(Some((close + 1, HtmlToken::Ignored)));
    }
    if let Some(name) = body.strip_prefix('/') {
        let Some(name) = normalized_tag(name.trim()) else {
            return Ok(Some((close + 1, malformed_tag())));
        };
        return Ok(Some((close + 1, HtmlToken::Close { name })));
    }

    let self_closing = body.ends_with('/');
    if self_closing {
        body = body[..body.len() - 1].trim_end();
    }
    let name_end = body.find(char::is_whitespace).unwrap_or(body.len());
    let Some(name) = normalized_tag(&body[..name_end]) else {
        return Ok(Some((close + 1, malformed_tag())));
    };
    let (attributes, issues) = parse_attributes(&body[name_end..], &name, tokenizer_budget)?;
    Ok(Some((
        close + 1,
        HtmlToken::Open {
            name,
            attributes,
            self_closing,
            issues,
        },
    )))
}

pub(super) fn looks_like_tag_candidate(input: &str) -> bool {
    let Some(after_open) = input.strip_prefix('<') else {
        return false;
    };
    let mut characters = after_open.chars();
    match characters.next() {
        Some(character) if character.is_ascii_alphabetic() || matches!(character, '!' | '?') => {
            true
        }
        Some('/') => characters
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic()),
        _ => false,
    }
}

pub(super) fn has_unterminated_attribute_quote(input: &str) -> bool {
    let mut quote = None;
    for character in input.chars().skip(1) {
        match (quote, character) {
            (Some(active), current) if active == current => quote = None,
            (None, '\'' | '"') => quote = Some(character),
            _ => {}
        }
    }
    quote.is_some()
}

fn malformed_tag() -> HtmlToken {
    HtmlToken::Malformed {
        issues: HtmlTokenIssues {
            malformed_tag: true,
            ..HtmlTokenIssues::default()
        },
    }
}

pub(super) fn is_style_tag(tag: &str) -> bool {
    matches!(tag, "b" | "i" | "u" | "s" | "span" | "font")
}

pub(super) fn link(
    attributes: &[HtmlAttribute],
    style: &mut StyleOverride,
    issues: &mut HtmlAttributeApplicationIssues,
) -> Option<LinkRef> {
    let Some(href) = attribute(attributes, "href") else {
        issues.invalid_attribute_value = true;
        return None;
    };
    let Ok(target) = UiRichLinkTarget::parse(href) else {
        issues.invalid_attribute_value = true;
        return None;
    };
    apply_link_style(style);
    Some(LinkRef {
        target,
        tooltip: attribute(attributes, "title").map(Arc::from),
    })
}

pub(super) fn inline_image(
    attributes: &[HtmlAttribute],
    issues: &mut HtmlAttributeApplicationIssues,
) -> Option<InlineObjectRef> {
    let Some(source) = attribute(attributes, "src") else {
        issues.invalid_attribute_value = true;
        return None;
    };
    let Some(source) = controlled_resource_locator(source) else {
        issues.invalid_attribute_value = true;
        return None;
    };
    let width = optional_attribute_value(
        attributes,
        "width",
        parse_positive_size,
        DEFAULT_INLINE_IMAGE_SIZE_PX,
        issues,
    );
    let height = optional_attribute_value(attributes, "height", parse_positive_size, width, issues);
    let baseline = optional_attribute_value(
        attributes,
        "baseline",
        parse_inline_baseline,
        InlineBaseline::default(),
        issues,
    );
    Some(InlineObjectRef::Image {
        texture: ResourceId::from_locator(&source),
        size: Vec2::new(width, height),
        baseline,
        alternative_text: attribute(attributes, "alt").map(str::to_owned),
        tooltip: attribute(attributes, "title").map(str::to_owned),
    })
}

pub(super) fn bbcode_inline_image(
    value: Option<&str>,
    attributes: &[(String, String)],
) -> Option<InlineObjectRef> {
    let source = value.or_else(|| bbcode_attribute_value(attributes, "src"))?;
    let source = controlled_resource_locator(source)?;
    Some(InlineObjectRef::Image {
        texture: ResourceId::from_locator(&source),
        size: Vec2::new(DEFAULT_INLINE_IMAGE_SIZE_PX, DEFAULT_INLINE_IMAGE_SIZE_PX),
        baseline: InlineBaseline::Baseline,
        alternative_text: bbcode_attribute_value(attributes, "alt").map(str::to_owned),
        tooltip: bbcode_attribute_value(attributes, "title").map(str::to_owned),
    })
}

pub(super) fn bbcode_link(
    value: Option<&str>,
    attributes: &[(String, String)],
    style: &mut StyleOverride,
) -> Option<LinkRef> {
    let href = value.or_else(|| bbcode_attribute_value(attributes, "href"))?;
    let target = UiRichLinkTarget::parse(href).ok()?;
    apply_link_style(style);
    Some(LinkRef {
        target,
        tooltip: bbcode_attribute_value(attributes, "title").map(Arc::from),
    })
}

const DEFAULT_INLINE_IMAGE_SIZE_PX: f32 = 16.0;
const DEFAULT_LINK_COLOR: [f32; 4] = [0.2, 0.45, 0.95, 1.0];

fn apply_link_style(style: &mut StyleOverride) {
    style.underline = Some(true);
    style.color = Some(Vec4::new(
        DEFAULT_LINK_COLOR[0],
        DEFAULT_LINK_COLOR[1],
        DEFAULT_LINK_COLOR[2],
        DEFAULT_LINK_COLOR[3],
    ));
}

fn parse_inline_baseline(value: &str) -> Option<InlineBaseline> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("baseline") {
        Some(InlineBaseline::Baseline)
    } else if value.eq_ignore_ascii_case("center") {
        Some(InlineBaseline::Center)
    } else if value.eq_ignore_ascii_case("top") {
        Some(InlineBaseline::Top)
    } else if value.eq_ignore_ascii_case("bottom") {
        Some(InlineBaseline::Bottom)
    } else {
        None
    }
}

pub(super) fn apply_style_tag(
    tag: &str,
    attributes: &[HtmlAttribute],
    style: &mut StyleOverride,
    issues: &mut HtmlAttributeApplicationIssues,
) -> bool {
    match tag {
        "b" => style.weight = Some(700),
        "i" => style.italic = Some(true),
        "u" => style.underline = Some(true),
        "s" => style.strike = Some(true),
        "span" => apply_span_style(attributes, style, issues),
        "font" => apply_font_attributes(attributes, style, issues),
        _ => return false,
    }
    true
}

pub(super) fn decode_entities(input: &str) -> Cow<'_, str> {
    decode_entities_with_issues(input).0
}

pub(super) fn decode_entities_with_issues(input: &str) -> (Cow<'_, str>, HtmlEntityIssues) {
    decode_entities_with_issues_observing(input, |_, _, _| {})
}

pub(super) fn decode_entities_with_issues_observing(
    input: &str,
    mut observe_decoded_fragment: impl FnMut((usize, usize), &str, bool),
) -> (Cow<'_, str>, HtmlEntityIssues) {
    if !input.contains('&') {
        observe_decoded_fragment((0, input.len()), input, false);
        return (Cow::Borrowed(input), HtmlEntityIssues::default());
    }
    let mut decoded = String::with_capacity(input.len());
    let mut issues = HtmlEntityIssues::default();
    let mut remaining = input;
    let mut source_offset = 0;
    while let Some(offset) = remaining.find('&') {
        let source_fragment = &remaining[..offset];
        decoded.push_str(source_fragment);
        observe_decoded_fragment(
            (source_offset, source_offset + offset),
            source_fragment,
            false,
        );
        remaining = &remaining[offset..];
        source_offset += offset;
        let Some(end) = remaining
            .find(';')
            .filter(|end| *end <= MAX_ENTITY_BODY_LEN)
        else {
            decoded.push('&');
            observe_decoded_fragment((source_offset, source_offset + 1), "&", false);
            remaining = &remaining[1..];
            source_offset += 1;
            issues.malformed_entity = true;
            continue;
        };
        let entity = &remaining[1..end];
        if let Some(character) = decode_entity(entity) {
            decoded.push(character);
            let mut encoded = [0; 4];
            observe_decoded_fragment(
                (source_offset, source_offset + end + 1),
                character.encode_utf8(&mut encoded),
                true,
            );
            remaining = &remaining[end + 1..];
        } else {
            let source_fragment = &remaining[..=end];
            decoded.push_str(source_fragment);
            observe_decoded_fragment(
                (source_offset, source_offset + end + 1),
                source_fragment,
                false,
            );
            remaining = &remaining[end + 1..];
            if entity.is_empty()
                || entity.starts_with('#')
                || !entity
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
            {
                issues.malformed_entity = true;
            } else {
                issues.unrecognized_entity = true;
            }
        }
        source_offset += end + 1;
    }
    decoded.push_str(remaining);
    observe_decoded_fragment(
        (source_offset, source_offset + remaining.len()),
        remaining,
        false,
    );
    (Cow::Owned(decoded), issues)
}

const MAX_ENTITY_BODY_LEN: usize = 16;

enum TagCloseOffset {
    Closed(usize),
    UnterminatedQuote(usize),
}

fn tag_close_offset(input: &str) -> Option<TagCloseOffset> {
    let mut quote = None;
    let mut first_quoted_close = None;
    for (offset, character) in input.char_indices().skip(1) {
        match (quote, character) {
            (Some(active), current) if active == current => quote = None,
            (None, '\'' | '"') => quote = Some(character),
            (None, '>') => return Some(TagCloseOffset::Closed(offset)),
            (Some(_), '>') => {
                first_quoted_close.get_or_insert(offset);
            }
            _ => {}
        }
    }
    quote
        .and(first_quoted_close)
        .map(TagCloseOffset::UnterminatedQuote)
}

fn normalized_tag(tag: &str) -> Option<String> {
    let tag = tag.trim().to_ascii_lowercase();
    (!tag.is_empty()
        && tag
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-'))
    .then_some(tag)
}

fn parse_attributes(
    mut input: &str,
    tag: &str,
    tokenizer_budget: RichTokenizerBudget,
) -> Result<(Vec<HtmlAttribute>, HtmlTokenIssues), RichTextParseError> {
    let mut attributes = Vec::new();
    let mut attribute_bytes = 0;
    let mut issues = HtmlTokenIssues::default();
    let reports_attribute_issues = is_supported_tag(tag);
    while !input.trim_start().is_empty() {
        input = input.trim_start();
        let name_end = input
            .find(|character: char| character.is_whitespace() || character == '=')
            .unwrap_or(input.len());
        if name_end == 0 {
            issues.malformed_attribute |= reports_attribute_issues;
            input = &input[input.chars().next().map(char::len_utf8).unwrap_or(1)..];
            continue;
        }
        let name = &input[..name_end];
        input = &input[name_end..];
        input = input.trim_start();
        let mut value = "";
        if let Some(after_equals) = input.strip_prefix('=') {
            input = after_equals.trim_start();
            let (parsed, remaining) = parse_attribute_value(input);
            value = parsed;
            input = remaining;
        }
        if name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        {
            attribute_bytes = tokenizer_budget.admit_attribute(
                attributes.len(),
                attribute_bytes,
                name.len(),
                value.len(),
            )?;
            attributes.push(HtmlAttribute {
                name: name.to_ascii_lowercase(),
                value: value.to_string(),
            });
            issues.unsupported_attribute |=
                reports_attribute_issues && !is_supported_attribute(tag, name);
        } else {
            issues.malformed_attribute |= reports_attribute_issues;
        }
    }
    Ok((attributes, issues))
}

fn is_supported_tag(tag: &str) -> bool {
    is_style_tag(tag) || matches!(tag, "br" | "a" | "img")
}

fn is_supported_attribute(tag: &str, attribute: &str) -> bool {
    match tag {
        "span" => attribute.eq_ignore_ascii_case("style"),
        "font" => matches_ascii_case(attribute, &["color", "size", "face"]),
        "a" => matches_ascii_case(attribute, &["href", "title"]),
        "img" => matches_ascii_case(
            attribute,
            &["src", "width", "height", "baseline", "alt", "title"],
        ),
        "b" | "i" | "u" | "s" | "br" => false,
        _ => false,
    }
}

fn matches_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn parse_attribute_value(input: &str) -> (&str, &str) {
    let Some(first) = input.chars().next() else {
        return ("", "");
    };
    if matches!(first, '\'' | '"') {
        let start = first.len_utf8();
        let quoted = &input[start..];
        if let Some(end) = quoted.find(first) {
            return (&quoted[..end], &quoted[end + first.len_utf8()..]);
        }
        return (quoted, "");
    }
    let end = input.find(char::is_whitespace).unwrap_or(input.len());
    (&input[..end], &input[end..])
}

fn attribute<'a>(attributes: &'a [HtmlAttribute], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|attribute| attribute.name == name)
        .map(|attribute| attribute.value.as_str())
}

fn apply_span_style(
    attributes: &[HtmlAttribute],
    style: &mut StyleOverride,
    issues: &mut HtmlAttributeApplicationIssues,
) {
    let Some(declarations) = attribute(attributes, "style") else {
        return;
    };
    for declaration in declarations.split(';') {
        if declaration.trim().is_empty() {
            continue;
        }
        let Some((property, value)) = declaration.split_once(':') else {
            issues.invalid_attribute_value = true;
            continue;
        };
        let property = property.trim().to_ascii_lowercase();
        let value = value.trim();
        match property.as_str() {
            "color" => {
                if let Some(color) = parse_hex_color(value) {
                    style.color = Some(color);
                } else {
                    issues.invalid_attribute_value = true;
                }
            }
            "font-size" => {
                if let Some(size) = parse_positive_size(value) {
                    style.font_size = Some(size);
                } else {
                    issues.invalid_attribute_value = true;
                }
            }
            "font-weight" => {
                if let Some(weight) = parse_font_weight(value) {
                    style.weight = Some(weight);
                } else {
                    issues.invalid_attribute_value = true;
                }
            }
            "font-style" if value.eq_ignore_ascii_case("italic") => style.italic = Some(true),
            "font-style" if value.eq_ignore_ascii_case("normal") => style.italic = Some(false),
            "font-style" => issues.invalid_attribute_value = true,
            "text-decoration" if !apply_text_decoration(value, style) => {
                issues.invalid_attribute_value = true;
            }
            "text-decoration" => {}
            _ => issues.unsupported_style_property = true,
        }
    }
}

fn apply_font_attributes(
    attributes: &[HtmlAttribute],
    style: &mut StyleOverride,
    issues: &mut HtmlAttributeApplicationIssues,
) {
    if let Some(value) = attribute(attributes, "color") {
        if let Some(color) = parse_hex_color(value) {
            style.color = Some(color);
        } else {
            issues.invalid_attribute_value = true;
        }
    }
    if let Some(value) = attribute(attributes, "size") {
        if let Some(size) = parse_positive_size(value) {
            style.font_size = Some(size);
        } else {
            issues.invalid_attribute_value = true;
        }
    }
    if let Some(value) = attribute(attributes, "face") {
        let family = value.trim();
        if family.is_empty() {
            issues.invalid_attribute_value = true;
        } else {
            style.family = Some(FontFamilyName::from(family));
        }
    }
}

fn optional_attribute_value<T: Copy>(
    attributes: &[HtmlAttribute],
    name: &str,
    parse: impl FnOnce(&str) -> Option<T>,
    default: T,
    issues: &mut HtmlAttributeApplicationIssues,
) -> T {
    let Some(value) = attribute(attributes, name) else {
        return default;
    };
    match parse(value) {
        Some(value) => value,
        None => {
            issues.invalid_attribute_value = true;
            default
        }
    }
}

fn parse_positive_size(value: &str) -> Option<f32> {
    let value = value
        .trim()
        .strip_suffix("px")
        .unwrap_or(value.trim())
        .trim();
    value
        .parse::<f32>()
        .ok()
        .filter(|size| size.is_finite() && *size > 0.0)
}

fn parse_font_weight(value: &str) -> Option<u16> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        Some(400)
    } else if value.eq_ignore_ascii_case("bold") {
        Some(700)
    } else {
        value
            .parse::<u16>()
            .ok()
            .filter(|weight| (1..=1000).contains(weight))
    }
}

fn apply_text_decoration(value: &str, style: &mut StyleOverride) -> bool {
    let mut any = false;
    for token in value.split_ascii_whitespace() {
        if token.eq_ignore_ascii_case("underline") {
            style.underline = Some(true);
            any = true;
        } else if token.eq_ignore_ascii_case("line-through") {
            style.strike = Some(true);
            any = true;
        } else if token.eq_ignore_ascii_case("none") {
            style.underline = Some(false);
            style.strike = Some(false);
            any = true;
        }
    }
    any
}

#[cfg(test)]
#[path = "html_subset/allocation_free_keyword_tests.rs"]
mod allocation_free_keyword_tests;

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        entity if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        entity if entity.starts_with('#') => {
            entity[1..].parse::<u32>().ok().and_then(char::from_u32)
        }
        _ => None,
    }
}

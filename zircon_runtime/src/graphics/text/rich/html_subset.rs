//! Tokenizer and projection helpers for the deliberately bounded HTML V1 subset.

use zircon_runtime_interface::resource::{ResourceId, ResourceLocator, ResourceScheme};

use crate::core::framework::render::{
    FontFamilyName, InlineBaseline, InlineObjectRef, LinkRef, StyleOverride,
};
use crate::core::math::{Vec2, Vec4};

use super::bbcode::parse_hex_color;

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
    },
    Close {
        name: String,
    },
    Ignored,
}

pub(super) fn token_at(input: &str) -> Option<(usize, HtmlToken)> {
    if !input.starts_with('<') {
        return None;
    }
    let close = tag_close_offset(input)?;
    let mut body = input[1..close].trim();
    if body.is_empty() {
        return None;
    }
    if body.starts_with('!') || body.starts_with('?') {
        return Some((close + 1, HtmlToken::Ignored));
    }
    if let Some(name) = body.strip_prefix('/') {
        return Some((
            close + 1,
            HtmlToken::Close {
                name: normalized_tag(name.trim())?,
            },
        ));
    }

    let self_closing = body.ends_with('/');
    if self_closing {
        body = body[..body.len() - 1].trim_end();
    }
    let name_end = body.find(char::is_whitespace).unwrap_or(body.len());
    let name = normalized_tag(&body[..name_end])?;
    let attributes = parse_attributes(&body[name_end..]);
    Some((
        close + 1,
        HtmlToken::Open {
            name,
            attributes,
            self_closing,
        },
    ))
}

pub(super) fn is_style_tag(tag: &str) -> bool {
    matches!(tag, "b" | "i" | "u" | "s" | "span" | "font")
}

pub(super) fn link(attributes: &[HtmlAttribute], style: &mut StyleOverride) -> Option<LinkRef> {
    let href = attribute(attributes, "href")?;
    let href = controlled_locator(href)?.to_string();
    apply_link_style(style);
    Some(LinkRef { href })
}

pub(super) fn inline_image(attributes: &[HtmlAttribute]) -> Option<InlineObjectRef> {
    let source = controlled_locator(attribute(attributes, "src")?)?;
    let width = attribute(attributes, "width")
        .and_then(parse_positive_size)
        .unwrap_or(DEFAULT_INLINE_IMAGE_SIZE_PX);
    let height = attribute(attributes, "height")
        .and_then(parse_positive_size)
        .unwrap_or(width);
    let baseline = attribute(attributes, "baseline")
        .and_then(parse_inline_baseline)
        .unwrap_or_default();
    Some(InlineObjectRef::Image {
        texture: ResourceId::from_locator(&source),
        size: Vec2::new(width, height),
        baseline,
    })
}

pub(super) fn bbcode_inline_image(source: &str) -> Option<InlineObjectRef> {
    let source = controlled_locator(source)?;
    Some(InlineObjectRef::Image {
        texture: ResourceId::from_locator(&source),
        size: Vec2::new(DEFAULT_INLINE_IMAGE_SIZE_PX, DEFAULT_INLINE_IMAGE_SIZE_PX),
        baseline: InlineBaseline::Baseline,
    })
}

pub(super) fn bbcode_link(href: &str, style: &mut StyleOverride) -> Option<LinkRef> {
    let href = controlled_locator(href)?.to_string();
    apply_link_style(style);
    Some(LinkRef { href })
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

fn controlled_locator(value: &str) -> Option<ResourceLocator> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let locator = if value.contains("://") {
        ResourceLocator::parse(value).ok()?
    } else {
        ResourceLocator::new(ResourceScheme::Res, value, None).ok()?
    };
    matches!(
        locator.scheme(),
        ResourceScheme::Res
            | ResourceScheme::Library
            | ResourceScheme::Package
            | ResourceScheme::Builtin
    )
    .then_some(locator)
}

fn parse_inline_baseline(value: &str) -> Option<InlineBaseline> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Some(InlineBaseline::Baseline),
        "center" => Some(InlineBaseline::Center),
        "top" => Some(InlineBaseline::Top),
        "bottom" => Some(InlineBaseline::Bottom),
        _ => None,
    }
}

pub(super) fn apply_style_tag(
    tag: &str,
    attributes: &[HtmlAttribute],
    style: &mut StyleOverride,
) -> bool {
    match tag {
        "b" => style.weight = Some(700),
        "i" => style.italic = Some(true),
        "u" => style.underline = Some(true),
        "s" => style.strike = Some(true),
        "span" => apply_span_style(attributes, style),
        "font" => apply_font_attributes(attributes, style),
        _ => return false,
    }
    true
}

pub(super) fn decode_entities(input: &str) -> String {
    let mut decoded = String::with_capacity(input.len());
    let mut remaining = input;
    while let Some(offset) = remaining.find('&') {
        decoded.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
        let Some(end) = remaining
            .find(';')
            .filter(|end| *end <= MAX_ENTITY_BODY_LEN)
        else {
            decoded.push('&');
            remaining = &remaining[1..];
            continue;
        };
        let entity = &remaining[1..end];
        if let Some(character) = decode_entity(entity) {
            decoded.push(character);
            remaining = &remaining[end + 1..];
        } else {
            decoded.push_str(&remaining[..=end]);
            remaining = &remaining[end + 1..];
        }
    }
    decoded.push_str(remaining);
    decoded
}

const MAX_ENTITY_BODY_LEN: usize = 16;

fn tag_close_offset(input: &str) -> Option<usize> {
    let mut quote = None;
    for (offset, character) in input.char_indices().skip(1) {
        match (quote, character) {
            (Some(active), current) if active == current => quote = None,
            (None, '\'' | '"') => quote = Some(character),
            (None, '>') => return Some(offset),
            _ => {}
        }
    }
    None
}

fn normalized_tag(tag: &str) -> Option<String> {
    let tag = tag.trim().to_ascii_lowercase();
    (!tag.is_empty()
        && tag
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-'))
    .then_some(tag)
}

fn parse_attributes(mut input: &str) -> Vec<HtmlAttribute> {
    let mut attributes = Vec::new();
    while !input.trim_start().is_empty() {
        input = input.trim_start();
        let name_end = input
            .find(|character: char| character.is_whitespace() || character == '=')
            .unwrap_or(input.len());
        if name_end == 0 {
            input = &input[input.chars().next().map(char::len_utf8).unwrap_or(1)..];
            continue;
        }
        let name = input[..name_end].to_ascii_lowercase();
        input = &input[name_end..];
        input = input.trim_start();
        let mut value = String::new();
        if let Some(after_equals) = input.strip_prefix('=') {
            input = after_equals.trim_start();
            let (parsed, remaining) = parse_attribute_value(input);
            value = parsed.to_string();
            input = remaining;
        }
        if name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        {
            attributes.push(HtmlAttribute { name, value });
        }
    }
    attributes
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

fn apply_span_style(attributes: &[HtmlAttribute], style: &mut StyleOverride) {
    let Some(declarations) = attribute(attributes, "style") else {
        return;
    };
    for declaration in declarations.split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        let property = property.trim().to_ascii_lowercase();
        let value = value.trim();
        match property.as_str() {
            "color" => {
                if let Some(color) = parse_hex_color(value) {
                    style.color = Some(color);
                }
            }
            "font-size" => {
                if let Some(size) = parse_positive_size(value) {
                    style.font_size = Some(size);
                }
            }
            "font-weight" => {
                if let Some(weight) = parse_font_weight(value) {
                    style.weight = Some(weight);
                }
            }
            "font-style" if value.eq_ignore_ascii_case("italic") => style.italic = Some(true),
            "font-style" if value.eq_ignore_ascii_case("normal") => style.italic = Some(false),
            "text-decoration" => apply_text_decoration(value, style),
            _ => {}
        }
    }
}

fn apply_font_attributes(attributes: &[HtmlAttribute], style: &mut StyleOverride) {
    if let Some(color) = attribute(attributes, "color").and_then(parse_hex_color) {
        style.color = Some(color);
    }
    if let Some(size) = attribute(attributes, "size").and_then(parse_positive_size) {
        style.font_size = Some(size);
    }
    if let Some(family) = attribute(attributes, "face")
        .map(str::trim)
        .filter(|family| !family.is_empty())
    {
        style.family = Some(FontFamilyName::from(family));
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
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(400),
        "bold" => Some(700),
        value => value
            .parse::<u16>()
            .ok()
            .filter(|weight| (1..=1000).contains(weight)),
    }
}

fn apply_text_decoration(value: &str, style: &mut StyleOverride) {
    let mut any = false;
    for token in value.split_ascii_whitespace() {
        match token.to_ascii_lowercase().as_str() {
            "underline" => {
                style.underline = Some(true);
                any = true;
            }
            "line-through" => {
                style.strike = Some(true);
                any = true;
            }
            "none" => {
                style.underline = Some(false);
                style.strike = Some(false);
                any = true;
            }
            _ => {}
        }
    }
    if !any {
        return;
    }
}

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

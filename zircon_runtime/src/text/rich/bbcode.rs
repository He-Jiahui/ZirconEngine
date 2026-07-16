use crate::core::math::Vec4;
use crate::text::{FontFamilyName, StyleOverride};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BbCodeToken {
    Open {
        name: String,
        value: Option<String>,
        attributes: Vec<(String, String)>,
    },
    Close {
        name: String,
    },
}

pub(super) fn token_at(input: &str) -> Option<(usize, BbCodeToken)> {
    if !input.starts_with('[') {
        return None;
    }
    let close = input.find(']')?;
    let body = input[1..close].trim();
    if body.is_empty() {
        return None;
    }

    let token = if let Some(name) = body.strip_prefix('/') {
        let name = normalized_tag(name)?;
        BbCodeToken::Close { name }
    } else {
        let (name, value, attributes) = parse_open_body(body)?;
        BbCodeToken::Open {
            name,
            value,
            attributes,
        }
    };
    Some((close + 1, token))
}

pub(super) fn apply_builtin_style(
    tag: &str,
    value: Option<&str>,
    style: &mut StyleOverride,
) -> bool {
    match tag {
        "b" => style.weight = Some(700),
        "i" => style.italic = Some(true),
        "u" => style.underline = Some(true),
        "s" => style.strike = Some(true),
        "code" => style.code = Some(true),
        "color" => {
            let Some(color) = value.and_then(parse_hex_color) else {
                return false;
            };
            style.color = Some(color);
        }
        "bgcolor" => {
            let Some(color) = value.and_then(parse_hex_color) else {
                return false;
            };
            style.bg_color = Some(color);
        }
        "size" => {
            let Some(size) = value
                .and_then(|value| value.parse::<f32>().ok())
                .filter(|size| size.is_finite() && *size > 0.0)
            else {
                return false;
            };
            style.font_size = Some(size);
        }
        "font" => {
            let Some(family) = value.map(str::trim).filter(|value| !value.is_empty()) else {
                return false;
            };
            style.family = Some(FontFamilyName::from(family));
        }
        _ => return false,
    }
    true
}

pub(super) fn literal_tag_text(tag: &str) -> Option<&'static str> {
    match tag {
        "lb" => Some("["),
        "rb" => Some("]"),
        "br" => Some("\n"),
        "lrm" => Some("\u{200e}"),
        "rlm" => Some("\u{200f}"),
        "lre" => Some("\u{202a}"),
        "rle" => Some("\u{202b}"),
        "pdf" => Some("\u{202c}"),
        "lro" => Some("\u{202d}"),
        "rlo" => Some("\u{202e}"),
        "alm" => Some("\u{061c}"),
        "lri" => Some("\u{2066}"),
        "rli" => Some("\u{2067}"),
        "fsi" => Some("\u{2068}"),
        "pdi" => Some("\u{2069}"),
        "zwj" => Some("\u{200d}"),
        "zwnj" => Some("\u{200c}"),
        "wj" => Some("\u{2060}"),
        "shy" => Some("\u{00ad}"),
        _ => None,
    }
}

pub(super) fn is_parser_reserved_tag(tag: &str) -> bool {
    literal_tag_text(tag).is_some()
        || matches!(
            tag,
            "img"
                | "url"
                | "left"
                | "center"
                | "right"
                | "fill"
                | "indent"
                | "p"
                | "ul"
                | "ol"
                | "li"
        )
}

pub(super) fn attribute_value<'a>(
    attributes: &'a [(String, String)],
    name: &str,
) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(attribute, _)| attribute == name)
        .map(|(_, value)| value.as_str())
}

pub(super) fn normalized_tag(tag: &str) -> Option<String> {
    let tag = tag.trim().to_ascii_lowercase();
    (!tag.is_empty()
        && tag
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_'))
    .then_some(tag)
}

fn unquoted(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn parse_open_body(body: &str) -> Option<(String, Option<String>, Vec<(String, String)>)> {
    let name_end = body
        .char_indices()
        .find_map(|(index, character)| {
            (character == '=' || character.is_whitespace()).then_some(index)
        })
        .unwrap_or(body.len());
    let name = normalized_tag(&body[..name_end])?;
    let remainder = body[name_end..].trim_start();
    if let Some(value) = remainder.strip_prefix('=') {
        return Some((name, Some(unquoted(value.trim()).to_string()), Vec::new()));
    }
    Some((name, None, parse_attributes(remainder)))
}

fn parse_attributes(mut input: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    while !input.trim_start().is_empty() {
        input = input.trim_start();
        let key_end = input
            .char_indices()
            .find_map(|(index, character)| {
                (character == '=' || character.is_whitespace()).then_some(index)
            })
            .unwrap_or(input.len());
        let Some(key) = normalized_tag(&input[..key_end]) else {
            break;
        };
        input = input[key_end..].trim_start();
        let Some(rest) = input.strip_prefix('=') else {
            break;
        };
        input = rest.trim_start();
        let (value, rest) = attribute_token(input);
        if value.is_empty() {
            break;
        }
        attributes.push((key, value.to_string()));
        input = rest;
    }
    attributes
}

fn attribute_token(input: &str) -> (&str, &str) {
    let Some(first) = input.chars().next() else {
        return ("", "");
    };
    if first == '"' || first == '\'' {
        let body = &input[first.len_utf8()..];
        return body
            .find(first)
            .map(|end| (&body[..end], &body[end + first.len_utf8()..]))
            .unwrap_or((body, ""));
    }
    let end = input
        .char_indices()
        .find_map(|(index, character)| character.is_whitespace().then_some(index))
        .unwrap_or(input.len());
    (&input[..end], &input[end..])
}

pub(super) fn parse_hex_color(value: &str) -> Option<Vec4> {
    let hex = value.trim().strip_prefix('#')?;
    let bytes = match hex.len() {
        3 | 4 => {
            let mut bytes = [255_u8; 4];
            for (index, digit) in hex.bytes().enumerate() {
                let nibble = hex_nibble(digit)?;
                bytes[index] = nibble * 17;
            }
            bytes
        }
        6 | 8 => {
            let mut bytes = [255_u8; 4];
            for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
                bytes[index] = hex_nibble(pair[0])? * 16 + hex_nibble(pair[1])?;
            }
            bytes
        }
        _ => return None,
    };
    Some(Vec4::new(
        f32::from(bytes[0]) / 255.0,
        f32::from(bytes[1]) / 255.0,
        f32::from(bytes[2]) / 255.0,
        f32::from(bytes[3]) / 255.0,
    ))
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

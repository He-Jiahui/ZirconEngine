use crate::core::framework::render::FontFamilyName;
use crate::core::{framework::render::StyleOverride, math::Vec4};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BbCodeToken {
    Open { name: String, value: Option<String> },
    Close { name: String },
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
        let (name, value) = body
            .split_once('=')
            .map(|(name, value)| (name, Some(unquoted(value.trim()).to_string())))
            .unwrap_or((body, None));
        let name = normalized_tag(name)?;
        BbCodeToken::Open { name, value }
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

fn normalized_tag(tag: &str) -> Option<String> {
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

fn parse_hex_color(value: &str) -> Option<Vec4> {
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

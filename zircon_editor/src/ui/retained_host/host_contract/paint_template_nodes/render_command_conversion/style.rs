use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextAlign},
};

use super::super::super::data::FrameRect;
use super::super::super::paint_theme::PALETTE;

const FALLBACK_TEXT: [u8; 4] = PALETTE.text;

pub(super) fn frame_from_ui(frame: UiFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn runtime_foreground_color(style: &UiResolvedStyle) -> [u8; 4] {
    parse_style_color(style.foreground_color.as_deref()).unwrap_or(FALLBACK_TEXT)
}

pub(super) fn aligned_text_x(frame: &FrameRect, text: &str, style: &UiResolvedStyle) -> f32 {
    let estimated_width = text.chars().count() as f32 * (style.font_size.max(1.0) * 0.5);
    match style.text_align {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - estimated_width).max(0.0) * 0.5,
        UiTextAlign::Right => frame.x + (frame.width - estimated_width).max(0.0),
    }
}

pub(super) fn parse_style_color(value: Option<&str>) -> Option<[u8; 4]> {
    let value = value?.trim();
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        3 => Some([
            parse_nibble(hex.as_bytes()[0])? * 17,
            parse_nibble(hex.as_bytes()[1])? * 17,
            parse_nibble(hex.as_bytes()[2])? * 17,
            255,
        ]),
        4 => Some([
            parse_nibble(hex.as_bytes()[0])? * 17,
            parse_nibble(hex.as_bytes()[1])? * 17,
            parse_nibble(hex.as_bytes()[2])? * 17,
            parse_nibble(hex.as_bytes()[3])? * 17,
        ]),
        6 => Some([
            parse_hex_pair(hex, 0)?,
            parse_hex_pair(hex, 2)?,
            parse_hex_pair(hex, 4)?,
            255,
        ]),
        8 => Some([
            parse_hex_pair(hex, 0)?,
            parse_hex_pair(hex, 2)?,
            parse_hex_pair(hex, 4)?,
            parse_hex_pair(hex, 6)?,
        ]),
        _ => None,
    }
}

fn parse_hex_pair(hex: &str, offset: usize) -> Option<u8> {
    let bytes = hex.as_bytes();
    Some(parse_nibble(bytes[offset])? * 16 + parse_nibble(bytes[offset + 1])?)
}

fn parse_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

/// 可解析的显式文本色覆盖宿主缺省色；颜色缺失或无法解析时回退到当前色板。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn runtime_foreground_color(
    style: &UiResolvedStyle,
) -> [u8; 4] {
    runtime_foreground_color_from_host(style, current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn parse_style_color(
    value: Option<&str>,
) -> Option<[u8; 4]> {
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

fn runtime_foreground_color_from_host(
    style: &UiResolvedStyle,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    parse_style_color(style.foreground_color.as_deref()).unwrap_or(palette.text)
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

#[cfg(test)]
#[path = "tests/colors.rs"]
mod tests;

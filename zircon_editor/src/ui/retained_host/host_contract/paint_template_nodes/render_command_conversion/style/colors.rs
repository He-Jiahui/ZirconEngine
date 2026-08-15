use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn runtime_foreground_fallback_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.text = [10, 11, 12, 255];
        let style = UiResolvedStyle::default();

        assert_eq!(
            runtime_foreground_color_from_host(&style, palette),
            [10, 11, 12, 255]
        );
    }

    #[test]
    fn runtime_foreground_declared_color_overrides_palette() {
        let palette = PALETTE;
        let style = UiResolvedStyle {
            foreground_color: Some("#123456".to_owned()),
            ..UiResolvedStyle::default()
        };

        assert_eq!(
            runtime_foreground_color_from_host(&style, palette),
            [0x12, 0x34, 0x56, 255]
        );
    }
}

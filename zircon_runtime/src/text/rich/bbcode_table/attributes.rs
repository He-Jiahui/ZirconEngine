use crate::text::{
    MAX_RICH_TABLE_ROW_SPAN, RichTableCellBoxStyle, RichTableCellPadding, RichTableColumn,
};

use super::super::bbcode::{attribute_value, parse_hex_color};
use super::placement::CellPlacement;

const DEFAULT_CELL_SPAN: u16 = 1;
const MAX_COLUMN_EXPAND_RATIO: u16 = 1024;
const MAX_CELL_PADDING_PX: f32 = 4096.0;
const CELL_PADDING_COMPONENT_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CellAttributes {
    pub column_span: u16,
    pub row_span: u16,
    pub box_style: RichTableCellBoxStyle,
}

pub(super) fn parse_cell_attributes(
    attributes: &[(String, String)],
    column_count: usize,
) -> CellAttributes {
    CellAttributes {
        column_span: parse_span(
            attribute_value(attributes, "colspan"),
            u16::try_from(column_count.max(1))
                .expect("table construction bounds the compact column count"),
        ),
        row_span: parse_span(
            attribute_value(attributes, "rowspan"),
            MAX_RICH_TABLE_ROW_SPAN,
        ),
        box_style: parse_cell_box_style(attributes),
    }
}

fn parse_cell_box_style(attributes: &[(String, String)]) -> RichTableCellBoxStyle {
    let (odd_row_background, even_row_background) = attribute_value(attributes, "bg")
        .and_then(parse_background_colors)
        .map(|colors| (Some(colors.0), Some(colors.1)))
        .unwrap_or_default();
    RichTableCellBoxStyle {
        padding: attribute_value(attributes, "padding").and_then(parse_padding),
        odd_row_background,
        even_row_background,
        border_color: attribute_value(attributes, "border").and_then(parse_hex_color),
    }
}

fn parse_background_colors(
    value: &str,
) -> Option<(crate::core::math::Vec4, crate::core::math::Vec4)> {
    let mut colors = value.split(',').map(str::trim);
    let odd = parse_hex_color(colors.next()?)?;
    let even = match colors.next() {
        Some(value) => parse_hex_color(value)?,
        None => odd,
    };
    colors.next().is_none().then_some((odd, even))
}

fn parse_padding(value: &str) -> Option<RichTableCellPadding> {
    let mut components = [0.0_f32; CELL_PADDING_COMPONENT_COUNT];
    let mut values = value.split(',').map(str::trim);
    for component in &mut components {
        let value = values.next()?.parse::<f32>().ok()?;
        if !value.is_finite() {
            return None;
        }
        *component = value.clamp(0.0, MAX_CELL_PADDING_PX);
    }
    if values.next().is_some() {
        return None;
    }
    Some(RichTableCellPadding {
        left: components[0],
        top: components[1],
        right: components[2],
        bottom: components[3],
    })
}

pub(super) fn configure_columns(
    columns: &mut [RichTableColumn],
    placement: &CellPlacement,
    value: Option<&str>,
    attributes: &[(String, String)],
) {
    let start = usize::from(placement.column_index).min(columns.len());
    let end = start
        .saturating_add(usize::from(placement.column_span))
        .min(columns.len());
    for column in &mut columns[start..end] {
        configure_column(column, value, attributes);
    }
}

fn configure_column(
    column: &mut RichTableColumn,
    value: Option<&str>,
    attributes: &[(String, String)],
) {
    if let Some(ratio) = attribute_value(attributes, "expand")
        .or(value)
        .and_then(parse_expand_ratio)
    {
        column.expand = true;
        column.expand_ratio = ratio;
    }
    if let Some(shrink) = attribute_value(attributes, "shrink").and_then(parse_bool) {
        column.shrink = shrink;
    }
}

fn parse_span(value: Option<&str>, maximum: u16) -> u16 {
    value
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .map(|value| {
            u16::try_from(value.min(u64::from(maximum.max(DEFAULT_CELL_SPAN))))
                .expect("span is clamped to its compact representation")
        })
        .unwrap_or(DEFAULT_CELL_SPAN)
}

fn parse_expand_ratio(value: &str) -> Option<u16> {
    value
        .trim()
        .parse::<u16>()
        .ok()
        .map(|ratio| ratio.clamp(1, MAX_COLUMN_EXPAND_RATIO))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

use crate::core::framework::render::ParagraphOverride;
use zircon_runtime_interface::ui::surface::UiTextAlign;

use super::bbcode::attribute_value;

const MAX_BLOCK_NESTING: usize = 32;
const MAX_PARAGRAPH_INDENT: f32 = 4096.0;

#[derive(Clone, Debug)]
enum ListKind {
    Unordered { bullet: String },
    Ordered { marker: OrderedMarker, next: u32 },
}

#[derive(Clone, Copy, Debug)]
enum OrderedMarker {
    Decimal,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
}

#[derive(Clone, Debug)]
struct ListState {
    tag: &'static str,
    kind: ListKind,
}

#[derive(Clone, Debug)]
pub(super) enum BlockOpen {
    Container,
    Paragraph {
        name: String,
        paragraph: ParagraphOverride,
        prefix: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub(super) enum BlockClose {
    Container,
    Paragraph { name: String },
}

#[derive(Clone, Debug, Default)]
pub(super) struct BbCodeBlockState {
    lists: Vec<ListState>,
    suppressed_list_depth: usize,
}

impl BbCodeBlockState {
    pub(super) fn open(
        &mut self,
        name: &str,
        value: Option<&str>,
        attributes: &[(String, String)],
    ) -> Option<BlockOpen> {
        match name {
            "indent" => Some(BlockOpen::Paragraph {
                name: name.to_string(),
                paragraph: ParagraphOverride {
                    indent_level: Some(parse_indent_level(value).unwrap_or(1)),
                    ..ParagraphOverride::default()
                },
                prefix: None,
            }),
            "p" => Some(BlockOpen::Paragraph {
                name: name.to_string(),
                paragraph: ParagraphOverride {
                    align: attribute_value(attributes, "align")
                        .or(value)
                        .and_then(parse_align),
                    indent: attribute_value(attributes, "indent").and_then(parse_indent),
                    ..ParagraphOverride::default()
                },
                prefix: None,
            }),
            "ul" => {
                if self.lists.len() < MAX_BLOCK_NESTING {
                    let bullet = attribute_value(attributes, "bullet")
                        .or(value)
                        .map(str::trim)
                        .filter(|bullet| !bullet.is_empty())
                        .unwrap_or("•")
                        .to_string();
                    self.lists.push(ListState {
                        tag: "ul",
                        kind: ListKind::Unordered { bullet },
                    });
                } else {
                    self.suppressed_list_depth = self.suppressed_list_depth.saturating_add(1);
                }
                Some(BlockOpen::Container)
            }
            "ol" => {
                if self.lists.len() < MAX_BLOCK_NESTING {
                    let marker = attribute_value(attributes, "type")
                        .or(value)
                        .and_then(parse_ordered_marker)
                        .unwrap_or(OrderedMarker::Decimal);
                    self.lists.push(ListState {
                        tag: "ol",
                        kind: ListKind::Ordered { marker, next: 1 },
                    });
                } else {
                    self.suppressed_list_depth = self.suppressed_list_depth.saturating_add(1);
                }
                Some(BlockOpen::Container)
            }
            "li" => {
                if self.suppressed_list_depth > 0 {
                    return Some(BlockOpen::Paragraph {
                        name: name.to_string(),
                        paragraph: ParagraphOverride {
                            indent_level: Some(1),
                            ..ParagraphOverride::default()
                        },
                        prefix: None,
                    });
                }
                let prefix = self.lists.last_mut().map(list_prefix)?;
                Some(BlockOpen::Paragraph {
                    name: name.to_string(),
                    paragraph: ParagraphOverride {
                        indent_level: Some(1),
                        ..ParagraphOverride::default()
                    },
                    prefix: Some(prefix),
                })
            }
            _ => None,
        }
    }

    pub(super) fn close(&mut self, name: &str) -> Option<BlockClose> {
        match name {
            "indent" | "p" | "li" => Some(BlockClose::Paragraph {
                name: name.to_string(),
            }),
            "ul" | "ol" => {
                if self.suppressed_list_depth > 0 {
                    self.suppressed_list_depth -= 1;
                } else if let Some(position) = self.lists.iter().rposition(|list| list.tag == name)
                {
                    self.lists.truncate(position);
                }
                Some(BlockClose::Container)
            }
            _ => None,
        }
    }
}

fn parse_align(value: &str) -> Option<UiTextAlign> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" => Some(UiTextAlign::Left),
        "center" => Some(UiTextAlign::Center),
        "right" => Some(UiTextAlign::Right),
        "fill" | "justify" => Some(UiTextAlign::Justify),
        "start" => Some(UiTextAlign::Start),
        "end" => Some(UiTextAlign::End),
        _ => None,
    }
}

fn parse_indent(value: &str) -> Option<f32> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|indent| indent.is_finite() && *indent >= 0.0)
        .map(|indent| indent.min(MAX_PARAGRAPH_INDENT))
}

fn parse_indent_level(value: Option<&str>) -> Option<u16> {
    value?
        .trim()
        .parse::<u16>()
        .ok()
        .map(|level| level.clamp(1, MAX_BLOCK_NESTING as u16))
}

fn parse_ordered_marker(value: &str) -> Option<OrderedMarker> {
    match value.trim() {
        "1" => Some(OrderedMarker::Decimal),
        "a" => Some(OrderedMarker::AlphaLower),
        "A" => Some(OrderedMarker::AlphaUpper),
        "i" => Some(OrderedMarker::RomanLower),
        "I" => Some(OrderedMarker::RomanUpper),
        _ => None,
    }
}

fn list_prefix(list: &mut ListState) -> String {
    let marker = match &mut list.kind {
        ListKind::Unordered { bullet } => bullet.clone(),
        ListKind::Ordered { marker, next } => {
            let value = ordered_marker_text(*next, *marker);
            *next = next.saturating_add(1);
            value
        }
    };
    format!("{marker} ")
}

fn ordered_marker_text(value: u32, marker: OrderedMarker) -> String {
    let marker = match marker {
        OrderedMarker::Decimal => value.to_string(),
        OrderedMarker::AlphaLower => alpha_marker(value, false),
        OrderedMarker::AlphaUpper => alpha_marker(value, true),
        OrderedMarker::RomanLower => roman_marker(value).to_ascii_lowercase(),
        OrderedMarker::RomanUpper => roman_marker(value),
    };
    format!("{marker}.")
}

fn alpha_marker(mut value: u32, uppercase: bool) -> String {
    value = value.max(1);
    let base = if uppercase { b'A' } else { b'a' };
    let mut marker = Vec::new();
    while value > 0 {
        value -= 1;
        marker.push(char::from(base + (value % 26) as u8));
        value /= 26;
    }
    marker.into_iter().rev().collect()
}

fn roman_marker(value: u32) -> String {
    if !(1..=3999).contains(&value) {
        return value.to_string();
    }
    const ROMAN: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut remaining = value;
    let mut marker = String::new();
    for &(unit, symbol) in ROMAN {
        while remaining >= unit {
            marker.push_str(symbol);
            remaining -= unit;
        }
    }
    marker
}

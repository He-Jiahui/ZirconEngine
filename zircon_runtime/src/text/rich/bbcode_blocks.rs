use crate::text::{ParagraphOverride, RichListItemKind, RichOrderedListMarker, TextAlign};

use super::admission::RichTextParseError;
use super::bbcode::attribute_value;

const MAX_PARAGRAPH_INDENT_LEVEL: u16 = 32;
const MAX_PARAGRAPH_INDENT: f32 = 4096.0;

#[derive(Clone, Debug)]
enum ListKind {
    Unordered {
        bullet: String,
    },
    Ordered {
        marker: RichOrderedListMarker,
        next: u32,
    },
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
        list_item: Option<PendingListItem>,
    },
}

#[derive(Clone, Debug)]
pub(super) struct PendingListItem {
    pub(super) prefix: String,
    pub(super) kind: RichListItemKind,
    pub(super) level: u32,
}

#[derive(Clone, Debug)]
pub(super) enum BlockClose {
    Container,
    Paragraph { name: String },
}

#[derive(Clone, Debug)]
pub(super) struct BbCodeBlockState {
    lists: Vec<ListState>,
    max_depth: usize,
}

impl BbCodeBlockState {
    pub(super) fn new(max_depth: usize) -> Self {
        Self {
            lists: Vec::new(),
            max_depth,
        }
    }

    pub(super) fn open(
        &mut self,
        name: &str,
        value: Option<&str>,
        attributes: &[(String, String)],
        active_paragraph_depth: usize,
    ) -> Result<Option<BlockOpen>, RichTextParseError> {
        let opened = match name {
            "indent" => Some(BlockOpen::Paragraph {
                name: name.to_string(),
                paragraph: ParagraphOverride {
                    indent_level: Some(parse_indent_level(value).unwrap_or(1)),
                    ..ParagraphOverride::default()
                },
                list_item: None,
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
                list_item: None,
            }),
            "ul" => {
                self.admit_list_depth(active_paragraph_depth)?;
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
                Some(BlockOpen::Container)
            }
            "ol" => {
                self.admit_list_depth(active_paragraph_depth)?;
                let marker = attribute_value(attributes, "type")
                    .or(value)
                    .and_then(parse_ordered_marker)
                    .unwrap_or(RichOrderedListMarker::Decimal);
                self.lists.push(ListState {
                    tag: "ol",
                    kind: ListKind::Ordered { marker, next: 1 },
                });
                Some(BlockOpen::Container)
            }
            "li" => {
                let level = u32::try_from(self.lists.len()).map_err(|_| {
                    RichTextParseError::ArtifactIndexCapacityExceeded {
                        index_kind: "list nesting level",
                        actual: self.lists.len(),
                        max: u32::MAX as usize,
                    }
                })?;
                let Some(list) = self.lists.last_mut() else {
                    return Ok(None);
                };
                Some(BlockOpen::Paragraph {
                    name: name.to_string(),
                    paragraph: ParagraphOverride {
                        indent_level: Some(1),
                        ..ParagraphOverride::default()
                    },
                    list_item: Some(pending_list_item(list, level)?),
                })
            }
            _ => None,
        };
        Ok(opened)
    }

    pub(super) fn close(&mut self, name: &str) -> Option<BlockClose> {
        match name {
            "indent" | "p" | "li" => Some(BlockClose::Paragraph {
                name: name.to_string(),
            }),
            "ul" | "ol" => {
                if let Some(position) = self.lists.iter().rposition(|list| list.tag == name) {
                    self.lists.truncate(position);
                }
                Some(BlockClose::Container)
            }
            _ => None,
        }
    }

    pub(super) const fn depth(&self) -> usize {
        self.lists.len()
    }

    fn admit_list_depth(&self, active_paragraph_depth: usize) -> Result<(), RichTextParseError> {
        let attempted_depth = self
            .lists
            .len()
            .saturating_add(active_paragraph_depth)
            .saturating_add(1);
        if attempted_depth > self.max_depth {
            return Err(RichTextParseError::BlockDepthBudgetExceeded {
                attempted_depth,
                max_depth: self.max_depth,
            });
        }
        Ok(())
    }
}

fn parse_align(value: &str) -> Option<TextAlign> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("left") {
        Some(TextAlign::Left)
    } else if value.eq_ignore_ascii_case("center") {
        Some(TextAlign::Center)
    } else if value.eq_ignore_ascii_case("right") {
        Some(TextAlign::Right)
    } else if value.eq_ignore_ascii_case("fill") || value.eq_ignore_ascii_case("justify") {
        Some(TextAlign::Justify)
    } else if value.eq_ignore_ascii_case("start") {
        Some(TextAlign::Start)
    } else if value.eq_ignore_ascii_case("end") {
        Some(TextAlign::End)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "bbcode_blocks/allocation_free_align_tests.rs"]
mod allocation_free_align_tests;

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
        .map(|level| level.clamp(1, MAX_PARAGRAPH_INDENT_LEVEL))
}

fn parse_ordered_marker(value: &str) -> Option<RichOrderedListMarker> {
    match value.trim() {
        "1" => Some(RichOrderedListMarker::Decimal),
        "a" => Some(RichOrderedListMarker::AlphaLower),
        "A" => Some(RichOrderedListMarker::AlphaUpper),
        "i" => Some(RichOrderedListMarker::RomanLower),
        "I" => Some(RichOrderedListMarker::RomanUpper),
        _ => None,
    }
}

fn pending_list_item(
    list: &mut ListState,
    level: u32,
) -> Result<PendingListItem, RichTextParseError> {
    let (marker_text, kind) = match &mut list.kind {
        ListKind::Unordered { bullet } => (bullet.clone(), RichListItemKind::Unordered),
        ListKind::Ordered { marker, next } => {
            let ordinal = *next;
            let next_ordinal = ordinal.checked_add(1).ok_or(
                RichTextParseError::ArtifactIndexCapacityExceeded {
                    index_kind: "ordered list ordinal",
                    actual: u32::MAX as usize,
                    max: (u32::MAX - 1) as usize,
                },
            )?;
            *next = next_ordinal;
            (
                ordered_marker_text(ordinal, *marker),
                RichListItemKind::Ordered {
                    ordinal,
                    marker: *marker,
                },
            )
        }
    };
    Ok(PendingListItem {
        prefix: format!("{marker_text} "),
        kind,
        level,
    })
}

fn ordered_marker_text(value: u32, marker: RichOrderedListMarker) -> String {
    let marker = match marker {
        RichOrderedListMarker::Decimal => value.to_string(),
        RichOrderedListMarker::AlphaLower => alpha_marker(value, false),
        RichOrderedListMarker::AlphaUpper => alpha_marker(value, true),
        RichOrderedListMarker::RomanLower => roman_marker(value).to_ascii_lowercase(),
        RichOrderedListMarker::RomanUpper => roman_marker(value),
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

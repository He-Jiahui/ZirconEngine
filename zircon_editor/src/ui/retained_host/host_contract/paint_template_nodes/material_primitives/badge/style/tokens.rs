use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::first_non_empty;

pub(super) fn badge_color_token(node: &TemplatePaneNodeData) -> &str {
    if let Some(token) = badge_color_variant(&node.component_variant) {
        return token;
    }
    first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()])
}

fn badge_color_variant(component_variant: &str) -> Option<&'static str> {
    let mut best = u8::MAX;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        let candidate = if part.eq_ignore_ascii_case("primary") {
            1
        } else if part.eq_ignore_ascii_case("secondary") {
            2
        } else if part.eq_ignore_ascii_case("error") {
            3
        } else if part.eq_ignore_ascii_case("danger") {
            4
        } else if part.eq_ignore_ascii_case("info") {
            5
        } else if part.eq_ignore_ascii_case("success") {
            6
        } else if part.eq_ignore_ascii_case("warning") {
            7
        } else if part.eq_ignore_ascii_case("default") {
            8
        } else {
            continue;
        };
        best = best.min(candidate);
        if best == 1 {
            break;
        }
    }
    match best {
        1 => Some("primary"),
        2 => Some("secondary"),
        3 => Some("error"),
        4 => Some("danger"),
        5 => Some("info"),
        6 => Some("success"),
        7 => Some("warning"),
        8 => Some("default"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tokens/single_scan_color_tests.rs"]
mod single_scan_color_tests;

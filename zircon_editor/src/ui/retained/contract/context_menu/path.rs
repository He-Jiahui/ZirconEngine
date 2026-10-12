use super::super::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) fn target_value_text(
    hit: &TemplateNodePointerHit,
) -> SharedString {
    if !hit.value_text.is_empty() {
        return hit.value_text.clone();
    }
    hit.control_id.clone()
}

pub(in crate::ui::retained_host::host_contract) fn push_path_segment(
    path: &mut String,
    value: &str,
) {
    let segment_start = path.len();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            path.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_' | '.') {
            if ch != '-' || path.len() > segment_start {
                path.push(ch);
            }
        } else if ch.is_whitespace() && path.len() > segment_start {
            path.push('-');
        }
    }
    while path.len() > segment_start && path.ends_with('-') {
        path.pop();
    }
}

#[cfg(test)]
#[path = "tests/path.rs"]
mod tests;

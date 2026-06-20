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

pub(in crate::ui::retained_host::host_contract) fn path_segment(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if matches!(ch, '-' | '_' | '.') {
                Some(ch)
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

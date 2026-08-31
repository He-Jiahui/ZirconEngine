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
mod tests {
    use super::*;

    #[test]
    fn path_segment_writes_normalized_text_without_an_intermediate_string() {
        assert_eq!(normalized(" --Scene Tree / Props-- "), "scene-tree--props");
        assert_eq!(normalized("A_B.C"), "a_b.c");
        assert_eq!(normalized("---"), "");
    }

    fn normalized(value: &str) -> String {
        let mut path = String::from("workbench://test/");
        let prefix_len = path.len();
        push_path_segment(&mut path, value);
        path[prefix_len..].to_string()
    }
}

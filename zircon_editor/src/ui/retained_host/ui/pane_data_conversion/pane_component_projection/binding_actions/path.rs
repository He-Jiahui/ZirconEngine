pub(in super::super) fn binding_path_action_id(binding_id: &str) -> String {
    let mut output = String::with_capacity(binding_id.len());
    push_binding_path_action_id(&mut output, binding_id);
    output
}

pub(in super::super) fn push_binding_path_action_id(output: &mut String, binding_id: &str) {
    let mut first_segment = true;
    for segment in binding_id
        .split(['/', '.', ':'])
        .filter(|segment| !segment.is_empty())
    {
        if !first_segment {
            output.push('.');
        }
        first_segment = false;
        push_snake_segment(output, segment);
    }
}

fn push_snake_segment(output: &mut String, value: &str) {
    let segment_start = output.len();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else {
            if output.len() > segment_start && !output.ends_with('_') {
                output.push('_');
            }
            previous_was_separator = true;
        }
    }
    if output.len() > segment_start && output.ends_with('_') {
        output.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::binding_path_action_id;

    #[test]
    fn normalizes_multiple_path_separators_without_dropping_empty_normalized_segments() {
        assert_eq!(
            binding_path_action_id("UiComponentShowcase/ArrayField:SetElement"),
            "ui_component_showcase.array_field.set_element"
        );
        assert_eq!(binding_path_action_id("Alpha/---/Beta"), "alpha..beta");
    }
}

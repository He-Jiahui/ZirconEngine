pub(super) fn binding_path_action_id(binding_id: &str) -> String {
    binding_id
        .split(['/', '.', ':'])
        .filter(|segment| !segment.is_empty())
        .map(camel_to_snake_segment)
        .collect::<Vec<_>>()
        .join(".")
}

fn camel_to_snake_segment(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !output.ends_with('_') {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !output.ends_with('_') {
            output.push('_');
            previous_was_separator = true;
        }
    }
    output.trim_matches('_').to_string()
}

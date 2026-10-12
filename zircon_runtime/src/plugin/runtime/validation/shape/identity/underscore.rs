pub(super) fn validate_runtime_plugin_package_id_underscore(
    context: &str,
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) {
    let mut segment_ends_with_underscore = false;
    let mut previous_was_underscore = false;
    let invalid = value.bytes().any(|byte| {
        if byte == b'.' {
            let invalid = segment_ends_with_underscore;
            segment_ends_with_underscore = false;
            previous_was_underscore = false;
            invalid
        } else if byte == b'_' {
            let invalid = previous_was_underscore;
            segment_ends_with_underscore = true;
            previous_was_underscore = true;
            invalid
        } else {
            segment_ends_with_underscore = false;
            previous_was_underscore = false;
            false
        }
    }) || segment_ends_with_underscore;

    if invalid {
        diagnostics.push(format!(
            "{context} {field_name} `{value}` segments must not end with an underscore or contain repeated underscores"
        ));
    }
}

#[cfg(test)]
#[path = "tests/underscore.rs"]
mod tests;
